//! Role: Deliver prepared host events to the handlers of their kind.
use super::{a11y, common, diagnostics, env, gpu, ime, key, platform, pointer, text};
use crate::view::drawable::events;
use kivact_engine::contract::HostEvent;
use kivact_runtime::Runtime;

pub(crate) fn dispatch(rt: &Runtime, host_event: &HostEvent) {
    let prepared = prepare_activation(rt, host_event);
    let host_event = prepared.as_ref().unwrap_or(host_event);
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
        _ => dispatch_host(rt, host_event),
    }
}

fn dispatch_host(rt: &Runtime, host_event: &HostEvent) {
    match host_event {
        HostEvent::DuplicateLayoutId { node, id } => {
            diagnostics::duplicate_layout_id(rt, *node, id)
        }
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
            layout,
        } => {
            set_layout_rect(rt, *node, *layout);
            env::layout_changed(rt, *node, *rect, *size, *text_truncated)
        }
        HostEvent::CloseRequested => env::close_requested(rt),
        HostEvent::EffectResult { request, result } => {
            platform::effect_result(rt, *request, result)
        }
        HostEvent::SnapshotResult { request, result } => {
            crate::view::snapshots::resolve(rt, *request, result)
        }
        HostEvent::ImageStatusChanged { node, status } => set_image_status(rt, *node, *status),
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
        HostEvent::DrawableNotHosted { node, drawable } => {
            diagnostics::drawable_not_hosted(rt, *node, *drawable)
        }
        HostEvent::TurnsCut { turns } => diagnostics::turns_cut(rt, *turns),
        HostEvent::ExternalImported { .. }
        | HostEvent::GpuAdapterChanged(_)
        | HostEvent::DrawableFrame(_)
        | HostEvent::DrawableResized { .. }
        | HostEvent::DrawableTextures { .. }
        | HostEvent::DrawableExportLost(_)
        | HostEvent::GpuExportAvailable => dispatch_export_host(rt, host_event),
        HostEvent::PresenceChanged { .. }
        | HostEvent::Pointer(_)
        | HostEvent::Wheel(_)
        | HostEvent::File(_)
        | HostEvent::Activate { .. }
        | HostEvent::HoverChanged { .. }
        | HostEvent::MousePosition(_)
        | HostEvent::Key(_)
        | HostEvent::ModifiersChanged(_)
        | HostEvent::TextChanged { .. }
        | HostEvent::SelectionChanged { .. }
        | HostEvent::Ime { .. }
        | HostEvent::FocusChanged { .. }
        | HostEvent::ExitComplete { .. }
        | HostEvent::FocusResult { .. }
        | HostEvent::A11y { .. } => unreachable!("invariant: input event already dispatched"),
    }
}

/// Deliver the GPU and drawable export events of the host.
fn dispatch_export_host(rt: &Runtime, host_event: &HostEvent) {
    match host_event {
        HostEvent::ExternalImported { image, result } => gpu::external_imported(rt, *image, result),
        HostEvent::GpuAdapterChanged(adapter) => gpu::adapter_changed(rt, adapter.as_ref()),
        HostEvent::DrawableFrame(frame) => events::frame(frame),
        HostEvent::DrawableResized { id } => events::resized(*id),
        HostEvent::DrawableTextures {
            id,
            generation,
            result,
        } => events::textures(*id, *generation, result),
        HostEvent::DrawableExportLost(loss) => events::lost(loss.sink, loss.generation),
        HostEvent::GpuExportAvailable => events::gpu_available(),
        _ => unreachable!("invariant: only export events are routed here"),
    }
}

/// Record the image status of an observed node for the views that read it.
fn set_image_status(rt: &Runtime, node: kivact_types::NodeId, status: kivact_types::ImageStatus) {
    let observation = crate::view::state::ViewState::tree(rt)
        .get(node)
        .and_then(|node| node.observation);
    if let Some(observation) = observation {
        observation.image_status.set(status);
    }
}

/// Record the window layout box of an observed node for the scroll-driven
/// progress values that read it.
fn set_layout_rect(rt: &Runtime, node: kivact_types::NodeId, layout: Option<kivact_types::Rect>) {
    let observation = crate::view::state::ViewState::tree(rt)
        .get(node)
        .and_then(|node| node.observation);
    if let Some(observation) = observation {
        observation.layout_rect.set(layout);
    }
}

fn prepare_activation(rt: &Runtime, event: &HostEvent) -> Option<HostEvent> {
    if !matches!(
        event,
        HostEvent::Activate {
            pointer: Some(_),
            ..
        }
    ) {
        return None;
    }
    let mut prepared = event.clone();
    crate::view::state::ViewState::tree(rt)
        .engine()
        .prepare_input(&mut prepared);
    Some(prepared)
}
