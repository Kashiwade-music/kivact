//! Role: Input injection and the input log of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;
use kivact_types::RequestId;

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn inject_input(rt: &Runtime, params: p::InjectInputParams, reply: Responder) {
    let _ = (rt, params);
    reply.send(unsupported());
}

pub(super) fn list_inputs(
    rt: &Runtime,
    params: p::ListInputsParams,
) -> Result<p::InputLog, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn get_ime_state(
    rt: &Runtime,
    params: p::WindowParams,
) -> Result<p::ImeState, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

/// Answer the `inject_input` call whose inputs were delivered.
pub(super) fn delivered(_rt: &Runtime, _request: RequestId) {}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::InjectInput(params) => inject_input(rt, params, reply),
        p::Call::ListInputs(params) => {
            reply.send(list_inputs(rt, params).map(p::Reply::ListInputs))
        }
        p::Call::GetImeState(params) => {
            reply.send(get_ime_state(rt, params).map(p::Reply::GetImeState))
        }
        other => return Some((other, reply)),
    }
    None
}
