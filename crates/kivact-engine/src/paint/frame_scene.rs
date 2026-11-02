//! Role: Build a frame's draw commands from the paint list, step by step.

use super::state::{Entry, Step};
use crate::model::Model;
use crate::model::interact::visual;
use arvane::asset::{PixelSize, fit_image};
use kivact_lower::effects::Tint;
use kivact_lower::shape::BoxShape;
use kivact_lower::{border, effects, scene, shape};
use kivact_types::protocol::DrawCommand as D;
use kivact_wire::{HostEvent, NodeId, NodeType, PaintProps, Point, Rect, Size};

impl Model {
    /// Build the frame's display list from the paint list (`list`): every
    /// listed node with its current content. Every shown image is drawn in
    /// it: a new frame of the asset store's recency.
    pub(crate) fn scene(&mut self, engine: &mut arvane::Engine) -> kivact_types::protocol::Scene {
        self.paint_list();
        self.assets.borrow_mut().begin_frame();
        let mut out = kivact_lower::SceneOut::default();
        // How much the transforms around a node enlarge it, outermost
        // first: its images are rasterized for that much more.
        let mut zoom: Vec<f32> = Vec::new();
        for entry in &self.paint.list.entries {
            let at = zoom.last().copied().unwrap_or(1.0);
            match entry {
                Entry::Open(step) => {
                    let at = match step
                        .transform
                        .and_then(|t| self.paint.list.transforms.get(t as usize))
                    {
                        Some(t) => {
                            let z = at * t.max_scale();
                            zoom.push(z);
                            z
                        }
                        None => at,
                    };
                    self.paint
                        .raster
                        .set(self.paint.scale * scene::raster_zoom(at));
                    self.paint_open(step, engine, &mut out);
                }
                Entry::Close(step) => {
                    self.paint
                        .raster
                        .set(self.paint.scale * scene::raster_zoom(at));
                    self.paint_close(step, engine, &mut out);
                    if step.transform.is_some() {
                        zoom.pop();
                    }
                }
            }
        }
        self.paint.raster.set(self.paint.scale);
        for (node, count) in self.paint.dropped_draws.take() {
            self.push(HostEvent::DrawDropped { node, count });
        }
        out.into_scene()
    }

    /// The color matrix at `index` of the paint list.
    fn matrix(&self, index: Option<u32>) -> Option<kivact_types::protocol::ColorMatrix> {
        self.paint.list.matrices.get(index? as usize).copied()
    }

