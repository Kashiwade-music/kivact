//! Role: Build a frame's composition from the paint list, step by step.

use super::state::{Entry, PaintList, PaintPass, Step};
use crate::contract::{HostEvent, NodeType};
use crate::lower::effects::{self, Tint};
use crate::lower::shape::BoxShape;
use crate::lower::{border, scene, shape};
use crate::model::Model;
use crate::model::interact::visual;
use arvane::asset::{PixelSize, fit_image};
use kivact_types::render::{Composition, RenderCmd as D};
use kivact_types::{NodeId, PaintProps, Point, Rect, Size};

impl Model {
    /// The frame's composition: the paint list brought up to date, drawn at
    /// the window's scale into the root layer.
    pub(crate) fn composition(&mut self, engine: &mut arvane::Engine) -> Composition {
        self.paint_list();
        self.sync_playback();
        let pass = PaintPass {
            scale: self.paint.scale,
            purpose: crate::lower::context::Purpose::Window,
            size: self.window(),
        };
        let composition = self.compose_list(engine, &self.paint.list, pass);
        self.report_dropped_draws();
        composition
    }

    /// Queue the commands dropped since the last report for the client.
    pub(super) fn report_dropped_draws(&mut self) {
        for (node, count) in self.paint.dropped_draws.take() {
            self.push(HostEvent::DrawDropped { node, count });
        }
    }

    /// The composition of `list`: every listed node with its current
    /// content in the root layer of `pass.size`. Every shown image is drawn
    /// in it: a new frame of the asset store's recency.
    pub(super) fn compose_list(
        &self,
        engine: &mut arvane::Engine,
        list: &PaintList,
        pass: PaintPass,
    ) -> Composition {
        self.assets.borrow_mut().begin_frame();
        let mut out = crate::lower::SceneOut::default();
        // How much the transforms around a node enlarge it, outermost
        // first: its images are rasterized for that much more.
        let mut zoom: Vec<f64> = Vec::new();
        for entry in &list.entries {
            let at = zoom.last().copied().unwrap_or(1.0);
            match entry {
                Entry::Open(step) => {
                    let at = match step.transform.and_then(|t| list.transforms.get(t as usize)) {
                        Some(t) => {
                            let z = at * t.max_scale();
                            zoom.push(z);
                            z
                        }
                        None => at,
                    };
                    let here = PaintPass {
                        scale: pass.scale * scene::raster_zoom(at),
                        ..pass
                    };
                    self.paint_open(step, list, here, engine, &mut out);
                }
                Entry::Close(step) => {
                    let here = PaintPass {
                        scale: pass.scale * scene::raster_zoom(at),
                        ..pass
                    };
                    self.paint_close(step, here, engine, &mut out);
                    if step.transform.is_some() {
                        zoom.pop();
                    }
                }
            }
        }
        crate::lower::composition::root_composition(out.into_scene(), pass.size)
    }

    /// Where `id` is painted when its layout box starts at `o`: its
    /// paint, its box and its canvas (inside the border).
    fn placed(&self, id: NodeId, o: Point) -> (PaintProps, Rect, Rect) {
        let layout = self.computed_layout(id).unwrap_or_default();
        let p = visual::paint(self, id);
        let bounds = Rect {
            x: o.x,
            y: o.y,
            width: layout.size.width,
            height: layout.size.height,
        };
        // Canvas scenes start inside the border.
        let canvas = visual::padding_box(bounds, &p);
        (p, bounds, canvas)
    }

    /// Paint `step`'s node up to its children: push its transform, open
    /// its group, draw its box (when its own paint shows), then (inside
    /// its clip when it clips) its canvas scene and its content.
    fn paint_open(
        &self,
        step: &Step,
        list: &PaintList,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut crate::lower::SceneOut,
    ) {
        let host = &self;
        let id = step.id;
        if let Some(t) = step.transform.and_then(|t| list.transforms.get(t as usize)) {
            out.push(D::PushTransform(*t));
        }
        let Some(n) = host.node(id) else {
            return;
        };
        let (p, bounds, canvas) = self.placed(id, step.origin);
        let tint = tint(step);
        let outline = BoxShape::of(bounds, &p.radius);
        let clip_path = p
            .effects
            .as_ref()
            .and_then(|e| e.clip_path.as_ref())
            .map(|s| shape::clip_path(s, bounds));
        if step.own
            && let (Some(ring), Some(clip)) = (&p.ring, &clip_path)
        {
            // Around the clip path, beneath the node's paint.
            border::focus_ring(out, &outline, Some(clip), ring, &tint);
        }
        if step.grouped {
            let opacity = if step.group { p.opacity } else { 1.0 };
            out.push(D::PushGroup(effects::group(&p, clip_path.clone(), opacity)));
        }
        if step.own {
            self.paint_box(&p, &outline, clip_path.is_some(), &tint, pass, out);
        }
        if let Some(clip) = step.presence_clip {
            out.push(D::PushClip(clip));
        }
        paint_content_clip(step, bounds, &p, out);
        if !step.own {
            return;
        }
        if host.editors[n.index.0].is_none()
            && let Some(scene) = &n.scenes()[0]
        {
            self.paint_scene(id, 0, scene, canvas, &tint, engine, out, pass);
        }
        match n.kind() {
            _ if host.editors[n.index.0].is_some() => {
                self.paint_input(id, bounds, step.clip, &tint, engine, out);
                if let Some(scene) = &n.scenes()[0] {
                    self.paint_scene(id, 0, scene, canvas, &tint, engine, out, pass);
                }
            }
            NodeType::Text => {
                let layout = host.computed_layout(id).unwrap_or_default();
                let area = visual::content_at(bounds, layout.content_box);
                self.paint_text(id, area, &tint, pass, engine, out);
            }
            _ => {}
        }
        if host.editors[n.index.0].is_none() && n.image().is_some() {
            self.paint_image(id, n, canvas, &tint, pass, out);
        }
    }

