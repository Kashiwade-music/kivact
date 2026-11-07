//! Role: Build a frame's composition from the paint list, step by step.
//!
//! Invariants:
//! - A listed node opens what [`Step::opened`] says, in that order. A layer
//!   opens first, and its own transform and effects are its layer's. The
//!   box is drawn between the outer openings (layer, transform, group) and
//!   the inner ones (the presence clip and the padding-box clip), so the box
//!   is not clipped to its padding. Each opening is closed by its own pop,
//!   in reverse order, after the node's draw-over and scrollbars.
//! - A layer's raster scale enlarges what is inside it like a transform's.

use super::state::{Entry, LayerSpec, Opening, PaintList, PaintPass, Step};
use crate::contract::{HostEvent, NodeType};
use crate::lower::effects::{self, Tint};
use crate::lower::sanitize;
use crate::lower::shape::BoxShape;
use crate::lower::{LayerProps, SceneOut, border, scene, shape};
use crate::model::Model;
use crate::model::interact::visual;
use arvane::asset::{PixelSize, fit_image};
use kivact_types::render::{Composition, LayerEffects, LayerKey, Paint, RenderCmd as D};
use kivact_types::scene::Shape;
use kivact_types::{BlendMode, NodeId, PaintProps, Point, Rect, Size};

/// The parts of one listed node that its opening and closing read.
struct Place<'a> {
    step: &'a Step,
    list: &'a PaintList,
    paint: &'a PaintProps,
    /// Its border box, in the space of its origin.
    bounds: Rect,
    /// Its clip path in the same space.
    clip_path: Option<Shape>,
}

