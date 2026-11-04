//! Role: Node references expose exact targets and shared host observations.
//!
//! `NodeRef`: a Copy reference to one mounted node, used for
//! focus, hover, scrolling and geometry. No string keys.

use std::cell::Cell;
use std::panic::Location;
use std::rc::Rc;

use kivact_types::{FocusTarget, NodeId};

use crate::view::state::{Bound, LayoutObservation, NodeRefState, RefState, ViewState};
use kivact_runtime::{
    AppId, ErrorKind, Owned, Owner, Runtime, Signal, UiError, Value, require_effectful,
    require_setup,
};
use kivact_style::draw::Rect;

/// A Copy reference to a node, bound with `.node_ref(r)` exactly once.
/// Equality identifies the same reference handle (App, key and generation),
/// not distinct handles bound to the same mounted target.
#[derive(Clone, Copy, PartialEq)]
pub struct NodeRef {
    pub(crate) app: AppId,
    pub(crate) state: Owned<NodeRefState>,
}

pub(crate) enum RovingMove {
    Previous,
    Next,
    First,
    Last,
}

/// A reference to a node of the current setup owner, bound with
/// `.node_ref(r)` on exactly one view; read its rect, focus it or scroll it
/// into view. Setup only.
///
/// ```ignore
/// let field = node_ref();
/// node().focusable(true).node_ref(field)
/// ```
#[track_caller]
pub fn node_ref() -> NodeRef {
    let at = Location::caller();
    NodeRef::owned_by(Owner::current("node_ref", at), at)
}

impl NodeRef {
    /// A reference stored under `owner`, disposed with it.
    pub(crate) fn owned_by(owner: Owner, at: &'static Location<'static>) -> Self {
        let app = owner.app();
        let bound = owner.signal(RefState::Unbound, at);
        let rect_value = resolved_rect(app, bound);
        let scroll_value = resolved_scroll(app, bound);
        let state = Rc::new(NodeRefState {
            owner,
            bound,
            observes_layout: Cell::new(LayoutObservation::NotRequested),
            rect_value,
            scroll_value,
        });
        let state = owner.store("NodeRef", state, at);
        NodeRef { app, state }
    }
}

impl NodeRef {
    /// The runtime and state of this ref; located `WrongApp` and `Disposed`.
    pub(crate) fn state(&self, at: &'static Location<'static>) -> (Rc<Runtime>, Rc<NodeRefState>) {
        let s = self.state.get_at(at);
        let rt = Runtime::get(self.app).expect("invariant: a resolved handle's App is registered");
        (rt, s)
    }

    pub(crate) fn bind(&self, rt: &Rc<Runtime>, id: NodeId, at: &'static Location<'static>) {
        self.bind_target(rt, Bound::Node(id), None, at);
    }

    /// Region binding cleanup belongs to its current adopted attachment owner.
    pub(crate) fn bind_region(
        &self,
        rt: &Rc<Runtime>,
        id: NodeId,
        region: crate::draw::RegionId,
        owner: Owner,
        hovered: Value<bool>,
        at: &'static Location<'static>,
    ) {
        self.bind_target(
            rt,
            Bound::Region {
                node: id,
                region,
                hovered,
            },
            Some(owner),
            at,
        );
    }

    fn bind_target(
        &self,
        rt: &Rc<Runtime>,
        bound: Bound,
        adoption_owner: Option<Owner>,
        at: &'static Location<'static>,
    ) {
        let id = bound.node();
        let node_owner = ViewState::tree(rt).get(id).map(|node| node.owner);
        if self.app != rt.id() {
            rt.report(
                node_owner,
                UiError::new(
                    ErrorKind::WrongApp,
                    "`.node_ref(r)` got a NodeRef of another App; it was not bound",
                    Some(at),
                ),
            );
            return;
        }
        let Some(state) = self.state.find() else {
            rt.report(
                node_owner,
                UiError::new(
                    ErrorKind::Disposed {
                        created_at: None,
                        disposed_by: None,
                    },
                    "`.node_ref(r)` got a NodeRef whose owner was disposed; it was not bound",
                    Some(at),
                ),
            );
            return;
        };
        if let Some(old) = state.bound.try_get_untracked().and_then(RefState::bound)
            && ViewState::tree(rt).get(old.node()).is_some()
        {
            rt.report(
                Some(state.owner),
                UiError::new(
                    ErrorKind::Message,
                    "NodeRef is already bound to a mounted node",
                    Some(at),
                ),
            );
            return;
        }
        // One host-owned callback pair serves every ref, including references
        // whose item owner will disappear while this canvas survives.
        if ViewState::tree_mut(rt).observe_node(id, at).is_none() {
            return;
        }
        if matches!(state.observes_layout.get(), LayoutObservation::Requested) {
            ViewState::tree_mut(rt).set_observe(id, true);
        }
        ViewState::tree_mut(rt)
            .get_mut(id)
            .expect("invariant: the observed node exists")
            .references
            .push(state.bound);
        state.bound.set(RefState::Bound(bound));
        if let Some(owner) = adoption_owner.or(node_owner) {
            let handle = self.state;
            owner.on_cleanup(move || {
                let Some(state) = handle.find() else {
                    return;
                };
                if state.bound.try_get_untracked().and_then(RefState::bound) != Some(bound) {
                    return;
                }
                state.bound.set(RefState::Detached);
            });
        }
    }

