//! Role: Attach a switch gesture machine and publish its recipe projection.
use super::{
    geometry::Geometry,
    machine::{self, Event, State, SwitchGesture},
};
use crate::values::{ControlSize, SwitchDragState};
use kivact_core::custom::{Part, clock_now};
use kivact_core::state::{Binding, Value, machine, memo};
use kivact_core::style::use_theme;
use kivact_core::view::{Handled, MouseButton, PointerKind, PointerPhase};
use kivact_interaction::ToggleState;
pub(crate) type Visual = Option<Value<Option<SwitchDragState>>>;
pub(crate) fn switch(
    control: Part<ToggleState>,
    value: Binding<bool>,
    size: Value<ControlSize>,
    on_edit: Option<kivact_core::state::Callback<kivact_core::view::EditPhase>>,
) -> (Part<ToggleState>, Visual) {
    if !use_theme()
        .get()
        .ext::<kivact_interaction::InteractionPolicy>()
        .switch
        .drag
        .0
    {
        return (control, None);
    }
    let edit = kivact_interaction::EditSession::new(move |initial| value.set(initial));
    if let Some(observer) = on_edit {
        edit.on_edit(move |phase| observer.call(phase));
    }
    let session = machine(
        State::Gesture(SwitchGesture::Idle),
        move || machine::Machine {
            value: value.get_untracked(),
            geometry: Geometry::new(size.get()),
            now: clock_now(),
        },
        move |_, effect| match effect {
            machine::Effect::Set(next) => {
                edit.once(value.get_untracked(), || value.set(next));
            }
            machine::Effect::Suppress(event) => event.suppress_activation(),
        },
    );
    let state = control.state();
    session.feed(move || state.get().input.disabled, Event::Disabled);
    let visual = memo(move || session.get().visual());
    let control = Part::new(state, move |host| {
        host.part(control).on_pointer(move |event| {
            let touch = event.kind() == PointerKind::Touch;
            if touch && !event.is_primary() {
                return Handled::Yes;
            }
            let before = session.get();
            session.send(Event::Pointer(event));
            let after = session.get();
            let handled = match event.phase() {
                PointerPhase::Down(press) => {
                    touch && event.is_primary() && press.button == MouseButton::Primary
                }
                PointerPhase::Move => after.handles(event),
                PointerPhase::Up(press) if press.button == MouseButton::Primary => {
                    before.handles(event)
                        || (touch && before.tracks(event))
                        || (before.tracks(event) && matches!(after, State::Released { .. }))
                }
                _ => false,
            };
            if handled { Handled::Yes } else { Handled::No }
        })
    });
    (control, Some(visual))
}
