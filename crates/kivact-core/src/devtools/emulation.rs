//! Role: The environment emulation of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn get_emulation(rt: &Runtime, params: p::Empty) -> Result<p::Emulation, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn set_emulation(
    rt: &Runtime,
    params: p::Emulation,
) -> Result<p::Emulation, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

/// Restore every emulated value to the App's own environment.
pub(super) fn clear(_rt: &Runtime) {}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::GetEmulation(params) => {
            reply.send(get_emulation(rt, params).map(p::Reply::GetEmulation))
        }
        p::Call::SetEmulation(params) => {
            reply.send(set_emulation(rt, params).map(p::Reply::SetEmulation))
        }
        other => return Some((other, reply)),
    }
    None
}