    /// The bound node if it is still mounted.
    pub(crate) fn node(&self) -> Option<NodeId> {
        let s = self.state.find()?;
        let bound = s.bound.try_get_untracked().and_then(RefState::bound)?;
        let rt = Runtime::get(self.app)?;
        ViewState::tree(&rt).get(bound.node()).map(|_| bound.node())
    }

    /// The complete binding while its scene target still exists.
    pub(crate) fn target(&self) -> Option<FocusTarget> {
        let state = self.state.find()?;
        live_target(
            self.app,
            state
                .bound
                .try_get_untracked()
                .and_then(RefState::bound)?
                .target(),
        )
    }

    /// Reactive node id (tracked).
    pub(crate) fn node_value(&self, at: &'static Location<'static>) -> Option<NodeId> {
        self.target_value(at).map(|t| t.node)
    }

    /// The exact live binding, tracking rebinding and region scene changes.
    pub(crate) fn target_value(&self, at: &'static Location<'static>) -> Option<FocusTarget> {
        let (rt, s) = self.state(at);
        let bound = s.bound.get().bound()?;
        if matches!(bound, Bound::Region { .. })
            && let Some(observation) = ViewState::tree(&rt)
                .get(bound.node())
                .and_then(|node| node.observation)
        {
            observation.scene_revision.track();
        }
        drop(rt);
        self.target()
    }

    /// Whether the frontmost hit at a window point belongs to this node or its descendants.
    #[track_caller]
    pub fn hit_test(&self, point: crate::draw::Point) -> bool {
        let (rt, _) = self.state(Location::caller());
        let Some(id) = self.node() else { return false };
        ViewState::tree(&rt).reader().hit_test(id, point)
    }

    /// Select every character in the bound text input.
    #[track_caller]
    pub fn select_all(&self) {
        let (rt, _) = self.state(Location::caller());
        require_effectful("NodeRef::select_all");
        if let Some(id) = self.node() {
            ViewState::tree(&rt).engine().select_all(id);
            rt.request_turn();
        }
    }
    /// The bound node's display lifetime phase, or `Leaving` while unbound.
    /// Setup only. References bound later follow their new node.
    #[track_caller]
    pub fn presence(&self) -> Value<kivact_types::Presence> {
        let at = Location::caller();
        let (_, state) = self.state(at);
        require_setup("NodeRef::presence");
        let binding = state.bound;
        let app = self.app;
        kivact_runtime::memo(move || {
            let Some(bound) = binding.try_get().and_then(RefState::bound) else {
                return kivact_types::Presence::Leaving;
            };
            let source = Runtime::get(app)
                .and_then(|rt| ViewState::tree_mut(&rt).presence(bound.node(), at));
            source
                .and_then(|source| source.try_get())
                .unwrap_or(kivact_types::Presence::Leaving)
        })
    }

