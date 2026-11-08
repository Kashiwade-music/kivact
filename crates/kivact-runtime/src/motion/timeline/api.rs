//! Role: The `Timeline` value, its tracks, segments and playback cell (design §12.6).
//!
//! Invariants:
//! - A segment's start is fixed when it is added (or moved with
//!   [`Segment::at`] right after): later additions never move it.
//! - A track's value at a time is its last segment that has started, shown
//!   at that time; a segment starts from the value its track had at its
//!   start, computed once from the segments before it (`Plan`).
//! - A tick publishes only the tracks whose value changed.
//! - Calls and completions run as effects of the timeline's watcher, never
//!   inside a frame step, so user code runs outside the motion driver.

use std::cell::{Cell, RefCell};
use std::panic::Location;
use std::rc::Rc;

use kivact_types::motion::{PlayDirection, Transition};

use super::super::Repeat;
use super::super::animated::shift;
use super::super::state::{Anim, LoopMotion, MotionState};
use super::super::transition::Kind;
use super::super::values::Animatable;
use super::curve::{seg_value, spring_rest_ms};
use super::plan::{Plan, TrackSlot, TrackState, build_plan, span_of};
use super::segment::{SegmentDef, SegmentKind};
use crate::{
    Callback, EffectKind, ErrorKind, IntoValue, Owned, Owner, Signal, UiError, Value,
    require_effectful, require_setup,
};

/// A timeline: tracks driven along one clock or a progress value. Build it in
/// setup; control it from handlers and effects. Copy handle.
#[derive(Clone, Copy, PartialEq)]
pub struct Timeline {
    cell: Owned<Rc<TimelineCell>>,
}

/// A timeline owned by the current setup owner (setup only).
#[track_caller]
pub fn timeline() -> Timeline {
    let at = Location::caller();
    let owner = Owner::current("timeline", at);
    let rt = owner.runtime();
    let cell = Rc::new(TimelineCell::new(owner, at));
    let anim: Rc<dyn Anim> = cell.clone();
    MotionState::of(&rt).register_anim(&anim);
    drop(anim);
    let events = cell.events;
    let weak = Rc::downgrade(&cell);
    crate::reactive::watch(
        move || events.get(),
        EffectKind::Internal,
        move |_| {
            if let Some(cell) = weak.upgrade() {
                cell.run_pending();
            }
        },
    );
    Timeline {
        cell: owner.store("Timeline", cell, at),
    }
}

/// A value a timeline publishes. Read it like any value.
pub struct TimelineTrack<T: 'static> {
    timeline: Timeline,
    index: u32,
    value: Value<T>,
}

impl<T: 'static> Clone for TimelineTrack<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for TimelineTrack<T> {}

impl<T: Animatable> TimelineTrack<T> {
    /// Current value of the track (tracked).
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Current value of the track without tracking.
    pub fn get_untracked(&self) -> T {
        self.value.get_untracked()
    }
}

impl<T: Animatable> IntoValue<T> for TimelineTrack<T> {
    fn into_prop(self) -> crate::Prop<T> {
        crate::Prop::value(self.value)
    }
}

/// Where a segment, label or call starts: relative to the previous segment,
/// or at an absolute time, or at a label.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum At {
    /// After the previous segment ends, plus this many ms.
    AfterPrevious(f64),
    /// When the previous segment starts, plus this many ms.
    WithPrevious(f64),
    /// At this many ms from the start.
    Time(f64),
    /// At a label, plus this many ms.
    Label(&'static str, f64),
}

/// A segment of a timeline (Copy handle), returned by `to` and `call`. `at`
/// moves its start.
#[derive(Clone, Copy)]
pub struct Segment {
    timeline: Timeline,
    index: u32,
}

impl Segment {
    /// Move this segment's start to `at` (setup only; call it right after the
    /// segment is added).
    #[track_caller]
    pub fn at(self, at: At) -> Segment {
        let location = Location::caller();
        require_setup("Segment::at");
        let cell = self.timeline.cell(location);
        let index = self.index as usize;
        let prev = cell.span_before(index);
        let start = cell.resolve(at, prev, location);
        cell.segments.borrow_mut()[index].start = start;
        cell.rebuild();
        self
    }
}

impl Timeline {
    fn cell(&self, at: &'static Location<'static>) -> Rc<TimelineCell> {
        self.cell.get_at(at)
    }

