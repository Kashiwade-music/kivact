//! Role: Read the modifier keys held, from a handler that gets no event.

use super::input::Modifiers;
use crate::view::state::ViewState;

/// The modifier keys held, as the host reported them last (with the
/// latest key, pointer or wheel event, or when they changed between
/// events). Untracked: it is a reading at the call, like
/// `KeyboardEvent.getModifierState` in the DOM. Read it in a handler
/// that gets no event (`on_press`, a control's `Binding` write) to see
/// whether Ctrl or Meta is held at that activation.
///
/// Like the reads of `kivact::platform`, it resolves the nearest owner
/// at the call (setup, handler, task or effect); a located `SetupOnly`
/// misuse outside every owner.
///
/// ```ignore
/// button("Open").on_press(move || {
///     let held = modifiers();
///     open_panel(held.ctrl || held.meta);
/// })
/// ```
#[track_caller]
pub fn modifiers() -> kivact_runtime::Value<Modifiers> {
    let rt =
        kivact_runtime::Runtime::current().expect("invariant: modifiers requires a running App");
    ViewState::of(&rt).modifiers()
}
