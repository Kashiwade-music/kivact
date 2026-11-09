//! Role: The reported errors of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::{Reported, Runtime};

use super::convert;
use crate::view::devtools_state::DevtoolsState;

/// The wire form of one reported error.
fn error_entry(entry: Reported) -> p::ErrorEntry {
    p::ErrorEntry {
        seq: entry.seq,
        kind: format!("{:?}", entry.error.kind()),
        message: entry.error.message().to_owned(),
        source: entry.error.location().map(convert::location),
        turn: None,
        handled: entry.handled,
    }
}

pub(super) fn list_errors(
    rt: &Runtime,
    params: p::ListErrorsParams,
) -> Result<p::ErrorList, p::RpcError> {
    let since = params.since.unwrap_or(0);
    let mut errors: Vec<p::ErrorEntry> = rt
        .reported()
        .into_iter()
        .filter(|entry| entry.seq > since)
        .map(error_entry)
        .collect();
    if let Some(limit) = params.limit {
        let keep = limit as usize;
        if errors.len() > keep {
            errors.drain(..errors.len() - keep);
        }
    }
    Ok(p::ErrorList { errors })
}

/// Notify the Errors subscribers of the errors reported since the last turn.
pub(super) fn after_turn(rt: &Runtime) {
    let i = DevtoolsState::inspector(rt);
    let reported = rt.reported();
    let last = reported.last().map_or(0, |entry| entry.seq);
    let since = i.session.errors_sent.borrow().get().copied().unwrap_or(0);
    if last <= since {
        return;
    }
    let wanted = i.wants(p::Topic::Errors);
    for entry in reported.into_iter().filter(|entry| entry.seq > since) {
        if wanted {
            i.notify(p::Notification::ErrorReported(error_entry(entry)));
        }
    }
    i.session.errors_sent.borrow_mut().update(last);
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::ListErrors(params) => {
            reply.send(list_errors(rt, params).map(p::Reply::ListErrors))
        }
        other => return Some((other, reply)),
    }
    None
}
