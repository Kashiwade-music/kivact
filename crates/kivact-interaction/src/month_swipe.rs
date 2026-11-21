//! Role: Turn the shown month with a horizontal touch swipe within the date bounds.
use crate::{DragAxis, DragVelocity, project_release};
use kivact_core::custom::{Part, PartState, clock_now, part};
use kivact_core::geometry::Point;
use kivact_core::i18n::{CalendarDate, CalendarMonth};
use kivact_core::state::{IntoValue, Machine, MachineHandle, Signal, Step, Value, machine, memo};
use kivact_core::view::{
    A11yAction, Handled, PointerEvent, PointerId, PointerKind, PointerPhase, ReadingDirection,
    use_direction,
};
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
/// How far the months a swipe turned to enter from.
const ENTER_OFFSET: f64 = 44.0;

/// Where a month swipe is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MonthSwipePhase {
    Idle,
    /// The shown months follow the finger by `shift` px.
    Dragging {
        shift: f64,
    },
    /// A release turned `months` months (0 once they entered); the trailing activation is ignored.
    Released {
        months: i32,
    },
}

impl MonthSwipePhase {
    /// How far the shown months follow the finger now.
    pub fn shift(self) -> f64 {
        match self {
            Self::Dragging { shift } => shift,
            _ => 0.0,
        }
    }
    /// The horizontal offset the newly shown months enter from: the side the
    /// swipe brought them in from, or none.
    pub fn entering(self, rtl: bool) -> f64 {
        match self {
            Self::Released { months } if months != 0 => {
                f64::from(months.signum()) * if rtl { -1.0 } else { 1.0 } * ENTER_OFFSET
            }
            _ => 0.0,
        }
    }
}

#[derive(Clone, PartialEq)]
enum Swipe {
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

enum Event {
    Pointer(PointerEvent),
    IndependentActivation,
    /// The months a swipe turned to are mounted.
    Entered,
}

struct SwipeSnapshot {
    rtl: bool,
    now: u64,
    /// The first shown month is the earliest the bounds allow.
    at_min: bool,
    /// The first shown month is the latest the bounds allow.
    at_max: bool,
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

/// A touch swipe on a calendar that turns its shown month.
#[derive(Clone, Copy)]
pub struct MonthSwipeApi {
    session: MachineHandle<SwipeSnapshot>,
    phase: Value<MonthSwipePhase>,
}

/// A touch swipe that turns `month` within `min` and `max`. Setup only.
#[track_caller]
pub fn use_month_swipe(
    month: Signal<CalendarMonth>,
    min: impl IntoValue<Option<CalendarDate>>,
    max: impl IntoValue<Option<CalendarDate>>,
) -> MonthSwipeApi {
    let (min, max) = (min.into_value(), max.into_value());
    let rtl = use_direction() == ReadingDirection::Rtl;
    let session = machine(
        Swipe::Idle,
        move || {
            let current = month.get_untracked();
            SwipeSnapshot {
                rtl,
                now: clock_now(),
                at_min: min.get().is_some_and(|min| current <= min.month_of()),
                at_max: max.get().is_some_and(|max| current >= max.month_of()),
            }
        },
        move |_, months: i32| {
            // A flick turns as many months as it projects, stopping at the
            // bounds; it never moves the other way.
            let current = month.get_untracked();
            let next = current.add_months(months);
            let (next, moves) = if months > 0 {
                let next = max
                    .get_untracked()
                    .map_or(next, |max| next.min(max.month_of()));
                (next, next > current)
            } else {
                let next = min
                    .get_untracked()
                    .map_or(next, |min| next.max(min.month_of()));
                (next, next < current)
            };
            if moves {
                month.set(next);
            }
        },
    );
    let phase = memo(move || match session.get() {
        Swipe::Dragging { shift, .. } => MonthSwipePhase::Dragging { shift },
        Swipe::Released { months } => MonthSwipePhase::Released { months },
        Swipe::Idle | Swipe::Pending { .. } => MonthSwipePhase::Idle,
    });
    MonthSwipeApi { session, phase }
}

impl MonthSwipeApi {
    /// Where the swipe is.
    pub fn phase(self) -> Value<MonthSwipePhase> {
        self.phase
    }
    /// The months a swipe turned to are mounted.
    pub fn entered(self) {
        self.session.send(Event::Entered);
    }
    /// The calendar's root: it captures the touch while dragging. Setup only.
    #[track_caller]
    pub fn area(self) -> Part<PartState> {
        let input = part(|input| input);
        let session = self.session;
        Part::new(input.state(), move |host| {
            host.part(input).on_pointer(move |event: PointerEvent| {
                session.send(Event::Pointer(event));
                if matches!(session.get(), Swipe::Dragging { .. }) {
                    event.capture();
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
        })
    }
    /// A day cell: its pointer feeds the swipe, and a key or the Activate
    /// action is an independent activation. Setup only.
    #[track_caller]
    pub fn day(self) -> Part<PartState> {
        let input = part(|input| input);
        let session = self.session;
        Part::new(input.state(), move |host| {
            host.part(input)
                .on_pointer(move |event: PointerEvent| {
                    let dragging = matches!(session.get(), Swipe::Dragging { .. });
                    session.send(Event::Pointer(event));
                    if matches!(session.get(), Swipe::Dragging { .. }) {
                        event.capture();
                        Handled::Yes
                    } else if dragging {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
                .on_key(move |_| {
                    session.send(Event::IndependentActivation);
                    Handled::No
                })
                .a11y_action(A11yAction::Activate, move |_| {
                    session.send(Event::IndependentActivation);
                    Handled::No
                })
        })
    }
}
