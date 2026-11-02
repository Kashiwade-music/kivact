//! Role: Retain what only core knows about a node and write directly to the engine.
//!
//! Fragments belong only to core. Host nodes are created immediately, including
//! detached list subtrees. Properties, scenes, layout rects, the test id and
//! whether layout is observed live only in the engine and are read through
//! its [`Reader`]; owners, handlers, call sites and the diagnostic origin of
//! each prop belong to core.
//!
//! There is one tree: the props and scenes of a node live in the engine's
//! tree only. The core tree ([`TNode`]) holds what the engine cannot: the
//! structure with fragments (the engine links host nodes only), owners,
//! handlers, creation sites and, per dynamic prop, the effect that wrote it
//! (`Prop`, an enum the engine's setter table generates, so core never
//! branches on strings). `Tree::update` takes that effect from
//! `Runtime::running_effect`; no copy of a prop value or of the write sequence is
//! kept.
//!
//! Invariants:
//! - The owned `Writer` and the `Reader` share the engine's `Rc` and borrow
//!   its model only while one of their calls runs. An effect writes in
//!   place with a comparing setter (an equal write records nothing); the
//!   engine's `Changes` and `Model::sync` derive the rest at commit.
//! - The tree exists from the host's start (`ViewState::start`): its writer
//!   and its root node are never missing.
//! - While several native `TextChanged` events wait, core counts only the
//!   undelivered edits per node, so an old controlled value never
//!   overwrites a newer native edit; the batch's counts live in a
//!   `TextEditsAhead` guard (`view::text_edits`).
//! - Ordinary callbacks run after the engine borrow is released. The one
//!   exception is a primitive's measure function, which runs no user
//!   handler and drops no user value (see `crate::primitive`).

use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;

use kivact_engine::bridge::{Prop, PropValue};
use kivact_engine::bridge::{Reader, Writer};
use kivact_wire::{
    IdMap, NodeId, NodeKind, NodeProperties, NodeType, Rect, Scene, SceneLayer, Size,
};

use crate::view::handler_layers::EventHandlers;
use crate::view::input::{KeyEvent, PointerEvent, WheelEvent};
use kivact_reactive::{NodeId as EffectId, Owner, Signal, Trigger, UiError};
use kivact_wire::A11yAction;

/// Scroll position report of a scroll container.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScrollState {
    /// Scroll offset of the content.
    pub offset: kivact_wire::Point,
    /// Size of the visible area.
    pub viewport: kivact_wire::Size,
}

/// Event handlers of one host node.
///
/// The key, key release, pointer, wheel and file handlers are one layered
/// list per event kind (`handler_layers`). The others hold one handler: a
/// later registration replaces or composes with the earlier one.
#[derive(Clone, Default)]
pub(crate) struct Handlers {
    /// Default action (click, Enter, Space, `A11yAction::Activate`).
    pub activate: Option<Rc<dyn Fn(Option<kivact_style::draw::RegionId>) -> Result<(), UiError>>>,
    /// Key handlers.
    pub key: EventHandlers<KeyEvent>,
    /// OS file drag handler (one: `on_file` replaces it).
    pub file: EventHandlers<crate::view::FileEvent>,
    /// Key release handlers.
    pub key_up: EventHandlers<KeyEvent>,
    /// Pointer handlers.
    pub pointer: EventHandlers<PointerEvent>,
    /// Wheel handlers.
    pub wheel: EventHandlers<WheelEvent>,
    /// The input method's events for the node's IME session.
    pub ime: EventHandlers<kivact_wire::ImeEvent>,
    /// Assistive-technology actions other than `Activate`, at most one
    /// handler per action (`on_a11y` replaces an earlier one).
    pub a11y: Vec<(
        A11yAction,
        Rc<dyn Fn(crate::view::A11yEvent) -> Result<(), UiError>>,
    )>,
    /// Text edits of a TextInput.
    pub text: Option<Rc<dyn Fn(crate::view::editing_data::TextEditEvent) -> Result<(), UiError>>>,
    /// Session changes of a TextInput (`TextEditing::on_selection`).
    pub selection: Option<crate::view::editing_data::SessionHandler>,
    /// Scroll reports.
    pub scroll: Option<Rc<dyn Fn(ScrollState)>>,
    /// Layout reports (observed nodes): where the box is drawn (window
    /// bounds, its transforms applied) and its layout size.
    pub layout: Option<Rc<dyn Fn(Rect, Size)>>,
}

