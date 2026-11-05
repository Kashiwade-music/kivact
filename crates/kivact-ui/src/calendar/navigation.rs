//! Role: Month navigation announcements and touch swipe ownership.
use kivact_core::geometry::Point;
use kivact_core::i18n::{CalendarMonth, Locale, NameWidth};
use kivact_core::state::{Machine, Step};
use kivact_core::view::{PointerEvent, PointerId, PointerKind, PointerPhase};
use kivact_interaction::DragAxis;
const MONTH_SWIPE_SLOP: f64 = 6.0;
#[derive(Clone, PartialEq)]
pub(super) enum Swipe {
    Idle,
    /// A completed touch drag suppresses its trailing native activation.
    Released,
    Pending {
        id: PointerId,
        start: Point,
    },
    Dragging {
        id: PointerId,
        start: Point,
        delta: f64,
    },
}
pub(super) enum Event {
    Pointer(PointerEvent),
    IndependentActivation,
}
pub(super) struct SwipeSnapshot {
    pub(super) rtl: bool,
}
impl Machine for SwipeSnapshot {
    type State = Swipe;
    type Event = Event;
    type Effect = i32;
    fn transition(&self, state: &Swipe, event: Event) -> Step<Swipe, Event, i32> {
        let Event::Pointer(event) = event else {
            return Step::to(Swipe::Idle);
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
            (_, PointerPhase::Down(_)) => Step::to(Swipe::Pending {
                id: event.id(),
                start: event.window_pos(),
            }),
            (Swipe::Pending { id, start }, PointerPhase::Move) if *id == event.id() => {
                let delta = Point::new(
                    event.window_pos().x - start.x,
                    event.window_pos().y - start.y,
                );
                if delta.y.abs() > delta.x.abs() && delta.y.abs() > MONTH_SWIPE_SLOP {
                    return Step::to(Swipe::Idle);
                }
                match DragAxis::Horizontal.past_slop(delta, MONTH_SWIPE_SLOP) {
                    Some(delta) => Step::to(Swipe::Dragging {
                        id: *id,
                        start: *start,
                        delta,
                    }),
                    None => Step::stay(),
                }
            }
            (Swipe::Dragging { id, start, .. }, PointerPhase::Move) if *id == event.id() => {
                Step::to(Swipe::Dragging {
                    id: *id,
                    start: *start,
                    delta: event.window_pos().x - start.x,
                })
            }
            (Swipe::Dragging { id, delta, .. }, PointerPhase::Up(_)) if *id == event.id() => {
                Step::to(Swipe::Released).with(if (*delta < 0.0) != self.rtl { 1 } else { -1 })
            }
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
