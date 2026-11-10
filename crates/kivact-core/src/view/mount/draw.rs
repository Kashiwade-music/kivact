//! Role: Mounting binds the draw layers of a host node.

use std::panic::Location;
use std::rc::Rc;

use kivact_engine::contract::SceneLayer;
use kivact_runtime::{EffectKind, Signal, UiError};
use kivact_style::draw::{Recording, Size};
use kivact_types::NodeId;

use super::facts::LocalPaintSource;
use crate::view::descriptor::Mcx;
use crate::view::element_data::{DrawFn, LayoutDrawFn, Registry};
use crate::view::state::ViewState;

/// Stage 5's input: the draw layers, the style's local paint and the canvas regions.
pub(super) struct DrawInput {
    // xtask-allow(state-fields): option: a draw layer below the children; absent means none
    pub(super) below: Option<DrawFn>,
    // xtask-allow(state-fields): option: a draw layer over the children; absent means none
    pub(super) over: Option<DrawFn>,
    // xtask-allow(state-fields): option: a layout-driven draw; absent means none
    pub(super) with_layout: Option<LayoutDrawFn>,
    // xtask-allow(state-fields): option: the style's local paint; absent means none
    pub(super) local_paint: Option<LocalPaintSource>,
    // xtask-allow(state-fields): option: the canvas regions the node declares; absent means none
    pub(super) regions: Option<Rc<Registry>>,
    /// The node's layout size, written by the layout observer `resolve` installed.
    pub(super) size: Signal<Size>,
}

/// Stage 5: bind the node's draw effect: `.draw`, `.draw_over` and the local
/// paint of its style, with the regions its canvas declares. The layers are
/// re-recorded when the node's size or anything they read changes.
pub(super) fn bind(cx: &Mcx<'_>, id: NodeId, input: DrawInput, at: &'static Location<'static>) {
    let owner = cx.owner;
    let DrawInput {
        below,
        over,
        with_layout,
        local_paint,
        regions,
        size,
    } = input;
    let theme = cx.theme();
    let observation = with_layout
        .as_ref()
        .and_then(|_| ViewState::tree_mut(cx.rt).observe_node(id, at));
    for (layer, f, paint, layout_fn) in [
        (SceneLayer::Draw, below, local_paint, None),
        (SceneLayer::DrawOver, over, None, with_layout),
    ] {
        if f.is_none() && paint.is_none() && layout_fn.is_none() {
            continue;
        }
        let region_paint = regions.clone();
        let region_reported = kivact_types::cell::ReportOnceSet::new();
        let weak = cx.weak();
        let reported = kivact_types::cell::ReportOnce::new();
        cx.effect(EffectKind::Render, at, move || {
            let sz = size.get();
            let theme = theme.map(|t| t.get()).unwrap_or_default();
            let measurer = weak
                .upgrade()
                .map(|rt| ViewState::tree_mut(&rt).reader().text_measurer(id));
            let layout = observation.and_then(|observation| {
                observation.editing_revision.track();
                observation.measured.get();
                observation.scroll_geometry.get();
                let rt = weak.upgrade()?;
                ViewState::tree_mut(&rt).reader().editing_canvas_layout(id)
            });
            let Recording {
                scene,
                unsupported: issues,
                region_errors,
            } = kivact_style::internal::record(sz, &theme, |d| {
                if let Some(measurer) = measurer {
                    d.set_text_measurer(measurer);
                }
                if let Some(regions) = &region_paint {
                    d.declare_regions(regions.resolve());
                }
                if let Some(f) = &f {
                    f(d, sz);
                }
                if let (Some(f), Some(layout)) = (&layout_fn, &layout) {
                    f(d, layout);
                }
                if let Some(source) = &paint {
                    source.revision.track();
                    // The storage borrow ends before the callback, which is
                    // user code.
                    let compiled = source.value.borrow().clone();
                    if let Some(compiled) = compiled {
                        compiled.paint_retained(d, sz);
                    }
                }
            });
            if let Some(rt) = weak.upgrade() {
                ViewState::tree_mut(&rt).set_scene(id, layer, scene);
                for message in region_errors {
                    if region_reported.note(message.clone()) {
                        rt.report(
                            Some(owner),
                            UiError::new(kivact_runtime::ErrorKind::Message, message, Some(at)),
                        );
                    }
                }
                report_unsupported(&rt, owner, id, issues, &reported, at);
            }
        });
    }
}

/// Bind the draw effect of a styled box without draw layers when its style
/// first has a paint callback (`LocalPaint::OnFirstPaint`): the layout size
/// observer and the Draw layer's effect that `bind` makes at mount for a
/// box with draw layers. `local_paint` already holds the callback.
pub(super) fn bind_style_paint(
    cx: &Mcx<'_>,
    id: NodeId,
    local_paint: LocalPaintSource,
    at: &'static Location<'static>,
) {
    let size = cx.owner.signal(Size::new(0.0, 0.0), at);
    if let Some(handlers) = ViewState::tree_mut(cx.rt).handlers_mut(id) {
        handlers.observe_layout(move |_, s| size.set(s));
    }
    let input = DrawInput {
        below: None,
        over: None,
        with_layout: None,
        local_paint: Some(local_paint),
        regions: None,
        size,
    };
    bind(cx, id, input, at);
}

fn report_unsupported(
    rt: &kivact_runtime::Runtime,
    owner: kivact_runtime::Owner,
    id: NodeId,
    issues: u32,
    reported: &kivact_types::cell::ReportOnce,
    at: &'static Location<'static>,
) {
    if issues > 0 && reported.note() {
        rt.note(format!("node {}: {issues} draw commands unsupported (invalid pixels or box-only shadow values)", id.0));
        rt.report(
                        Some(owner),
                        UiError::new(
                            kivact_runtime::ErrorKind::Unsupported,
                            format!("{issues} draw command(s) contain invalid RGBA pixels or a canvas shadow with spread/inset; unsupported values dropped"),
                            Some(at),
                        ),
                    );
    }
}
