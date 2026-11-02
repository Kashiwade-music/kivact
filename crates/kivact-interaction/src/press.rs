//! Role: The pointer presses that several behaviors read the same way.

use kivact_core::view::{MouseButton, PointerEvent, PointerPhase};

/// Whether `event` is the second press of a primary-button multi-click.
pub(crate) fn is_double_press(event: &PointerEvent) -> bool {
    matches!(
        event.phase,
        PointerPhase::Down(press) if press.button == MouseButton::Primary && press.clicks == 2
    )
}
