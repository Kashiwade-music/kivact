//! Role: Mounting binds the draw layers of a host node.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_reactive::{EffectKind, Signal, UiError};
use kivact_style::draw::{Draw, Recording, Size};
use kivact_wire::{NodeId, SceneLayer};

use super::facts::NodeFacts;
use crate::view::common::{Common, Registry};
use crate::view::descriptor::Mcx;
use crate::view::state::ViewState;

/// Stage 7: bind the node's draw effect: `.draw`, `.draw_over` and the local
/// paint of its style, with the regions its canvas declares. The layers are
/// re-recorded when the node's size or anything they read changes.
pub(super) fn bind(
    cx: &Mcx<'_>,
    id: NodeId,
    c: &mut Common,
    facts: &NodeFacts,
    regions: Option<Rc<Registry>>,
    at: &'static Location<'static>,
) {
    if !facts.has_draw {
        return;
    }
    let owner = cx.owner;
    let local_paint = &facts.local_paint;
    let size: Signal<Size> = cx.owner.signal(Size::new(0.0, 0.0), at);
    if let Some(h) = ViewState::tree_mut(cx.rt).handlers_mut(id) {
        h.layout = Some(Rc::new(move |_, s| size.set(s)));
    }
    let theme = cx.theme();
    for (layer, f) in [
        (SceneLayer::Draw, c.draw.take()),
        (SceneLayer::DrawOver, c.draw_over.take()),
    ] {
        let paint = if layer == SceneLayer::Draw {
            local_paint.clone()
        } else {
            None
        };
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
            let measurer = weak.upgrade().and_then(|rt| {
                ViewState::tree_mut(&rt)
                    .reader()
                    .and_then(|reader| reader.text_measurer(id))
            });
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
