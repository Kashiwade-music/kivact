//! Role: Retain what only core knows about a node and write directly to the engine.
//!
//! Engine nodes, including transparent flow boxes, own all topology and props.
//! CoreNodes retains only owner resources, handlers and diagnostic origins.
//! ViewNodes owns the writer and host-wide bookkeeping, borrowing the model
//! only during a Reader or Writer call. Removed user resources are dropped
//! after this service's borrow ends.

use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;

use kivact_engine::api::{Prop, PropValue};
use kivact_engine::api::{Reader, Writer};
use kivact_engine::contract::{NodeKind, NodeProperties, NodeType, SceneLayer};
use kivact_types::{DrawList, IdMap, NodeId, Rect, Size};

use crate::view::handler_layers::EventHandlers;
use crate::view::input::{Handled, KeyEvent, PointerEvent, WheelEvent};
use kivact_runtime::{Handler, NodeId as EffectId, Notifier, Owner, Signal};
use kivact_types::A11yAction;

/// Scroll position report of a scroll container.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScrollState {
    /// Scroll offset of the content.
    pub offset: kivact_types::Point,
    /// Size of the visible area.
    pub viewport: kivact_types::Size,
    pub content: kivact_types::Size,
}

fn classify_scroll(report: ScrollState) -> super::node_status_data::ScrollState {
    let extent = [
        (
            report.offset.x,
            (report.content.width - report.viewport.width).max(0.0),
        ),
        (
            report.offset.y,
            (report.content.height - report.viewport.height).max(0.0),
        ),
    ];
    let axes: Vec<_> = extent.into_iter().filter(|(_, max)| *max > 0.0).collect();
    if axes.is_empty() {
        super::node_status_data::ScrollState::NotScrollable
    } else if axes.iter().all(|(offset, _)| *offset <= 0.0) {
        super::node_status_data::ScrollState::AtStart
    } else if axes.iter().all(|(offset, max)| *offset >= *max) {
        super::node_status_data::ScrollState::AtEnd
    } else {
        super::node_status_data::ScrollState::Middle
    }
}

type SecondaryContextHandler = Handler<PointerEvent>;

// xtask-allow(state-fields): independent optional handlers, one per event kind; absence means no subscription.
/// Event handlers of one host node.
///
/// The key, key release, pointer, wheel and file handlers are one layered
/// list per event kind (`handler_layers`). The others hold one handler: a
/// later registration replaces or composes with the earlier one.
#[derive(Clone, Default)]
pub(crate) struct Handlers {
    /// Default action (click, Enter, Space, `A11yAction::Activate`).
    pub activate: Option<Handler<Option<kivact_style::draw::RegionId>>>,
    /// Key handlers.
    pub key: EventHandlers<KeyEvent>,
    /// OS file drag handler (one: `on_file` replaces it).
    pub file: EventHandlers<crate::view::FileEvent>,
    /// Key release handlers.
    pub key_up: EventHandlers<KeyEvent>,
    /// Pointer handlers.
    pub pointer: EventHandlers<PointerEvent>,
    /// Fallback for an unclaimed secondary button event in this subtree.
    pub secondary_context: Option<SecondaryContextHandler>,
    /// Wheel handlers.
    pub wheel: EventHandlers<WheelEvent>,
    /// The input method's events for the node's IME session.
    pub ime: EventHandlers<kivact_types::ImeInput>,
    /// Assistive-technology actions other than `Activate`, at most one
    /// handler per action (`on_a11y` replaces an earlier one).
    pub a11y: Vec<(A11yAction, Handler<crate::view::A11yEvent>)>,
    /// Text edits of a TextInput.
    pub text: Option<Handler<crate::view::editing_data::TextEditEvent>>,
    /// Session changes of a TextInput (`TextEditing::on_selection_change`).
    pub selection: Option<crate::view::editing_data::SessionHandler>,
    pub edit_phase: Option<Handler<crate::view::EditPhase>>,
    /// Loading or decoding failed for the node's image source.
    pub image_error: Option<Handler<kivact_runtime::UiError>>,
    /// Scroll reports.
    pub scroll: Option<Handler<ScrollState>>,
    /// Layout reports (observed nodes): where the box is drawn (window
    /// bounds, its transforms applied) and its layout size.
    pub layout: Option<Handler<(Rect, Size)>>,
}

