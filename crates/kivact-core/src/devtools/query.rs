//! Role: The role, text and test-id query methods of the DevTools target.

use std::panic::Location;
use std::time::Duration;

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_engine::api::Reader;
use kivact_runtime::Runtime;
use kivact_types::NodeId;

use super::convert;
use super::tree::tree_node;
use crate::view::devtools_state::{DevtoolsState, Pending, QueryBy};
use crate::view::state::ViewState;
use crate::view::tree::ViewNodes;

/// The wait of a `find_*` call that names no timeout.
const FIND_TIMEOUT_MS: u32 = 5000;

pub(super) fn get_by_role(rt: &Runtime, params: p::RoleQuery) -> Result<p::FoundNode, p::RpcError> {
    one(rt, &QueryBy::Role(params))
}

pub(super) fn query_by_role(
    rt: &Runtime,
    params: p::RoleQuery,
) -> Result<p::MaybeNode, p::RpcError> {
    maybe(rt, &QueryBy::Role(params))
}

pub(super) fn find_by_role(rt: &Runtime, params: p::RoleQuery, reply: Responder) {
    find(rt, QueryBy::Role(params), reply);
}

pub(super) fn all_by_role(
    rt: &Runtime,
    params: p::RoleQuery,
) -> Result<p::FoundNodes, p::RpcError> {
    all(rt, &QueryBy::Role(params))
}

pub(super) fn get_by_text(rt: &Runtime, params: p::TextQuery) -> Result<p::FoundNode, p::RpcError> {
    one(rt, &QueryBy::Text(params))
}

pub(super) fn query_by_text(
    rt: &Runtime,
    params: p::TextQuery,
) -> Result<p::MaybeNode, p::RpcError> {
    maybe(rt, &QueryBy::Text(params))
}

pub(super) fn find_by_text(rt: &Runtime, params: p::TextQuery, reply: Responder) {
    find(rt, QueryBy::Text(params), reply);
}

pub(super) fn all_by_text(
    rt: &Runtime,
    params: p::TextQuery,
) -> Result<p::FoundNodes, p::RpcError> {
    all(rt, &QueryBy::Text(params))
}

pub(super) fn get_by_test_id(
    rt: &Runtime,
    params: p::TestIdQuery,
) -> Result<p::FoundNode, p::RpcError> {
    one(rt, &QueryBy::TestId(params))
}

pub(super) fn query_by_test_id(
    rt: &Runtime,
    params: p::TestIdQuery,
) -> Result<p::MaybeNode, p::RpcError> {
    maybe(rt, &QueryBy::TestId(params))
}

pub(super) fn find_by_test_id(rt: &Runtime, params: p::TestIdQuery, reply: Responder) {
    find(rt, QueryBy::TestId(params), reply);
}

pub(super) fn all_by_test_id(
    rt: &Runtime,
    params: p::TestIdQuery,
) -> Result<p::FoundNodes, p::RpcError> {
    all(rt, &QueryBy::TestId(params))
}

/// The window and scope of a query.
fn bounds_of(by: &QueryBy) -> (p::WindowId, Option<p::NodeId>) {
    match by {
        QueryBy::Role(params) => (params.window, params.within),
        QueryBy::Text(params) => (params.window, params.within),
        QueryBy::TestId(params) => (params.window, params.within),
    }
}

/// The timeout of a `find_*` call, with the default filled in.
fn timeout_of(by: &QueryBy) -> u32 {
    let timeout = match by {
        QueryBy::Role(params) => params.timeout_ms,
        QueryBy::Text(params) => params.timeout_ms,
        QueryBy::TestId(params) => params.timeout_ms,
    };
    timeout.unwrap_or(FIND_TIMEOUT_MS)
}

/// The visible nodes of the query's window and scope that match, in tree order.
fn search(rt: &Runtime, by: &QueryBy) -> Result<Vec<NodeId>, p::RpcError> {
    let (raw_window, within) = bounds_of(by);
    let tree = ViewState::tree(rt);
    let reader = tree.reader();
    let window = convert::window_from(raw_window)
        .filter(|window| reader.windows().contains(window))
        .ok_or_else(|| window_not_found(raw_window))?;
    if let Some(within) = within
        && tree.get(NodeId(within)).is_none()
    {
        return Err(node_not_found(within));
    }
    let start = match within {
        Some(within) => NodeId(within),
        None => reader
            .root(window)
            .ok_or_else(|| window_not_found(raw_window))?,
    };
    let mut found = Vec::new();
    let mut stack = vec![start];
    while let Some(id) = stack.pop() {
        if tree.hidden(id) == Some(true) {
            continue;
        }
        if !tree.is_visible(id) {
            continue;
        }
        if matches_node(&tree, &reader, id, by) {
            found.push(id);
        }
        for child in reader.children(id).into_iter().rev() {
            stack.push(child);
        }
    }
    Ok(found)
}

fn matches_node(tree: &ViewNodes, reader: &Reader, id: NodeId, by: &QueryBy) -> bool {
    match by {
        QueryBy::Role(params) => {
            let Some(semantics) = reader.semantics(id) else {
                return false;
            };
            let role = semantics.role.unwrap_or(kivact_types::Role::Generic);
            role == convert::role_from(params.role)
                && params.name.as_ref().is_none_or(|name| {
                    semantics.name.as_deref().unwrap_or_default() == name.as_str()
                })
        }
        QueryBy::Text(params) => tree.content(id).is_some_and(|text| text == params.text),
        QueryBy::TestId(params) => {
            reader
                .semantics(id)
                .and_then(|semantics| semantics.test_id)
                .as_deref()
                == Some(params.id.as_str())
        }
    }
}