    /// The node's own box: shadows, background, border and ring
    /// (see the module invariants).
    fn paint_box(
        &self,
        p: &PaintProps,
        outline: &BoxShape,
        clip_path: bool,
        tint: &Tint,
        pass: PaintPass,
        out: &mut crate::lower::SceneOut,
    ) {
        let knockout = !border::opaque(p.background.as_ref(), tint);
        border::shadows(out, outline, &p.shadows, false, knockout, tint);
        if let Some(fill) = &p.background {
            let area = outline.rect();
            let origin = Point {
                x: area.x,
                y: area.y,
            };
            self.fill(out, outline, fill, tint, origin, pass);
        }
        border::shadows(out, outline, &p.shadows, true, false, tint);
        if let Some(b) = &p.border {
            border::border(out, outline, b, tint);
        }
        if let Some(ring) = &p.ring
            && !clip_path
        {
            border::focus_ring(out, outline, None, ring, tint);
        }
    }

    /// Paint `step`'s node after its children: its `DrawOver` scene, then
    /// pop its clip, its group and its transform.
    fn paint_close(
        &self,
        step: &Step,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut crate::lower::SceneOut,
    ) {
        let id = step.id;
        if step.own
            && let Some(n) = self.node(id)
            && let Some(scene) = &n.scenes()[1]
        {
            let (_, _, canvas) = self.placed(id, step.origin);
            let tint = tint(step);
            self.paint_scene(id, 1, scene, canvas, &tint, engine, out, pass);
        }
        let bars = if step.own {
            self.scrollbar_bars(id)
        } else {
            [None, None]
        };
        if bars.iter().any(Option::is_some) {
            let props = self.scrollbar_style(id);
            let tint = tint(step);
            for bar in bars.into_iter().flatten() {
                for (rect, color) in [(bar.track, props.track), (bar.thumb, props.thumb)] {
                    let rect = Rect {
                        x: step.origin.x + rect.x,
                        y: step.origin.y + rect.y,
                        width: rect.width,
                        height: rect.height,
                    };
                    out.push(D::FillShape {
                        shape: kivact_types::scene::Shape::Rect(rect),
                        paint: kivact_types::render::Paint::Solid(tint.color(color)),
                    });
                }
            }
        }
        if step.clips {
            out.push(if step.rounded {
                D::PopGroup
            } else {
                D::PopClip
            });
        }
        if step.presence_clip.is_some() {
            out.push(D::PopClip);
        }
        if step.grouped {
            out.push(D::PopGroup);
        }
        if step.transform.is_some() {
            out.push(D::PopTransform);
        }
    }

    /// The image of `Image` node `id` showing `asset` in its box `canvas`
    /// (see the module invariants); nothing while it loads or after it
    /// failed.
    fn paint_image(
        &self,
        id: NodeId,
        n: &crate::tree::node::HostNode,
        canvas: Rect,
        tint: &Tint,
        pass: PaintPass,
        out: &mut crate::lower::SceneOut,
    ) {
        let Some(source) = n.image() else {
            return;
        };
        let mut assets = self.assets.borrow_mut();
        let Some(natural) = assets.natural(source) else {
            return;
        };
        let fit = n.image_data().and_then(|d| d.fit).unwrap_or_default();

        let monochrome = n.image_data().and_then(|d| d.monochrome).unwrap_or(false);
        // The node's text color, inherited, with its and its ancestors' states.
        let tint_color = monochrome.then(|| self.text_color(id));
        let placed = fit_image(fit, natural, canvas);
        let device = PixelSize::of(
            Size {
                width: placed.rect.width,
                height: placed.rect.height,
            },
            pass.scale,
        );
        let frame = self.playback_frame(id);
        // An overflowing SVG is rasterized for the part the box shows.
        let Some((image, region)) =
            assets.pixels(source, device, placed.visible(canvas), tint_color, frame)
        else {
            return;
        };
        drop(assets);
        if placed.clip {
            out.push(D::PushClip(canvas));
        }
        out.push(D::Image {
            image,
            rect: region.place(placed.rect),
            opacity: tint.alpha,
        });
        if placed.clip {
            out.push(D::PopClip);
        }
    }
}

fn paint_content_clip(step: &Step, bounds: Rect, p: &PaintProps, out: &mut crate::lower::SceneOut) {
    if step.clips {
        // Closed as the list says (a rect clip or a rounded group).
        let padding =
            BoxShape::of(bounds, &p.radius).inset(shape::border_widths(p.border.as_ref()));
        if step.rounded {
            out.push(effects::clip_group(padding.shape()));
        } else {
            out.push(D::PushClip(padding.rect()));
        }
    }
}

/// How `step`'s colors are drawn.
fn tint(step: &Step) -> Tint {
    Tint { alpha: step.alpha }
}
