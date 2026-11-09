//! Role: Focus management and navigation.
//!
//! Keyboard focus as a reactive source of the view state: the node the host
//! last reported in `FocusChanged`, and a `Selector` over it that
//! `NodeRef::focused` reads.
//!
//! Invariants:
//! - This source is the only copy of the keyboard focus on the core side.
//!   The client writes it inside the turn (`FocusChanged`), and everything
//!   that needs the focus (key dispatch, `NodeRef::{focused,
//!   focus_within}`, the modal focus restore) reads it.
//! - Both nodes are created once, under the root owner, when the host starts
//!   the App ([`FocusKeys`](super::state::FocusKeys)). They exist before any App code runs.
//! - Node ids are never reused, so a stale id in the source never matches a
//!   node mounted later.

use kivact_runtime::Runtime;
use kivact_types::{FocusTarget, WindowId};

use crate::view::state::ViewState;

/// The focused node of `window` (untracked).
pub(crate) fn focused(rt: &Runtime, window: WindowId) -> Option<FocusTarget> {
    let keys = ViewState::of(rt).focus_keys(window)?;
    keys.node.try_get_untracked().flatten()
}

/// The host reported that `node` has the keyboard focus in `window` now.
pub(crate) fn set_focused(
    rt: &Runtime,
    window: WindowId,
    node: Option<FocusTarget>,
    visible: bool,
) {
    let Some(keys) = ViewState::of(rt).focus_keys(window) else {
        return;
    };
    // Only the focus source is written here: no owner applies.
    rt.enter(|| {
        keys.node.set(node);
        keys.visible.set(visible);
    });
}
