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
#[derive(Clone, Copy, PartialEq)]
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
        match event {
            Event::Pointer(event) => {
                match gesture(*state, self.readiness, self.bounds, self.slop, event) {
                    Some(next) => Step::to(next),
                    None => Step::stay(),
                }
            }
            Event::Reset if matches!(state, SheetGesture::Released(_)) => {
                Step::to(SheetGesture::Idle)
            }
            Event::Reset => Step::stay(),
        }
    }
}

/// The gesture's next state from one pointer event, or `None` when it stays.
///
/// A press starts a contact only while `readiness` permits it; a move or
/// release follows the pressed contact and releases inside `bounds` record
/// whether the contact was a tap.
pub(crate) fn gesture(
    state: SheetGesture,
    readiness: Readiness,
    bounds: Option<Rect>,
    slop: f64,
    event: PointerEvent,
) -> Option<SheetGesture> {
    if matches!(event.phase(), PointerPhase::Down(_)) {
        return if event.kind() == PointerKind::Touch
            && event.is_primary()
            && matches!(readiness, Readiness::Ready)
        {
            Some(SheetGesture::Pressed {
                pointer: event.id(),
                origin: event.window_pos(),
            })
        } else if matches!(state, SheetGesture::Released(_)) {
            Some(SheetGesture::Idle)
        } else {
            None
        };
    }
    let (pointer, outcome) = match state {
        SheetGesture::Pressed { pointer, origin } => {
            let pos = event.window_pos();
            let distance = (pos.x - origin.x).hypot(pos.y - origin.y);
            let moved = DragAxis::Horizontal
                .past_slop(Point::new(distance, 0.0), slop)
                .is_some();
            (pointer, if moved { Outcome::Moved } else { Outcome::Tap })
        }
        SheetGesture::Moved { pointer } => (pointer, Outcome::Moved),
        _ => return None,
    };
    if pointer != event.id() {
        return None;
    }
    match event.phase() {
        PointerPhase::Move if outcome == Outcome::Moved => Some(SheetGesture::Moved { pointer }),
        PointerPhase::Up(_) => Some(
            if bounds.is_some_and(|rect| rect.contains(event.window_pos())) {
                SheetGesture::Released(outcome)
            } else {
                SheetGesture::Idle
            },
        ),
        PointerPhase::Cancel => Some(SheetGesture::Idle),
        _ => None,
    }
}
