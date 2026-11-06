//! Role: Month navigation announcements and touch swipe ownership.
use kivact_core::geometry::Point;
use kivact_core::i18n::{CalendarMonth, Locale, NameWidth};
use kivact_core::state::{Machine, Step};
use kivact_core::view::{PointerEvent, PointerId, PointerKind, PointerPhase};
use kivact_interaction::{DragAxis, DragVelocity, project_release};
use std::time::Duration;
const MONTH_SWIPE_SLOP: f64 = 6.0;
/// The horizontal travel a release needs to turn the month at all.
const MONTH_TURN_DISTANCE: f64 = 32.0;
/// How far ahead the release speed projects the travel.
const PROJECTION: Duration = Duration::from_millis(499);
/// The projected travel that turns one more month.
const MONTH_TRAVEL: f64 = 308.0;
/// The most months one flick turns.
const MAX_MONTHS_PER_FLICK: f64 = 3.0;
/// The farthest the shown months follow the finger.
const MAX_SHIFT: f64 = 154.0;
/// The share of the travel shown when the drag points past a date bound.
const EDGE_RESISTANCE: f64 = 0.15;
/// The recent motion the release speed is measured over.
const VELOCITY_WINDOW: Duration = Duration::from_millis(100);
#[derive(Clone, PartialEq)]
pub(super) enum Swipe {
    Idle,
    /// A completed touch drag suppresses its trailing native activation.
    /// `months` is how many months it turned, until the new months enter.
    Released {
        months: i32,
    },
    Pending {
        id: PointerId,
        start: Point,
        velocity: DragVelocity,
    },
    Dragging {
        id: PointerId,
        start: Point,
        delta: f64,
        /// How far the shown months follow the finger.
        shift: f64,
        velocity: DragVelocity,
    },
}
/// How far the months a swipe turned to enter from.
const ENTER_OFFSET: f64 = 44.0;
impl Swipe {
    /// How far the shown months follow the finger now.
    pub(super) fn shift(&self) -> f64 {
        match self {
            Swipe::Dragging { shift, .. } => *shift,
            _ => 0.0,
        }
    }
    /// The horizontal offset the newly shown months enter from: the side the
    /// swipe brought them in from, or none.
    pub(super) fn entering(&self, rtl: bool) -> f64 {
        match self {
            Swipe::Released { months } if *months != 0 => {
                f64::from(months.signum()) * if rtl { -1.0 } else { 1.0 } * ENTER_OFFSET
            }
            _ => 0.0,
        }
    }
}
pub(super) enum Event {
    Pointer(PointerEvent),
    IndependentActivation,
    /// The months a swipe turned to are mounted.
    Entered,
}
// xtask-allow(state-fields): independent per-transition inputs read from the reading direction, the clock and the date bounds.
pub(super) struct SwipeSnapshot {
    pub(super) rtl: bool,
    pub(super) now: u64,
    /// The first shown month is the earliest the bounds allow.
    pub(super) at_min: bool,
    /// The first shown month is the latest the bounds allow.
    pub(super) at_max: bool,
}
impl SwipeSnapshot {
    fn sign(&self) -> f64 {
        if self.rtl { -1.0 } else { 1.0 }
    }
    /// The live offset of the shown months for a drag of `delta`, resisting
    /// a drag toward a month past the bounds.
    fn shift(&self, delta: f64) -> f64 {
        let toward = delta * self.sign();
        let edge = (toward > 0.0 && self.at_min) || (toward < 0.0 && self.at_max);
        delta.clamp(-MAX_SHIFT, MAX_SHIFT) * if edge { EDGE_RESISTANCE } else { 1.0 }
    }
    /// The months a release turns: none below the turn distance, otherwise
    /// one to three by the travel projected from the release speed.
    fn months(&self, delta: f64, speed: f64) -> i32 {
        if delta.abs() < MONTH_TURN_DISTANCE {
            return 0;
        }
        let projected = project_release(delta, speed, PROJECTION) * self.sign();
        let direction = (-delta * self.sign()).signum() as i32;
        let distance = (projected.abs() / MONTH_TRAVEL)
            .round()
            .clamp(1.0, MAX_MONTHS_PER_FLICK) as i32;
        direction * distance
    }
}
impl SwipeSnapshot {
    /// A move of the touch that has not yet chosen an axis.
    fn pending_move(
        &self,
        id: PointerId,
        start: Point,
        velocity: &DragVelocity,
        pos: Point,
    ) -> Step<Swipe, Event, i32> {
        let delta = Point::new(pos.x - start.x, pos.y - start.y);
        if delta.y.abs() > delta.x.abs() && delta.y.abs() > MONTH_SWIPE_SLOP {
            return Step::to(Swipe::Idle);
        }
        let mut velocity = velocity.clone();
        velocity.record(self.now, pos);
        match DragAxis::Horizontal.past_slop(delta, MONTH_SWIPE_SLOP) {
            Some(_) => Step::to(Swipe::Dragging {
                id,
                start,
                delta: delta.x,
                shift: self.shift(delta.x),
                velocity,
            }),
            None => Step::to(Swipe::Pending {
                id,
                start,
                velocity,
            }),
        }
    }
    /// A move or the release of a horizontal swipe.
    fn drag(
        &self,
        id: PointerId,
        start: Point,
        velocity: &DragVelocity,
        event: PointerEvent,
    ) -> Step<Swipe, Event, i32> {
        let pos = event.window_pos();
        let delta = pos.x - start.x;
        let mut velocity = velocity.clone();
        velocity.record(self.now, pos);
        if matches!(event.phase(), PointerPhase::Move) {
            return Step::to(Swipe::Dragging {
                id,
                start,
                delta,
                shift: self.shift(delta),
                velocity,
            });
        }
        let months = self.months(delta, velocity.pixels_per_ms(self.now).x);
        let next = Step::to(Swipe::Released { months });
        if months == 0 { next } else { next.with(months) }
    }
}
impl Machine for SwipeSnapshot {
    type State = Swipe;
    type Event = Event;
    type Effect = i32;
    fn transition(&self, state: &Swipe, event: Event) -> Step<Swipe, Event, i32> {
        let event = match event {
            Event::Pointer(event) => event,
            Event::IndependentActivation => return Step::to(Swipe::Idle),
            Event::Entered => {
                return match state {
                    Swipe::Released { months } if *months != 0 => {
                        Step::to(Swipe::Released { months: 0 })
                    }
                    _ => Step::stay(),
                };
            }
        };
        if event.kind() != PointerKind::Touch {
            return if matches!(event.phase(), PointerPhase::Down(_)) {
                Step::to(Swipe::Idle)
            } else {
                Step::stay()
            };
        }
        if !event.is_primary() {
            return Step::stay();
        }
        match (state, event.phase()) {
            (_, PointerPhase::Down(_)) => {
                let mut velocity = DragVelocity::new(VELOCITY_WINDOW);
                velocity.reset(self.now, event.window_pos());
                Step::to(Swipe::Pending {
                    id: event.id(),
                    start: event.window_pos(),
                    velocity,
                })
            }
            (
                Swipe::Pending {
                    id,
                    start,
                    velocity,
                },
                PointerPhase::Move,
            ) if *id == event.id() => self.pending_move(*id, *start, velocity, event.window_pos()),
            (
                Swipe::Dragging {
                    id,
                    start,
                    velocity,
                    ..
                },
                PointerPhase::Move | PointerPhase::Up(_),
            ) if *id == event.id() => self.drag(*id, *start, velocity, event),
            (_, PointerPhase::Up(_) | PointerPhase::Cancel) => Step::to(Swipe::Idle),
            _ => Step::stay(),
        }
    }
}
pub(super) enum MonthEvent {
    Navigate(CalendarMonth),
    Observe(CalendarMonth),
}
pub(super) struct MonthSnapshot {
    pub(super) locale: Locale,
}
impl Machine for MonthSnapshot {
    type State = String;
    type Event = MonthEvent;
    type Effect = CalendarMonth;
    fn transition(&self, _: &String, event: MonthEvent) -> Step<String, MonthEvent, CalendarMonth> {
        let month = match event {
            MonthEvent::Navigate(month) | MonthEvent::Observe(month) => month,
        };
        let label = format!(
            "{} {}",
            self.locale.month_name(month.month(), NameWidth::Long),
            month.year()
        );
        let next = Step::to(label);
        match event {
            MonthEvent::Navigate(month) => next.with(month),
            MonthEvent::Observe(_) => next,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum InitialFocus {
    Pending,
    Applied,
}
pub(super) struct FocusSnapshot {
    pub(super) requested: bool,
}
impl Machine for FocusSnapshot {
    type State = InitialFocus;
    type Event = kivact_interaction::CalendarGridApi;
    type Effect = kivact_interaction::CalendarGridApi;
    fn transition(
        &self,
        state: &InitialFocus,
        grid: Self::Event,
    ) -> Step<InitialFocus, Self::Event, Self::Effect> {
        match state {
            InitialFocus::Applied => Step::stay(),
            InitialFocus::Pending => {
                let next = Step::to(InitialFocus::Applied);
                if self.requested {
                    next.with(grid)
                } else {
                    next
                }
            }
        }
    }
}
