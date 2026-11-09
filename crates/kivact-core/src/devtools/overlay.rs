//! Role: Highlight, overlays, pick mode and screenshots of the DevTools target.

use std::collections::BTreeSet;
use std::panic::Location;

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_engine::api::Reader;
use kivact_engine::contract::{
    DevtoolsOverlay, EngineRecords, OverlayBox, OverlayFill, OverlayLabel, OverlayOutline,
    PickPhase,
};
use kivact_runtime::Runtime;
use kivact_types::layout::{Display, FlexDirection};
use kivact_types::{NodeId, Point, Rect, Rgba, WindowId};

use super::convert;
use super::lifecycle;
use crate::view::devtools_state::{DevtoolsState, Highlight, Inspector, overlays_off};
use crate::view::state::ViewState;

/// The layout outlines and gaps of the layout overlay.
const LAYOUT_COLOR: u32 = 0x8e7cc3;
/// The boxes of the nodes the last turn redrew.
const REDRAW_COLOR: u32 = 0xff5a5a;
/// The boxes of the nodes the last turn changed.
const HEAT_COLOR: u32 = 0xffa94d;
/// The boxes of the layout findings.
const LINT_COLOR: u32 = 0xe5484d;

pub(super) fn highlight(rt: &Runtime, params: p::HighlightParams) -> Result<p::Empty, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    match params.node {
        Some(raw) => {
            let id = NodeId(raw);
            if ViewState::tree(rt).get(id).is_none() {
                return Err(node_not_found(raw));
            }
            i.session.highlight.set(Some(Highlight {
                node: id,
                layout: params.layout,
            }));
        }
        None => i.session.highlight.set(None),
    }
    Ok(p::Empty {})
}

pub(super) fn set_overlays(
    rt: &Runtime,
    params: p::OverlaySettings,
) -> Result<p::Empty, p::RpcError> {
    *DevtoolsState::inspector(rt).session.overlays.borrow_mut() = params;
    lifecycle::apply_recording(rt);
    Ok(p::Empty {})
}

pub(super) fn get_overlays(rt: &Runtime, _: p::Empty) -> Result<p::OverlaySettings, p::RpcError> {
    Ok(DevtoolsState::inspector(rt)
        .session
        .overlays
        .borrow()
        .clone())
}

pub(super) fn start_pick(
    rt: &Runtime,
    params: p::StartPickParams,
) -> Result<p::Empty, p::RpcError> {
    let window = convert::window_from(params.window)
        .filter(|window| ViewState::tree(rt).reader().windows().contains(window))
        .ok_or_else(|| window_not_found(params.window))?;
    start_pick_in(rt, window);
    Ok(p::Empty {})
}

pub(super) fn stop_pick(rt: &Runtime, _: p::Empty) -> Result<p::Empty, p::RpcError> {
    DevtoolsState::inspector(rt).session.pick.set(None);
    ViewState::tree(rt).engine().set_pick(None);
    Ok(p::Empty {})
}

pub(super) fn screenshot(rt: &Runtime, params: p::ScreenshotParams, reply: Responder) {
    let node = match screenshot_node(rt, &params) {
        Ok(node) => node,
        Err(error) => {
            reply.send(Err(error));
            return;
        }
    };
    let snapshot = crate::view::snapshots::request(rt, node, rt.root(), Location::caller());
    kivact_runtime::spawn(async move {
        let image = match snapshot.await {
            Ok(pixels) => {
                match kivact_devtools_agent::encode_png(pixels.width, pixels.height, &pixels.rgba) {
                    Ok(bytes) => Ok(p::Image {
                        png_base64: kivact_devtools_agent::base64(&bytes),
                        width: pixels.width,
                        height: pixels.height,
                    }),
                    Err(error) => Err(p::RpcError::new(
                        p::code::INTERNAL_ERROR,
                        format!("encode the screenshot: {error}"),
                    )),
                }
            }
            Err(error) => Err(snapshot_error(&error)),
        };
        reply.send(image.map(p::Reply::Screenshot));
    });
}

