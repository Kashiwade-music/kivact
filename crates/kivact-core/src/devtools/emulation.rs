//! Role: The environment emulation of the DevTools target.

use kivact_devtools_agent::Responder;
use kivact_devtools_protocol as p;
use kivact_engine::contract::{EngineEmulation, HostRequest};
use kivact_runtime::{ErrorKind, Runtime};
use kivact_types::ThemeHint;

use crate::view::devtools_state::{DevtoolsState, emulation_off};
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

pub(super) fn get_emulation(rt: &Runtime, _: p::Empty) -> Result<p::Emulation, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    Ok(i.hooks.emulation.get_untracked())
}

pub(super) fn set_emulation(
    rt: &Runtime,
    params: p::Emulation,
) -> Result<p::Emulation, p::RpcError> {
    let i = DevtoolsState::inspector(rt);
    i.hooks.emulation.set(params.clone());
    let emulation = EngineEmulation {
        theme_hint: params.theme.map(|theme| match theme {
            p::ThemeEmulation::Light => ThemeHint::Light,
            p::ThemeEmulation::Dark => ThemeHint::Dark,
        }),
        scale: params.scale,
        touch: params.touch,
    };
    engine_effect(rt, HostRequest::Emulate(emulation));
    engine_effect(rt, HostRequest::ReducedMotion(params.reduced_motion));
    Ok(params)
}

/// Restore every emulated value to the App's own environment.
pub(super) fn clear(rt: &Runtime) {
    let i = DevtoolsState::inspector(rt);
    i.hooks.emulation.set(emulation_off());
    engine_effect(rt, HostRequest::Emulate(EngineEmulation::default()));
    engine_effect(rt, HostRequest::ReducedMotion(None));
}

pub(super) fn route(rt: &Runtime, call: p::Call, reply: Responder) -> Option<(p::Call, Responder)> {
    match call {
        p::Call::GetEmulation(params) => {
            reply.send(get_emulation(rt, params).map(p::Reply::GetEmulation))
        }
        p::Call::SetEmulation(params) => {
            reply.send(set_emulation(rt, params).map(p::Reply::SetEmulation))
        }
        other => return Some((other, reply)),
    }
    None
}