impl Handlers {
    /// The activation, text and accessibility-action handlers, moved out so
    /// an attached part can contribute them as defaults.
    pub(crate) fn take_part_actions(&mut self) -> PartActions {
        PartActions {
            activate: self.activate.take(),
            text: self.text.take(),
            a11y: std::mem::take(&mut self.a11y),
        }
    }

    /// The accessibility actions these handlers offer.
    pub(crate) fn actions(&self) -> Vec<A11yAction> {
        offered_actions(self.activate.is_some(), &self.a11y)
    }

    /// Add a layout observer after the ones already there (all run, in registration order).
    pub(crate) fn observe_layout(&mut self, f: impl Fn(Rect, Size) + 'static) {
        let previous = self.layout.take();
        self.layout = Some(Rc::new(move |bounds, size| {
            if let Some(previous) = &previous {
                previous(bounds, size);
            }
            f(bounds, size);
        }));
    }

    /// Add a scroll observer after the ones already there (all run, in registration order).
    pub(crate) fn observe_scroll(&mut self, f: impl Fn(ScrollState) + 'static) {
        let previous = self.scroll.take();
        self.scroll = Some(Rc::new(move |state| {
            if let Some(previous) = &previous {
                previous(state);
            }
            f(state);
        }));
    }
}

/// The activation, text and accessibility-action defaults of the parts
/// attached to a node or a canvas region, composed in attachment order (an
/// earlier part runs first, and its error stops the later ones). These kinds
/// have one handler each, so they are not layered.
#[derive(Clone, Default)]
pub(crate) struct PartActions {
    pub activate: Option<Rc<dyn Fn(Option<kivact_style::draw::RegionId>) -> Result<(), UiError>>>,
    pub text: Option<Rc<dyn Fn(crate::view::editing_data::TextEditEvent) -> Result<(), UiError>>>,
    pub a11y: Vec<(
        A11yAction,
        Rc<dyn Fn(crate::view::A11yEvent) -> Result<(), UiError>>,
    )>,
}

impl PartActions {
    /// Compose `later` after what is already here.
    pub(crate) fn compose(&mut self, later: PartActions) {
        if let Some(handler) = later.text {
            let previous = self.text.take();
            self.text = Some(Rc::new(move |event| {
                if let Some(previous) = &previous {
                    previous(event.clone())?;
                }
                handler(event)
            }));
        }
        if let Some(handler) = later.activate {
            let previous = self.activate.take();
            self.activate = Some(Rc::new(move |region| {
                if let Some(previous) = &previous {
                    previous(region)?;
                }
                handler(region)
            }));
        }
        for (action, handler) in later.a11y {
            if let Some((_, previous)) = self.a11y.iter_mut().find(|(a, _)| *a == action) {
                let before = previous.clone();
                *previous = Rc::new(move |event| {
                    before(event.clone())?;
                    handler(event)
                });
            } else {
                self.a11y.push((action, handler));
            }
        }
    }

    /// The accessibility actions these defaults offer.
    pub(crate) fn actions(&self) -> Vec<A11yAction> {
        offered_actions(self.activate.is_some(), &self.a11y)
    }
}

fn offered_actions<T>(activate: bool, a11y: &[(A11yAction, T)]) -> Vec<A11yAction> {
    let mut actions = Vec::new();
    if activate {
        actions.push(A11yAction::Activate);
    }
    for (action, _) in a11y {
        if !actions.contains(action) {
            actions.push(*action);
        }
    }
    actions
}

/// Sources shared by all node and region references bound to one host node.
#[derive(Clone, Copy)]
pub(crate) struct NodeObservation {
    pub(crate) rect: Signal<Option<crate::draw::Rect>>,
    pub(crate) scroll: Signal<crate::draw::Point>,
    pub(crate) scene_revision: Trigger,
}

