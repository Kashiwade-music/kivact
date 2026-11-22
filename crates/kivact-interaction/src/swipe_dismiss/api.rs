//! Role: The swipe handle: its phase, held state and the parts it drives.

use kivact_core::custom::{Part, PartState, clock_now, part};
use kivact_core::geometry::Rect;
use kivact_core::state::{Callback, IntoHandler, MachineHandle, Value, machine, memo};
use kivact_core::view::{Handled, PointerEvent, PointerPhase};

use super::machine::{self, Event, Swipe};
use super::types::{SwipeDismissPolicy, SwipeInputs, SwipePhase};

#[derive(Clone, Copy)]
pub struct SwipeDismissApi {
    session: MachineHandle<machine::Machine>,
    inputs: SwipeInputs,
    phase: Value<SwipePhase>,
    held: Value<bool>,
}
/// Create a swipe that calls `on_dismiss` once a release qualifies.
/// `rect` is the surface's rectangle. Setup only.
#[track_caller]
pub fn use_swipe_dismiss<M>(
    policy: SwipeDismissPolicy,
    rect: Value<Option<Rect>>,
    on_dismiss: impl IntoHandler<(), M>,
) -> SwipeDismissApi {
    let on_dismiss = Callback::from_handler(on_dismiss.into_handler());
    let session = machine(
        Swipe::Rest,
        move || machine::Machine {
            policy,
            rect: rect.get(),
            now: clock_now(),
        },
        move |_, machine::Effect::Dismiss| on_dismiss.call(()),
    );
    let phase = memo(move || session.get().phase());
    let held = memo(move || !matches!(session.get().phase(), SwipePhase::Rest));
    SwipeDismissApi {
        session,
        inputs: policy.inputs,
        phase,
        held,
    }
}
impl SwipeDismissApi {
    pub fn phase(self) -> Value<SwipePhase> {
        self.phase
    }
    /// True from a press through the drag and a dismissing release (a toast holds its timer).
    pub fn is_held(self) -> Value<bool> {
        self.held
    }
    /// The part the contact presses and moves: the toast card or the drawer's handle. Setup only.
    #[track_caller]
    pub fn control(self) -> Part<PartState> {
        let input = part(|input| input);
        Part::new(input.state(), move |host| {
            host.part(input)
                .on_pointer(move |event: PointerEvent| self.pointer(&event))
        })
    }
    /// A part for a button inside the surface: its press never starts a swipe. Setup only.
    #[track_caller]
    pub fn protect(self) -> Part<PartState> {
        let input = part(|input| input);
        Part::new(input.state(), move |host| {
            host.part(input).on_pointer(move |event: PointerEvent| {
                if matches!(event.phase(), PointerPhase::Down(_)) {
                    self.session.send(Event::Protect(event.id()));
                }
                Handled::No
            })
        })
    }
    fn pointer(self, event: &PointerEvent) -> Handled {
        let before = self.session.state().get_untracked();
        self.session.send(Event::Pointer(*event));
        let after = self.session.state().get_untracked();
        let pos = event.window_pos();
        if !self.inputs.accepts(event.kind())
            || !pos.x.is_finite()
            || !pos.y.is_finite()
            || matches!(before, Swipe::Released { .. })
        {
            return Handled::No;
        }
        let mine = |swipe: &Swipe| matches!(swipe, Swipe::Dragging { contact, .. } if contact.pointer == event.id());
        match event.phase() {
            PointerPhase::Down(_) => {
                if matches!(&after, Swipe::Pressed(contact) if contact.pointer == event.id()) {
                    event.capture();
                }
                Handled::No
            }
            PointerPhase::Move if mine(&after) => Handled::Yes,
            PointerPhase::Up(_) if mine(&before) => Handled::Yes,
            _ => Handled::No,
        }
    }
}
