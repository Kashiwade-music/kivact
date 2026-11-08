//! Role: Frames, counters, snapshots, owners and trace export of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn list_frames(
    rt: &Runtime,
    params: p::ListFramesParams,
) -> Result<p::FrameList, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn get_counters(rt: &Runtime, params: p::Empty) -> Result<p::Counters, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn take_snapshot(
    rt: &Runtime,
    params: p::Empty,
) -> Result<p::SnapshotInfo, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn diff_snapshots(
    rt: &Runtime,
    params: p::DiffSnapshotsParams,
) -> Result<p::SnapshotDiff, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn export_trace(
    rt: &Runtime,
    params: p::ExportTraceParams,
) -> Result<p::ExportedFiles, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::ListFrames(params) => {
            reply.send(list_frames(rt, params).map(p::Reply::ListFrames))
        }
        p::Call::GetCounters(params) => {
            reply.send(get_counters(rt, params).map(p::Reply::GetCounters))
        }
        p::Call::TakeSnapshot(params) => {
            reply.send(take_snapshot(rt, params).map(p::Reply::TakeSnapshot))
        }
        p::Call::DiffSnapshots(params) => {
            reply.send(diff_snapshots(rt, params).map(p::Reply::DiffSnapshots))
        }
        p::Call::ExportTrace(params) => {
            reply.send(export_trace(rt, params).map(p::Reply::ExportTrace))
        }
        other => return Some((other, reply)),
    }
    None
}