impl Model {
    /// The frame's composition: the paint list brought up to date, drawn at
    /// the window's scale into the root layer.
    pub(crate) fn composition(&mut self, engine: &mut arvane::Engine) -> Composition {
        self.paint_list();
        self.sync_playback();
        let pass = PaintPass {
            scale: self.paint.scale,
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
        let mut out = SceneOut::default();
        // How much the transforms and layers around a node enlarge it,
        // outermost first: its images are rasterized for that much more.
        let mut zoom: Vec<f64> = Vec::new();
        for entry in &list.entries {
            let at = zoom.last().copied().unwrap_or(1.0);
            match entry {
                Entry::Open(step) => {
                    let at = match zoom_of(step, list) {
                        Some(z) => {
                            let z = at * z;
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
                    if zoom_of(step, list).is_some() {
                        zoom.pop();
                    }
                }
            }
        }
        out.into_composition(pass.size)
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

    /// Paint `step`'s node up to its children: open what it opens, draw its
    /// box (when its own paint shows), then (inside its clips when it clips)
    /// its canvas scene and its content.
    fn paint_open(
        &self,
        step: &Step,
        list: &PaintList,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut SceneOut,
    ) {
        let host = &self;
        let id = step.id;
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
        let place = Place {
            step,
            list,
            paint: &p,
            bounds,
            clip_path: clip_path.clone(),
        };
        let (outer, inner): (Vec<Opening>, Vec<Opening>) = step
            .opened()
            .into_iter()
            .partition(|kind| matches!(kind, Opening::Layer | Opening::Transform | Opening::Group));
        for kind in outer {
            self.open(kind, &place, pass, out);
        }
        if step.own {
            if let (Some(ring), Some(clip)) = (&p.ring, &clip_path) {
                // Around the clip path, beneath the node's paint.
                border::focus_ring(out, &outline, Some(clip), ring, &tint);
            }
            self.paint_box(&p, &outline, clip_path.is_some(), &tint, pass, out);
        }
        for kind in inner {
            self.open(kind, &place, pass, out);
        }
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

    /// Open one thing of `place`'s node.
    fn open(&self, kind: Opening, place: &Place, pass: PaintPass, out: &mut SceneOut) {
        let step = place.step;
        match kind {
            Opening::Layer => out.open_layer(self.node_layer(place, pass)),
            Opening::Transform => {
                if let Some(t) = step
                    .transform
                    .and_then(|t| place.list.transforms.get(t as usize))
                {
                    out.push(D::PushTransform(*t));
                }
            }
            Opening::Group => {
                let opacity = if step.group { place.paint.opacity } else { 1.0 };
                let clip = place.clip_path.clone();
                out.push(D::PushGroup(effects::group(place.paint, clip, opacity)));
            }
            Opening::Presence => {
                if let Some(clip) = step.presence_clip {
                    out.push(D::PushClip(clip));
                }
            }
            Opening::Clip => out.push(D::PushClip(self.padding_box(place).rect())),
            Opening::ClipGroup => out.push(effects::clip_group(self.padding_box(place).shape())),
        }
    }

    /// The padding box of `place`'s node: what its children are clipped to.
    fn padding_box(&self, place: &Place) -> BoxShape {
        BoxShape::of(place.bounds, &place.paint.radius)
            .inset(shape::border_widths(place.paint.border.as_ref()))
    }

    /// The layer properties of `place`'s node: its layer's transform, bounds
    /// and plane, and its effects (opacity, blend, clip path, filters, masks
    /// and backdrop) in its own space.
    fn node_layer(&self, place: &Place, pass: PaintPass) -> LayerProps {
        let spec = place
            .step
            .layer
            .and_then(|index| place.list.layers.get(index as usize))
            .copied()
            .unwrap_or(LayerSpec {
                matrix: kivact_types::geometry3d::Matrix4::IDENTITY,
                bounds: place.bounds,
                plane: kivact_types::render::Plane::Flat,
                backface: kivact_types::paint::BackfaceVisibility::Visible,
                grid: kivact_types::render::RasterGrid::Target,
            });
        let context = self.painting(pass);
        let effects = place.paint.effects.as_deref();
        let layer_effects = LayerEffects {
            opacity: sanitize::unit(place.paint.opacity, 1.0),
            blend: effects.map_or(BlendMode::Normal, |e| e.blend),
            composite: kivact_types::render::Composite::SourceOver,
            filters: effects.map_or_else(Vec::new, |e| context.layer_filters(&e.filter)),
            clip: place.clip_path.clone(),
            masks: effects.map_or_else(Vec::new, |e| context.layer_masks(&e.masks, place.bounds)),
            backdrop: effects.and_then(|e| {
                let shape = BoxShape::of(place.bounds, &place.paint.radius).shape();
                context.layer_backdrop(&e.backdrop, shape)
            }),
        };
        // The context's failures are the frame's: a window keeps painting.
        let _ = context.finish();
        LayerProps {
            key: LayerKey::Node(place.step.id),
            transform: spec.matrix,
            bounds: spec.bounds,
            plane: spec.plane,
            backface: spec.backface,
            grid: spec.grid,
            effects: layer_effects,
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
        out: &mut SceneOut,
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

    /// Paint `step`'s node after its children: its `DrawOver` scene and its
    /// scrollbars, then close what it opened, innermost first.
    fn paint_close(
        &self,
        step: &Step,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut SceneOut,
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
        self.paint_bars(step, out);
        for kind in step.opened().into_iter().rev() {
            match kind {
                Opening::Layer => out.close_layer(),
                Opening::Transform => out.push(D::PopTransform),
                Opening::Group | Opening::ClipGroup => out.push(D::PopGroup),
                Opening::Presence | Opening::Clip => out.push(D::PopClip),
            }
        }
    }

    /// The scrollbars of `step`'s node (when it draws its own).
    fn paint_bars(&self, step: &Step, out: &mut SceneOut) {
        let bars = if step.own {
            self.scrollbar_bars(step.id)
        } else {
            [None, None]
        };
        if !bars.iter().any(Option::is_some) {
            return;
        }
        let props = self.scrollbar_style(step.id);
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
                    shape: Shape::Rect(rect),
                    paint: Paint::Solid(tint.color(color)),
                });
            }
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
        out: &mut SceneOut,
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

/// The zoom a listed node's transform or layer enlarges its content by
/// (`None`: neither).
fn zoom_of(step: &Step, list: &PaintList) -> Option<f64> {
    let transform = step
        .transform
        .and_then(|t| list.transforms.get(t as usize))
        .map(|t| t.max_scale());
    let layer = step
        .layer
        .and_then(|l| list.layers.get(l as usize))
        .map(|l| l.matrix.as_affine().map_or(1.0, |t| t.max_scale()));
    transform.or(layer)
}

/// How `step`'s colors are drawn.
fn tint(step: &Step) -> Tint {
    Tint { alpha: step.alpha }
}
