//! Role: Frames, counters, snapshots, owners and trace export of the DevTools target.

use std::collections::BTreeMap;

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::{LiveSnapshot, MotionState, Runtime, TaskState};

use super::convert;
use crate::client::state::HostState;
use crate::view::devtools_state::DevtoolsState;
use crate::view::state::ViewState;

/// Snapshots kept for `diff_snapshots`.
const SNAPSHOT_CAP: usize = 10;

pub(super) fn list_frames(
    rt: &Runtime,
    params: p::ListFramesParams,
) -> Result<p::FrameList, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    let since = params.since.unwrap_or(0);
    let mut frames = Vec::new();
    {
        let history = i.session.frames.borrow();
        if params.window.is_none() {
            frames.extend(
                history
                    .values()
                    .flat_map(|queue| queue.iter())
                    .filter(|frame| frame.end_ns > since)
                    .cloned(),
            );
        } else if let Some(window) = params.window.and_then(convert::window_from)
            && let Some(queue) = history.get(&window)
        {
            frames.extend(queue.iter().filter(|frame| frame.end_ns > since).cloned());
        }
    }
    frames.sort_by_key(|frame| frame.end_ns);
    if let Some(limit) = params.limit {
        let limit = limit as usize;
        if frames.len() > limit {
            frames.drain(..frames.len() - limit);
        }
    }
    Ok(p::FrameList { frames })
}

pub(super) fn get_counters(rt: &Runtime, params: p::Empty) -> Result<p::Counters, p::RpcError> {
    let _ = params;
    let live = rt.live_counts();
    let motion = MotionState::of(rt).stats();
    let tasks = TaskState::of(rt).stats();
    Ok(p::Counters {
        live_reactive: live.reactive as u64,
        live_owners: live.owners as u64,
        live_objects: live.objects as u64,
        live_constants: live.constants as u64,
        tree_nodes: ViewState::tree(rt).len() as u64,
        timers: tasks.timers as u64,
        frame_loops: motion.loops as u64,
        animations: motion.animations as u64,
        turns: HostState::of(rt).turns(),
        outstanding_blocking: tasks.outstanding_blocking as u64,
    })
}

pub(super) fn take_snapshot(
    rt: &Runtime,
    params: p::Empty,
) -> Result<p::SnapshotInfo, p::RpcError> {
    let _ = params;
    let i = DevtoolsState::inspector(rt);
    let id = i.session.next_snapshot.get() + 1;
    i.session.next_snapshot.set(id);
    let snapshot = rt.live_snapshot();
    let info = p::SnapshotInfo {
        id,
        taken_ns: rt.now_ns(),
        owners: snapshot.owners.len() as u32,
        objects: snapshot.objects.len() as u32,
    };
    let mut snapshots = i.session.snapshots.borrow_mut();
    if snapshots.len() == SNAPSHOT_CAP {
        snapshots.pop_front();
    }
    snapshots.push_back((id, snapshot));
    Ok(info)
}

pub(super) fn diff_snapshots(
    rt: &Runtime,
    params: p::DiffSnapshotsParams,
) -> Result<p::SnapshotDiff, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    let snapshots = i.session.snapshots.borrow();
    let before = snapshots
        .iter()
        .find(|(id, _)| *id == params.before)
        .map(|(_, snapshot)| snapshot);
    let after = snapshots
        .iter()
        .find(|(id, _)| *id == params.after)
        .map(|(_, snapshot)| snapshot);
    let (Some(before), Some(after)) = (before, after) else {
        return Err(p::RpcError::new(
            p::code::INVALID_PARAMS,
            "unknown snapshot id",
        ));
    };
    let before_owners = owner_entries(before);
    let after_owners = owner_entries(after);
    let before_objects = object_entries(before);
    let after_objects = object_entries(after);
    Ok(p::SnapshotDiff {
        added_owners: missing_entries(&after_owners, &before_owners),
        removed_owners: missing_entries(&before_owners, &after_owners),
        added_objects: missing_entries(&after_objects, &before_objects),
        removed_objects: missing_entries(&before_objects, &after_objects),
    })
}

fn owner_entries(snapshot: &LiveSnapshot) -> BTreeMap<u64, p::OwnerEntry> {
    snapshot
        .owners
        .iter()
        .map(|owner| {
            (
                owner.id.raw(),
                p::OwnerEntry {
                    id: owner.id.raw(),
                    parent: owner.parent.map(|parent| parent.raw()),
                    source: convert::location(owner.created_at),
                },
            )
        })
        .collect()
}

fn object_entries(snapshot: &LiveSnapshot) -> BTreeMap<u64, p::ObjectEntry> {
    snapshot
        .objects
        .iter()
        .map(|object| {
            (
                object.id,
                p::ObjectEntry {
                    id: object.id,
                    owner: object.owner.raw(),
                    name: object.name.to_owned(),
                    source: convert::location(object.created_at),
                },
            )
        })
        .collect()
}

fn missing_entries<T: Clone>(from: &BTreeMap<u64, T>, other: &BTreeMap<u64, T>) -> Vec<T> {
    from.iter()
        .filter(|(id, _)| !other.contains_key(id))
        .map(|(_, entry)| entry.clone())
        .collect()
}

pub(super) fn export_trace(
    rt: &Runtime,
    params: p::ExportTraceParams,
) -> Result<p::ExportedFiles, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    let turns: Vec<p::TurnGraph> = i.session.turns.borrow().iter().cloned().collect();
    let mut frames: Vec<p::FrameInfo> = i
        .session
        .frames
        .borrow()
        .values()
        .flatten()
        .cloned()
        .collect();
    frames.sort_by_key(|frame| frame.end_ns);
    let json = kivact_devtools_protocol::chrome_trace(&turns, &frames);
    let written = i.agent.borrow().write_trace(&json, params.path.as_deref());
    let path = written.map_err(|error| {
        p::RpcError::new(
            p::code::INTERNAL_ERROR,
            format!("cannot write the trace: {error}"),
        )
    })?;
    Ok(p::ExportedFiles {
        paths: vec![path.to_string_lossy().into_owned()],
    })
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
