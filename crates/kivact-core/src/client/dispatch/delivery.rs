//! Role: Deliver prepared host events to the handlers of their kind.
use super::{a11y, common, diagnostics, env, gpu, ime, key, platform, pointer, text};
use crate::view::drawable::events;
use crate::view::state::ViewState;
use kivact_engine::contract::HostEvent;
use kivact_runtime::Runtime;
use kivact_types::{FocusTarget, WindowId};

pub(crate) fn dispatch(rt: &Runtime, host_event: &HostEvent) {
    // An event that names a window already closed has no record to apply to.
    if let Some(window) = event_window(host_event)
        && ViewState::of(rt).window_record(window).is_none()
    {
        return;
    }
    let prepared = prepare_activation(rt, host_event);
    let host_event = prepared.as_ref().unwrap_or(host_event);
    common::track_modality(rt, host_event);
    common::track_modifiers(rt, host_event);
    record_input(rt, host_event);
    if focus_event(rt, host_event) {
        return;
    }
    deliver(rt, host_event);
}

/// Record a pointer, wheel, key, IME or file event for the DevTools input log.
fn record_input(rt: &Runtime, host_event: &HostEvent) {
    let window = match host_event {
        HostEvent::Pointer(route) => route.window,
        HostEvent::Wheel(route) => route.window,
        HostEvent::File(route) => route.window,
        HostEvent::Key(route) => route.window,
        HostEvent::Ime { node, .. } => {
            let Some(window) = ViewState::tree(rt).reader().window_of(*node) else {
                return;
            };
            window
        }
        _ => return,
    };
    crate::view::devtools_hooks::input_event(rt, host_event, window);
}

