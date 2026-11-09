//! Role: The app, window and element tree methods of the DevTools target.

use std::collections::{BTreeMap, HashMap};

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_engine::contract::NodeKind as EngineNodeKind;
use kivact_runtime::Runtime;
use kivact_types::{NodeId, WindowId};

use super::convert;
use crate::view::devtools_state::DevtoolsState;
use crate::view::state::{ViewState, current_window};

/// One element of the retained tree as the wire sees it.
pub(crate) fn tree_node(rt: &Runtime, id: NodeId) -> Option<p::ElementInfo> {
    let tree = ViewState::tree(rt);
    let node = tree.get(id)?;
    let reader = tree.reader();
    let window = reader.window_of(id)?;
    let semantics = reader.semantics(id);
    let kind = reader
        .with_node(id, |props| {
            props.map(|props| match &props.kind {
                EngineNodeKind::Root => p::NodeKind::Root,
                EngineNodeKind::Box(_) => p::NodeKind::Box,
                EngineNodeKind::Text(_) => p::NodeKind::Text,
                EngineNodeKind::Image(_) => p::NodeKind::Image,
                EngineNodeKind::Overlay(_) => p::NodeKind::Overlay,
            })
        })
        .unwrap_or(p::NodeKind::Box);
    let update_count = DevtoolsState::inspector(rt)
        .session
        .update_counts
        .borrow()
        .get(&id)
        .copied()
        .unwrap_or(0);
    Some(p::ElementInfo {
        id: id.0,
        window: convert::window(window),
        parent: reader.parent(id).map(|parent| parent.0),
        children: reader.children(id).iter().map(|child| child.0).collect(),
        kind,
        role: semantics.as_ref().and_then(|s| s.role).map(convert::role),
        name: semantics.as_ref().and_then(|s| s.name.clone()),
        text: tree
            .content(id)
            .map(|text| text.chars().take(80).collect::<String>()),
        test_id: semantics.as_ref().and_then(|s| s.test_id.clone()),
        source: node.at.map(convert::location),
        hidden: tree.hidden(id).unwrap_or(false),
        leaving: matches!(reader.presence(id), kivact_types::Presence::Leaving),
        update_count,
    })
}

pub(super) fn get_app_info(rt: &Runtime, _: p::Empty) -> Result<p::AppInfo, p::RpcError> {
    let app = DevtoolsState::inspector(rt).agent.borrow().app().clone();
    Ok(app)
}

pub(super) fn list_windows(rt: &Runtime, _: p::Empty) -> Result<p::WindowList, p::RpcError> {
    let state = ViewState::of(rt);
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let focused = reader.focused_window();
    let mut windows = Vec::new();
    for window in reader.windows() {
        let Some(record) = state.window_record(window) else {
            continue;
        };
        let environment = record.env.environment();
        let parent =
            (window != WindowId::MAIN).then(|| convert::window(current_window(record.declared_in)));
        windows.push(p::WindowInfo {
            id: convert::window(window),
            title: record.title.get_untracked(),
            bounds: p::Rect {
                x: 0.0,
                y: 0.0,
                width: environment.window_size.width,
                height: environment.window_size.height,
            },
            scale: environment.scale_factor,
            focused: focused == Some(window),
            mode: match record.mode {
                kivact_types::WindowMode::Normal => p::WindowMode::Normal,
                kivact_types::WindowMode::Tool => p::WindowMode::Tool,
                kivact_types::WindowMode::Modal => p::WindowMode::Modal,
            },
            parent,
        });
    }
    Ok(p::WindowList { windows })
}

pub(super) fn get_tree(
    rt: &Runtime,
    params: p::GetTreeParams,
) -> Result<p::TreeSnapshot, p::RpcError> {
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let window = convert::window_from(params.window)
        .filter(|window| reader.windows().contains(window))
        .ok_or_else(|| window_not_found(params.window))?;
    let root = match params.root {
        Some(raw) => {
            let root = NodeId(raw);
            if tree.get(root).is_none() {
                return Err(p::RpcError::new(
                    p::code::NODE_NOT_FOUND,
                    format!("node {raw} does not exist"),
                ));
            }
            root
        }
        None => reader
            .root(window)
            .ok_or_else(|| window_not_found(params.window))?,
    };
    let revision = reader.stats().revision;
    let mut nodes = Vec::new();
    let mut stack = vec![(root, 0_u32)];
    while let Some((id, depth)) = stack.pop() {
        if let Some(info) = tree_node(rt, id) {
            nodes.push(info);
        }
        if params.depth.is_none_or(|limit| depth < limit) {
            for child in reader.children(id).into_iter().rev() {
                stack.push((child, depth + 1));
            }
        }
    }
    Ok(p::TreeSnapshot {
        window: convert::window(window),
        root: root.0,
        revision,
        nodes,
    })
}

fn window_not_found(raw: p::WindowId) -> p::RpcError {
    p::RpcError::new(
        p::code::WINDOW_NOT_FOUND,
        format!("window {raw} does not exist"),
    )
}

/// Notify the Windows subscribers when the open windows changed.
pub(crate) fn notify_windows_changed(rt: &Runtime) {
    let i = DevtoolsState::inspector(rt);
    if !i.wants(p::Topic::Windows) {
        return;
    }
    let Ok(list) = list_windows(rt, p::Empty {}) else {
        return;
    };
    let changed = {
        let mut sent = i.session.sent_windows.borrow_mut();
        sent.update(list.windows.clone()).is_some()
    };
    if changed {
        i.notify(p::Notification::WindowsChanged(list));
    }
}

/// Notify the Tree subscribers of the nodes added, changed or removed.
pub(crate) fn notify_tree_changes(rt: &Runtime) {
    let i = DevtoolsState::inspector(rt);
    if !i.wants(p::Topic::Tree) {
        i.session.sent_tree.borrow_mut().clear();
        return;
    }
    let current = current_nodes(rt);
    let mut changes: BTreeMap<p::WindowId, (Vec<p::ElementInfo>, Vec<p::NodeId>)> = BTreeMap::new();
    {
        let sent = i.session.sent_tree.borrow();
        for (id, info) in &current {
            if sent.get(id) != Some(info) {
                changes.entry(info.window).or_default().0.push(info.clone());
            }
        }
        for (id, old) in sent.iter() {
            if !current.contains_key(id) {
                changes.entry(old.window).or_default().1.push(id.0);
            }
        }
    }
    let revision = ViewState::tree(rt).reader().stats().revision;
    for (window, (upserted, removed)) in changes {
        i.notify(p::Notification::TreeChanged(p::TreeChanged {
            window,
            revision,
            upserted,
            removed,
        }));
    }
    *i.session.sent_tree.borrow_mut() = current;
}

fn current_nodes(rt: &Runtime) -> HashMap<NodeId, p::ElementInfo> {
    let mut nodes = HashMap::new();
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    for window in reader.windows() {
        let Some(root) = reader.root(window) else {
            continue;
        };
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if let Some(info) = tree_node(rt, id) {
                nodes.insert(id, info);
            }
            stack.extend(reader.children(id));
        }
    }
    nodes
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::GetAppInfo(params) => {
            reply.send(get_app_info(rt, params).map(p::Reply::GetAppInfo))
        }
        p::Call::ListWindows(params) => {
            reply.send(list_windows(rt, params).map(p::Reply::ListWindows))
        }
        p::Call::GetTree(params) => reply.send(get_tree(rt, params).map(p::Reply::GetTree)),
        other => return Some((other, reply)),
    }
    None
}
