//! Role: The tables a script shows in the DevTools app.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;

use crate::view::devtools_state::DevtoolsState;

pub(super) fn show_table(
    rt: &Runtime,
    params: p::ShowTableParams,
) -> Result<p::Empty, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    if i.wants(p::Topic::Scripts) {
        i.notify(p::Notification::ScriptTable(params));
    }
    Ok(p::Empty {})
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::ShowTable(params) => reply.send(show_table(rt, params).map(p::Reply::ShowTable)),
        other => return Some((other, reply)),
    }
    None
}
