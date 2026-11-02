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
//!   the App ([`keys`]). They exist before any App code runs.
//! - Node ids are never reused, so a stale id in the source never matches a
//!   node mounted later.

use kivact_reactive::Signal;
use kivact_reactive::selector;
use kivact_reactive::{Owner, Runtime};
use kivact_wire::FocusTarget;
use std::panic::Location;

use crate::view::state::{FocusKeys, ViewState};

/// Create the focus source and its selector under the root `owner`.
pub(crate) fn keys(owner: Owner, at: &'static Location<'static>) -> FocusKeys {
    let node: Signal<Option<FocusTarget>> = owner.signal(None::<FocusTarget>, at);
    // `selector` registers under the setup owner.
    let selector = owner.with_setup(|| selector(node));
    let visible = owner.signal(false, at);
    FocusKeys {
        node,
        selector,
        visible,
    }
}

/// The focused node (untracked).
pub(crate) fn focused(rt: &Runtime) -> Option<FocusTarget> {
    let keys = ViewState::of(rt).focus_keys();
    keys.node.try_get_untracked().flatten()
}

/// The host reported that `node` has the keyboard focus now.
pub(crate) fn set_focused(rt: &Runtime, node: Option<FocusTarget>, visible: bool) {
    let keys = ViewState::of(rt).focus_keys();
    // Only the focus source is written here: no owner applies.
    rt.enter(|| {
        keys.node.set(node);
        keys.visible.set(visible);
    });
}
