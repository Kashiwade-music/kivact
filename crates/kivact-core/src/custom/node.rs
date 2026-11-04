//! Role: Opaque mounted node identity and component focus queries.

use crate::view::owner::Owner;
use crate::view::state::ViewState;
use crate::view::{NodeRef, focus, requests};
use kivact_engine::api::PropValue;
use kivact_runtime::{AppId, Runtime, require_effectful};
use kivact_types::FocusTarget;
use std::panic::Location;
use std::rc::Rc;

/// The App-scoped identity of a mounted node, including cleanup snapshots.
#[derive(Clone, Copy, Debug)]
pub struct NodeToken {
    pub(crate) app: AppId,
    pub(crate) node: FocusTarget,
    pub(super) visible: bool,
}
impl PartialEq for NodeToken {
    fn eq(&self, other: &Self) -> bool {
        self.app == other.app && self.node == other.node
    }
}
impl Eq for NodeToken {}
impl std::hash::Hash for NodeToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.app, state);
        std::hash::Hash::hash(&self.node, state);
    }
}
impl NodeToken {
    /// Request best-effort focus restoration; explicit focus takes priority.
    #[track_caller]
    pub fn restore_focus(self) {
        if let Some(rt) = Runtime::get(self.app) {
            require_effectful("NodeToken::restore_focus");
            requests::request_restore_focus(&rt, self.node, None, self.visible);
        }
    }
}

impl Owner {
    /// Read the tracked last-known focused node, including removed nodes.
    #[track_caller]
    pub fn focused_node(self) -> Option<NodeToken> {
        let rt = self.runtime();
        let keys = ViewState::of(&rt).focus_keys();
        keys.node.get().map(|node| NodeToken {
            app: self.0.app(),
            node,
            visible: keys.visible.get(),
        })
    }
}

/// Read the running App's focused node without subscribing.
/// Returns `None` outside an App's frame.
#[track_caller]
pub fn focused_node() -> Option<NodeToken> {
    let rt = Runtime::current()?;
    focus::focused(&rt).map(|node| NodeToken {
        app: rt.id(),
        node,
        visible: ViewState::of(&rt).focus_keys().visible.get_untracked(),
    })
}

/// Component-author queries over a node reference.
pub trait NodeRefExt {
    /// Synchronize an application-controlled value without an edit notification.
    ///
    /// Call outside pure computations. Selection is preserved and clamped;
    /// active composition delays synchronization. Application writes may update
    /// disabled or read-only editors. Unbound references and regions do nothing.
    /// The same `Rc` given again changes nothing unless the text was
    /// edited since; another one is compared with the text once.
    #[track_caller]
    fn set_edit_value(&self, value: impl Into<Rc<str>>);

    /// Copy the mounted node's editing state, text included, without
    /// subscribing. Changes made to the copy are recorded: given back with
    /// [`NodeRefExt::set_edit_state`] before the editor's text changes,
    /// only they are applied. Returns `None` for an unbound reference, a
    /// region or a non-editor.
    fn edit_state(&self) -> Option<super::EditState>;
    /// Read the mounted editor's selection and composition without copying
    /// its text or subscribing. Returns `None` like [`NodeRefExt::edit_state`].
    fn edit_session(&self) -> Option<super::TextSession>;
    /// Read the shaped lines `lines` (clamped to the editor's lines) and
    /// their carets in logical window coordinates, after scrolling and
    /// transforms, with the caret and the number of lines. Only those
    /// lines are shaped and read: ask for the lines needed (`0..0` for the
    /// caret alone), not every line of a long document. Returns `None` if
    /// no editor can be shaped.
    fn text_layout(&self, lines: std::ops::Range<usize>) -> Option<super::TextLayout>;
    /// Replace the mounted editor's state outside pure computations.
    ///
    /// Disabled editors reject the update; read-only editors allow selection
    /// changes only. The node's character limit applies to committed text.
    /// Value and composition changes notify edit handlers; caret-only moves do
    /// not. Unbound references and regions have no effect.
    #[track_caller]
    fn set_edit_state(&self, state: super::EditState);
    /// Replace the mounted editor's selection and composition outside
    /// pure computations, without reading or copying its text.
    ///
    /// The session is clamped into the text (to char boundaries).
    /// Disabled editors reject it; read-only editors take the selection
    /// but no composition. A new or ended composition notifies edit
    /// handlers; the session handlers ([`TextEditing::on_selection_change`]) see
    /// every change. Unbound references and regions have no effect.
    ///
    /// [`TextEditing::on_selection_change`]: super::TextEditing::on_selection_change
    #[track_caller]
    fn set_edit_session(&self, session: super::TextSession);

    /// The currently mounted node without subscribing.
    #[track_caller]
    fn mounted_node(&self) -> Option<NodeToken>;
    /// The currently mounted node, subscribing to rebinding.
    /// The caller location identifies misuse in deferred reads.
    #[track_caller]
    fn tracked_node(&self) -> Option<NodeToken>;
    /// Whether two handles identify the same reference, even while unbound.
    #[track_caller]
    fn same_ref(&self, other: NodeRef) -> bool;
    /// Track mount and inherited visibility, including while unmounted.
    #[track_caller]
    fn is_visible(&self) -> bool;
    /// Track whether this mounted node is itself a live region or is inside one.
    /// Unbound references return false.
    #[track_caller]
    fn is_in_live_region(&self) -> bool;
    /// Whether this node itself holds focus, without subscribing.
    #[track_caller]
    fn holds_focus(&self) -> bool;
    /// Whether a strict descendant holds focus, without subscribing.
    #[track_caller]
    fn holds_focus_within(&self) -> bool;
    /// Whether focus was inside a captured panel, including removed focus.
    #[track_caller]
    fn focus_was_in(&self, panel: NodeToken) -> bool;
    /// Restore focus to the first eligible node at or below this reference.
    #[track_caller]
    fn focus_first_inside(&self);
    /// Whether the mounted overlay currently has modal semantics.
    #[track_caller]
    fn is_modal(&self) -> bool;
}

