//! Role: The role, text and test-id query methods of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn get_by_role(rt: &Runtime, params: p::RoleQuery) -> Result<p::FoundNode, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn query_by_role(
    rt: &Runtime,
    params: p::RoleQuery,
) -> Result<p::MaybeNode, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn find_by_role(rt: &Runtime, params: p::RoleQuery, reply: Responder) {
    let _ = (rt, params);
    reply.send(unsupported());
}

pub(super) fn all_by_role(
    rt: &Runtime,
    params: p::RoleQuery,
) -> Result<p::FoundNodes, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn get_by_text(rt: &Runtime, params: p::TextQuery) -> Result<p::FoundNode, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn query_by_text(
    rt: &Runtime,
    params: p::TextQuery,
) -> Result<p::MaybeNode, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn find_by_text(rt: &Runtime, params: p::TextQuery, reply: Responder) {
    let _ = (rt, params);
    reply.send(unsupported());
}

pub(super) fn all_by_text(
    rt: &Runtime,
    params: p::TextQuery,
) -> Result<p::FoundNodes, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn get_by_test_id(
    rt: &Runtime,
    params: p::TestIdQuery,
) -> Result<p::FoundNode, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn query_by_test_id(
    rt: &Runtime,
    params: p::TestIdQuery,
) -> Result<p::MaybeNode, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn find_by_test_id(rt: &Runtime, params: p::TestIdQuery, reply: Responder) {
    let _ = (rt, params);
    reply.send(unsupported());
}

pub(super) fn all_by_test_id(
    rt: &Runtime,
    params: p::TestIdQuery,
) -> Result<p::FoundNodes, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

/// Answer the `find_*` calls whose element appeared or whose deadline passed.
pub(super) fn after_turn(_rt: &Runtime) {}

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