    /// Reactive scroll offset of the bound node; zero while unbound.
    #[track_caller]
    pub fn scroll_offset(&self) -> Value<crate::draw::Point> {
        let (rt, state) = self.state(Location::caller());
        drop(rt);
        state.scroll_value
    }
    /// Move the bound scroll container to an absolute logical offset.
    #[track_caller]
    pub fn scroll_to(&self, offset: crate::draw::Point, behavior: ScrollBehavior) {
        self.scroll_request(offset, false, behavior);
    }
    /// Move the bound scroll container by a logical delta.
    #[track_caller]
    pub fn scroll_by(&self, delta: crate::draw::Point, behavior: ScrollBehavior) {
        self.scroll_request(delta, true, behavior);
    }
    #[track_caller]
    fn scroll_request(&self, point: crate::draw::Point, relative: bool, behavior: ScrollBehavior) {
        let (rt, _) = self.state(Location::caller());
        require_effectful("NodeRef scroll");
        if let Some(id) = self.node() {
            ViewState::tree(&rt).engine().scroll(
                id,
                point,
                relative,
                behavior == ScrollBehavior::Smooth,
            );
            rt.request_turn();
        }
    }
    /// Request focus; failure goes to the error sink.
    ///
    /// The focus indicator follows the last input, the way a browser's
    /// `:focus-visible` does for a script's `focus()`: it is visible unless
    /// the last key or pointer press the App got was a pointer press. A
    /// list that moves the focus to the row a click selected shows no
    /// keyboard ring, while the same list driven by arrow keys does (a key
    /// press makes the next focus visible again, a press with the pointer
    /// makes it invisible). Before any input the indicator is visible.
    #[track_caller]
    pub fn focus(&self) {
        let at = Location::caller();
        let (rt, s) = self.state(at);
        require_effectful("NodeRef::focus");
        let owner = s.owner;
        super::requests::request_focus(
            &rt,
            self.target(),
            super::state::FocusWaiter::Report(owner, at),
            !ViewState::of(&rt).pointer_modality(),
        );
    }

    /// Activate the control at or below the bound node as a click on it
    /// would, the way pressing an HTML `<label>` activates its control: the
    /// first node at or below the bound one that can take the focus
    /// (focusable, not disabled, no hidden node on the way; the node itself
    /// first) gets the focus without a visible indicator, then its press
    /// handler runs (`on_press`, the toggle of a check box, the selection of
    /// a tab, ...). A text field has no press handler: it only takes the
    /// focus. Bind the ref to the control itself, or to a wrapper around a
    /// control built elsewhere (a field around an input).
    ///
    /// The press handler runs after the handler that calls this has
    /// returned, in the same turn; the focus follows the rules of
    /// [`Self::restore_focus`]. Nothing happens, and nothing is reported,
    /// when the ref is unbound or disposed, no node at or below it can take
    /// the focus, or the node is removed, disabled or outside the open
    /// modal by the time the press would run.
    ///
    /// ```ignore
    /// let agree = node_ref();
    /// row((
    ///     checkbox("I agree", agreed).hide_label(true).node_ref(agree),
    ///     text("I agree to the terms").on_press(move || agree.activate()),
    /// ))
    /// ```
    #[track_caller]
    pub fn activate(&self) {
        let Some(rt) = Runtime::get(self.app) else {
            return;
        };
        require_effectful("NodeRef::activate");
        let Some(root) = self.node() else {
            return;
        };
        let target = ViewState::tree(&rt).first_focus_target(root);
        if let Some(node) = target {
            super::requests::request_restore_focus(&rt, node, None, false);
            super::requests::request_activation(&rt, node);
        }
    }

    /// Give the focus back to the bound node after closing a part (a
    /// window, a panel, an inline editor), if it may still take it when
    /// the turn's writes are sent. Unlike [`Self::focus`], a target that
    /// went away is no error: nothing happens when the ref is disposed or
    /// unbound, nor when by then its owner is gone, its node was removed,
    /// is hidden (by itself or an ancestor), disabled or not focusable, or
    /// lies outside the modal that stays open. A [`Self::focus`] of the same
    /// turn wins over it. A request the
    /// host refuses all the same goes to the error sink, located here. The
    /// focus indicator follows the last input as for [`Self::focus`].
    #[track_caller]
    pub fn restore_focus(&self) {
        let at = Location::caller();
        let Some(rt) = Runtime::get(self.app) else {
            return;
        };
        require_effectful("NodeRef::restore_focus");
        let Some(state) = self.state.find() else {
            return;
        };
        if let Some(node) = self.target() {
            let visible = !ViewState::of(&rt).pointer_modality();
            super::requests::request_restore_focus(&rt, node, Some((state.owner, at)), visible);
        }
    }

