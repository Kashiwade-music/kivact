//! Role: The review of the DevTools target.

use std::panic::Location;

use kivact_devtools_agent::{NewComment, Responder};
use kivact_devtools_protocol as p;
use kivact_runtime::{Owner, Runtime};
use kivact_types::{NodeId, WindowId};

use super::convert;
use super::node;
use super::tree;
use crate::view::devtools_apply;
use crate::view::devtools_state::{DevtoolsState, Inspector};
use crate::view::state::ViewState;

/// The window title, target and options of one comment, read before a screenshot wait.
struct PreparedComment {
    window: WindowId,
    window_title: String,
    target: p::ReviewTarget,
    style_changes: Vec<p::StyleChange>,
    text: String,
    /// The node to capture and its owner when a screenshot was asked for.
    snapshot: Option<(NodeId, Owner, &'static Location<'static>)>,
}

pub(super) fn get_review(rt: &Runtime, _: p::Empty) -> Result<p::ReviewDocument, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    sync_environment(i);
    Ok(i.agent.borrow_mut().review().document().clone())
}

pub(super) fn add_comment(rt: &Runtime, params: p::AddCommentParams, reply: Responder) {
    let prepared = match prepare(rt, &params) {
        Ok(prepared) => prepared,
        Err(error) => {
            reply.send(Err(error));
            return;
        }
    };
    let Some((id, owner, at)) = prepared.snapshot else {
        reply.send(submit(rt, prepared, None).map(p::Reply::AddComment));
        return;
    };
    kivact_runtime::spawn(async move {
        let image = match capture(id, owner, at).await {
            Ok(image) => Some(image),
            Err(error) => {
                reply.send(Err(error));
                return;
            }
        };
        let Some(rt) = Runtime::current() else {
            reply.send(Err(internal("the App is no longer running")));
            return;
        };
        reply.send(submit(&rt, prepared, image).map(p::Reply::AddComment));
    });
}

pub(super) fn edit_comment(
    rt: &Runtime,
    params: p::EditCommentParams,
) -> Result<p::ReviewItem, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    sync_environment(i);
    i.agent
        .borrow_mut()
        .review()
        .edit(&params.comment, params.text, now_ms())
        .map_err(|error| p::RpcError::new(p::code::INVALID_PARAMS, error))
}

pub(super) fn remove_comment(
    rt: &Runtime,
    params: p::RemoveCommentParams,
) -> Result<p::Empty, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    sync_environment(i);
    i.agent
        .borrow_mut()
        .review()
        .remove(&params.comment, now_ms())
        .map_err(|error| p::RpcError::new(p::code::INVALID_PARAMS, error))?;
    Ok(p::Empty {})
}

pub(super) fn export_review(rt: &Runtime, _: p::Empty) -> Result<p::ExportedReview, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    sync_environment(i);
    i.agent
        .borrow_mut()
        .review()
        .export()
        .map_err(|error| internal(error.to_string()))
}

/// Read the window, target and style changes of a comment before any wait.
fn prepare(rt: &Runtime, params: &p::AddCommentParams) -> Result<PreparedComment, p::RpcError> {
    let Some(window) = convert::window_from(params.window) else {
        return Err(window_not_found(params.window));
    };
    let record = ViewState::of(rt)
        .window_record(window)
        .ok_or_else(|| window_not_found(params.window))?;
    let target = match params.node {
        Some(raw) => node_target(rt, raw)?,
        None => p::ReviewTarget::Window {},
    };
    let style_changes = match &target {
        p::ReviewTarget::Node { node, .. } => node_style_changes(rt, NodeId(*node)),
        p::ReviewTarget::Window {} => Vec::new(),
    };
    let snapshot = if params.screenshot {
        Some(snapshot_target(rt, window, params.node)?)
    } else {
        None
    };
    Ok(PreparedComment {
        window,
        window_title: record.title.get_untracked(),
        target,
        style_changes,
        text: params.text.clone(),
        snapshot,
    })
}

/// The target of a comment on one node, as it is now.
fn node_target(rt: &Runtime, raw: p::NodeId) -> Result<p::ReviewTarget, p::RpcError> {
    let id = NodeId(raw);
    let info = tree::tree_node(rt, id).ok_or_else(|| node_not_found(raw))?;
    let bounds = {
        let tree = ViewState::tree(rt);
        match tree.reader().box_model(id) {
            Some(model) => convert::rect(model.border),
            None => p::Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        }
    };
    Ok(p::ReviewTarget::Node {
        node: raw,
        role: info.role,
        name: info.name,
        test_id: info.test_id,
        source: info.source,
        call_chain: node::call_chain(rt, id),
        bounds,
        states: node::states(rt, id),
    })
}

