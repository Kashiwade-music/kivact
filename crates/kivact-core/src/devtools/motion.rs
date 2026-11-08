//! Role: Animations, the animation clock and task lists of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn list_animations(
    rt: &Runtime,
    params: p::Empty,
) -> Result<p::AnimationList, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn set_time_scale(
    rt: &Runtime,
    params: p::TimeScaleParams,
) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn list_tasks(rt: &Runtime, params: p::Empty) -> Result<p::TaskList, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

/// Restore the animation clock to its normal speed.
pub(super) fn reset_time_scale(_rt: &Runtime) {}

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