    /// Whether the keyboard focus is on `panel` or below it, or on a node
    /// removed since (the panel's subtree just unmounted, say), as the core
    /// last knows it. Meant for the cleanup of a part whose node `panel` was.
    pub(crate) fn focus_was_in_raw(&self, panel: NodeId) -> bool {
        let Some(rt) = Runtime::get(self.app) else {
            return false;
        };
        let Some(focused) = super::focus::focused(&rt) else {
            return false;
        };
        let tree = ViewState::tree(&rt);
        !tree.contains(focused.node) || tree.is_within(focused.node, panel)
    }

    /// Whether the keyboard focus is on the bound node, as the core last
    /// knows it (untracked).
    pub(crate) fn holds_focus_raw(&self) -> bool {
        let (Some(rt), Some(node)) = (Runtime::get(self.app), self.target()) else {
            return false;
        };
        super::focus::focused(&rt) == Some(node)
    }

    /// Whether the keyboard focus is on a node below the bound one, as the
    /// core last knows it (untracked).
    pub(crate) fn holds_focus_within_raw(&self) -> bool {
        let (Some(rt), Some(node)) = (Runtime::get(self.app), self.node()) else {
            return false;
        };
        let Some(focused) = super::focus::focused(&rt) else {
            return false;
        };
        self.target().is_some_and(|target| {
            target.region.is_none()
                && focused != target
                && ViewState::tree(&rt).is_within(focused.node, node)
        })
    }

    /// Ask for the focus on the first node at or below the bound one that can
    /// take it (focusable, not disabled, no hidden node on the way), when
    /// there is one and it may still take it once the turn's writes are sent.
    /// It is a restore in the sense of [`Self::restore_focus`]: nothing is
    /// reported when there is no such node or it cannot take the focus, and
    /// an explicit [`Self::focus`] of the same turn wins.
    pub(crate) fn focus_first_inside_raw(&self) {
        let Some(rt) = Runtime::get(self.app) else {
            return;
        };
        let Some(root) = self.node() else {
            return;
        };
        let target = ViewState::tree(&rt).first_focus_target(root);
        if let Some(node) = target {
            super::requests::request_restore_focus(&rt, node, None, true);
        }
    }

    pub(crate) fn move_roving_focus(&self, movement: RovingMove) -> bool {
        let (Some(rt), Some(root)) = (Runtime::get(self.app), self.node()) else {
            return false;
        };
        let Some(focus) = super::focus::focused(&rt) else {
            return false;
        };
        let targets = ViewState::tree(&rt).reader().roving_targets(root);
        let Some(index) = targets.iter().position(|target| *target == focus) else {
            return false;
        };
        if targets.len() < 2 {
            return false;
        }
        let next = match movement {
            RovingMove::Previous => (index + targets.len() - 1) % targets.len(),
            RovingMove::Next => (index + 1) % targets.len(),
            RovingMove::First => 0,
            RovingMove::Last => targets.len() - 1,
        };
        super::requests::request_restore_focus(&rt, targets[next], None, true);
        true
    }

    /// Ask the host to scroll ancestors so the node is visible. Nothing
    /// happens while the node is not in the window (unbound, or built but
    /// not mounted yet).
    #[track_caller]
    pub fn scroll_into_view(&self) {
        let at = Location::caller();
        let (rt, _) = self.state(at);
        require_effectful("NodeRef::scroll_into_view");
        if let Some(id) = self.node() {
            ViewState::tree(&rt).engine().scroll_into_view(id);
            rt.request_turn();
        }
    }

    /// Whether the native text layout omitted text with an ellipsis or line
    /// clamp. False for an unbound reference or a node without shaped text.
    #[track_caller]
    pub fn text_truncated(&self) -> Value<bool> {
        let _ = self.rect();
        let app = self.app;
        let binding = self.state(Location::caller()).1.bound;
        kivact_runtime::memo(move || {
            binding
                .try_get()
                .and_then(RefState::bound)
                .filter(|bound| bound.target().region.is_none())
                .and_then(|bound| observation(app, bound.target()))
                .and_then(|observation| observation.text_truncated.try_get())
                .unwrap_or(false)
        })
    }

    /// Window-space rect of the bound node, updated by the host. The first call starts
    /// observing the layout; the value follows the ref when it is bound again to a new node,
    /// and is `None` while it is unbound.
    #[track_caller]
    pub fn rect(&self) -> Value<Option<Rect>> {
        let at = Location::caller();
        let (rt, s) = self.state(at);
        s.observes_layout.set(LayoutObservation::Requested);
        if let Some(bound) = s.bound.try_get_untracked().and_then(RefState::bound) {
            ViewState::tree_mut(&rt).set_observe(bound.node(), true);
        }
        s.rect_value
    }
}

