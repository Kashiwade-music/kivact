//! Role: Animations, the animation clock and task lists of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_engine::contract::HostRequest;
use kivact_runtime::{ErrorKind, MotionState, Runtime, TaskState};

use super::convert;
use crate::view::requests::request_effect;
use crate::view::state::EffectWaiter;

/// Send one DevTools effect to the engine through the view's request counter.
fn engine_effect(rt: &Runtime, request: HostRequest) {
    let waiter = EffectWaiter::Report {
        owner: None,
        at: std::panic::Location::caller(),
        failed: ErrorKind::Platform,
    };
    request_effect(rt, request, waiter);
}

pub(super) fn list_animations(rt: &Runtime, _: p::Empty) -> Result<p::AnimationList, p::RpcError> {
    let motion = MotionState::of(rt);
    let animations = motion
        .animations()
        .into_iter()
        .enumerate()
        .map(|(index, info)| p::AnimationInfo {
            id: index as u64,
            kind: match info.kind {
                kivact_runtime::AnimKind::Animated => p::AnimationKind::Animated,
                kivact_runtime::AnimKind::Timeline => p::AnimationKind::Timeline,
                kivact_runtime::AnimKind::FrameLoop => p::AnimationKind::FrameLoop,
            },
            source: info.created_at.map(convert::location),
            owner: info.owner.raw(),
            running: info.is_running(),
            paused: info.is_paused(),
            decorative: info.decorative,
        })
        .collect();
    Ok(p::AnimationList {
        animations,
        time_scale: motion.time_scale(),
    })
}

pub(super) fn set_time_scale(
    rt: &Runtime,
    params: p::TimeScaleParams,
) -> Result<p::Empty, p::RpcError> {
    if !(0.0..=4.0).contains(&params.scale) {
        return Err(p::RpcError::new(
            p::code::INVALID_PARAMS,
            "time scale must be between 0 and 4",
        ));
    }
    MotionState::of(rt).set_time_scale(rt, params.scale);
    engine_effect(rt, HostRequest::TimeScale(params.scale));
    Ok(p::Empty {})
}

pub(super) fn list_tasks(rt: &Runtime, _: p::Empty) -> Result<p::TaskList, p::RpcError> {
    let state = TaskState::of(rt);
    let mut tasks = Vec::new();
    for (index, timer) in state.list().into_iter().enumerate() {
        tasks.push(p::TaskInfo {
            id: index as u64,
            kind: p::TaskKind::Timer,
            source: Some(convert::location(timer.created_at)),
            owner: timer.owner.map(|owner| owner.raw()),
            phase: p::AsyncPhase::Pending,
            deadline_ns: Some(timer.deadline_ns),
        });
    }
    for object in rt.live_snapshot().objects {
        let kind = match object.name {
            "Task" => p::TaskKind::Task,
            "Resource" => p::TaskKind::Resource,
            "Action" => p::TaskKind::Action,
            _ => continue,
        };
        tasks.push(p::TaskInfo {
            id: object.id,
            kind,
            source: Some(convert::location(object.created_at)),
            owner: Some(object.owner.raw()),
            phase: p::AsyncPhase::Idle,
            deadline_ns: None,
        });
    }
    Ok(p::TaskList {
        tasks,
        outstanding_blocking: state.stats().outstanding_blocking as u32,
    })
}

/// Restore the animation clock to its normal speed.
pub(super) fn reset_time_scale(rt: &Runtime) {
    MotionState::of(rt).set_time_scale(rt, 1.0);
    engine_effect(rt, HostRequest::TimeScale(1.0));
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::ListAnimations(params) => {
            reply.send(list_animations(rt, params).map(p::Reply::ListAnimations))
        }
        p::Call::SetTimeScale(params) => {
            reply.send(set_time_scale(rt, params).map(p::Reply::SetTimeScale))
        }
        p::Call::ListTasks(params) => reply.send(list_tasks(rt, params).map(p::Reply::ListTasks)),
        other => return Some((other, reply)),
    }
    None
}