fn deliver(rt: &Runtime, host_event: &HostEvent) {
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
            if let Some(target) = route.target {
                pointer::file(
                    rt,
                    host_event,
                    target.node,
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
        HostEvent::HoverChanged { window, target } => pointer::hover_changed(rt, *window, *target),
        HostEvent::MousePosition { window, position } => {
            crate::view::hover::set_pointer_position(rt, *window, *position)
        }
        HostEvent::Key(_) => key::key(rt, host_event),
        // Recorded by `track_modifiers` above; nothing else to deliver.
        HostEvent::ModifiersChanged { .. } => {}
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
        HostEvent::ExitComplete { node } => {
            let callback = crate::view::state::ViewState::tree_mut(rt).take_exit_callback(*node);
            if let Some(callback) = callback
                && let Err(error) = callback.call(())
            {
                rt.report(None, error);
            }
        }
        HostEvent::A11y {
            node,
            region,
            request,
        } => a11y::a11y(rt, host_event, *node, *region, request),
        _ => dispatch_host(rt, host_event),
    }
}

/// Deliver a focus change or result; `false` for every other event.
fn focus_event(rt: &Runtime, host_event: &HostEvent) -> bool {
    match host_event {
        HostEvent::FocusChanged {
            window,
            node,
            region,
            visible,
        } => {
            text::focus_changed(rt, *window, *node, *region, *visible);
            crate::view::devtools_hooks::focus_changed(
                rt,
                *window,
                node.map(|node| FocusTarget {
                    node,
                    region: *region,
                }),
                *visible,
            );
        }
        HostEvent::FocusResult { request, result } => {
            text::focus_result(rt, *request, result.clone());
            crate::view::devtools_hooks::focus_result(rt);
        }
        _ => return false,
    }
    true
}

/// The window an event applies to, when it names one.
fn event_window(event: &HostEvent) -> Option<WindowId> {
    match event {
        HostEvent::Pointer(routed) => Some(routed.window),
        HostEvent::Wheel(routed) => Some(routed.window),
        HostEvent::File(routed) => Some(routed.window),
        HostEvent::Key(routed) => Some(routed.window),
        HostEvent::ModifiersChanged { window, .. }
        | HostEvent::FocusChanged { window, .. }
        | HostEvent::HoverChanged { window, .. }
        | HostEvent::MousePosition { window, .. }
        | HostEvent::EnvChanged { window, .. }
        | HostEvent::HostFailure { window, .. }
        | HostEvent::GpuAdapterChanged { window, .. }
        | HostEvent::CloseRequested { window }
        | HostEvent::WindowOpened { window, .. }
        | HostEvent::WindowFailed { window, .. }
        | HostEvent::InjectDelivered { window, .. } => Some(*window),
        // App-wide events.
        HostEvent::FrameTick { .. } | HostEvent::AppMotion { .. } => None,
        _ => None,
    }
}

fn dispatch_host(rt: &Runtime, host_event: &HostEvent) {
    if dispatch_export_host(rt, host_event) || dispatch_window_host(rt, host_event) {
        return;
    }
    match host_event {
        HostEvent::DuplicateLayoutId { node, id } => {
            diagnostics::duplicate_layout_id(rt, *node, id)
        }
        HostEvent::EnvChanged {
            window,
            environment,
            surface_active,
        } => env::env_changed(rt, *window, environment, *surface_active),
        HostEvent::FrameTick {
            now_ns,
            estimated_present_ns,
            clock,
            ..
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
        } => env::layout_changed(rt, *node, *rect, *size, *text_truncated, *layout),
        HostEvent::EffectResult { request, result } => {
            platform::effect_result(rt, *request, result)
        }
        HostEvent::SnapshotResult { request, result } => {
            crate::view::snapshots::resolve(rt, *request, result)
        }
        HostEvent::ImageStatusChanged { node, status } => set_image_status(rt, *node, *status),
        HostEvent::AssetFailed { node, message } => platform::asset_failed(rt, *node, message),
        HostEvent::HostFailure {
            revision, reason, ..
        } => platform::host_failure(rt, *revision, reason),
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
        HostEvent::DrawableNotHosted { .. } | HostEvent::DrawableDrawnTwice { .. } => {
            dispatch_drawable_report(rt, host_event)
        }
        HostEvent::TurnsCut { turns } => diagnostics::turns_cut(rt, *turns),
        HostEvent::DevtoolsPick { .. } | HostEvent::DevtoolsKey { .. } => {
            crate::view::devtools_hooks::host_event(rt, host_event)
        }
        _ => unreachable!("invariant: input and routed events are dispatched before host events"),
    }
}

/// The window lifecycle events; `false` for every other event.
fn dispatch_window_host(rt: &Runtime, host_event: &HostEvent) -> bool {
    match host_event {
        HostEvent::CloseRequested { window } => env::close_requested(rt, *window),
        HostEvent::WindowOpened {
            window,
            environment,
            surface_active,
        } => env::window_opened(rt, *window, environment, *surface_active),
        HostEvent::WindowFailed { window, reason } => env::window_failed(rt, *window, reason),
        HostEvent::InjectDelivered { .. } => {
            crate::view::devtools_hooks::host_event(rt, host_event)
        }
        HostEvent::AppMotion {
            drawable,
            frame_period_ns,
        } => {
            rt.enter(|| {
                ViewState::of(rt)
                    .app_motion
                    .set(crate::view::state::AppMotion {
                        drawable: *drawable,
                        frame_period_ns: *frame_period_ns,
                    })
            });
        }
        _ => return false,
    }
    true
}

/// Report a drawable the engine could not place: a drawing in a node that
/// does not host it, or a drawable drawn in both slots.
fn dispatch_drawable_report(rt: &Runtime, host_event: &HostEvent) {
    match host_event {
        HostEvent::DrawableNotHosted { node, drawable } => {
            diagnostics::drawable_not_hosted(rt, *node, *drawable)
        }
        HostEvent::DrawableDrawnTwice { node, drawable } => {
            diagnostics::drawable_drawn_twice(rt, *node, *drawable)
        }
        _ => unreachable!("invariant: only drawable reports are routed here"),
    }
}

/// Deliver the GPU and drawable export events of the host; `false` for every
/// other event.
fn dispatch_export_host(rt: &Runtime, host_event: &HostEvent) -> bool {
    match host_event {
        HostEvent::ExternalImported { image, result } => gpu::external_imported(rt, *image, result),
        HostEvent::GpuAdapterChanged { window, adapter } => {
            gpu::adapter_changed(rt, *window, adapter.as_ref())
        }
        HostEvent::DrawableFrame(frame) => events::frame(frame),
        HostEvent::DrawableResized { id } => events::resized(*id),
        HostEvent::DrawableTextures {
            id,
            generation,
            result,
        } => events::textures(*id, *generation, result),
        HostEvent::DrawableExportLost(loss) => events::lost(loss.sink, loss.generation),
        HostEvent::GpuExportAvailable => events::gpu_available(),
        _ => return false,
    }
    true
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