impl NodeRef {
    /// Whether the bound node has the keyboard focus, as a reactive value.
    /// It follows Tab, clicks, `NodeRef::focus` and assistive technology,
    /// and is `false` while the ref is unbound or its node is unmounted. It
    /// also follows a node bound after the call or bound again after the
    /// old node was replaced. A focus move notifies only the readers of the
    /// node that lost and of the node that gained the focus. Setup only (it
    /// creates a memo).
    ///
    /// ```ignore
    /// let field = node_ref();
    /// let focused = field.focused();
    /// column((
    ///     node().name("Name").focusable(true).node_ref(field),
    ///     text(move || if focused.get() { "editing" } else { "" }.to_string()),
    /// ))
    /// ```
    #[track_caller]
    pub fn focused(&self) -> Value<bool> {
        let at = Location::caller();
        let (rt, s) = self.state(at);
        require_setup("NodeRef::focused");
        let keys = ViewState::of(&rt).focus_keys();
        let app = rt.id();
        let node = s.bound;
        drop(rt);
        kivact_runtime::memo(move || {
            let Some(bound) = node.get().bound() else {
                return false;
            };
            let id = bound.target();
            if id.region.is_some()
                && let Some(rt) = Runtime::get(app)
                && let Some(observation) = ViewState::tree(&rt)
                    .get(id.node)
                    .and_then(|node| node.observation)
            {
                observation.scene_revision.track();
            }
            let Some(id) = bound_node(app, node) else {
                return false;
            };
            keys.selector.is(&id)
        })
    }

    /// Whether this exact target has focus that requests a visible indicator.
    #[track_caller]
    pub fn focus_visible(&self) -> Value<bool> {
        let at = Location::caller();
        let (rt, _) = self.state(at);
        let focused = self.focused();
        let keys = ViewState::of(&rt).focus_keys();
        kivact_runtime::memo(move || focused.get() && keys.visible.get())
    }

    /// Whether the keyboard focus is on the bound node or on a node inside
    /// its subtree (overlays mounted inside it included), as a reactive
    /// value; `false` while the ref is unbound or its node is unmounted.
    /// Use it for "close the menu when the focus leaves it".
    ///
    /// Cost: every focus move re-runs each `focus_within` value once and
    /// walks the parent chain of the newly focused node, so it costs
    /// O(tree depth) per focus move per value. It is meant for a few
    /// subscribers (menus, dialogs, comboboxes), not for every row of a
    /// list (use [`NodeRef::focused`] there). Setup only (it creates a
    /// memo).
    #[track_caller]
    pub fn focus_within(&self) -> Value<bool> {
        let at = Location::caller();
        let (rt, s) = self.state(at);
        require_setup("NodeRef::focus_within");
        let keys = ViewState::of(&rt).focus_keys();
        let app = rt.id();
        let node = s.bound;
        drop(rt);
        kivact_runtime::memo(move || {
            let Some(me) = bound_node(app, node) else {
                return false;
            };
            let Some(inner) = keys.node.try_get().flatten() else {
                return false;
            };
            let Some(rt) = Runtime::get(app) else {
                return false;
            };
            if me.region.is_some() {
                inner == me
            } else {
                ViewState::tree(&rt).is_within(inner.node, me.node)
            }
        })
    }

    /// Whether the pointer is over the bound node or a node inside its
    /// subtree (overlays mounted inside it included), as a reactive value,
    /// the same meaning as [`Style::hover`](crate::style::Style::hover)
    /// (CSS `:hover`): the hit the host reports, so a descendant with its own
    /// `on_pointer`, a disabled descendant or a pointer capture elsewhere
    /// does not hide it. It is `false` while the ref is unbound or its
    /// node is unmounted, hidden or outside the open modal, and once the
    /// pointer left the window (also with a button held) or the platform
    /// cancelled it. A ref bound after this call, or bound again to a new
    /// node, is followed too. Use it for "show the tooltip while the
    /// pointer is over the button".
    ///
    /// It follows the pointer's input: a layout change under a pointer
    /// that does not move does not hit-test again. Cost: like
    /// [`NodeRef::focus_within`], every change of the hovered node re-runs
    /// each `hovered` value once and walks the parent chain of the
    /// new hit, O(tree depth) per value, so it is meant for a few
    /// subscribers, not for every row of a list. Setup only (it creates a
    /// memo).
    #[track_caller]
    pub fn hovered(&self) -> Value<bool> {
        let at = Location::caller();
        let (rt, s) = self.state(at);
        require_setup("NodeRef::hovered");
        let hover = crate::view::hover::source(&rt, at);
        let app = rt.id();
        let node = s.bound;
        drop(rt);
        let owner_hover = within(app, node, hover);
        kivact_runtime::memo(move || {
            if !owner_hover.get() {
                return false;
            }
            match node.get().bound() {
                Some(Bound::Region { hovered, .. }) => hovered.get(),
                Some(Bound::Node(_)) => true,
                None => false,
            }
        })
    }
}