    /// A track starting at `initial` (setup only).
    #[track_caller]
    pub fn track<T: Animatable>(&self, initial: T) -> TimelineTrack<T> {
        let at = Location::caller();
        require_setup("Timeline::track");
        let cell = self.cell(at);
        let mut components = Vec::with_capacity(T::COMPONENTS);
        initial.to_components(&mut components);
        let value = cell.owner.signal(initial, at);
        let index = cell.tracks.borrow().len();
        cell.tracks.borrow_mut().push(Box::new(TrackState {
            value,
            initial: components.clone(),
            last: RefCell::new(components),
        }));
        cell.rebuild();
        TimelineTrack {
            timeline: *self,
            index: index as u32,
            value: Value::from(value),
        }
    }

    /// A segment that moves `track` to `target` with `transition`, starting
    /// after the previous segment (setup only). Use [`Segment::at`] to move it.
    #[track_caller]
    pub fn to<T: Animatable>(
        &self,
        track: TimelineTrack<T>,
        target: T,
        transition: Transition,
    ) -> Segment {
        let at = Location::caller();
        require_setup("Timeline::to");
        assert!(
            track.timeline == *self,
            "Timeline::to: the track belongs to another timeline"
        );
        let cell = self.cell(at);
        let mut transition = transition;
        if transition.repeat == Repeat::Forever {
            cell.owner.report(UiError::new(
                ErrorKind::Message,
                "a timeline segment cannot repeat forever; it plays once",
                Some(at),
            ));
            transition.repeat = Repeat::Times(1);
        }
        let one_ms = match &transition.kind {
            Kind::Spring(spring) => {
                if let Some(problem) = spring.problem() {
                    cell.owner.report(UiError::new(
                        ErrorKind::Message,
                        format!("spring: {problem}"),
                        Some(at),
                    ));
                }
                spring_rest_ms(spring)
            }
            Kind::Tween { ms, .. } | Kind::Keyframes { ms, .. } => *ms,
        };
        let repeats = match transition.repeat {
            Repeat::Times(n) => f64::from(n),
            Repeat::Forever => 1.0,
        };
        let len = transition.delay_ms + one_ms * repeats;
        let mut to = Vec::with_capacity(T::COMPONENTS);
        target.to_components(&mut to);
        let index = cell.push(
            SegmentDef {
                start: 0.0,
                len,
                kind: SegmentKind::Track {
                    track: track.index as usize,
                    to,
                    transition,
                    one_ms,
                },
            },
            At::AfterPrevious(0.0),
            at,
        );
        Segment {
            timeline: *self,
            index: index as u32,
        }
    }

    /// A named time on the timeline (setup only).
    #[track_caller]
    pub fn label(&self, name: &'static str, at: At) {
        let location = Location::caller();
        require_setup("Timeline::label");
        let cell = self.cell(location);
        let time = cell.resolve(at, cell.last_span(), location);
        cell.labels.borrow_mut().push((name, time));
    }

    /// A call of `f` at `at`: a segment of length zero (setup only).
    #[track_caller]
    pub fn call(&self, at: At, f: impl Into<Callback<()>>) -> Segment {
        let location = Location::caller();
        require_setup("Timeline::call");
        let cell = self.cell(location);
        let index = cell.push(
            SegmentDef {
                start: 0.0,
                len: 0.0,
                kind: SegmentKind::Call(f.into()),
            },
            at,
            location,
        );
        Segment {
            timeline: *self,
            index: index as u32,
        }
    }

    /// Play the whole timeline this many times (setup only).
    #[track_caller]
    pub fn repeat(self, repeat: Repeat) -> Timeline {
        require_setup("Timeline::repeat");
        self.cell(Location::caller()).repeat.set(repeat);
        self
    }

    /// Play every other repetition backwards (setup only).
    #[track_caller]
    pub fn alternate(self) -> Timeline {
        require_setup("Timeline::alternate");
        self.cell(Location::caller())
            .alternate
            .set(PlayDirection::Alternate);
        self
    }

    /// Keep the timeline running under reduced motion (setup only).
    #[track_caller]
    pub fn essential(self) -> Timeline {
        require_setup("Timeline::essential");
        self.cell(Location::caller())
            .motion
            .set(LoopMotion::Essential);
        self
    }

