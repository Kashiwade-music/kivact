//! Role: The element detail and owner-tree methods of the DevTools target.

use std::collections::{HashMap, HashSet};

use kivact_devtools_agent::{Agent, Responder};
use kivact_devtools_protocol as p;
use kivact_engine::api::Reader;
use kivact_runtime::{LiveOwner, OwnerId, Runtime, Turn};
use kivact_types::{CheckedState, FocusTarget, NodeId};

use super::convert;
use super::styles;
use super::tree;
use crate::diagnostics::why::{WhyDep, WhyProp, explain};
use crate::view::devtools_apply;
use crate::view::devtools_state::DevtoolsState;
use crate::view::state::ViewState;
use crate::view::tree::{Handlers, ViewNodes};

/// The call chain of `id`: where it was created and the owners above it.
pub(super) fn call_chain(rt: &Runtime, id: NodeId) -> Vec<p::CallFrame> {
    let tree = ViewState::tree(rt);
    let Some(node) = tree.get(id) else {
        return Vec::new();
    };
    devtools_apply::call_sites(rt, node.owner, node.at)
        .into_iter()
        .map(|at| p::CallFrame {
            location: convert::location(at),
            app_code: convert::app_code(at.file()),
        })
        .collect()
}

/// The interactive states of `id`.
pub(super) fn states(rt: &Runtime, id: NodeId) -> Vec<p::InteractiveState> {
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let semantics = reader.semantics(id);
    let (hovered, pressed) = reader.pointer_state(id);
    let focused = reader
        .window_of(id)
        .and_then(|window| reader.focused(window))
        .is_some_and(|target| target.node == id);
    let mut states = Vec::new();
    if hovered {
        states.push(p::InteractiveState::Hovered);
    }
    if pressed {
        states.push(p::InteractiveState::Pressed);
    }
    if focused {
        states.push(p::InteractiveState::Focused);
    }
    if semantics
        .as_ref()
        .is_some_and(|semantics| semantics.disabled)
    {
        states.push(p::InteractiveState::Disabled);
    }
    if semantics.is_some_and(|semantics| semantics.selected == Some(true)) {
        states.push(p::InteractiveState::Selected);
    }
    states
}

pub(super) fn get_node(
    rt: &Runtime,
    params: p::GetNodeParams,
) -> Result<p::NodeDetail, p::RpcError> {
    let id = NodeId(params.node);
    let node = tree::tree_node(rt, id).ok_or_else(|| node_not_found(params.node))?;
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let bounds = reader
        .box_model(id)
        .map_or_else(empty_box_model, |model| p::BoxModel {
            margin: convert::rect(model.margin),
            border: convert::rect(model.border),
            padding: convert::rect(model.padding),
            content: convert::rect(model.content),
        });
    let semantics = semantics_info(reader.semantics(id).as_ref());
    let handlers = handler_info(tree.get(id).and_then(|node| node.handlers.as_deref()));
    let why = why_props(rt, id);
    Ok(p::NodeDetail {
        node,
        bounds,
        call_chain: call_chain(rt, id),
        styles: styles::layers(rt, id),
        computed: computed(&reader, id),
        semantics,
        handlers,
        states: states(rt, id),
        why,
    })
}

fn empty_box_model() -> p::BoxModel {
    let empty = p::Rect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };
    p::BoxModel {
        margin: empty.clone(),
        border: empty.clone(),
        padding: empty.clone(),
        content: empty,
    }
}

fn node_not_found(raw: p::NodeId) -> p::RpcError {
    p::RpcError::new(
        p::code::NODE_NOT_FOUND,
        format!("node {raw} does not exist"),
    )
}