fn screenshot_node(rt: &Runtime, params: &p::ScreenshotParams) -> Result<NodeId, p::RpcError> {
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let window = convert::window_from(params.window)
        .filter(|window| reader.windows().contains(window))
        .ok_or_else(|| window_not_found(params.window))?;
    match params.node {
        Some(raw) => {
            let id = NodeId(raw);
            if tree.get(id).is_none() {
                return Err(node_not_found(raw));
            }
            Ok(id)
        }
        None => reader
            .root(window)
            .ok_or_else(|| window_not_found(params.window)),
    }
}

fn snapshot_error(error: &kivact_runtime::UiError) -> p::RpcError {
    let code = match error.kind() {
        kivact_runtime::ErrorKind::Unsupported => p::code::UNSUPPORTED,
        _ => p::code::INTERNAL_ERROR,
    };
    p::RpcError::new(code, error.message().to_owned())
}

/// Report one pick event to the DevTools and the App.
pub(super) fn pick_event(rt: &Runtime, window: WindowId, node: Option<NodeId>, phase: PickPhase) {
    let i = DevtoolsState::inspector(rt);
    if phase == PickPhase::Hover
        && let Some(node) = node
    {
        let layout = i
            .session
            .highlight
            .get()
            .is_some_and(|highlight| highlight.layout);
        i.session.highlight.set(Some(Highlight { node, layout }));
    }
    if i.wants(p::Topic::Pick) {
        i.notify(p::Notification::Pick(p::PickEvent {
            window: convert::window(window),
            node: node.map(|node| node.0),
            phase: match phase {
                PickPhase::Hover => p::PickPhase::Hover,
                PickPhase::Select => p::PickPhase::Select,
                PickPhase::Cancel => p::PickPhase::Cancel,
            },
        }));
    }
    if phase != PickPhase::Hover {
        i.session.pick.set(None);
        ViewState::tree(rt).engine().set_pick(None);
    }
}

/// Begin pick mode in `window`.
pub(super) fn start_pick_in(rt: &Runtime, window: WindowId) {
    let i = DevtoolsState::inspector(rt);
    i.session.pick.set(Some(window));
    ViewState::tree(rt).engine().set_pick(Some(window));
}

/// Rebuild the overlays of every window from the current settings.
pub(super) fn after_turn(rt: &Runtime, records: &EngineRecords) {
    let i = DevtoolsState::inspector(rt);
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let scope = paint_scope(&reader, i);
    let mut sent = i.session.sent_overlays.borrow_mut();
    for window in reader.windows() {
        let overlay = build_overlay(&reader, i, window, records, &scope);
        if sent.get(&window) != Some(&overlay) {
            tree.engine().set_devtools_overlay(window, overlay.clone());
            sent.insert(window, overlay);
        }
    }
}

/// The nodes paint flashing and the update heatmap draw.
enum Scope {
    /// Every node.
    All,
    /// The highlighted node and its descendants.
    Subtree(BTreeSet<NodeId>),
    /// No node: `selected_only` is on and no node is highlighted.
    Empty,
}

impl Scope {
    fn admits(&self, id: NodeId) -> bool {
        match self {
            Scope::All => true,
            Scope::Subtree(nodes) => nodes.contains(&id),
            Scope::Empty => false,
        }
    }
}

/// The scope the settings choose for this turn's paint flashing and heatmap.
fn paint_scope(reader: &Reader, i: &Inspector) -> Scope {
    if !i.session.overlays.borrow().selected_only {
        return Scope::All;
    }
    match i.session.highlight.get() {
        Some(highlight) => Scope::Subtree(subtree(reader, highlight.node)),
        None => Scope::Empty,
    }
}

/// `root` and every node below it.
fn subtree(reader: &Reader, root: NodeId) -> BTreeSet<NodeId> {
    let mut nodes = BTreeSet::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        if nodes.insert(id) {
            stack.extend(reader.children(id));
        }
    }
    nodes
}