/// The node and owner a screenshot of this comment captures.
fn snapshot_target(
    rt: &Runtime,
    window: WindowId,
    node: Option<p::NodeId>,
) -> Result<(NodeId, Owner, &'static Location<'static>), p::RpcError> {
    let tree = ViewState::tree(rt);
    let id = match node {
        Some(raw) => NodeId(raw),
        None => tree
            .reader()
            .root(window)
            .ok_or_else(|| window_not_found(convert::window(window)))?,
    };
    let entry = tree.get(id).ok_or_else(|| node_not_found(id.0))?;
    let at = match entry.at {
        Some(at) => at,
        None => Location::caller(),
    };
    Ok((id, entry.owner, at))
}

/// The overrides that apply to `id`: call-site overrides and node overrides.
fn node_style_changes(rt: &Runtime, id: NodeId) -> Vec<p::StyleChange> {
    let (sites, all) = {
        let tree = ViewState::tree(rt);
        let Some(entry) = tree.get(id) else {
            return Vec::new();
        };
        let all = DevtoolsState::inspector(rt).hooks.overrides.get_untracked();
        (devtools_apply::call_sites(rt, entry.owner, entry.at), all)
    };
    all.into_iter()
        .filter(|entry| scope_applies(&entry.scope, id, &sites))
        .map(|entry| p::StyleChange {
            scope: entry.scope,
            property: entry.property,
            before: None,
            after: entry.value,
        })
        .collect()
}

/// Whether an override scope applies to `id`.
fn scope_applies(
    scope: &p::OverrideScope,
    id: NodeId,
    sites: &[&'static Location<'static>],
) -> bool {
    match scope {
        p::OverrideScope::Node { node } => *node == id.0,
        p::OverrideScope::CallSite { location } => sites.iter().any(|site| {
            site.file() == location.file
                && site.line() == location.line
                && site.column() == location.column
        }),
        p::OverrideScope::Token { .. } => false,
    }
}

/// Capture `id` as PNG bytes in a task-owned snapshot wait.
async fn capture(
    id: NodeId,
    owner: Owner,
    at: &'static Location<'static>,
) -> Result<Vec<u8>, p::RpcError> {
    let rt = Runtime::current().ok_or_else(|| internal("the App is no longer running"))?;
    let pixels = crate::view::snapshots::request(&rt, id, owner, at)
        .await
        .map_err(|error| internal(error.to_string()))?;
    kivact_devtools_agent::encode_png(pixels.width, pixels.height, pixels.rgba.as_ref())
        .map_err(|error| internal(error.to_string()))
}

/// Add the comment of `prepared` to the agent's review.
fn submit(
    rt: &Runtime,
    prepared: PreparedComment,
    image_png: Option<Vec<u8>>,
) -> Result<p::ReviewItem, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    sync_environment(i);
    let comment = NewComment {
        window: convert::window(prepared.window),
        window_title: prepared.window_title,
        target: prepared.target,
        style_changes: prepared.style_changes,
        image_png,
        text: prepared.text,
    };
    i.agent
        .borrow_mut()
        .review()
        .add(comment, now_ms())
        .map_err(|error| internal(error.to_string()))
}

/// Record the current emulation in the review document.
fn sync_environment(i: &Inspector) {
    let environment = i.hooks.emulation.get_untracked();
    i.agent.borrow_mut().review().set_environment(environment);
}

fn internal(message: impl Into<String>) -> p::RpcError {
    p::RpcError::new(p::code::INTERNAL_ERROR, message.into())
}

fn node_not_found(raw: p::NodeId) -> p::RpcError {
    p::RpcError::new(
        p::code::NODE_NOT_FOUND,
        format!("node {raw} does not exist"),
    )
}

fn window_not_found(raw: p::WindowId) -> p::RpcError {
    p::RpcError::new(
        p::code::WINDOW_NOT_FOUND,
        format!("window {raw} does not exist"),
    )
}

/// The wall clock in Unix milliseconds (review dates are RFC 3339 in the agent).
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::GetReview(params) => reply.send(get_review(rt, params).map(p::Reply::GetReview)),
        p::Call::AddComment(params) => add_comment(rt, params, reply),
        p::Call::EditComment(params) => {
            reply.send(edit_comment(rt, params).map(p::Reply::EditComment))
        }
        p::Call::RemoveComment(params) => {
            reply.send(remove_comment(rt, params).map(p::Reply::RemoveComment))
        }
        p::Call::ExportReview(params) => {
            reply.send(export_review(rt, params).map(p::Reply::ExportReview))
        }
        other => return Some((other, reply)),
    }
    None
}