fn semantics_info(semantics: Option<&kivact_types::Semantics<FocusTarget>>) -> p::SemanticsInfo {
    let Some(semantics) = semantics else {
        return p::SemanticsInfo {
            role: None,
            name: None,
            description: None,
            value: None,
            states: Vec::new(),
            actions: Vec::new(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            focusable: false,
        };
    };
    p::SemanticsInfo {
        role: semantics.role.map(convert::role),
        name: semantics.name.clone(),
        description: semantics.description.clone(),
        value: semantics.value_text.clone(),
        states: semantics_states(semantics),
        actions: semantics
            .actions
            .iter()
            .flatten()
            .map(|action| action_info(*action))
            .collect(),
        labelled_by: semantics
            .labelled_by
            .iter()
            .map(|target| target.node.0)
            .collect(),
        described_by: semantics
            .described_by
            .iter()
            .map(|target| target.node.0)
            .collect(),
        controls: semantics
            .controls
            .iter()
            .map(|target| target.node.0)
            .collect(),
        focusable: semantics.focusable,
    }
}

fn semantics_states(semantics: &kivact_types::Semantics<FocusTarget>) -> Vec<String> {
    let mut states = Vec::new();
    if semantics.disabled {
        states.push("disabled".to_owned());
    }
    if semantics.hidden {
        states.push("hidden".to_owned());
    }
    if semantics.selected == Some(true) {
        states.push("selected".to_owned());
    }
    if semantics.expanded == Some(true) {
        states.push("expanded".to_owned());
    }
    match semantics.checked {
        Some(CheckedState::Checked) => states.push("checked".to_owned()),
        Some(CheckedState::Indeterminate) => states.push("mixed".to_owned()),
        Some(CheckedState::Unchecked) => states.push("unchecked".to_owned()),
        None => {}
    }
    if semantics.required == Some(true) {
        states.push("required".to_owned());
    }
    if semantics.read_only == Some(true) {
        states.push("read_only".to_owned());
    }
    if semantics.invalid == Some(true) {
        states.push("invalid".to_owned());
    }
    if semantics.busy == Some(true) {
        states.push("busy".to_owned());
    }
    states
}

/// The protocol action of a Kivact action.
fn action_info(action: kivact_types::A11yAction) -> p::A11yAction {
    use kivact_types::A11yAction as A;
    match action {
        A::Activate => p::A11yAction::Activate,
        A::Focus => p::A11yAction::Focus,
        A::Increment => p::A11yAction::Increment,
        A::Decrement => p::A11yAction::Decrement,
        A::Expand => p::A11yAction::Expand,
        A::Collapse => p::A11yAction::Collapse,
        A::Toggle => p::A11yAction::Toggle,
        A::Select => p::A11yAction::Select,
        A::ScrollIntoView => p::A11yAction::ScrollIntoView,
        A::SetValue => p::A11yAction::SetValue,
        A::ShowMenu => p::A11yAction::ShowMenu,
        A::Dismiss => p::A11yAction::Dismiss,
    }
}

fn handler_info(handlers: Option<&Handlers>) -> Vec<p::HandlerInfo> {
    let Some(handlers) = handlers else {
        return Vec::new();
    };
    let mut events = Vec::new();
    if handlers.activate.is_some() {
        events.push("activate");
    }
    if !handlers.key.is_empty() {
        events.push("key");
    }
    if !handlers.file.is_empty() {
        events.push("file");
    }
    if !handlers.key_up.is_empty() {
        events.push("key_up");
    }
    if !handlers.pointer.is_empty() {
        events.push("pointer");
    }
    if handlers.secondary_context.is_some() {
        events.push("secondary_context");
    }
    if !handlers.wheel.is_empty() {
        events.push("wheel");
    }
    if !handlers.ime.is_empty() {
        events.push("ime");
    }
    if !handlers.a11y.is_empty() {
        events.push("a11y");
    }
    if handlers.text.is_some() {
        events.push("text");
    }
    if handlers.selection.is_some() {
        events.push("selection");
    }
    if handlers.edit_phase.is_some() {
        events.push("edit_phase");
    }
    if handlers.image_error.is_some() {
        events.push("image_error");
    }
    if handlers.scroll.is_some() {
        events.push("scroll");
    }
    if handlers.layout.is_some() {
        events.push("layout");
    }
    events
        .into_iter()
        .map(|event| p::HandlerInfo {
            event: event.to_owned(),
        })
        .collect()
}

fn why_props(rt: &Runtime, id: NodeId) -> Vec<p::WhyProp> {
    let why = explain(rt, id);
    let i = DevtoolsState::inspector(rt);
    let mut agent = i.agent.borrow_mut();
    why.props
        .iter()
        .map(|prop| why_prop(&mut agent, prop))
        .collect()
}

fn why_prop(agent: &mut Agent, prop: &WhyProp) -> p::WhyProp {
    p::WhyProp {
        prop: prop.prop.clone(),
        effect_at: convert::location(prop.effect_at),
        reads: prop.reads.iter().map(|dep| why_dep(agent, dep)).collect(),
    }
}

fn why_dep(agent: &mut Agent, dep: &WhyDep) -> p::WhyDep {
    let created_at = convert::location(dep.created_at);
    let last_write = dep.last_write;
    p::WhyDep {
        kind: dep.kind.to_owned(),
        ty: dep.ty.to_owned(),
        name_hint: agent.name_hint(&created_at),
        created_at,
        last_write: last_write.map(|(at, _)| convert::location(at)),
        last_write_turn: last_write.and_then(|(_, turn)| match turn {
            Turn::Number(number) => Some(number),
            Turn::Setup => None,
        }),
        reads: dep.reads.iter().map(|read| why_dep(agent, read)).collect(),
    }
}

fn computed(reader: &Reader, id: NodeId) -> Vec<p::StyleProperty> {
    let mut out = Vec::new();
    describe_paint(&reader.effective_paint(id), &mut out);
    describe_text(&reader.computed_text(id), &mut out);
    describe_layout(&reader.layout_props(id), &mut out);
    out
}

fn property(name: &str, value: impl std::fmt::Debug) -> p::StyleProperty {
    p::StyleProperty {
        name: name.to_owned(),
        value: format!("{value:?}"),
        overridden: false,
    }
}

fn describe_paint(paint: &kivact_types::PaintProps, out: &mut Vec<p::StyleProperty>) {
    out.extend([
        property("background", &paint.background),
        property("border", paint.border),
        property("ring", paint.ring),
        property("radius", paint.radius),
        property("shadows", &paint.shadows),
        property("opacity", paint.opacity),
        property("transform", paint.transform),
        property("perspective", paint.perspective),
        property("z_index", paint.z_index),
        property("clip", paint.clip),
        property("cursor", paint.cursor),
        property("effects", &paint.effects),
    ]);
}

fn describe_text(text: &kivact_types::ComputedTextStyle, out: &mut Vec<p::StyleProperty>) {
    out.extend([
        property("font_size", text.font_size),
        property("weight", text.weight),
        property("color", text.color),
        property("line_height", text.line_height),
        property("family", &text.family),
        property("italic", text.italic),
        property("letter_spacing", text.letter_spacing),
        property("word_spacing", text.word_spacing),
        property("align", text.align),
        property("wrap", text.wrap),
        property("decoration", text.decoration),
        property("overflow", text.overflow),
        property("max_lines", text.max_lines),
        property("transform", text.transform),
        property("tabular_nums", text.tabular_nums),
    ]);
}

fn describe_layout(layout: &kivact_types::LayoutProps, out: &mut Vec<p::StyleProperty>) {
    out.extend([
        property("display", layout.display),
        property("direction", layout.direction),
        property("align_items", layout.align_items),
        property("justify", layout.justify),
        property("wrap", layout.wrap),
        property("row_gap", layout.row_gap),
        property("column_gap", layout.column_gap),
        property("padding", layout.padding),
        property("border", layout.border),
        property("margin", layout.margin),
        property("width", layout.width),
        property("height", layout.height),
        property("min_width", layout.min_width),
        property("min_height", layout.min_height),
        property("max_width", layout.max_width),
        property("max_height", layout.max_height),
        property("grow", layout.grow),
        property("shrink", layout.shrink),
        property("basis", layout.basis),
        property("position", layout.position),
        property("inset", layout.inset),
        property("overflow_x", layout.overflow_x),
        property("overflow_y", layout.overflow_y),
        property("hidden", layout.hidden),
        property("visibility", layout.visibility),
        property("aspect_ratio", layout.aspect_ratio),
    ]);
    if let Some(grid) = &layout.grid {
        out.push(property("grid", grid));
    }
}

pub(super) fn get_owners(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::OwnerTree, p::RpcError> {
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let window = convert::window_from(params.window)
        .filter(|window| reader.windows().contains(window))
        .ok_or_else(|| window_not_found(params.window))?;
    let root_node = reader
        .root(window)
        .ok_or_else(|| window_not_found(params.window))?;
    let root_owner = tree
        .get(root_node)
        .map(|node| node.owner.id())
        .ok_or_else(|| window_not_found(params.window))?;
    let mut window_roots = HashSet::new();
    for open in reader.windows() {
        if let Some(root) = reader.root(open)
            && let Some(node) = tree.get(root)
        {
            window_roots.insert(node.owner.id());
        }
    }
    let nodes = owner_nodes(&tree);
    drop(reader);
    drop(tree);
    let snapshot = rt.live_snapshot();
    let (owners, children) = owner_links(&snapshot.owners, root_owner, &window_roots);
    let root = build_owner_tree(root_owner, &children, &owners, &nodes);
    Ok(p::OwnerTree { roots: vec![root] })
}

fn window_not_found(raw: p::WindowId) -> p::RpcError {
    p::RpcError::new(
        p::code::WINDOW_NOT_FOUND,
        format!("window {raw} does not exist"),
    )
}

/// The host nodes each owner created, by owner and ascending node id.
fn owner_nodes(tree: &ViewNodes) -> HashMap<OwnerId, Vec<NodeId>> {
    let mut nodes: HashMap<OwnerId, Vec<NodeId>> = HashMap::new();
    for id in tree.ids() {
        if let Some(node) = tree.get(id) {
            nodes.entry(node.owner.id()).or_default().push(id);
        }
    }
    for list in nodes.values_mut() {
        list.sort_by_key(|id| id.0);
    }
    nodes
}

/// The live owners below `root` and the children of each, ascending by id.
fn owner_links<'a>(
    live: &'a [LiveOwner],
    root: OwnerId,
    window_roots: &HashSet<OwnerId>,
) -> (
    HashMap<OwnerId, &'a LiveOwner>,
    HashMap<OwnerId, Vec<OwnerId>>,
) {
    let mut owners = HashMap::new();
    let mut children: HashMap<OwnerId, Vec<OwnerId>> = HashMap::new();
    for owner in live {
        owners.insert(owner.id, owner);
        if owner.id != root && window_roots.contains(&owner.id) {
            continue;
        }
        if let Some(parent) = owner.parent {
            children.entry(parent).or_default().push(owner.id);
        }
    }
    for list in children.values_mut() {
        list.sort();
    }
    (owners, children)
}

/// Build `root`'s owner tree bottom-up without recursion.
fn build_owner_tree(
    root: OwnerId,
    children: &HashMap<OwnerId, Vec<OwnerId>>,
    owners: &HashMap<OwnerId, &LiveOwner>,
    nodes: &HashMap<OwnerId, Vec<NodeId>>,
) -> p::OwnerNode {
    let mut order = Vec::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        order.push(id);
        if let Some(list) = children.get(&id) {
            stack.extend(list.iter().rev().copied());
        }
    }
    let mut built: HashMap<OwnerId, p::OwnerNode> = HashMap::new();
    for id in order.into_iter().rev() {
        let node = owner_node(id, children, owners, nodes, &built);
        built.insert(id, node);
    }
    built
        .remove(&root)
        .expect("invariant: the root owner was built")
}

