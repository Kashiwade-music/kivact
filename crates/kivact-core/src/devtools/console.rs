//! Role: The reported errors of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn list_errors(
    rt: &Runtime,
    params: p::ListErrorsParams,
) -> Result<p::ErrorList, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

/// Notify the Errors subscribers of the errors reported since the last turn.
pub(super) fn after_turn(_rt: &Runtime) {}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::ListErrors(params) => {
            reply.send(list_errors(rt, params).map(p::Reply::ListErrors))
        }
        other => return Some((other, reply)),
    }
    None
}
