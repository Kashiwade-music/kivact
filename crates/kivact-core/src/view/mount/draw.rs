//! Role: Mounting binds the draw layers of a host node.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_reactive::{EffectKind, Signal, UiError};
use kivact_style::draw::{Draw, Recording, Size};
use kivact_wire::{NodeId, SceneLayer};

use super::facts::LocalPaintSource;
use crate::view::common::{DrawFn, Registry};
use crate::view::descriptor::Mcx;
use crate::view::state::ViewState;

/// Stage 5's input: the draw layers, the style's local paint and the canvas regions.
pub(super) struct DrawInput {
    pub(super) below: Option<DrawFn>,
    pub(super) over: Option<DrawFn>,
    pub(super) local_paint: Option<LocalPaintSource>,
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
        local_paint,
        regions,
        size,
    } = input;
    let theme = cx.theme();
    for (layer, f, paint) in [
        (SceneLayer::Draw, below, local_paint),
        (SceneLayer::DrawOver, over, None),
    ] {
        if f.is_none() && paint.is_none() {
            continue;
        }
        let region_paint = regions.clone();
        let region_reported = RefCell::new(std::collections::HashSet::new());
        let weak = cx.weak();
        let reported = std::cell::Cell::new(false);
        cx.effect(EffectKind::Render, at, move || {
            let sz = size.get();
            let theme = theme.map(|t| t.get()).unwrap_or_default();
            let measurer = weak
                .upgrade()
                .and_then(|rt| ViewState::tree_mut(&rt).reader().text_measurer(id));
            let Recording {
                scene,
                unsupported: issues,
                region_errors,
            } = Draw::record(sz, &theme, |d| {
                if let Some(measurer) = measurer {
                    d.set_text_measurer(measurer);
                }
                if let Some(regions) = &region_paint {
                    d.declare_regions(regions.resolve());
                }
                if let Some(f) = &f {
                    f(d, sz);
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
                    if region_reported.borrow_mut().insert(message.clone()) {
                        rt.report(Some(owner), UiError::new(
                            kivact_reactive::ErrorKind::Message, message, Some(at)));
                    }
                }
                if issues > 0 && !reported.replace(true) {
                    rt.note(format!("node {}: {issues} draw commands unsupported (invalid pixels or box-only shadow values)", id.0));
                    rt.report(
                        Some(owner),
                        UiError::new(
                            kivact_reactive::ErrorKind::Unsupported,
                            format!("{issues} draw command(s) contain invalid RGBA pixels or a canvas shadow with spread/inset; unsupported values dropped"),
                            Some(at),
                        ),
                    );
                }
            }
        });
    }
}
