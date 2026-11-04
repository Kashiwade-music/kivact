//! Role: Mounting reports a node that can be pressed but cannot be reached.

use std::panic::Location;

use crate::view::descriptor::Mcx;
use kivact_runtime::{ErrorKind, UiError};

/// A node with `on_press` that is neither a focus stop nor has a role is
/// pressable only with a pointer: the keyboard and assistive technology do
/// not reach it. Call it for a node whose declared modifiers make it pressable with a pointer
/// only. Reported once, at mount, in a debug build; the node is mounted as it is.
pub(super) fn report_unreachable_press(cx: &Mcx<'_>, at: &'static Location<'static>) {
    if !cfg!(debug_assertions) {
        return;
    }
    cx.rt.report(
        Some(cx.owner),
        UiError::new(
            ErrorKind::Unsupported,
            "`.on_press(..)` on a node with neither a role nor keyboard focus can only be              pressed with a pointer; use `button(..)`, or add `.role(..)` and              `.focusable(true)`, or `.role(Role::Generic)` for a deliberate pointer-only              forwarder such as a label",
            Some(at),
        ),
    );
}