/// Records [`Tree::remove`] deleted. They hold the nodes' handlers, so
/// user values: the caller drops them after the tree borrow ends, as the
/// runtime drops what a disposal released (G-01, design §4「arena と
/// handle」).
pub(crate) struct Removed(#[allow(dead_code, reason = "held only to be dropped later")] Vec<TNode>);

/// One node of the retained tree (host node or fragment): only what the
/// engine cannot hold. Everything the engine knows about a host node (its
/// props, rect, test id, whether its layout is observed) is read through
/// [`Tree::reader`].
pub(crate) struct TNode {
    /// `None` for fragments.
    pub kind: Option<NodeType>,
    /// Parent node, if attached. The engine links host nodes only; core
    /// keeps its own links because fragments sit between them: where a
    /// fragment's children go on the host (`top_hosts`, `next_host_after`)
    /// and which subtree a fragment removes depend on them, and detached
    /// subtrees (list rows not yet inserted) are linked here first.
    pub parent: Option<NodeId>,
    /// Child nodes in order, fragments included (see `parent`).
    pub children: Vec<NodeId>,
    /// Owner whose disposal removes the node.
    pub owner: Owner,
    /// Where the builder of a host node was called (`#[track_caller]` of
    /// `node()`, `text(..)`, `button(..)`, ...). `None` for the root, for
    /// fragments, for the wrappers a builder adds (stack layers, list rows)
    /// and for the text of a plain string child; these take the location of
    /// their nearest located ancestor. Snapshots and diagnostics name nodes
    /// by it.
    pub at: Option<&'static Location<'static>>,
    /// Event handlers, if any.
    pub handlers: Option<Box<Handlers>>,
    /// The graph sources node references read (`Tree::observe_node`).
    pub(crate) observation: Option<NodeObservation>,
    /// The effect that last set each dynamic prop (diagnostics::why).
    pub(crate) origins: Vec<(Prop, EffectId)>,
}

impl TNode {
    /// A detached node of `owner` (`None`: a fragment), with no children,
    /// handlers or origins yet.
    fn new(kind: Option<NodeType>, owner: Owner) -> TNode {
        TNode {
            kind,
            parent: None,
            children: Vec::new(),
            owner,
            at: None,
            handlers: None,
            observation: None,
            origins: Vec::new(),
        }
    }

    /// Whether the node is a host node (not a fragment).
    pub(crate) fn is_host(&self) -> bool {
        self.kind.is_some()
    }
}

/// Core metadata and direct access to the engine tree.
pub(crate) struct Tree {
    pub(crate) nodes: IdMap<TNode>,
    next_id: u64,
    /// The root node (`NodeType::Root`), made with the tree; it lives as long as the App.
    pub root: NodeId,
    engine: Writer,
    future_text_edits: IdMap<usize>,
    removes: bool,
    /// Lazily installed for choice candidates; only Hidden changes notify it.
    visibility_revision: Option<Trigger>,
}

impl Tree {
    /// The tree of an App the host started: `engine` writes the engine's
    /// tree, and the root node (`NodeType::Root`, of the root `owner`) is
    /// created in it.
    pub(crate) fn new(engine: Writer, owner: Owner) -> Self {
        let root = NodeId(1);
        let mut nodes = IdMap::default();
        nodes.insert(root, TNode::new(Some(NodeType::Root), owner));
        engine.create(root, NodeKind::new(NodeType::Root));
        Tree {
            nodes,
            next_id: 1,
            root,
            engine,
            future_text_edits: IdMap::default(),
            removes: false,
            visibility_revision: None,
        }
    }

    fn root_node(&self) -> &TNode {
        self.nodes
            .get(&self.root)
            .expect("invariant: the root node lives as long as the App")
    }

    /// The root owner (the owner of the root node).
    pub(crate) fn root_owner(&self) -> Owner {
        self.root_node().owner
    }

    /// Release excess metadata capacity without discarding live nodes,
    /// reactive handles, pending edits or user resources.
    pub(crate) fn release_capacity(&mut self) {
        for node in self.nodes.values_mut() {
            node.children.shrink_to_fit();
            node.origins.shrink_to_fit();
        }
        self.nodes.shrink_to_fit();
        self.future_text_edits.shrink_to_fit();
    }

    /// A new node of `owner` (`None`: a fragment).
    pub(crate) fn create(&mut self, kind: Option<NodeType>, owner: Owner) -> NodeId {
        self.next_id += 1;
        let id = NodeId(self.next_id);
        self.nodes.insert(id, TNode::new(kind, owner));
        if let Some(kind) = kind {
            self.engine().create(id, NodeKind::new(kind));
        }
        id
    }

    /// Node `id`, if it exists.
    pub(crate) fn get(&self, id: NodeId) -> Option<&TNode> {
        self.nodes.get(&id)
    }

    /// Node `id` for change, if it exists.
    pub(crate) fn get_mut(&mut self, id: NodeId) -> Option<&mut TNode> {
        self.nodes.get_mut(&id)
    }

    /// The handlers of node `id`, created on first use.
    pub(crate) fn handlers_mut(&mut self, id: NodeId) -> Option<&mut Handlers> {
        let n = self.nodes.get_mut(&id)?;
        Some(n.handlers.get_or_insert_with(Default::default))
    }

    /// The engine's reader.
    pub(crate) fn reader(&self) -> Reader {
        self.engine.reader()
    }

    /// A value-less visibility notification owned by the App root, created
    /// only when a choice controller needs to track ancestor visibility.
    pub(crate) fn visibility_revision(&mut self, at: &'static Location<'static>) -> Trigger {
        if let Some(revision) = self.visibility_revision {
            return revision;
        }
        let owner = self.root_owner();
        let revision = owner.trigger(at);
        self.visibility_revision = Some(revision);
        revision
    }

    /// Create the host-owned record before any reference publishes its binding.
    pub(crate) fn observe_node(
        &mut self,
        id: NodeId,
        at: &'static Location<'static>,
    ) -> Option<NodeObservation> {
        let node = self.nodes.get_mut(&id)?;
        if let Some(observation) = node.observation {
            return Some(observation);
        }
        let observation = NodeObservation {
            rect: node.owner.signal(None::<crate::draw::Rect>, at),
            scroll: node.owner.signal(crate::draw::Point::default(), at),
            scene_revision: node.owner.trigger(at),
        };
        let handlers = node.handlers.get_or_insert_with(Default::default);
        let rect = observation.rect;
        handlers.observe_layout(move |bounds, _| {
            rect.set(Some(crate::draw::Rect::new(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            )));
        });
        let scroll = observation.scroll;
        handlers.observe_scroll(move |state| scroll.set(state.offset));
        node.observation = Some(observation);
        Some(observation)
    }

    /// Match the host's Hidden inheritance from current engine properties.
    /// Focusable and disabled remain node-local rules.
    pub(crate) fn is_visible(&self, mut id: NodeId) -> bool {
        loop {
            let Some(node) = self.get(id) else {
                return false;
            };
            if self.with_node(id, |node| {
                node.is_some_and(|node| node.input.hidden == Some(true))
            }) {
                return false;
            }
            let Some(parent) = node.parent else {
                return true;
            };
            id = parent;
        }
    }

    fn notify_visibility(&self, changed: bool) {
        if changed && let Some(revision) = self.visibility_revision {
            revision.notify_from_render();
        }
    }

    /// The owned engine writer.
    pub(crate) fn engine(&self) -> &Writer {
        &self.engine
    }

    pub(crate) fn with_node<R>(
        &self,
        id: NodeId,
        f: impl FnOnce(Option<&NodeProperties>) -> R,
    ) -> R {
        self.reader().with_node(id, f)
    }

    pub(crate) fn content(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| match &node.kind {
                NodeKind::Text(data) => data.content.clone(),
                _ => None,
            })
        })
    }
    #[cfg(feature = "test")]
    pub(crate) fn input_value(&self, id: NodeId) -> Option<String> {
        self.reader()
            .edit_state(id)
            .map(kivact_editing::EditState::into_value)
    }
    #[cfg(feature = "test")]
    pub(crate) fn placeholder(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| {
                node.editing
                    .as_ref()
                    .and_then(|data| data.placeholder.clone())
            })
        })
    }
    pub(crate) fn overlay(&self, id: NodeId) -> Option<kivact_wire::OverlayProps> {
        self.with_node(id, |node| {
            node.and_then(|node| match &node.kind {
                NodeKind::Overlay(data) => *data,
                _ => None,
            })
        })
    }
    pub(crate) fn name(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_ref())
                .and_then(|semantics| semantics.name.clone())
        })
    }
    pub(crate) fn role(&self, id: NodeId) -> Option<kivact_wire::Role> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_ref())
                .and_then(|semantics| semantics.role)
        })
    }
    #[cfg(feature = "test")]
    pub(crate) fn value(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_ref())
                .and_then(|semantics| semantics.value.clone())
        })
    }
    #[cfg(feature = "test")]
    pub(crate) fn selected(&self, id: NodeId) -> Option<bool> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_ref())
                .and_then(|semantics| semantics.selected)
        })
    }
    #[cfg(feature = "test")]
    pub(crate) fn checked(&self, id: NodeId) -> Option<kivact_wire::CheckedState> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_ref())
                .and_then(|semantics| semantics.checked)
        })
    }
    #[cfg(feature = "test")]
    pub(crate) fn expanded(&self, id: NodeId) -> Option<bool> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_ref())
                .and_then(|semantics| semantics.expanded)
        })
    }
    pub(crate) fn hidden(&self, id: NodeId) -> Option<bool> {
        self.with_node(id, |node| node.and_then(|node| node.input.hidden))
    }
    pub(crate) fn disabled(&self, id: NodeId) -> Option<bool> {
        self.with_node(id, |node| node.and_then(|node| node.input.disabled))
    }
    pub(crate) fn focusable(&self, id: NodeId) -> Option<bool> {
        self.with_node(id, |node| node.and_then(|node| node.input.focusable))
    }
    /// Write `value` to node `id`, attributing it to `writer`, the
    /// Render/Internal effect running (`Runtime::running_effect`).
    pub(crate) fn update(
        &mut self,
        id: NodeId,
        value: PropValue,
        writer: Option<EffectId>,
    ) -> bool {
        let prop = value.prop();
        let Some(node) = self.nodes.get_mut(&id) else {
            return false;
        };
        if let Some(writer) = writer {
            match node.origins.iter_mut().find(|(set, _)| *set == prop) {
                Some(entry) => entry.1 = writer,
                None => node.origins.push((prop, writer)),
            }
        }
        if prop == Prop::InputValue
            && self
                .future_text_edits
                .get(&id)
                .is_some_and(|count| *count > 0)
        {
            return false;
        }
        let hides = prop == Prop::Hidden;
        let was_hidden = hides && self.hidden(id) == Some(true);
        let changed = self.engine().set(id, value);
        self.notify_visibility(hides && was_hidden != (self.hidden(id) == Some(true)));
        changed
    }

    pub(super) fn count_text_edits(&mut self, ids: impl Iterator<Item = NodeId>) {
        debug_assert!(
            self.future_text_edits.is_empty(),
            "invariant: one batch is delivered at a time"
        );
        for id in ids {
            *self.future_text_edits.entry(id).or_default() += 1;
        }
    }

    pub(super) fn consume_text_edit(&mut self, id: NodeId) {
        if let Some(count) = self.future_text_edits.get_mut(&id) {
            *count -= 1;
            if *count == 0 {
                self.future_text_edits.remove(&id);
            }
        }
    }

    pub(super) fn clear_text_edits(&mut self) {
        self.future_text_edits.clear();
    }
    pub(crate) fn take_removes(&mut self) -> bool {
        std::mem::take(&mut self.removes)
    }

    pub(crate) fn scene(&self, id: NodeId, layer: SceneLayer) -> Option<Arc<Scene>> {
        self.reader().scene(id, layer)
    }

    pub(crate) fn set_scene(&mut self, id: NodeId, layer: SceneLayer, scene: Scene) {
        if self.scene(id, layer).as_deref() == Some(&scene) {
            return;
        }
        self.engine().set_scene(id, layer, Arc::new(scene));
        if let Some(observation) = self.get(id).and_then(|node| node.observation) {
            observation.scene_revision.notify_from_render();
        }
    }

    pub(crate) fn set_observe(&mut self, id: NodeId, on: bool) {
        if self.nodes.contains_key(&id) {
            self.engine().observe_layout(id, on);
        }
    }

    /// The test id of node `id` (`.test_id(..)`), as the engine holds it.
    #[cfg(feature = "test")]
    pub(crate) fn test_id(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_ref())
                .and_then(|semantics| semantics.test_id.clone())
        })
    }

    /// Whether node `id` has a record: it was created and not removed (it may be a
    /// detached list row not inserted yet).
    pub(crate) fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    /// Nearest host ancestor (the node itself excluded).
    pub(crate) fn host_parent(&self, id: NodeId) -> Option<NodeId> {
        let mut cur = self.nodes.get(&id)?.parent;
        while let Some(p) = cur {
            let n = self.nodes.get(&p)?;
            if n.is_host() {
                return Some(p);
            }
            cur = n.parent;
        }
        None
    }

    /// Host nodes at the top of `id`'s subtree (itself if it is a host node).
    pub(crate) fn top_hosts(&self, id: NodeId, out: &mut Vec<NodeId>) {
        let Some(n) = self.nodes.get(&id) else { return };
        if n.is_host() {
            out.push(id);
            return;
        }
        for c in &n.children {
            self.top_hosts(*c, out);
        }
    }

    /// The App's view: the first host node of what the App mounted below
    /// the root (`None` while the App's view has no host node).
    pub(crate) fn app_view(&self) -> Option<NodeId> {
        self.root_node()
            .children
            .iter()
            .find_map(|c| self.first_host(*c))
    }

    fn first_host(&self, id: NodeId) -> Option<NodeId> {
        let n = self.nodes.get(&id)?;
        if n.is_host() {
            return Some(id);
        }
        n.children.iter().find_map(|c| self.first_host(*c))
    }

    /// First host node after `id` within its host parent (insert position).
    pub(crate) fn next_host_after(&self, id: NodeId) -> Option<NodeId> {
        let mut cur = id;
        loop {
            let parent = self.nodes.get(&cur)?.parent?;
            let pn = self.nodes.get(&parent)?;
            // Initial mounting appends children; avoid scanning the entire sibling
            // list for its last child on every append.
            if pn.children.last() == Some(&cur) {
                if pn.is_host() {
                    return None;
                }
                cur = parent;
                continue;
            }
            let pos = pn.children.iter().position(|c| *c == cur)?;
            if let Some(h) = pn.children[pos + 1..]
                .iter()
                .find_map(|c| self.first_host(*c))
            {
                return Some(h);
            }
            if pn.is_host() {
                return None;
            }
            cur = parent;
        }
    }

    /// Link `child` under `parent` at `index` (append if None), then attach
    /// its top host nodes to their nearest host parent.
    pub(crate) fn insert(&mut self, parent: NodeId, child: NodeId, index: Option<usize>) {
        let Some(p) = self.nodes.get_mut(&parent) else {
            return;
        };
        match index {
            Some(i) if i < p.children.len() => p.children.insert(i, child),
            _ => p.children.push(child),
        }
        if let Some(c) = self.nodes.get_mut(&child) {
            c.parent = Some(parent);
        }
        self.attach_writes(child);
    }

    /// Attach the top host nodes of a newly linked subtree.
    pub(crate) fn attach_writes(&mut self, child: NodeId) {
        let before = self.next_host_after(child);
        self.attach_writes_before(child, before);
    }

    /// Like `attach_writes` with a precomputed insertion point (lists).
    pub(crate) fn attach_writes_before(&mut self, child: NodeId, before: Option<NodeId>) {
        let Some(hp) = self.host_parent(child) else {
            return;
        };
        let mut tops = Vec::new();
        self.top_hosts(child, &mut tops);
        for h in tops {
            self.engine().insert(hp, h, before);
        }
    }

    /// Move `child`'s top host nodes before `before`
    /// (at the end when `None`); the tree links are not changed.
    pub(crate) fn move_writes_before(&mut self, child: NodeId, before: Option<NodeId>) {
        let mut tops = Vec::new();
        self.top_hosts(child, &mut tops);
        for h in tops {
            self.engine().move_before(h, before);
        }
    }

    /// Unlink and delete `id`'s subtree and its top engine nodes. The
    /// deleted records come back, to be dropped once the tree is no longer
    /// borrowed: their handlers hold user values whose `Drop` may use the
    /// tree.
    #[must_use = "drop the removed records after the tree borrow ends"]
    pub(crate) fn remove(&mut self, id: NodeId) -> Removed {
        let Some(n) = self.nodes.get(&id) else {
            return Removed(Vec::new());
        };
        let parent = n.parent;
        let mut tops = Vec::new();
        self.top_hosts(id, &mut tops);
        for h in tops {
            self.removes = true;
            self.engine().remove(h);
        }
        if let Some(p) = parent.and_then(|p| self.nodes.get_mut(&p))
            && let Some(pos) = p.children.iter().position(|c| *c == id)
        {
            p.children.remove(pos);
        }
        self.delete_records(id)
    }

    /// Remove a subtree whose parent link the caller rewrites itself (lists):
    /// removes engine nodes and core records without an O(N) unlink. The
    /// deleted records come back, as from [`Tree::remove`].
    #[must_use = "drop the removed records after the tree borrow ends"]
    pub(crate) fn remove_unlinked(&mut self, id: NodeId) -> Removed {
        let mut tops = Vec::new();
        self.top_hosts(id, &mut tops);
        for h in tops {
            self.removes = true;
            self.engine().remove(h);
        }
        self.delete_records(id)
    }

    /// First host node of a subtree (public to lists).
    pub(crate) fn first_host_of(&self, id: NodeId) -> Option<NodeId> {
        self.first_host(id)
    }

    /// Delete core bookkeeping after the engine subtree has been removed.
    fn delete_records(&mut self, id: NodeId) -> Removed {
        let mut removed = Vec::new();
        let mut stack = vec![id];
        while let Some(x) = stack.pop() {
            if let Some(mut n) = self.nodes.remove(&x) {
                self.future_text_edits.remove(&x);
                stack.append(&mut n.children);
                removed.push(n);
            }
        }
        Removed(removed)
    }

    /// Replace a fragment's child order; the caller updates engine attachments.
    pub(crate) fn set_children(&mut self, parent: NodeId, children: Vec<NodeId>) {
        if let Some(p) = self.nodes.get_mut(&parent) {
            p.children = children;
        }
    }

    /// Nearest ancestor-or-self host node with handlers matching `f`, not
    /// looking above `stop` (the open modal the search starts in).
    pub(crate) fn find_up<T>(
        &self,
        mut id: NodeId,
        stop: Option<NodeId>,
        f: impl Fn(&TNode) -> Option<T>,
    ) -> Option<(NodeId, T)> {
        loop {
            let n = self.nodes.get(&id)?;
            if let Some(t) = f(n) {
                return Some((id, t));
            }
            if Some(id) == stop {
                return None;
            }
            id = n.parent?;
        }
    }

    /// Number of nodes (the Testing Library's `Stats`).
    #[cfg(feature = "test")]
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl Tree {
    fn overlay_hidden(&self, id: NodeId) -> bool {
        self.with_node(id, |node| {
            node.is_some_and(|node| {
                node.input.hidden == Some(true)
                    || node.layout.as_ref().is_some_and(|layout| layout.hidden)
            })
        })
    }

    /// Whether `node` is `ancestor` or lies in its subtree.
    pub(crate) fn is_within(&self, mut node: NodeId, ancestor: NodeId) -> bool {
        loop {
            if node == ancestor {
                return true;
            }
            match self.nodes.get(&node).and_then(|n| n.parent) {
                Some(p) => node = p,
                None => return false,
            }
        }
    }

    /// The first node in tree order at or below `root` that is focusable and
    /// not disabled, with no hidden node on the way down from `root`: where an
    /// opened part puts the focus when the caller names no target. It is a
    /// candidate only; the restore request made for it is checked by
    /// `Reader::can_take_focus` when it is sent.
    pub(crate) fn first_focus_target(&self, root: NodeId) -> Option<kivact_wire::FocusTarget> {
        if self.overlay_hidden(root) {
            return None;
        }
        if self.is_focusable(root) {
            return Some(kivact_wire::FocusTarget {
                node: root,
                region: None,
            });
        }
        let mut regions: Vec<_> = [
            kivact_wire::SceneLayer::Draw,
            kivact_wire::SceneLayer::DrawOver,
        ]
        .into_iter()
        .filter_map(|layer| self.scene(root, layer))
        .flat_map(|scene| scene.regions.clone())
        .collect();
        regions.sort_by_key(|r| r.focus_order);
        if self.disabled(root) != Some(true)
            && let Some(region) = regions
                .iter()
                .find(|r| r.focusable && !r.disabled && !r.hidden)
        {
            return Some(kivact_wire::FocusTarget {
                node: root,
                region: Some(region.id),
            });
        }
        self.get(root)?
            .children
            .iter()
            .find_map(|child| self.first_focus_target(*child))
    }

    /// Focusable (a text input by default) and not disabled.
    fn is_focusable(&self, id: NodeId) -> bool {
        if self.get(id).is_none() {
            return false;
        }
        let text_input = self.with_node(id, |node| node.is_some_and(|node| node.editing.is_some()));
        self.focusable(id).unwrap_or(text_input) && self.disabled(id) != Some(true)
    }

    /// The topmost shown modal overlay (the engine's answer) when `id` lies
    /// in it: input that bubbles up from `id` (pointer, wheel and key handler
    /// chains) stops there and never reaches the nodes behind the modal.
    pub(crate) fn modal_boundary(&self, id: NodeId) -> Option<NodeId> {
        self.reader()
            .active_modal()
            .filter(|m| self.is_within(id, *m))
    }
}
