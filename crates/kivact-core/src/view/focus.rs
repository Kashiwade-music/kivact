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
//! - Both nodes are created once, under the root owner, when the root is
//!   mounted. Before that the focus reads as `None` and writes are dropped.
//! - Node ids are never reused, so a stale id in the source never matches a
//!   node mounted later.

use std::panic::Location;
use std::rc::Rc;

use kivact_reactive::Signal;
use kivact_reactive::selector;
use kivact_reactive::{Owner, Runtime};
use kivact_wire::FocusTarget;

use crate::view::state::{FocusKeys, ViewState};

/// Create the focus source and its selector under the root `owner`.
pub(crate) fn install(rt: &Rc<Runtime>, owner: Owner, at: &'static Location<'static>) {
    let node: Signal<Option<FocusTarget>> = owner.signal(None::<FocusTarget>, at);
    // `selector` registers under the setup owner.
    let selector = owner.with_setup(|| selector(node));
    let visible = owner.signal(false, at);
    ViewState::of(rt).set_focus_keys(FocusKeys {
        node,
        selector,
        visible,
    });
}

/// The focused node (untracked; `None` before the root is mounted).
pub(crate) fn focused(rt: &Runtime) -> Option<FocusTarget> {
    let keys = ViewState::of(rt).focus_keys()?;
    keys.node.try_get_untracked().flatten()
}

/// The host reported that `node` has the keyboard focus now.
pub(crate) fn set_focused(rt: &Runtime, node: Option<FocusTarget>, visible: bool) {
    let Some(keys) = ViewState::of(rt).focus_keys() else {
        return;
    };
    // Only the focus source is written here: no owner applies.
    rt.enter(|| {
        keys.node.set(node);
        keys.visible.set(visible);
    });
}
