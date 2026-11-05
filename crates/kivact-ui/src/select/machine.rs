//! Role: Recognize a touch sheet tap without duplicating select surface state.
use kivact_core::geometry::{Point, Rect};
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::{PointerEvent, PointerId, PointerKind, PointerPhase};
use kivact_interaction::DragAxis;
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Outcome {
    Tap,
    Moved,
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum SheetGesture {
    Idle,
    Pressed { pointer: PointerId, origin: Point },
    Moved { pointer: PointerId },
    Released(Outcome),
}
pub(crate) enum Event {
    Pointer(PointerEvent),
    Reset,
}
pub(crate) enum Effect {}
#[derive(Clone, Copy)]
pub(crate) enum Readiness {
    Ready,
    Unavailable,
}
pub(crate) struct Machine {
    pub readiness: Readiness,
    pub bounds: Option<Rect>,
    pub slop: f64,
}
impl MachineTrait for Machine {
    type State = SheetGesture;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &SheetGesture, event: Event) -> Step<SheetGesture, Event, Effect> {
        let Event::Pointer(event) = event else {
            return if matches!(state, SheetGesture::Released(_)) {
                Step::to(SheetGesture::Idle)
            } else {
                Step::stay()
            };
        };
        if matches!(event.phase(), PointerPhase::Down(_)) {
            return if event.kind() == PointerKind::Touch
                && event.is_primary()
                && matches!(self.readiness, Readiness::Ready)
            {
                Step::to(SheetGesture::Pressed {
                    pointer: event.id(),
                    origin: event.window_pos(),
                })
            } else if matches!(state, SheetGesture::Released(_)) {
                Step::to(SheetGesture::Idle)
            } else {
                Step::stay()
            };
        }
        let (pointer, outcome) = match *state {
            SheetGesture::Pressed { pointer, origin } => {
                let pos = event.window_pos();
                let distance = (pos.x - origin.x).hypot(pos.y - origin.y);
                let moved = DragAxis::Horizontal
                    .past_slop(Point::new(distance, 0.0), self.slop)
                    .is_some();
                (pointer, if moved { Outcome::Moved } else { Outcome::Tap })
            }
            SheetGesture::Moved { pointer } => (pointer, Outcome::Moved),
            _ => return Step::stay(),
        };
        if pointer != event.id() {
            return Step::stay();
        }
        match event.phase() {
            PointerPhase::Move if outcome == Outcome::Moved => {
                Step::to(SheetGesture::Moved { pointer })
            }
            PointerPhase::Up(_) => Step::to(
                if self
                    .bounds
                    .is_some_and(|rect| rect.contains(event.window_pos()))
                {
                    SheetGesture::Released(outcome)
                } else {
                    SheetGesture::Idle
                },
            ),
            PointerPhase::Cancel => Step::to(SheetGesture::Idle),
            _ => Step::stay(),
        }
    }
}