/// Read the host-owned record selected by a reference's current binding.
fn observation(app: AppId, target: FocusTarget) -> Option<crate::view::tree::NodeObservation> {
    let rt = Runtime::get(app)?;
    ViewState::tree(&rt).get(target.node)?.observation
}

/// Resolve exact transformed region bounds without allocating during a read.
#[track_caller]
fn resolved_rect(app: AppId, binding: Signal<RefState>) -> Value<Option<Rect>> {
    kivact_runtime::memo(move || {
        let target = binding.try_get().and_then(RefState::bound)?.target();
        let observation = observation(app, target)?;
        let owner_rect = observation.rect.try_get().flatten();
        let Some(region) = target.region else {
            return owner_rect;
        };
        observation.scene_revision.track();
        let rt = Runtime::get(app)?;
        let tree = ViewState::tree(&rt);
        let region = [
            kivact_engine::contract::SceneLayer::Draw,
            kivact_engine::contract::SceneLayer::DrawOver,
        ]
        .into_iter()
        .filter_map(|layer| tree.scene(target.node, layer))
        .find_map(|scene| scene.regions.iter().find(|item| item.id == region).cloned())?;
        let bounds = tree.reader().region_bounds(target.node, region.rect)?;
        Some(bounds)
    })
}

#[track_caller]
fn resolved_scroll(app: AppId, binding: Signal<RefState>) -> Value<crate::draw::Point> {
    kivact_runtime::memo(move || {
        binding
            .try_get()
            .and_then(RefState::bound)
            .and_then(|bound| observation(app, bound.target()))
            .and_then(|observation| observation.scroll.try_get())
            .unwrap_or_default()
    })
}

/// Whether the node `source` holds is the node a ref is bound to (through
/// its node source `node`) or lies in its subtree, as a memo.
fn within(app: AppId, node: Signal<RefState>, source: Signal<Option<NodeId>>) -> Value<bool> {
    kivact_runtime::memo(move || {
        let Some(me) = bound_node(app, node) else {
            return false;
        };
        let Some(inner) = source.try_get().flatten() else {
            return false;
        };
        let Some(rt) = Runtime::get(app) else {
            return false;
        };
        let tree = ViewState::tree(&rt);
        let mut cur = Some(inner);
        while let Some(id) = cur {
            if id == me.node {
                return true;
            }
            cur = tree.get(id).and_then(|n| n.parent);
        }
        false
    })
}

/// The node a ref is bound to (tracked through its node source), if it is
/// still mounted; `None` once the ref or the App is gone.
fn bound_node(app: AppId, node: Signal<RefState>) -> Option<FocusTarget> {
    live_target(app, node.try_get().and_then(RefState::bound)?.target())
}

/// Check exact identity against the currently retained scenes.
fn live_target(app: AppId, target: FocusTarget) -> Option<FocusTarget> {
    let rt = Runtime::get(app)?;
    let tree = ViewState::tree(&rt);
    tree.get(target.node)?;
    if let Some(region) = target.region
        && ![
            kivact_engine::contract::SceneLayer::Draw,
            kivact_engine::contract::SceneLayer::DrawOver,
        ]
        .into_iter()
        .filter_map(|l| tree.scene(target.node, l))
        .any(|s| s.regions.iter().any(|r| r.id == region))
    {
        return None;
    }
    Some(target)
}

/// The node `r` is bound to, as a reactive read (tracked; `None` while
/// unbound or unmounted).
pub(crate) fn node_value(r: &NodeRef, at: &'static Location<'static>) -> Option<NodeId> {
    r.node_value(at)
}

/// How a requested scroll reaches its target.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollBehavior {
    /// Apply immediately.
    #[default]
    Instant,
    /// Animate for 240 ms; user input cancels it.
    Smooth,
}
