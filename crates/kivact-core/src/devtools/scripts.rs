//! Role: The tables a script shows in the DevTools app.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;

pub(super) fn show_table(
    rt: &Runtime,
    params: p::ShowTableParams,
) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::ShowTable(params) => reply.send(show_table(rt, params).map(p::Reply::ShowTable)),
        other => return Some((other, reply)),
    }
    None
}