    /// Drive time by a progress value in `0..=1` instead of the clock; call
    /// it after the segments are added (setup only).
    #[track_caller]
    pub fn drive(self, progress: impl IntoValue<f64>) -> Timeline {
        let at = Location::caller();
        require_setup("Timeline::drive");
        let cell = self.cell(at);
        let source = progress.into_prop();
        cell.phase.set(TimelinePhase::Driven);
        cell.drive_to(crate::untrack(|| source.get()), at);
        let weak = Rc::downgrade(&cell);
        crate::reactive::watch(
            move || source.get(),
            EffectKind::Internal,
            move |p: &f64| {
                if let Some(cell) = weak.upgrade() {
                    cell.drive_to(*p, at);
                }
            },
        );
        self
    }

    /// Register a setup-owned callback for the completion of a playback
    /// (setup only). A driven timeline never completes.
    #[track_caller]
    pub fn on_complete(self, f: impl Into<Callback<()>>) -> Timeline {
        require_setup("Timeline::on_complete");
        self.cell(Location::caller())
            .completions
            .borrow_mut()
            .push(f.into());
        self
    }

    /// Start playing from the current time, or from the start when finished.
    #[track_caller]
    pub fn play(&self) {
        let at = Location::caller();
        require_effectful("Timeline::play");
        self.cell(at).play(at);
    }

    /// Stop playing, keeping the time.
    #[track_caller]
    pub fn pause(&self) {
        let at = Location::caller();
        require_effectful("Timeline::pause");
        self.cell(at).pause(at);
    }

    /// Play the other way from the current time.
    #[track_caller]
    pub fn reverse(&self) {
        let at = Location::caller();
        require_effectful("Timeline::reverse");
        self.cell(at).reverse(at);
    }

    /// Play again from the start (the end, when playing backwards).
    #[track_caller]
    pub fn restart(&self) {
        let at = Location::caller();
        require_effectful("Timeline::restart");
        self.cell(at).restart(at);
    }

    /// Jump to the end of the playback and complete it.
    #[track_caller]
    pub fn finish(&self) {
        let at = Location::caller();
        require_effectful("Timeline::finish");
        self.cell(at).finish(at);
    }

    /// Move to `ms` (clamped to the playback) and keep the play state.
    #[track_caller]
    pub fn seek(&self, ms: f64) {
        let at = Location::caller();
        require_effectful("Timeline::seek");
        self.cell(at).seek(ms, at);
    }

    /// Move to a label (reported and ignored when it is not defined).
    #[track_caller]
    pub fn seek_label(&self, name: &'static str) {
        let at = Location::caller();
        require_effectful("Timeline::seek_label");
        let cell = self.cell(at);
        match cell.label_time(name) {
            Some(time) => cell.seek(time, at),
            None => cell.report_missing_label(name, at),
        }
    }

    /// Set the playback rate (a positive value; others are reported and ignored).
    #[track_caller]
    pub fn set_rate(&self, rate: f64) {
        let at = Location::caller();
        require_effectful("Timeline::set_rate");
        self.cell(at).set_rate(rate, at);
    }

    /// The time (ms) last published (tracked).
    #[track_caller]
    pub fn time(&self) -> Value<f64> {
        Value::from(self.cell(Location::caller()).time)
    }

    /// The time as a fraction of the whole playback, in `0..=1` (tracked).
    #[track_caller]
    pub fn progress(&self) -> Value<f64> {
        let cell = self.cell(Location::caller());
        let time = cell.time;
        let whole = match cell.repeat.get() {
            Repeat::Times(n) => cell.duration() * f64::from(n),
            Repeat::Forever => cell.duration(),
        };
        cell.owner.with_setup(|| {
            crate::memo(move || {
                if whole > 0.0 {
                    (time.get() / whole).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            })
        })
    }

    /// The length of one repetition in ms: the latest end of a segment, label
    /// or call.
    #[track_caller]
    pub fn duration(&self) -> f64 {
        self.cell(Location::caller()).duration()
    }

    /// Whether the timeline is playing (tracked).
    #[track_caller]
    pub fn is_playing(&self) -> Value<bool> {
        let cell = self.cell(Location::caller());
        let phase = cell.phase;
        cell.owner.with_setup(|| {
            crate::memo(move || matches!(phase.get(), TimelinePhase::Playing { .. }))
        })
    }
}

/// Where a timeline stands. Time is in ms from the start of the playback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TimelinePhase {
    /// Not playing, at `time` (the state after setup, at 0).
    Idle { time: f64 },
    /// Playing from `from_time` since `start_ns` on the App's clock.
    Playing { start_ns: u64, from_time: f64 },
    /// Stopped by `pause` or a seek, at `time`.
    Paused { time: f64 },
    /// At an end after the last repetition.
    Finished { time: f64 },
    /// Time follows a progress value (`drive`).
    Driven,
}