    /// How `step`'s colors are drawn.
    fn tint(&self, step: &Step) -> Tint {
        Tint {
            alpha: step.alpha,
            matrix: self.matrix(step.matrix),
        }
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

    /// The layers `step` is composited in, outermost first: the clip path
    /// and mask over its painted box `bounds`, its group opacity, its
    /// blend mode and its layer filters.
    fn paint_layers(
        &self,
        step: &Step,
        p: &PaintProps,
        bounds: Rect,
    ) -> Vec<kivact_types::protocol::Layer> {
        let e = p.effects.as_deref();
        let outer = kivact_types::protocol::Layer {
            clip: e
                .and_then(|e| e.clip_path.as_ref())
                .map(|s| shape::clip_path(s, bounds)),
            mask: e.and_then(|e| e.mask.as_ref()).and_then(|m| {
                let area = bounds;
                // A mask still loading hides nothing yet.
                let origin = Point {
                    x: area.x,
                    y: area.y,
                };
                self.layer_paint(m, area, origin)
                    .map(|paint| kivact_types::protocol::Mask { rect: area, paint })
            }),
            ..kivact_types::protocol::Layer::default()
        };
        let opacity = if step.group { p.opacity } else { 1.0 };
        let parent = self.matrix(step.parent_matrix);
        let mut layers = effects::layers(p, outer, opacity, parent.as_ref());
        // As many as the list counted (it closes as many).
        layers.resize_with(
            usize::from(step.layers),
            kivact_types::protocol::Layer::default,
        );
        layers
    }

    /// Paint `step`'s node up to its children: push its transform, open
    /// its layers, draw its box (when its own paint shows), then (inside
    /// its clip when it clips) its canvas scene and its content.
    fn paint_open(
        &self,
        step: &Step,
        engine: &mut arvane::Engine,
        out: &mut kivact_lower::SceneOut,
    ) {
        let host = &self;
        let id = step.id;
        if let Some(t) = step
            .transform
            .and_then(|t| self.paint.list.transforms.get(t as usize))
        {
            out.push(D::PushTransform(*t));
        }
        let Some(n) = host.node(id) else {
            return;
        };
        let (p, bounds, canvas) = self.placed(id, step.origin);
        let tint = self.tint(step);
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
        if step.layers > 0 {
            for layer in self.paint_layers(step, &p, bounds) {
                out.push(D::PushLayer(layer));
            }
        }
        if step.own {
            self.paint_box(step, &p, &outline, clip_path.is_some(), &tint, out);
        }
        if step.clips {
            // Closed as the list says (a rect clip or a rounded layer).
            let padding =
                BoxShape::of(bounds, &p.radius).inset(shape::border_widths(p.border.as_ref()));
            if step.rounded {
                out.push(D::PushLayer(kivact_types::protocol::Layer {
                    clip: Some(padding.shape()),
                    ..kivact_types::protocol::Layer::default()
                }));
            } else {
                out.push(D::PushClip(padding.rect()));
            }
        }
        if !step.own {
            return;
        }
        if let Some(scene) = &n.scenes()[0] {
            self.paint_scene(id, 0, scene, canvas, &tint, engine, out);
        }
        match n.kind() {
            _ if host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .is_some() =>
            {
                self.paint_input(id, bounds, step.clip, &tint, engine, out)
            }
            NodeType::Text => {
                let layout = host.computed_layout(id).unwrap_or_default();
                let area = visual::content_at(bounds, layout.content_box);
                self.paint_text(id, area, &tint, engine, out);
            }
            _ => {}
        }
        if host.editors[n.index.0]
            .as_deref()
            .map(|editor| editor.editor.state())
            .is_none()
            && let Some(asset) = n.image()
        {
            self.paint_image(id, n, asset, canvas, &tint, out);
        }
    }

    /// The node's own box: shadows, backdrop, background, border and ring
    /// (see the module invariants).
    fn paint_box(
        &self,
        step: &Step,
        p: &PaintProps,
        outline: &BoxShape,
        clip_path: bool,
        tint: &Tint,
        out: &mut kivact_lower::SceneOut,
    ) {
        let knockout = !border::opaque(p.background.as_ref(), tint);
        border::shadows(out, outline, &p.shadows, false, knockout, tint);
        let parent = self.matrix(step.parent_matrix);
        if let Some(b) = effects::backdrop(p, outline.shape(), parent.as_ref()) {
            out.push(D::Backdrop(b));
        }
        if let Some(fill) = &p.background {
            let area = outline.rect();
            let origin = Point {
                x: area.x,
                y: area.y,
            };
            self.fill(out, outline, fill, tint, origin);
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
    /// pop its clip, its layers and its transform.
    fn paint_close(
        &self,
        step: &Step,
        engine: &mut arvane::Engine,
        out: &mut kivact_lower::SceneOut,
    ) {
        let id = step.id;
        if step.own
            && let Some(n) = self.node(id)
            && let Some(scene) = &n.scenes()[1]
        {
            let (_, _, canvas) = self.placed(id, step.origin);
            let tint = self.tint(step);
            self.paint_scene(id, 1, scene, canvas, &tint, engine, out);
        }
        let bars = if step.own {
            self.scrollbar_bars(id)
        } else {
            [None, None]
        };
        if bars.iter().any(Option::is_some) {
            let props = self.scrollbar_style(id);
            let tint = self.tint(step);
            for bar in bars.into_iter().flatten() {
                for (rect, color) in [(bar.track, props.track), (bar.thumb, props.thumb)] {
                    let rect = Rect {
                        x: step.origin.x + rect.x,
                        y: step.origin.y + rect.y,
                        width: rect.width,
                        height: rect.height,
                    };
                    out.push(D::FillShape {
                        shape: kivact_types::protocol::Shape::Rect(rect),
                        paint: kivact_types::protocol::Paint::Solid(tint.color(color)),
                    });
                }
            }
        }
        if step.clips {
            out.push(if step.rounded {
                D::PopLayer
            } else {
                D::PopClip
            });
        }
        out.extend(std::iter::repeat_n(D::PopLayer, usize::from(step.layers)));
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
        source: &kivact_types::Asset,
        canvas: Rect,
        tint: &Tint,
        out: &mut kivact_lower::SceneOut,
    ) {
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
            self.paint.raster.get(),
        );
        // An overflowing SVG is rasterized for the part the box shows.
        let Some((image, region)) =
            assets.pixels(source, device, placed.visible(canvas), tint_color)
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
            color_matrix: tint.matrix,
        });
        if placed.clip {
            out.push(D::PopClip);
        }
    }
}
