//! Role: Style overrides, forced states and theme tokens of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_runtime::Runtime;
use kivact_types::NodeId;

fn unsupported<T>() -> Result<T, p::RpcError> {
    Err(p::RpcError::new(
        p::code::UNSUPPORTED,
        "not implemented yet",
    ))
}

/// The style layers of `id`, highest priority first.
pub(super) fn layers(_rt: &Runtime, _id: NodeId) -> Vec<p::StyleLayerInfo> {
    Vec::new()
}

/// Remove every override and forced state.
pub(super) fn clear_all(_rt: &Runtime) {}

pub(super) fn set_style_override(
    rt: &Runtime,
    params: p::SetStyleOverrideParams,
) -> Result<p::StyleOverride, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn clear_style_overrides(
    rt: &Runtime,
    params: p::ClearStyleOverridesParams,
) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn list_style_overrides(
    rt: &Runtime,
    params: p::Empty,
) -> Result<p::StyleOverrideList, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn force_state(
    rt: &Runtime,
    params: p::ForceStateParams,
) -> Result<p::Empty, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn list_theme_tokens(
    rt: &Runtime,
    params: p::Empty,
) -> Result<p::ThemeTokenList, p::RpcError> {
    let _ = (rt, params);
    unsupported()
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::SetStyleOverride(params) => {
            reply.send(set_style_override(rt, params).map(p::Reply::SetStyleOverride))
        }
        p::Call::ClearStyleOverrides(params) => {
            reply.send(clear_style_overrides(rt, params).map(p::Reply::ClearStyleOverrides))
        }
        p::Call::ListStyleOverrides(params) => {
            reply.send(list_style_overrides(rt, params).map(p::Reply::ListStyleOverrides))
        }
        p::Call::ForceState(params) => {
            reply.send(force_state(rt, params).map(p::Reply::ForceState))
        }
        p::Call::ListThemeTokens(params) => {
            reply.send(list_theme_tokens(rt, params).map(p::Reply::ListThemeTokens))
        }
        other => return Some((other, reply)),
    }
    None
}
