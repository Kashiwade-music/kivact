//! Role: Highlight, overlays, pick mode and screenshots of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_engine::contract::{EngineRecords, PickPhase};
use kivact_runtime::Runtime;
use kivact_types::{NodeId, WindowId};

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

pub(super) fn highlight(rt: &Runtime, params: p::HighlightParams) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn set_overlays(
    rt: &Runtime,
    params: p::OverlaySettings,
) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn get_overlays(
    rt: &Runtime,
    params: p::Empty,
) -> Result<p::OverlaySettings, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn start_pick(
    rt: &Runtime,
    params: p::StartPickParams,
) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn stop_pick(rt: &Runtime, params: p::Empty) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn screenshot(rt: &Runtime, params: p::ScreenshotParams, reply: Responder) {
    let _ = (rt, params);
    reply.send(unsupported());
}

/// Report one pick event to the DevTools and the App.
pub(super) fn pick_event(
    _rt: &Runtime,
    _window: WindowId,
    _node: Option<NodeId>,
    _phase: PickPhase,
) {
}

/// Begin pick mode in `window`.
pub(super) fn start_pick_in(_rt: &Runtime, _window: WindowId) {}

/// Rebuild the overlays of every window from the current settings.
pub(super) fn after_turn(_rt: &Runtime, _records: &EngineRecords) {}

/// Stop pick, highlight and every overlay.
pub(super) fn clear(_rt: &Runtime) {}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::Highlight(params) => reply.send(highlight(rt, params).map(p::Reply::Highlight)),
        p::Call::SetOverlays(params) => {
            reply.send(set_overlays(rt, params).map(p::Reply::SetOverlays))
        }
        p::Call::GetOverlays(params) => {
            reply.send(get_overlays(rt, params).map(p::Reply::GetOverlays))
        }
        p::Call::StartPick(params) => reply.send(start_pick(rt, params).map(p::Reply::StartPick)),
        p::Call::StopPick(params) => reply.send(stop_pick(rt, params).map(p::Reply::StopPick)),
        p::Call::Screenshot(params) => screenshot(rt, params, reply),
        other => return Some((other, reply)),
    }
    None
}
