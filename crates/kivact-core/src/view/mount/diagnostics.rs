//! Role: Mounting reports a node that can be pressed but cannot be reached.

use std::panic::Location;

use crate::view::common::Common;
use crate::view::descriptor::Mcx;
use kivact_reactive::{ErrorKind, UiError};

/// A node with `on_press` that is neither a focus stop nor has a role is
/// pressable only with a pointer: the keyboard and assistive technology do
/// not reach it. Reported once, at mount, in a debug build; the node is
/// mounted as it is. Call it before the parts are routed, while `handlers`
/// still holds only the caller's `on_press`.
pub(super) fn report_unreachable_press(
    common: &Common,
    cx: &Mcx<'_>,
    at: &'static Location<'static>,
) {
    if !cfg!(debug_assertions) || common.handlers.activate.is_none() || common.role.is_some() {
        return;
    }
    let focusable = common.focusable == Some(true)
        || common.text_editing.is_some()
        || common
            .semantics
            .as_ref()
            .is_some_and(|semantics| semantics.focusable.is_some())
        || !common.part_constraints.focusable.is_empty();
    if focusable {
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
