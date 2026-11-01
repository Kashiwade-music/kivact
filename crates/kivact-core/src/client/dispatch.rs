//! Role: Host event delivery.
//! `dispatch` only routes each [`HostEvent`] to the module of its kind
//! (`pointer`, `key`, `text`, `a11y`, `env`, `platform`, `diagnostics`,
//! `gpu`); `common` holds what they share.
//!
//! Events for nodes that no longer exist are stale and
//! dropped (recorded in diagnostics). An asset failure is reported once per
//! node and message as `ErrorKind::Asset`, located where its node was
//! created.
//!
//! While overlays are shown, the engine decides which events are admitted
//! (`Reader::admit_input`, asked once per event right before it is
//! dispatched), where key, pointer and wheel handler chains stop and which
//! overlay an unhandled Escape dismisses (`crate::view::overlays`); these
//! modules apply its answers. Events they do not admit are dropped and
//! counted like stale events.

mod a11y;
mod common;
mod diagnostics;
mod env;
#[cfg(feature = "gpu-interop")]
mod gpu;
mod ime;
mod key;
mod platform;
mod pointer;
mod text;

use kivact_reactive::Runtime;
use kivact_wire::HostEvent;

pub(super) use pointer::run_requested_activations;

pub(crate) fn dispatch(rt: &Runtime, host_event: &HostEvent) {
    common::track_modality(rt, host_event);
    match host_event {
        HostEvent::Pointer {
            node,
            region,
            event,
        } => pointer::pointer(rt, host_event, *node, *region, *event),
        HostEvent::Wheel {
            node,
            region,
            delta,
            pos,
            modifiers,
        } => pointer::wheel(rt, host_event, *node, *region, *delta, *pos, *modifiers),
        HostEvent::File {
            node,
            phase,
            files,
            window_pos,
            ..
        } => pointer::file(rt, host_event, *node, *phase, files, *window_pos),
        HostEvent::Activate {
            node,
            region,
            pointer,
        } => pointer::activate_event(rt, host_event, *node, *region, *pointer),
        HostEvent::HoverChanged { node } => pointer::hover_changed(rt, *node),
        HostEvent::Key { .. } => key::key(rt, host_event),
        HostEvent::TextChanged {
            node,
            changes,
            session,
            text,
        } => text::text_changed(rt, host_event, *node, changes, session, text.as_deref()),
        HostEvent::SelectionChanged { node, session } => {
            text::selection_changed(rt, host_event, *node, session)
        }
        HostEvent::Ime { node, event } => ime::ime(rt, host_event, *node, event),
        HostEvent::FocusChanged {
            node,
            region,
            visible,
        } => text::focus_changed(rt, *node, *region, *visible),
        HostEvent::FocusResult {
            request,
            ok,
            reason,
        } => text::focus_result(rt, *request, *ok, reason.as_deref()),
        HostEvent::A11y {
            node,
            region,
            action,
            value,
        } => a11y::a11y(rt, host_event, *node, *region, *action, value.as_deref()),
        HostEvent::OverlayDismiss { node } => a11y::overlay_dismiss(rt, host_event, *node),
        HostEvent::EnvChanged(env) => env::env_changed(rt, env),
        HostEvent::FrameTick {
            now_ns,
            estimated_present_ns,
            clock,
        } => env::frame_tick(rt, *now_ns, *estimated_present_ns, clock),
        HostEvent::ScrollChanged {
            node,
            offset,
            viewport,
            ..
        } => env::scroll_changed(rt, *node, *offset, *viewport),
        HostEvent::LayoutChanged { node, rect, size } => {
            env::layout_changed(rt, *node, *rect, *size)
        }
        HostEvent::CloseRequested => env::close_requested(rt),
        HostEvent::EffectResult { request, result } => {
            platform::effect_result(rt, *request, result)
        }
        HostEvent::AssetFailed { node, message } => platform::asset_failed(rt, *node, message),
        HostEvent::HostFailure { revision, reason } => {
            platform::host_failure(rt, *revision, reason)
        }
        HostEvent::Lifecycle(event) => {
            if *event == kivact_wire::LifecycleEvent::MemoryWarning {
                platform::memory_warning(rt);
            }
            crate::platform::lifecycle::dispatch(rt, *event);
        }
        HostEvent::LayoutWarning { node, rule, amount } => {
            diagnostics::layout_warning(rt, *node, *rule, *amount)
        }
        HostEvent::DrawDropped { node, count } => diagnostics::draw_dropped(rt, *node, *count),
        HostEvent::TurnsCut { turns } => diagnostics::turns_cut(rt, *turns),
        #[cfg(feature = "gpu-interop")]
        HostEvent::ExternalImported { image, result } => gpu::external_imported(rt, *image, result),
        #[cfg(feature = "gpu-interop")]
        HostEvent::GpuAdapterChanged(adapter) => gpu::adapter_changed(rt, adapter.as_ref()),
        // A host with gpu-interop talking to a client without it.
        #[cfg(not(feature = "gpu-interop"))]
        HostEvent::ExternalImported { .. } | HostEvent::GpuAdapterChanged(_) => {}
    }
}