/// The direction the clock moves in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Forward,
    Backward,
}

impl Direction {
    fn flipped(self) -> Direction {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }
}

/// How playback proceeds; kept across every phase.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Playback {
    rate: f64,
    direction: Direction,
}

/// Something a tick or a control completed, to be run by the watcher.
#[derive(Clone, Copy)]
enum Event {
    /// A playback completed: the `on_complete` callbacks run.
    Complete,
    /// The call segment at this index is due.
    Call(usize),
}

/// The state of one timeline: its structure (setup) and its playback.
pub(crate) struct TimelineCell {
    owner: Owner,
    phase: Signal<TimelinePhase>,
    /// The time last published (ms).
    time: Signal<f64>,
    /// Counts the events the watcher must run.
    events: Signal<u64>,
    playback: Cell<Playback>,
    repeat: Cell<Repeat>,
    alternate: Cell<PlayDirection>,
    motion: Cell<LoopMotion>,
    segments: RefCell<Vec<SegmentDef>>,
    labels: RefCell<Vec<(&'static str, f64)>>,
    tracks: RefCell<Vec<Box<dyn TrackSlot>>>,
    plan: RefCell<Plan>,
    pending: RefCell<Vec<Event>>,
    completions: RefCell<Vec<Callback<()>>>,
}

impl TimelineCell {
    fn new(owner: Owner, at: &'static Location<'static>) -> Self {
        Self {
            owner,
            phase: owner.signal(TimelinePhase::Idle { time: 0.0 }, at),
            time: owner.signal(0.0, at),
            events: owner.signal(0, at),
            playback: Cell::new(Playback {
                rate: 1.0,
                direction: Direction::Forward,
            }),
            repeat: Cell::new(Repeat::Times(1)),
            alternate: Cell::new(PlayDirection::Normal),
            motion: Cell::new(LoopMotion::Decorative),
            segments: RefCell::new(Vec::new()),
            labels: RefCell::new(Vec::new()),
            tracks: RefCell::new(Vec::new()),
            plan: RefCell::new(Plan::default()),
            pending: RefCell::new(Vec::new()),
            completions: RefCell::new(Vec::new()),
        }
    }

    fn now(&self) -> u64 {
        self.owner.runtime().now_ns()
    }

    fn reduced(&self) -> bool {
        let rt = self.owner.runtime();
        MotionState::of(&rt).reduced_motion(&rt)
    }

    fn wake(&self) {
        self.owner.runtime().request_turn();
    }

    /// The length of one repetition (ms).
    fn duration(&self) -> f64 {
        let segments = self.segments.borrow();
        let labels = self.labels.borrow();
        segments
            .iter()
            .map(|s| s.start + s.len)
            .chain(labels.iter().map(|l| l.1))
            .fold(0.0, f64::max)
    }

    /// The whole playback (ms), repetitions included.
    fn total(&self) -> f64 {
        match self.repeat.get() {
            Repeat::Times(n) => self.duration() * f64::from(n),
            Repeat::Forever => f64::INFINITY,
        }
    }

    /// Where playback in the current direction ends.
    fn end_time(&self) -> f64 {
        match self.playback.get().direction {
            Direction::Forward => self.total(),
            Direction::Backward => 0.0,
        }
    }

    /// Where playback in the current direction starts.
    fn start_time(&self) -> f64 {
        match self.playback.get().direction {
            Direction::Forward => 0.0,
            Direction::Backward => {
                let total = self.total();
                if total.is_finite() { total } else { 0.0 }
            }
        }
    }

    fn label_time(&self, name: &str) -> Option<f64> {
        self.labels
            .borrow()
            .iter()
            .find(|l| l.0 == name)
            .map(|l| l.1)
    }