fn owner_node(
    id: OwnerId,
    children: &HashMap<OwnerId, Vec<OwnerId>>,
    owners: &HashMap<OwnerId, &LiveOwner>,
    nodes: &HashMap<OwnerId, Vec<NodeId>>,
    built: &HashMap<OwnerId, p::OwnerNode>,
) -> p::OwnerNode {
    let owner = owners.get(&id).copied();
    p::OwnerNode {
        id: id.raw(),
        label: owner.map_or_else(String::new, |owner| owner_label(owner.created_at)),
        location: owner.map(|owner| convert::location(owner.created_at)),
        nodes: nodes
            .get(&id)
            .map(|list| list.iter().map(|node| node.0).collect())
            .unwrap_or_default(),
        children: children
            .get(&id)
            .map(|list| {
                list.iter()
                    .filter_map(|child| built.get(child).cloned())
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn owner_label(at: &'static std::panic::Location<'static>) -> String {
    let file = at.file().rsplit(['/', '\\']).next().unwrap_or(at.file());
    format!("{file}:{}", at.line())
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::GetNode(params) => reply.send(get_node(rt, params).map(p::Reply::GetNode)),
        p::Call::GetOwners(params) => reply.send(get_owners(rt, params).map(p::Reply::GetOwners)),
        other => return Some((other, reply)),
    }
    None
}
