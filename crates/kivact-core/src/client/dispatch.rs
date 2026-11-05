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
//! Modifier changes are only recorded (`view::modifiers`).
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
mod gpu;
mod ime;
mod key;
mod platform;
mod pointer;
mod text;

use kivact_engine::contract::HostEvent;
use kivact_runtime::Runtime;

pub(crate) use pointer::run_requested_activations;

pub(crate) fn dispatch(rt: &Runtime, host_event: &HostEvent) {
    let mut prepared;
    let host_event = if matches!(
        host_event,
        HostEvent::Activate {
            pointer: Some(_),
            ..
        }
    ) {
        prepared = host_event.clone();
        crate::view::state::ViewState::tree(rt)
            .engine()
            .prepare_input(&mut prepared);
        &prepared
    } else {
        host_event
    };
    common::track_modality(rt, host_event);
    common::track_modifiers(rt, host_event);
    match host_event {
        HostEvent::PresenceChanged { node, presence } => {
            let source = crate::view::state::ViewState::tree(rt)
                .get(*node)
                .and_then(|node| node.presence);
            if let Some(source) = source {
                source.set(*presence);
            }
        }
        HostEvent::Pointer(route) => pointer::pointer(rt, host_event, *route),
        HostEvent::Wheel(route) => pointer::wheel(rt, host_event, *route),
        HostEvent::File(route) => {
            if let Some(node) = route.target {
                pointer::file(
                    rt,
                    host_event,
                    node,
                    route.input.phase,
                    &route.input.files,
                    route.input.position,
                );
            }
        }
        HostEvent::Activate {
            node,
            region,
            pointer,
        } => pointer::activate_event(rt, host_event, *node, *region, *pointer),
        HostEvent::HoverChanged { target } => pointer::hover_changed(rt, *target),
        HostEvent::MousePosition(position) => {
            crate::view::hover::set_pointer_position(rt, *position)
        }
        HostEvent::Key(_) => key::key(rt, host_event),
        // Recorded by `track_modifiers` above; nothing else to deliver.
        HostEvent::ModifiersChanged(_) => {}
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
        HostEvent::ExitComplete { node } => {
            let callback = crate::view::state::ViewState::tree_mut(rt).take_exit_callback(*node);
            if let Some(callback) = callback
                && let Err(error) = callback.call(())
            {
                rt.report(None, error);
            }
        }
        HostEvent::FocusResult { request, result } => {
            text::focus_result(rt, *request, result.clone())
        }
        HostEvent::A11y {
            node,
            region,
            request,
        } => a11y::a11y(rt, host_event, *node, *region, request),
        HostEvent::EnvChanged {
            environment,
            surface_active,
        } => env::env_changed(rt, environment, *surface_active),
        HostEvent::FrameTick {
            now_ns,
            estimated_present_ns,
            clock,
        } => env::frame_tick(rt, *now_ns, *estimated_present_ns, clock),
        HostEvent::ScrollChanged {
            node,
            offset,
            viewport,
            content,
        } => env::scroll_changed(rt, *node, *offset, *viewport, *content),
        HostEvent::LayoutChanged {
            node,
            rect,
            size,
            text_truncated,
        } => env::layout_changed(rt, *node, *rect, *size, *text_truncated),
        HostEvent::CloseRequested => env::close_requested(rt),
        HostEvent::EffectResult { request, result } => {
            platform::effect_result(rt, *request, result)
        }
        HostEvent::ImageStatusChanged { node, status } => {
            let observation = crate::view::state::ViewState::tree(rt)
                .get(*node)
                .and_then(|node| node.observation);
            if let Some(observation) = observation {
                observation.image_status.set(*status);
            }
        }
        HostEvent::AssetFailed { node, message } => platform::asset_failed(rt, *node, message),
        HostEvent::HostFailure { revision, reason } => {
            platform::host_failure(rt, *revision, reason)
        }
        HostEvent::Lifecycle(event) => {
            if *event == kivact_types::LifecycleEvent::MemoryWarning {
                platform::memory_warning(rt);
            }
            crate::platform::lifecycle::dispatch(rt, *event);
        }
        HostEvent::LayoutWarning { node, rule, amount } => {
            diagnostics::layout_warning(rt, *node, *rule, *amount)
        }
        HostEvent::DrawDropped { node, count } => diagnostics::draw_dropped(rt, *node, *count),
        HostEvent::TurnsCut { turns } => diagnostics::turns_cut(rt, *turns),
        HostEvent::ExternalImported { image, result } => gpu::external_imported(rt, *image, result),
        HostEvent::GpuAdapterChanged(adapter) => gpu::adapter_changed(rt, adapter.as_ref()),
    }
}