    fn last_span(&self) -> Option<(f64, f64)> {
        self.segments.borrow().last().map(span_of)
    }

    fn span_before(&self, index: usize) -> Option<(f64, f64)> {
        match index.checked_sub(1) {
            Some(prev) => self.segments.borrow().get(prev).map(span_of),
            None => None,
        }
    }

    /// The start (ms) of a segment, label or call placed by `at`; `prev` is the
    /// span of the segment before it.
    fn resolve(
        &self,
        at: At,
        prev: Option<(f64, f64)>,
        location: &'static Location<'static>,
    ) -> f64 {
        let after = prev.map_or(0.0, |(_, end)| end);
        match at {
            At::AfterPrevious(ms) => after + ms,
            At::WithPrevious(ms) => prev.map_or(0.0, |(start, _)| start) + ms,
            At::Time(ms) => ms,
            At::Label(name, ms) => match self.label_time(name) {
                Some(time) => time + ms,
                None => {
                    self.report_missing_label(name, location);
                    after + ms
                }
            },
        }
    }

    /// Report a label that is not defined; the caller then uses a fallback.
    fn report_missing_label(&self, name: &str, at: &'static Location<'static>) {
        self.owner.report(UiError::new(
            ErrorKind::Message,
            format!("timeline label `{name}` is not defined"),
            Some(at),
        ));
    }

    /// Add a segment placed by `at`; returns its index.
    fn push(&self, mut def: SegmentDef, at: At, location: &'static Location<'static>) -> usize {
        def.start = self.resolve(at, self.last_span(), location);
        let index = {
            let mut segments = self.segments.borrow_mut();
            segments.push(def);
            segments.len() - 1
        };
        self.rebuild();
        index
    }

    fn rebuild(&self) {
        let plan = build_plan(&self.segments.borrow(), &self.tracks.borrow());
        *self.plan.borrow_mut() = plan;
    }

    /// Queue an event for the watcher and wake it.
    fn announce(&self, event: Event) {
        self.pending.borrow_mut().push(event);
        self.events.update(|n| *n += 1);
    }

    /// Run the queued events, outside every frame step.
    fn run_pending(&self) {
        let pending = std::mem::take(&mut *self.pending.borrow_mut());
        for event in pending {
            let calls = match event {
                Event::Complete => self.completions.borrow().clone(),
                Event::Call(index) => match &self.segments.borrow()[index].kind {
                    SegmentKind::Call(f) => vec![*f],
                    SegmentKind::Track { .. } => continue,
                },
            };
            for f in calls {
                self.owner.run_handler(|| {
                    if let Err(err) = f.try_call(()) {
                        self.owner.report(err);
                    }
                });
            }
        }
    }

    /// The time inside one repetition (ms) for a playback time `time`; an
    /// alternate repetition that is odd runs backwards.
    fn local(&self, time: f64) -> f64 {
        let dur = self.duration();
        if dur <= 0.0 {
            return 0.0;
        }
        let last = match self.repeat.get() {
            Repeat::Times(n) => f64::from(n.saturating_sub(1)),
            Repeat::Forever => f64::INFINITY,
        };
        let iter = (time / dur).floor().clamp(0.0, last);
        let local = (time - iter * dur).clamp(0.0, dur);
        if self.alternate.get() == PlayDirection::Alternate && iter % 2.0 == 1.0 {
            dur - local
        } else {
            local
        }
    }

    /// Every track's value at the time `local` (ms) in one repetition.
    fn sample(&self, local: f64) -> Vec<Vec<f64>> {
        let segments = self.segments.borrow();
        let plan = self.plan.borrow();
        let tracks = self.tracks.borrow();
        tracks
            .iter()
            .enumerate()
            .map(|(track, slot)| {
                let order = &plan.order[track];
                let started = order.partition_point(|&s| segments[s].start <= local);
                match started.checked_sub(1) {
                    None => slot.initial(),
                    Some(k) => {
                        let s = order[k];
                        seg_value(&segments[s], &plan.from[s], local - segments[s].start)
                    }
                }
            })
            .collect()
    }

    /// Show playback time `time`: record it and publish the tracks.
    fn apply(&self, time: f64) {
        self.time.set(time);
        let values = self.sample(self.local(time));
        let tracks = self.tracks.borrow();
        for (slot, value) in tracks.iter().zip(&values) {
            slot.publish(value);
        }
    }

