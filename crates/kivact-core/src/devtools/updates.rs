//! Role: Recording, turns and frames of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_engine::contract::EngineRecords;
use kivact_runtime::Runtime;

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn start_recording(
    rt: &Runtime,
    params: p::RecordingSettings,
) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn stop_recording(rt: &Runtime, params: p::Empty) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn get_recording(
    rt: &Runtime,
    params: p::Empty,
) -> Result<p::RecordingSettings, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn list_turns(rt: &Runtime, params: p::ListTurnsParams, reply: Responder) {
    let _ = (rt, params);
    reply.send(unsupported());
}

/// Finish the graphs whose frame arrived, and record this turn's graph.
pub(super) fn after_turn(_rt: &Runtime, _records: &EngineRecords) {}

/// Finish every waiting graph and stop recording.
pub(super) fn stop(_rt: &Runtime) {}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::StartRecording(params) => {
            reply.send(start_recording(rt, params).map(p::Reply::StartRecording))
        }
        p::Call::StopRecording(params) => {
            reply.send(stop_recording(rt, params).map(p::Reply::StopRecording))
        }
        p::Call::GetRecording(params) => {
            reply.send(get_recording(rt, params).map(p::Reply::GetRecording))
        }
        p::Call::ListTurns(params) => list_turns(rt, params, reply),
        other => return Some((other, reply)),
    }
    None
}