fn build_overlay(
    reader: &Reader,
    i: &Inspector,
    window: WindowId,
    records: &EngineRecords,
    scope: &Scope,
) -> DevtoolsOverlay {
    let mut overlay = DevtoolsOverlay::default();
    if let Some(highlight) = i.session.highlight.get() {
        add_highlight(reader, window, highlight, &mut overlay);
    }
    let settings = i.session.overlays.borrow().clone();
    if settings.paint_flashing {
        add_paint_flashing(reader, window, records, scope, &mut overlay);
    }
    if settings.update_heatmap {
        add_update_heatmap(reader, window, i, scope, &mut overlay);
    }
    if settings.tab_order {
        add_tab_order(reader, window, &mut overlay);
    }
    if settings.layout_lint {
        add_layout_lint(reader, window, &mut overlay);
    }
    overlay
}

fn add_highlight(
    reader: &Reader,
    window: WindowId,
    highlight: Highlight,
    overlay: &mut DevtoolsOverlay,
) {
    if reader.window_of(highlight.node) != Some(window) {
        return;
    }
    let Some(model) = reader.box_model(highlight.node) else {
        return;
    };
    overlay.boxes.push(OverlayBox {
        margin: model.margin,
        border: model.border,
        padding: model.padding,
        content: model.content,
    });
    overlay.labels.push(OverlayLabel {
        at: Point {
            x: model.border.x,
            y: model.border.y,
        },
        text: highlight_label(reader, highlight.node, model.border),
    });
    if !highlight.layout {
        return;
    }
    for child in reader.children(highlight.node) {
        if let Some(child) = reader.box_model(child) {
            overlay
                .outlines
                .push(layout_outline(child.border, LAYOUT_COLOR));
        }
    }
    let layout = reader.layout_props(highlight.node);
    if layout.display == Display::Flex {
        add_gaps(
            reader,
            highlight.node,
            layout.direction,
            model.content,
            overlay,
        );
    }
}

fn highlight_label(reader: &Reader, id: NodeId, border: Rect) -> String {
    let semantics = reader.semantics(id);
    let role = semantics
        .as_ref()
        .and_then(|semantics| semantics.role)
        .map_or_else(|| "element".to_owned(), |role| format!("{role:?}"));
    let name = semantics
        .as_ref()
        .and_then(|semantics| semantics.name.clone())
        .unwrap_or_default();
    format!("{role} \"{name}\" {}×{}", border.width, border.height)
}

fn add_gaps(
    reader: &Reader,
    id: NodeId,
    direction: FlexDirection,
    content: Rect,
    overlay: &mut DevtoolsOverlay,
) {
    let rects: Vec<Rect> = reader
        .children(id)
        .into_iter()
        .filter_map(|child| reader.box_model(child))
        .map(|model| model.border)
        .collect();
    for pair in rects.windows(2) {
        let (previous, next) = (pair[0], pair[1]);
        let gap = match direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                let end = previous.x + previous.width;
                (end < next.x).then_some(Rect {
                    x: end,
                    y: content.y,
                    width: next.x - end,
                    height: content.height,
                })
            }
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                let end = previous.y + previous.height;
                (end < next.y).then_some(Rect {
                    x: content.x,
                    y: end,
                    width: content.width,
                    height: next.y - end,
                })
            }
        };
        if let Some(rect) = gap {
            overlay.fills.push(OverlayFill {
                rect,
                color: Rgba::from_hex(LAYOUT_COLOR, 0.35),
            });
        }
    }
}

fn add_paint_flashing(
    reader: &Reader,
    window: WindowId,
    records: &EngineRecords,
    scope: &Scope,
    overlay: &mut DevtoolsOverlay,
) {
    let mut seen = BTreeSet::new();
    for sync in &records.syncs {
        if sync.window != window {
            continue;
        }
        for id in &sync.redrawn {
            if !seen.insert(*id) || !scope.admits(*id) {
                continue;
            }
            if let Some(model) = reader.box_model(*id) {
                overlay
                    .outlines
                    .push(layout_outline(model.border, REDRAW_COLOR));
            }
        }
    }
}