    /// Queue the calls that a move from `from` to `to` crosses. A forward move
    /// fires the calls in `[from, to)` (in `[from, to]` when it reached the
    /// end); a backward move the mirror of that.
    fn fire_calls(&self, from: f64, to: f64, reached: bool) {
        let forward = self.playback.get().direction == Direction::Forward;
        let mut due = Vec::new();
        for (index, seg) in self.segments.borrow().iter().enumerate() {
            if !matches!(seg.kind, SegmentKind::Call(_)) {
                continue;
            }
            let c = seg.start;
            let hit = if forward {
                from <= c && if reached { c <= to } else { c < to }
            } else {
                c <= from && if reached { c >= to } else { c > to }
            };
            if hit {
                due.push(Event::Call(index));
            }
        }
        if !due.is_empty() {
            self.pending.borrow_mut().extend(due);
            self.events.update(|n| *n += 1);
        }
    }

    /// A tick: move to `time`, fire the calls passed, and finish at the end.
    fn advance(&self, time: f64) {
        let from = self.time.get_untracked();
        let forward = self.playback.get().direction == Direction::Forward;
        let reached = if forward {
            time >= self.total()
        } else {
            time <= 0.0
        };
        if reached {
            self.fire_calls(from, self.end_time(), true);
            self.finish_at_end();
        } else {
            self.fire_calls(from, time, false);
            self.apply(time);
        }
    }

    /// Finish at the end of the playback and queue its completion.
    fn finish_at_end(&self) {
        let end = self.end_time();
        self.apply(end);
        self.phase.set(TimelinePhase::Finished { time: end });
        self.announce(Event::Complete);
    }

    /// Stop a decorative playback under reduced motion: at the end when it has
    /// one; an endless playback stops where it is.
    fn stop_for_reduced(&self) {
        if self.total().is_finite() {
            self.finish_at_end();
        } else {
            let time = self.current_time();
            self.apply(time);
            self.phase.set(TimelinePhase::Paused { time });
        }
    }

    /// The time of a running playback at `now`, before the end check.
    fn playing_raw(&self, start_ns: u64, from_time: f64, now: u64) -> f64 {
        let playback = self.playback.get();
        let elapsed = now.saturating_sub(start_ns) as f64 / 1e6 * playback.rate;
        match playback.direction {
            Direction::Forward => from_time + elapsed,
            Direction::Backward => from_time - elapsed,
        }
    }

    /// The time now, within the playback.
    fn current_time(&self) -> f64 {
        match self.phase.get_untracked() {
            TimelinePhase::Playing {
                start_ns,
                from_time,
            } => self
                .playing_raw(start_ns, from_time, self.now())
                .clamp(0.0, self.total()),
            TimelinePhase::Idle { time }
            | TimelinePhase::Paused { time }
            | TimelinePhase::Finished { time } => time,
            TimelinePhase::Driven => self.time.get_untracked(),
        }
    }

    /// Start playing from `time`; under reduced motion a decorative timeline
    /// goes straight to its end.
    fn start_from(&self, time: f64) {
        self.apply(time);
        if self.reduced() && self.motion.get() == LoopMotion::Decorative {
            self.stop_for_reduced();
            return;
        }
        self.phase.set(TimelinePhase::Playing {
            start_ns: self.now(),
            from_time: time,
        });
    }

    /// Report a call of a control on a driven timeline; true when refused.
    fn refuse_driven(&self, api: &str, at: &'static Location<'static>) -> bool {
        if matches!(self.phase.get_untracked(), TimelinePhase::Driven) {
            self.owner.report(UiError::new(
                ErrorKind::Message,
                format!("{api} does not apply to a driven timeline"),
                Some(at),
            ));
            true
        } else {
            false
        }
    }

    fn play(&self, at: &'static Location<'static>) {
        if self.refuse_driven("Timeline::play", at) {
            return;
        }
        match self.phase.get_untracked() {
            TimelinePhase::Idle { time } | TimelinePhase::Paused { time } => self.start_from(time),
            TimelinePhase::Finished { .. } => self.start_from(self.start_time()),
            TimelinePhase::Playing { .. } | TimelinePhase::Driven => {}
        }
        self.wake();
    }

