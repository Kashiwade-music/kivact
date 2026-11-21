//! Role: Recognise a touch tap that should open a selection surface as a sheet.

use crate::DragAxis;
use kivact_core::custom::{Part, PartState, part};
use kivact_core::geometry::{Point, Rect};
use kivact_core::state::{IntoValue, Machine as MachineTrait, MachineHandle, Step};
use kivact_core::view::{
    Handled, NodeRef, PointerEvent, PointerId, PointerKind, PointerPhase, node_ref,
};

#[derive(Clone, Copy, PartialEq)]
enum Outcome {
    Tap,
    Moved,
}

#[derive(Clone, Copy, PartialEq)]
enum SheetGesture {
    Idle,
    Pressed { pointer: PointerId, origin: Point },
    Moved { pointer: PointerId },
    Released(Outcome),
}

enum Event {
    Pointer(PointerEvent),
    Reset,
}

enum Effect {}

#[derive(Clone, Copy, PartialEq)]
enum Readiness {
    Ready,
    Unavailable,
}

struct Machine {
    readiness: Readiness,
    bounds: Option<Rect>,
    slop: f64,
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
fn gesture(
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

/// Copy handle for a touch tap on a selection trigger or field.
#[derive(Clone, Copy)]
pub struct SheetTapApi {
    gesture: MachineHandle<Machine>,
    target: NodeRef,
}

/// A tap is recognised only while `available`; the slop is the theme's
/// `InteractionPolicy::select.sheet_slop`. Setup only.
#[track_caller]
pub fn use_sheet_tap(available: impl IntoValue<bool>) -> SheetTapApi {
    let available = available.into_value();
    let target = node_ref();
    let bounds = target.rect();
    let theme = kivact_core::style::use_theme();
    let gesture = kivact_core::state::machine(
        SheetGesture::Idle,
        move || Machine {
            readiness: if available.get() {
                Readiness::Ready
            } else {
                Readiness::Unavailable
            },
            bounds: bounds.get(),
            slop: theme
                .get()
                .ext::<crate::InteractionPolicy>()
                .select
                .sheet_slop,
        },
        |_, effect| match effect {},
    );
    SheetTapApi { gesture, target }
}

impl SheetTapApi {
    /// The touched node: a tap must end inside it, and a key pressed on it forgets a tap.
    #[track_caller]
    pub fn target(self) -> Part<PartState> {
        let input = part(|input| input);
        Part::new(input.state(), move |host| {
            host.part(input).node_ref(self.target).on_key(move |_| {
                self.gesture.send(Event::Reset);
                Handled::No
            })
        })
    }

    /// Feed one pointer event of the target, from the caller's own pointer handler.
    pub fn pointer(self, event: PointerEvent) {
        self.gesture.send(Event::Pointer(event));
    }

    /// Whether a touch tap completed inside the target; reading consumes it.
    pub fn take_tap(self) -> bool {
        let tap = matches!(
            self.gesture.state().get_untracked(),
            SheetGesture::Released(Outcome::Tap)
        );
        self.gesture.send(Event::Reset);
        tap
    }
}