fn add_update_heatmap(
    reader: &Reader,
    window: WindowId,
    i: &Inspector,
    scope: &Scope,
    overlay: &mut DevtoolsOverlay,
) {
    let turns = i.session.turns.borrow();
    let Some(turn) = turns.back() else {
        return;
    };
    let mut seen = BTreeSet::new();
    for id in &turn.changed_nodes {
        let node = NodeId(*id);
        if !seen.insert(node) || reader.window_of(node) != Some(window) || !scope.admits(node) {
            continue;
        }
        if let Some(model) = reader.box_model(node) {
            overlay
                .outlines
                .push(layout_outline(model.border, HEAT_COLOR));
        }
    }
}

fn add_tab_order(reader: &Reader, window: WindowId, overlay: &mut DevtoolsOverlay) {
    for (index, target) in reader.tab_order(window).into_iter().enumerate() {
        if let Some(model) = reader.box_model(target.node) {
            overlay.labels.push(OverlayLabel {
                at: Point {
                    x: model.border.x,
                    y: model.border.y,
                },
                text: (index + 1).to_string(),
            });
        }
    }
}

fn add_layout_lint(reader: &Reader, window: WindowId, overlay: &mut DevtoolsOverlay) {
    for finding in reader.layout_findings(window, &kivact_types::LayoutCheck::default()) {
        overlay
            .outlines
            .push(layout_outline(finding.text_rect, LINT_COLOR));
    }
}

fn layout_outline(rect: Rect, color: u32) -> OverlayOutline {
    OverlayOutline {
        rect,
        color: Rgba::from_hex(color, 1.0),
        width: 1.0,
    }
}

/// Stop pick, highlight and every overlay.
pub(super) fn clear(rt: &Runtime) {
    let i = DevtoolsState::inspector(rt);
    i.session.pick.set(None);
    i.session.highlight.set(None);
    *i.session.overlays.borrow_mut() = overlays_off();
    {
        let tree = ViewState::tree(rt);
        let writer = tree.engine();
        writer.set_pick(None);
        for window in tree.reader().windows() {
            writer.set_devtools_overlay(window, DevtoolsOverlay::default());
        }
    }
    i.session.sent_overlays.borrow_mut().clear();
    lifecycle::apply_recording(rt);
}

fn window_not_found(raw: p::WindowId) -> p::RpcError {
    p::RpcError::new(
        p::code::WINDOW_NOT_FOUND,
        format!("window {raw} does not exist"),
    )
}

fn node_not_found(raw: p::NodeId) -> p::RpcError {
    p::RpcError::new(
        p::code::NODE_NOT_FOUND,
        format!("node {raw} does not exist"),
    )
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::Highlight(params) => reply.send(highlight(rt, params).map(p::Reply::Highlight)),
        p::Call::SetOverlays(params) => {
            reply.send(set_overlays(rt, params).map(p::Reply::SetOverlays))
        }
        p::Call::GetOverlays(params) => {
            reply.send(get_overlays(rt, params).map(p::Reply::GetOverlays))
        }
        p::Call::StartPick(params) => reply.send(start_pick(rt, params).map(p::Reply::StartPick)),
        p::Call::StopPick(params) => reply.send(stop_pick(rt, params).map(p::Reply::StopPick)),
        p::Call::Screenshot(params) => screenshot(rt, params, reply),
        other => return Some((other, reply)),
    }
    None
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the overlay scope of the selected node.

    use super::Scope;
    use kivact_types::NodeId;
    use std::collections::BTreeSet;

    #[test]
    fn selected_scope_admits_the_subtree_only() {
        let scope = Scope::Subtree(BTreeSet::from([NodeId(2), NodeId(3)]));
        assert!(scope.admits(NodeId(2)));
        assert!(!scope.admits(NodeId(4)));
        assert!(!Scope::Empty.admits(NodeId(2)));
        assert!(Scope::All.admits(NodeId(4)));
    }
}