impl Handlers {
    pub(crate) fn merge_caller(&mut self, mut later: Self) {
        self.key.merge_caller(later.key);
        self.key_up.merge_caller(later.key_up);
        self.pointer.merge_caller(later.pointer);
        self.wheel.merge_caller(later.wheel);
        self.ime.merge_caller(later.ime);
        if !later.file.is_empty() {
            self.file = later.file;
        }
        macro_rules! replace_fields { ($($field:ident),*) => { $(if later.$field.is_some() { self.$field = later.$field.take(); })* }; }
        replace_fields!(
            activate,
            secondary_context,
            text,
            selection,
            edit_phase,
            image_error
        );
        for (action, handler) in later.a11y {
            if let Some(entry) = self.a11y.iter_mut().find(|(a, _)| *a == action) {
                entry.1 = handler;
            } else {
                self.a11y.push((action, handler));
            }
        }
        if let Some(handler) = later.layout {
            let earlier = self.layout.take();
            self.layout = Some(Handler::from_fn(move |value| {
                if let Some(earlier) = &earlier {
                    earlier.call(value)?;
                }
                handler.call(value)
            }));
        }
        if let Some(handler) = later.scroll {
            let earlier = self.scroll.take();
            self.scroll = Some(Handler::from_fn(move |value| {
                if let Some(earlier) = &earlier {
                    earlier.call(value)?;
                }
                handler.call(value)
            }));
        }
    }
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
        self.layout = Some(Handler::from_fn(move |(bounds, size)| {
            if let Some(previous) = &previous {
                previous.call((bounds, size))?;
            }
            f(bounds, size);
            Ok(Handled::No)
        }));
    }

    /// Add a scroll observer after the ones already there (all run, in registration order).
    pub(crate) fn observe_scroll(&mut self, f: impl Fn(ScrollState) + 'static) {
        let previous = self.scroll.take();
        self.scroll = Some(Handler::from_fn(move |state| {
            if let Some(previous) = &previous {
                previous.call(state)?;
            }
            f(state);
            Ok(Handled::No)
        }));
    }
}

// xtask-allow(state-fields): independent optional action defaults, composed independently by event kind.
/// The activation, text and accessibility-action defaults of the parts
/// attached to a node or a canvas region, composed in attachment order (an
/// earlier part runs first, and its error stops the later ones). These kinds
/// have one handler each, so they are not layered.
#[derive(Clone, Default)]
pub(crate) struct PartActions {
    pub activate: Option<Handler<Option<kivact_style::draw::RegionId>>>,
    pub text: Option<Handler<crate::view::editing_data::TextEditEvent>>,
    pub a11y: Vec<(A11yAction, Handler<crate::view::A11yEvent>)>,
}