fn one(rt: &Runtime, by: &QueryBy) -> Result<p::FoundNode, p::RpcError> {
    let found = search(rt, by)?;
    match found.as_slice() {
        [id] => found_node(rt, *id),
        [] => Err(no_match(by)),
        _ => Err(multiple_matches(found.len())),
    }
}

fn maybe(rt: &Runtime, by: &QueryBy) -> Result<p::MaybeNode, p::RpcError> {
    let found = search(rt, by)?;
    let node = match found.as_slice() {
        [id] => tree_node(rt, *id),
        _ => None,
    };
    Ok(p::MaybeNode { node })
}

fn all(rt: &Runtime, by: &QueryBy) -> Result<p::FoundNodes, p::RpcError> {
    let found = search(rt, by)?;
    let nodes = found
        .into_iter()
        .filter_map(|id| tree_node(rt, id))
        .collect();
    Ok(p::FoundNodes { nodes })
}

fn found_node(rt: &Runtime, id: NodeId) -> Result<p::FoundNode, p::RpcError> {
    let node = tree_node(rt, id).ok_or_else(|| node_not_found(id.0))?;
    Ok(p::FoundNode { node })
}

fn find(rt: &Runtime, by: QueryBy, reply: Responder) {
    let found = match search(rt, &by) {
        Ok(found) => found,
        Err(error) => {
            reply.send(Err(error));
            return;
        }
    };
    match found.as_slice() {
        [id] => reply.send(found_node(rt, *id).map(|node| reply_for(&by, node))),
        [] => wait(rt, by, reply),
        _ => reply.send(Err(multiple_matches(found.len()))),
    }
}

/// The reply variant of a found node, by the kind of query.
fn reply_for(by: &QueryBy, node: p::FoundNode) -> p::Reply {
    match by {
        QueryBy::Role(_) => p::Reply::FindByRole(node),
        QueryBy::Text(_) => p::Reply::FindByText(node),
        QueryBy::TestId(_) => p::Reply::FindByTestId(node),
    }
}

fn wait(rt: &Runtime, by: QueryBy, reply: Responder) {
    let (raw_window, _) = bounds_of(&by);
    let Some(window) = convert::window_from(raw_window) else {
        reply.send(Err(window_not_found(raw_window)));
        return;
    };
    let timeout = timeout_of(&by);
    let deadline_ns = rt.now_ns().saturating_add(u64::from(timeout) * 1_000_000);
    DevtoolsState::inspector(rt)
        .session
        .pending
        .borrow_mut()
        .push(Pending::Find {
            by,
            window,
            deadline_ns,
            reply,
        });
    kivact_runtime::spawn_in(
        rt.root(),
        async move {
            kivact_runtime::sleep(Duration::from_millis(u64::from(timeout))).await;
        },
        Location::caller(),
    );
}

/// Answer the `find_*` calls whose element appeared or whose deadline passed.
pub(super) fn after_turn(rt: &Runtime) {
    let i = DevtoolsState::inspector(rt);
    let pending = std::mem::take(&mut *i.session.pending.borrow_mut());
    for item in pending {
        let Pending::Find {
            by,
            window,
            deadline_ns,
            reply,
        } = item
        else {
            i.session.pending.borrow_mut().push(item);
            continue;
        };
        let found = match search(rt, &by) {
            Ok(found) => found,
            Err(error) => {
                reply.send(Err(error));
                continue;
            }
        };
        match found.as_slice() {
            [id] => reply.send(found_node(rt, *id).map(|node| reply_for(&by, node))),
            [] if rt.now_ns() >= deadline_ns => reply.send(Err(timed_out())),
            [] => i.session.pending.borrow_mut().push(Pending::Find {
                by,
                window,
                deadline_ns,
                reply,
            }),
            _ => reply.send(Err(multiple_matches(found.len()))),
        }
    }
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

fn no_match(by: &QueryBy) -> p::RpcError {
    let what = match by {
        QueryBy::Role(_) => "role",
        QueryBy::Text(_) => "text",
        QueryBy::TestId(_) => "test id",
    };
    p::RpcError::new(
        p::code::NO_MATCH,
        format!("no element matches the {what} query"),
    )
}

fn multiple_matches(count: usize) -> p::RpcError {
    p::RpcError::new(p::code::MULTIPLE_MATCHES, format!("{count} elements match"))
}

fn timed_out() -> p::RpcError {
    p::RpcError::new(p::code::TIMED_OUT, "the query timed out")
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::GetByRole(params) => reply.send(get_by_role(rt, params).map(p::Reply::GetByRole)),
        p::Call::QueryByRole(params) => {
            reply.send(query_by_role(rt, params).map(p::Reply::QueryByRole))
        }
        p::Call::FindByRole(params) => find_by_role(rt, params, reply),
        p::Call::AllByRole(params) => reply.send(all_by_role(rt, params).map(p::Reply::AllByRole)),
        p::Call::GetByText(params) => reply.send(get_by_text(rt, params).map(p::Reply::GetByText)),
        p::Call::QueryByText(params) => {
            reply.send(query_by_text(rt, params).map(p::Reply::QueryByText))
        }
        p::Call::FindByText(params) => find_by_text(rt, params, reply),
        p::Call::AllByText(params) => reply.send(all_by_text(rt, params).map(p::Reply::AllByText)),
        p::Call::GetByTestId(params) => {
            reply.send(get_by_test_id(rt, params).map(p::Reply::GetByTestId))
        }
        p::Call::QueryByTestId(params) => {
            reply.send(query_by_test_id(rt, params).map(p::Reply::QueryByTestId))
        }
        p::Call::FindByTestId(params) => find_by_test_id(rt, params, reply),
        p::Call::AllByTestId(params) => {
            reply.send(all_by_test_id(rt, params).map(p::Reply::AllByTestId))
        }
        other => return Some((other, reply)),
    }
    None
}
