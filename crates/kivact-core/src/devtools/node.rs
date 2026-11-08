//! Role: The element detail and owner-tree methods of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;
use kivact_types::NodeId;

/// The call chain of `id`: where it was created and the owners above it.
pub(super) fn call_chain(_rt: &Runtime, _id: NodeId) -> Vec<p::CallFrame> {
    Vec::new()
}

/// The interactive states of `id`.
pub(super) fn states(_rt: &Runtime, _id: NodeId) -> Vec<p::InteractiveState> {
    Vec::new()
}

pub(super) fn get_node(
    rt: &Runtime,
    params: p::GetNodeParams,
) -> Result<p::NodeDetail, p::RpcError> {
    let _ = (rt, params);
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn get_owners(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::OwnerTree, p::RpcError> {
    let _ = (rt, params);
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::GetNode(params) => reply.send(get_node(rt, params).map(p::Reply::GetNode)),
        p::Call::GetOwners(params) => reply.send(get_owners(rt, params).map(p::Reply::GetOwners)),
        other => return Some((other, reply)),
    }
    None
}