    fn pause(&self, at: &'static Location<'static>) {
        if self.refuse_driven("Timeline::pause", at) {
            return;
        }
        if let TimelinePhase::Playing { .. } = self.phase.get_untracked() {
            let time = self.current_time();
            self.apply(time);
            self.phase.set(TimelinePhase::Paused { time });
        }
    }

    fn reverse(&self, at: &'static Location<'static>) {
        if self.refuse_driven("Timeline::reverse", at) {
            return;
        }
        let time = self.current_time();
        let mut playback = self.playback.get();
        playback.direction = playback.direction.flipped();
        self.playback.set(playback);
        if let TimelinePhase::Playing { .. } = self.phase.get_untracked() {
            self.phase.set(TimelinePhase::Playing {
                start_ns: self.now(),
                from_time: time,
            });
        }
        self.wake();
    }

    fn restart(&self, at: &'static Location<'static>) {
        if self.refuse_driven("Timeline::restart", at) {
            return;
        }
        self.start_from(self.start_time());
        self.wake();
    }

    fn finish(&self, at: &'static Location<'static>) {
        if self.refuse_driven("Timeline::finish", at) {
            return;
        }
        if let TimelinePhase::Finished { .. } = self.phase.get_untracked() {
            return;
        }
        if !self.total().is_finite() {
            self.owner.report(UiError::new(
                ErrorKind::Message,
                "Timeline::finish needs a finite repeat",
                Some(at),
            ));
            return;
        }
        self.finish_at_end();
        self.wake();
    }

    fn seek(&self, ms: f64, at: &'static Location<'static>) {
        if self.refuse_driven("Timeline::seek", at) {
            return;
        }
        if !ms.is_finite() {
            self.owner.report(UiError::new(
                ErrorKind::Message,
                "timeline seek needs a finite time",
                Some(at),
            ));
            return;
        }
        let time = ms.clamp(0.0, self.total());
        let next = match self.phase.get_untracked() {
            TimelinePhase::Idle { .. } => TimelinePhase::Idle { time },
            TimelinePhase::Paused { .. } | TimelinePhase::Finished { .. } => {
                TimelinePhase::Paused { time }
            }
            TimelinePhase::Playing { .. } => TimelinePhase::Playing {
                start_ns: self.now(),
                from_time: time,
            },
            TimelinePhase::Driven => return,
        };
        self.apply(time);
        self.phase.set(next);
        self.wake();
    }

    fn set_rate(&self, rate: f64, at: &'static Location<'static>) {
        if !(rate.is_finite() && rate > 0.0) {
            self.owner.report(UiError::new(
                ErrorKind::Message,
                "timeline rate must be a positive number",
                Some(at),
            ));
            return;
        }
        let time = self.current_time();
        let mut playback = self.playback.get();
        playback.rate = rate;
        self.playback.set(playback);
        if let TimelinePhase::Playing { .. } = self.phase.get_untracked() {
            self.phase.set(TimelinePhase::Playing {
                start_ns: self.now(),
                from_time: time,
            });
        }
        self.wake();
    }

    /// Show the progress `p` (0..=1) of a driven timeline; a non-finite
    /// progress is reported and ignored.
    fn drive_to(&self, p: f64, at: &'static Location<'static>) {
        if !p.is_finite() {
            self.owner.report(UiError::new(
                ErrorKind::Message,
                "timeline progress must be a finite number",
                Some(at),
            ));
            return;
        }
        self.apply(p.clamp(0.0, 1.0) * self.duration());
    }
}

impl Anim for TimelineCell {
    fn is_running(&self) -> bool {
        matches!(self.phase.get_untracked(), TimelinePhase::Playing { .. })
    }

    fn finish_decorative(&self) {
        if self.is_running() && self.motion.get() == LoopMotion::Decorative {
            self.stop_for_reduced();
        }
    }

    fn shift_clock(&self, paused: u64, now: u64) {
        if let TimelinePhase::Playing {
            start_ns,
            from_time,
        } = self.phase.get_untracked()
        {
            self.phase.set(TimelinePhase::Playing {
                start_ns: shift(start_ns, paused, now),
                from_time,
            });
        }
    }

    fn step(&self, now: u64) {
        let TimelinePhase::Playing {
            start_ns,
            from_time,
        } = self.phase.get_untracked()
        else {
            return;
        };
        self.advance(self.playing_raw(start_ns, from_time, now));
    }
}