impl NodeRefExt for NodeRef {
    #[track_caller]
    fn set_edit_value(&self, value: impl Into<Rc<str>>) {
        let value = value.into();
        let Some(rt) = Runtime::get(self.app) else {
            return;
        };
        require_effectful("NodeRefExt::set_edit_value");
        let Some(target) = self.mounted_node() else {
            return;
        };
        if target.node.region.is_some() {
            return;
        }
        let changed = ViewState::tree_mut(&rt).update(
            target.node.node,
            PropValue::InputValue(value),
            rt.running_effect(),
        );
        if changed {
            rt.request_turn();
        }
    }

    fn edit_state(&self) -> Option<super::EditState> {
        let target = self.mounted_node()?;
        if target.node.region.is_some() {
            return None;
        }
        let rt = Runtime::get(target.app)?;
        ViewState::tree(&rt).reader().edit_state(target.node.node)
    }
    fn edit_session(&self) -> Option<super::TextSession> {
        let target = self.mounted_node()?;
        if target.node.region.is_some() {
            return None;
        }
        let rt = Runtime::get(target.app)?;
        ViewState::tree(&rt).reader().edit_session(target.node.node)
    }
    fn text_layout(&self, lines: std::ops::Range<usize>) -> Option<super::TextLayout> {
        let target = self.mounted_node()?;
        if target.node.region.is_some() {
            return None;
        }
        let rt = Runtime::get(target.app)?;
        ViewState::tree(&rt)
            .reader()
            .text_layout(target.node.node, lines)
    }
    #[track_caller]
    fn set_edit_state(&self, state: super::EditState) {
        let Some(rt) = Runtime::get(self.app) else {
            return;
        };
        require_effectful("NodeRefExt::set_edit_state");
        let Some(target) = self.mounted_node() else {
            return;
        };
        if target.node.region.is_some() {
            return;
        }
        let changed = ViewState::tree_mut(&rt).update(
            target.node.node,
            PropValue::EditState(state),
            rt.running_effect(),
        );
        if changed {
            rt.request_turn();
        }
    }
    #[track_caller]
    fn set_edit_session(&self, session: super::TextSession) {
        let Some(rt) = Runtime::get(self.app) else {
            return;
        };
        require_effectful("NodeRefExt::set_edit_session");
        let Some(target) = self.mounted_node() else {
            return;
        };
        if target.node.region.is_some() {
            return;
        }
        let changed = ViewState::tree_mut(&rt).update(
            target.node.node,
            PropValue::EditSession(session),
            rt.running_effect(),
        );
        if changed {
            rt.request_turn();
        }
    }

    #[track_caller]
    fn mounted_node(&self) -> Option<NodeToken> {
        self.target().map(|node| NodeToken {
            app: self.app,
            node,
            visible: true,
        })
    }
    #[track_caller]
    fn tracked_node(&self) -> Option<NodeToken> {
        self.target_value(Location::caller()).map(|node| NodeToken {
            app: self.app,
            node,
            visible: true,
        })
    }
    #[track_caller]
    fn same_ref(&self, other: NodeRef) -> bool {
        *self == other
    }
    #[track_caller]
    fn is_visible(&self) -> bool {
        let (rt, _) = self.state(Location::caller());
        let revision = ViewState::tree_mut(&rt).visibility_revision(Location::caller());
        revision.track();
        self.tracked_node()
            .is_some_and(|token| ViewState::tree(&rt).is_visible(token.node.node))
    }
    #[track_caller]
    fn is_in_live_region(&self) -> bool {
        let at = Location::caller();
        let (rt, _) = self.state(at);
        let revision = ViewState::tree_mut(&rt).live_region_revision(at);
        revision.track();
        self.tracked_node()
            .is_some_and(|token| ViewState::tree(&rt).is_in_live_region(token.node.node))
    }
    #[track_caller]
    fn holds_focus(&self) -> bool {
        NodeRef::holds_focus_raw(self)
    }
    #[track_caller]
    fn holds_focus_within(&self) -> bool {
        NodeRef::holds_focus_within_raw(self)
    }
    #[track_caller]
    fn focus_was_in(&self, panel: NodeToken) -> bool {
        if self.app != panel.app {
            super::raise(
                kivact_runtime::ErrorKind::WrongApp,
                "focus panel belongs to another App",
            );
        }
        NodeRef::focus_was_in_raw(self, panel.node.node)
    }
    #[track_caller]
    fn focus_first_inside(&self) {
        if Runtime::get(self.app).is_none() {
            return;
        }
        require_effectful("NodeRefExt::focus_first_inside");
        NodeRef::focus_first_inside_raw(self);
    }
    #[track_caller]
    fn is_modal(&self) -> bool {
        let Some(rt) = Runtime::get(self.app) else {
            return false;
        };
        self.node().is_some_and(|node| {
            ViewState::tree(&rt)
                .overlay(node)
                .is_some_and(|overlay| overlay.modal)
        })
    }
}