impl PartActions {
    /// Compose `later` after what is already here.
    pub(crate) fn compose(&mut self, later: PartActions) {
        if let Some(handler) = later.text {
            let previous = self.text.take();
            self.text = Some(Handler::from_fn(
                move |event: crate::view::editing_data::TextEditEvent| {
                    if let Some(previous) = &previous {
                        previous.call(event.clone())?;
                    }
                    handler.call(event)
                },
            ));
        }
        if let Some(handler) = later.activate {
            let previous = self.activate.take();
            self.activate = Some(Handler::from_fn(move |region| {
                if let Some(previous) = &previous {
                    previous.call(region)?;
                }
                handler.call(region)
            }));
        }
        for (action, handler) in later.a11y {
            if let Some((_, previous)) = self.a11y.iter_mut().find(|(a, _)| *a == action) {
                let before = previous.clone();
                *previous = Handler::from_fn(move |event: crate::view::A11yEvent| {
                    before.call(event.clone())?;
                    handler.call(event)
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

// xtask-allow(state-fields): independent observed engine facts, not a lifecycle phase.
/// Sources shared by all node and region references bound to one host node.
#[derive(Clone, Copy)]
pub(crate) struct NodeObservation {
    pub(crate) rect: Signal<Option<crate::geometry::Rect>>,
    pub(crate) text_truncated: Signal<bool>,
    pub(crate) scroll: Signal<crate::geometry::Point>,
    pub(crate) scroll_state: Signal<super::node_status_data::ScrollState>,
    pub(crate) image_status: Signal<kivact_types::ImageStatus>,
    pub(crate) scene_revision: Notifier,
    pub(crate) editing_revision: Notifier,
}

/// Records [`ViewNodes::remove`] deleted. They hold the nodes' handlers, so
/// user values: the caller drops them after the tree borrow ends, as the
/// runtime drops what a disposal released (G-01, design §4「arena と
/// handle」).
#[must_use]
pub(crate) struct Removed {
    pub(crate) ids: Vec<NodeId>,
    #[allow(
        dead_code,
        reason = "removed user records must drop after the tree borrow"
    )]
    records: Vec<CoreNode>,
}
// xtask-allow(state-fields): independent core resources; each optional subscription or callback is allocated only when requested.
/// One node of the retained tree (host node or fragment): only what the
/// engine cannot hold. Everything the engine knows about a host node (its
/// props, rect, test id, whether its layout is observed) is read through
/// [`ViewNodes::reader`].
pub(crate) struct CoreNode {
    pub(crate) references: Vec<Signal<crate::view::ref_data::RefState>>,
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
    pub(crate) exit_layout: crate::view::ExitLayout,
    pub(crate) exit_complete: Option<kivact_runtime::Handler<()>>,
    /// The graph sources node references read (`ViewNodes::observe_node`).
    pub(crate) observation: Option<NodeObservation>,
    /// Created only when a reference observes this node's lifetime.
    pub(crate) presence: Option<Signal<kivact_types::Presence>>,
    /// The effect that last set each dynamic prop (diagnostics::why).
    pub(crate) origins: Vec<(Prop, EffectId)>,
    pub(crate) semantic_origins: Vec<&'static Location<'static>>,
}

impl CoreNode {
    /// Metadata for a node of `owner`, without handlers or origins yet.
    fn new(owner: Owner) -> CoreNode {
        CoreNode {
            references: Vec::new(),
            owner,
            at: None,
            handlers: None,
            exit_layout: Default::default(),
            exit_complete: None,
            observation: None,
            presence: None,
            origins: Vec::new(),
            semantic_origins: Vec::new(),
        }
    }
}

/// Metadata keyed by the authoritative engine node IDs.
pub(crate) struct CoreNodes(IdMap<CoreNode>);

/// Core metadata and direct access to the engine tree.
pub(crate) struct ViewNodes {
    nodes: CoreNodes,
    next_id: u64,
    /// The root node (`NodeType::Root`), made with the tree; it lives as long as the App.
    pub root: NodeId,
    engine: Writer,
    pub(super) future_text_edits: Rc<std::cell::RefCell<TextEditsInFlight>>,
    leaving: IdMap<Leaving>,
    /// Choice candidate invalidation; only Hidden changes notify it.
    visibility_revision: Notifier,
    live_region_revision: Notifier,
}

impl ViewNodes {
    /// The tree of an App the host started: `engine` writes the engine's
    /// tree, and the root node (`NodeType::Root`, of the root `owner`) is
    /// created in it.
    pub(crate) fn new(engine: Writer, owner: Owner) -> Self {
        let root = NodeId(1);
        let mut nodes = IdMap::default();
        nodes.insert(root, CoreNode::new(owner));
        engine.create(root, NodeKind::new(NodeType::Root));
        ViewNodes {
            nodes: CoreNodes(nodes),
            next_id: 1,
            root,
            engine,
            future_text_edits: Rc::new(std::cell::RefCell::new(
                TextEditsInFlight(IdMap::default()),
            )),
            leaving: IdMap::default(),
            visibility_revision: owner.notifier(Location::caller()),
            live_region_revision: owner.notifier(Location::caller()),
        }
    }

    fn root_node(&self) -> &CoreNode {
        self.nodes
            .0
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
        for node in self.nodes.0.values_mut() {
            node.origins.shrink_to_fit();
        }
        self.nodes.0.shrink_to_fit();
        self.future_text_edits.borrow_mut().0.shrink_to_fit();
    }

    /// A new detached engine node and its owner metadata.
    pub(crate) fn create(&mut self, kind: NodeType, owner: Owner) -> NodeId {
        self.next_id += 1;
        let id = NodeId(self.next_id);
        self.nodes.0.insert(id, CoreNode::new(owner));
        self.engine().create(id, NodeKind::new(kind));
        id
    }

    /// Node `id`, if it exists.
    pub(crate) fn get(&self, id: NodeId) -> Option<&CoreNode> {
        self.nodes.0.get(&id)
    }

    /// Node `id` for change, if it exists.
    pub(crate) fn get_mut(&mut self, id: NodeId) -> Option<&mut CoreNode> {
        self.nodes.0.get_mut(&id)
    }

    /// The handlers of node `id`, created on first use.
    pub(crate) fn handlers_mut(&mut self, id: NodeId) -> Option<&mut Handlers> {
        let n = self.nodes.0.get_mut(&id)?;
        Some(n.handlers.get_or_insert_with(Default::default))
    }

    /// The engine's reader.
    pub(crate) fn reader(&self) -> Reader {
        self.engine.reader()
    }

    /// A value-less visibility notification owned by the App root, for
    /// choice controllers tracking ancestor visibility.
    pub(crate) fn visibility_revision(&self) -> Notifier {
        self.visibility_revision
    }

    /// Track changes to live-region ancestry.
    pub(crate) fn live_region_revision(&self) -> Notifier {
        self.live_region_revision
    }

    /// Create the host-owned record before any reference publishes its binding.
    pub(crate) fn observe_node(
        &mut self,
        id: NodeId,
        at: &'static Location<'static>,
    ) -> Option<NodeObservation> {
        let image_status = self.reader().image_status(id);
        let node = self.nodes.0.get_mut(&id)?;
        if let Some(observation) = node.observation {
            return Some(observation);
        }
        let observation = NodeObservation {
            rect: node.owner.signal(None::<crate::geometry::Rect>, at),
            text_truncated: node.owner.signal(false, at),
            scroll: node.owner.signal(crate::geometry::Point::default(), at),
            scroll_state: node
                .owner
                .signal(super::node_status_data::ScrollState::NotScrollable, at),
            image_status: node.owner.signal(image_status, at),
            scene_revision: node.owner.notifier(at),
            editing_revision: node.owner.notifier(at),
        };
        let handlers = node.handlers.get_or_insert_with(Default::default);
        let rect = observation.rect;
        handlers.observe_layout(move |bounds, _| {
            rect.set(Some(crate::geometry::Rect::new(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            )));
        });
        let scroll = observation.scroll;
        let scroll_state = observation.scroll_state;
        handlers.observe_scroll(move |state| {
            scroll.set(state.offset);
            scroll_state.set(classify_scroll(state));
        });
        node.observation = Some(observation);
        Some(observation)
    }

    /// Allocate one source per observed node, initialized from the engine.
    pub(crate) fn presence(
        &mut self,
        id: NodeId,
        at: &'static Location<'static>,
    ) -> Option<Signal<kivact_types::Presence>> {
        let initial = self.reader().presence(id);
        let node = self.nodes.0.get_mut(&id)?;
        Some(
            *node
                .presence
                .get_or_insert_with(|| node.owner.signal(initial, at)),
        )
    }

    /// Match the host's Hidden inheritance from current engine properties.
    /// Focusable and disabled remain node-local rules.
    pub(crate) fn is_visible(&self, mut id: NodeId) -> bool {
        loop {
            let Some(_) = self.get(id) else {
                return false;
            };
            if self.with_node(id, |node| {
                node.is_some_and(|node| node.input.hidden == Some(true))
            }) {
                return false;
            }
            let Some(parent) = self.reader().parent(id) else {
                return true;
            };
            id = parent;
        }
    }

    fn is_live_region(&self, id: NodeId) -> bool {
        self.with_node(id, |node| {
            let semantics = node.and_then(|node| node.semantics.as_deref());
            semantics.is_some_and(|semantics| {
                matches!(
                    semantics.role,
                    Some(
                        kivact_types::Role::Alert
                            | kivact_types::Role::Status
                            | kivact_types::Role::Log
                    )
                ) || matches!(
                    semantics.live_region,
                    Some(kivact_types::LiveRegion::Polite | kivact_types::LiveRegion::Assertive)
                )
            })
        })
    }

    pub(crate) fn is_in_live_region(&self, mut id: NodeId) -> bool {
        while self.get(id).is_some() {
            if self.is_live_region(id) {
                return true;
            }
            let Some(parent) = self.reader().parent(id) else {
                return false;
            };
            id = parent;
        }
        false
    }

    fn notify_visibility(&self, changed: bool) {
        if changed {
            self.visibility_revision.notify_from_render();
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

    pub(crate) fn overlay(&self, id: NodeId) -> Option<kivact_types::OverlayProps> {
        self.with_node(id, |node| {
            node.and_then(|node| match &node.kind {
                NodeKind::Overlay(data) => *data,
                _ => None,
            })
        })
    }
    pub(crate) fn name(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_deref())
                .and_then(|semantics| semantics.name.clone())
        })
    }
    pub(crate) fn role(&self, id: NodeId) -> Option<kivact_types::Role> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_deref())
                .and_then(|semantics| semantics.role)
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
        let Some(node) = self.nodes.0.get_mut(&id) else {
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
                .borrow()
                .0
                .get(&id)
                .is_some_and(|count| *count > 0)
        {
            return false;
        }
        let hides = prop == Prop::Hidden;
        let was_hidden = hides && self.hidden(id) == Some(true);
        let tracks_live = matches!(prop, Prop::Role | Prop::Semantics);
        let was_live = tracks_live && self.is_live_region(id);
        let changed = self.engine().set(id, value);
        self.notify_visibility(hides && was_hidden != (self.hidden(id) == Some(true)));
        if tracks_live && changed && was_live != self.is_live_region(id) {
            self.live_region_revision.notify_from_render();
        }
        changed
    }

    pub(crate) fn scene(&self, id: NodeId, layer: SceneLayer) -> Option<Arc<DrawList>> {
        self.reader().scene(id, layer)
    }

    pub(crate) fn set_scene(&mut self, id: NodeId, layer: SceneLayer, scene: DrawList) {
        if self.scene(id, layer).as_deref() == Some(&scene) {
            return;
        }
        self.engine().set_scene(id, layer, Arc::new(scene));
        if let Some(observation) = self.get(id).and_then(|node| node.observation) {
            observation.scene_revision.notify_from_render();
        }
    }

    pub(crate) fn set_observe(&mut self, id: NodeId, on: bool) {
        if self.nodes.0.contains_key(&id) {
            self.engine().observe_layout(id, on);
        }
    }

    /// Whether node `id` has a record: it was created and not removed (it may be a
    /// detached list row not inserted yet).
    pub(crate) fn contains(&self, id: NodeId) -> bool {
        self.nodes.0.contains_key(&id)
    }

    /// Remove metadata using the engine's logical subtree removal result.
    #[must_use = "drop the removed records after the tree borrow ends"]
    pub(crate) fn remove(&mut self, id: NodeId) -> Removed {
        if !self.contains(id) {
            return Removed {
                ids: Vec::new(),
                records: Vec::new(),
            };
        }
        let ids = self.remove_engine_subtree(id);
        let mut records = Vec::new();
        for id in &ids {
            self.future_text_edits.borrow_mut().0.remove(id);
            if let Some(record) = self.nodes.0.remove(id) {
                for reference in &record.references {
                    if reference
                        .try_get_untracked()
                        .and_then(crate::view::ref_data::RefState::bound)
                        .is_some_and(|bound| bound.node() == *id)
                    {
                        reference.set(crate::view::ref_data::RefState::Detached);
                    }
                }
                records.push(record);
            }
        }
        Removed { ids, records }
    }

    /// Transparent boxes preserve each top-level child's exit policy. Retained
    /// boxes wait for those child exits in the engine; core retains no links.
    fn remove_engine_subtree(&mut self, id: NodeId) -> Vec<NodeId> {
        let transparent = self.with_node(id, |node| {
            node.is_some_and(|node| {
                node.layout
                    .as_ref()
                    .is_some_and(|layout| layout.display == kivact_types::Display::Contents)
            })
        });
        let children = if transparent {
            self.reader().children(id)
        } else {
            Vec::new()
        };
        let mut descendants = Vec::new();
        for child in children {
            descendants.extend(self.remove_engine_subtree(child));
        }
        let Some(node) = self.nodes.0.get_mut(&id) else {
            return descendants;
        };
        let pop = node.exit_layout == crate::view::ExitLayout::Pop;
        let callback = node.exit_complete.take();
        let notify = callback.is_some();
        if notify {
            self.leaving.insert(id, Leaving { on_exit: callback });
        }
        let removed = self.engine().remove_with_exit(id, pop, notify);
        let mut ids = Vec::with_capacity(descendants.len() + removed.len());
        let mut seen = std::collections::HashSet::new();
        for node in removed
            .first()
            .copied()
            .into_iter()
            .chain(descendants)
            .chain(removed)
        {
            if seen.insert(node) {
                ids.push(node);
            }
        }
        ids
    }

    pub(crate) fn take_exit_callback(&mut self, id: NodeId) -> Option<kivact_runtime::Handler<()>> {
        self.leaving.remove(&id).and_then(|leaving| leaving.on_exit)
    }

    /// Nearest ancestor-or-self host node with handlers matching `f`, not
    /// looking above `stop` (the open modal the search starts in).
    pub(crate) fn find_up<T>(
        &self,
        mut id: NodeId,
        stop: Option<NodeId>,
        f: impl Fn(&CoreNode) -> Option<T>,
    ) -> Option<(NodeId, T)> {
        loop {
            let n = self.nodes.0.get(&id)?;
            if let Some(t) = f(n) {
                return Some((id, t));
            }
            if Some(id) == stop {
                return None;
            }
            id = self.reader().parent(id)?;
        }
    }

    /// Number of nodes (the Testing Library's `Stats`).
    pub(crate) fn len(&self) -> usize {
        self.nodes.0.len()
    }

    /// The IDs of all node records (the test harness's settle check).
    #[cfg(debug_assertions)]
    pub(crate) fn ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.0.keys().copied()
    }
}

impl ViewNodes {
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
            match self.reader().parent(node) {
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
    pub(crate) fn first_focus_target(&self, root: NodeId) -> Option<kivact_types::FocusTarget> {
        if self.overlay_hidden(root) {
            return None;
        }
        if self.is_focusable(root) {
            return Some(kivact_types::FocusTarget {
                node: root,
                region: None,
            });
        }
        let mut regions: Vec<_> = [
            kivact_engine::contract::SceneLayer::Draw,
            kivact_engine::contract::SceneLayer::DrawOver,
        ]
        .into_iter()
        .filter_map(|layer| self.scene(root, layer))
        .flat_map(|scene| scene.regions.clone())
        .collect();
        regions.sort_by_key(|r| r.focus_order);
        if self.disabled(root) != Some(true)
            && let Some(region) = regions
                .iter()
                .find(|r| r.semantics.focusable && !r.semantics.disabled && !r.semantics.hidden)
        {
            return Some(kivact_types::FocusTarget {
                node: root,
                region: Some(region.id),
            });
        }
        self.reader()
            .children(root)
            .iter()
            .find_map(|child| self.first_focus_target(*child))
    }

    /// Focusable (a text input by default) and eligible while disabled.
    fn is_focusable(&self, id: NodeId) -> bool {
        if self.get(id).is_none() {
            return false;
        }
        let text_input = self.with_node(id, |node| node.is_some_and(|node| node.editing.is_some()));
        let retain = self.with_node(id, |node| {
            node.is_some_and(|n| n.input.focusable_when_disabled == Some(true))
        });
        self.focusable(id).unwrap_or(text_input) && (retain || self.disabled(id) != Some(true))
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

struct Leaving {
    on_exit: Option<kivact_runtime::Handler<()>>,
}

pub(super) struct TextEditsInFlight(pub(super) IdMap<u32>);
