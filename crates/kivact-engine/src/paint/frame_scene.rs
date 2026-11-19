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

use super::state::{
    DrawKey, Entry, Grouping, LayerSpec, NodeDraw, Opening, PaintList, PaintPass, Recorded, Step,
};
use crate::contract::{HostEvent, NodeType};
use crate::lower::border_image::lower_border_image;
use crate::lower::effects::{self, Tint};
use crate::lower::fill::Boxes;
use crate::lower::sanitize;
use crate::lower::shape::BoxShape;
use crate::lower::{LayerProps, SceneOut, border, scene, shape};
use crate::model::Model;
use crate::model::drawables::export_of;
use crate::model::interact::visual;
use arvane::asset::{PixelSize, fit_image};
use arvane_protocol::{Composition, LayerEffects, LayerKey, RasterPaint, RenderCmd as D};
use kivact_types::scene::Shape;
use kivact_types::{BlendMode, NodeId, PaintProps, Point, Rect, Size};
use std::rc::Rc;

/// In debug builds with the test driver every kept drawing is checked before it is replayed: its paint against the node's paint now, its commands against the ones that paint draws now into a scratch output. A change that did not forget a drawing panics, naming the node; the frame, the kept drawings and the layers' versions are the replay's.
const VERIFY_DRAWS: bool = cfg!(all(debug_assertions, feature = "testing"));

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
        if self.paint_list() {
            self.evict_draws();
            self.refresh_shader_clocks();
            self.refresh_playback();
        }
        let pass = PaintPass {
            scale: self.paint.scale,
            size: self.window(),
            cache: true,
        };
        self.prepare_canvas(engine);
        let mut composition = self.compose_list(engine, &self.paint.list, pass);
        self.report_dropped_draws();
        self.append_exports(engine, &mut composition);
        composition
    }

    /// Add the layers of the exported drawables (`DrawableMode::Export`) that
    /// the app asked frames of, each as a layer no item of the screen draws.
    /// A drawable's layer is its subtree composed on its own: the layers
    /// are appended to `out`, their items renumbered, and the export names
    /// the subtree's root layer.
    fn append_exports(&self, engine: &mut arvane::Engine, out: &mut Composition) {
        for (id, root, ask) in self.export_requests() {
            let Some(bounds) = self.rect(root) else {
                continue;
            };
            let scale = ask.scale_at(self.paint.scale());
            let Some(layers) = self.subtree_layers(engine, id, root, scale) else {
                continue;
            };
            let base = out.layers.len() as u32;
            for mut layer in layers {
                for item in &mut layer.items {
                    if let arvane_protocol::LayerItem::Layer { index, .. } = item {
                        *index += base;
                    }
                }
                out.layers.push(layer);
            }
            out.exports.push(export_of(id, base, bounds, scale, &ask));
        }
    }

    /// Queue the commands dropped since the last report for the client.
    pub(super) fn report_dropped_draws(&mut self) {
        for (node, count) in self.paint.dropped_draws.take() {
            self.push(HostEvent::DrawDropped { node, count });
        }
        self.report_misplaced();
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

    /// The box and the canvas (inside the border) of `id` when its layout
    /// box starts at `o`, drawn with the paint `p`.
    fn boxes_at(&self, id: NodeId, o: Point, p: &PaintProps) -> (Rect, Rect) {
        let layout = self.computed_layout(id).unwrap_or_default();
        let bounds = Rect {
            x: o.x,
            y: o.y,
            width: layout.size.width,
            height: layout.size.height,
        };
        // Canvas scenes start inside the border.
        (bounds, visual::padding_box(bounds, p))
    }

    /// Where `step`'s node is drawn with `paint`: its place and its canvas
    /// (inside the border).
    fn place_at<'a>(
        &self,
        step: &'a Step,
        list: &'a PaintList,
        paint: &'a PaintProps,
    ) -> (Place<'a>, Rect) {
        let (bounds, canvas) = self.boxes_at(step.id, step.origin, paint);
        let clip_path = paint
            .effects
            .as_ref()
            .and_then(|e| e.clip_path.as_ref())
            .map(|s| s.in_rect(bounds));
        let place = Place {
            step,
            list,
            paint,
            bounds,
            clip_path,
        };
        (place, canvas)
    }

    /// The drawing kept for `step`'s node when it was made for this step
    /// and scale, taken out of its slot (`keep_draw` puts one back).
    fn take_draw(&self, step: &Step, pass: PaintPass) -> Option<Box<NodeDraw>> {
        if !pass.cache {
            return None;
        }
        let n = self.node(step.id)?;
        self.slots.draws[n.index.0]
            .take()
            .filter(|d| d.key == DrawKey::of(step, pass))
    }

    /// Keep `draw` for `id`'s next frame (`fresh`: it was made now).
    fn keep_draw(&self, id: NodeId, draw: Box<NodeDraw>, fresh: bool, pass: PaintPass) {
        if !pass.cache {
            return;
        }
        let Some(n) = self.node(id) else {
            return;
        };
        if fresh {
            self.paint.draw_owners.borrow_mut().insert((id, n.index.0));
        }
        self.slots.draws[n.index.0].set(Some(draw));
    }

    /// The node's own box: its focus ring around a clip path, then its
    /// shadows, background, border and ring (nothing unless it draws itself).
    fn draw_box(
        &self,
        place: &Place,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut SceneOut,
    ) {
        if !place.step.own {
            return;
        }
        let id = place.step.id;
        let p = place.paint;
        let tint = tint(place.step);
        let outline = BoxShape::of(place.bounds, &p.radius);
        if let (Some(ring), Some(clip)) = (&p.ring, &place.clip_path) {
            // Around the clip path, beneath the node's paint.
            border::focus_ring(out, &outline, Some(clip), ring, &tint);
        }
        let boxes = self.node_boxes(id, p, &outline, &tint, engine);
        self.paint_box(p, &boxes, place.clip_path.is_some(), &tint, pass, out);
    }

    /// The node's canvas scene and content (text, input lines, image),
    /// inside its clips (nothing unless it draws itself).
    fn draw_content(
        &self,
        place: &Place,
        canvas: Rect,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut SceneOut,
    ) {
        if !place.step.own {
            return;
        }
        let id = place.step.id;
        let Some(n) = self.node(id) else {
            return;
        };
        let tint = tint(place.step);
        if self.slots.editors[n.index.0].is_none()
            && let Some(scene) = &n.scenes()[0]
        {
            self.paint_scene(id, 0, scene, canvas, &tint, engine, out, pass);
        }
        match n.kind() {
            _ if self.slots.editors[n.index.0].is_some() => {
                self.paint_input(id, place.bounds, place.step.clip, &tint, engine, out);
                if let Some(scene) = &n.scenes()[0] {
                    self.paint_scene(id, 0, scene, canvas, &tint, engine, out, pass);
                }
            }
            NodeType::Text => {
                let layout = self.computed_layout(id).unwrap_or_default();
                let area = visual::content_at(place.bounds, layout.content_box);
                self.paint_text(id, area, &tint, pass, engine, out);
            }
            _ => {}
        }
        if self.slots.editors[n.index.0].is_none() && n.image().is_some() {
            self.paint_image(id, n, canvas, &tint, pass, out);
        }
    }

    /// The node's draw-over scene and scrollbars.
    fn draw_over(
        &self,
        step: &Step,
        p: &PaintProps,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut SceneOut,
    ) {
        let id = step.id;
        if step.own
            && let Some(n) = self.node(id)
            && let Some(scene) = &n.scenes()[1]
        {
            let (_, canvas) = self.boxes_at(id, step.origin, p);
            self.paint_scene(id, 1, scene, canvas, &tint(step), engine, out, pass);
        }
        self.paint_bars(step, out);
    }

    /// Drop the drawings of the nodes the rebuilt list does not list.
    pub(super) fn evict_draws(&mut self) {
        let listed: std::collections::HashSet<NodeId> = self
            .paint
            .list
            .entries
            .iter()
            .filter_map(|e| match e {
                Entry::Open(step) => Some(step.id),
                Entry::Close(_) => None,
            })
            .collect();
        let slots = &self.slots;
        self.paint.draw_owners.get_mut().retain(|(id, slot)| {
            let keep = listed.contains(id);
            if !keep {
                slots.draws[*slot].set(None);
            }
            keep
        });
    }

    /// RasterPaint `step`'s node up to its children: open what it opens, draw its
    /// box (when its own paint shows), then (inside its clips when it clips)
    /// its canvas scene and its content. A kept drawing is replayed (checked
    /// first under `VERIFY_DRAWS`); a live one is drawn again.
    fn paint_open(
        &self,
        step: &Step,
        list: &PaintList,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut SceneOut,
    ) {
        let id = step.id;
        if self.node(id).is_none() {
            return;
        }
        let kept = self.take_draw(step, pass);
        if VERIFY_DRAWS && let Some(draw) = &kept {
            self.verify_draw(step, list, draw, pass, engine);
        }
        let paint = kept
            .as_ref()
            .map_or_else(|| Rc::new(visual::paint(self, id)), |d| Rc::clone(&d.paint));
        let (place, canvas) = self.place_at(step, list, &paint);
        let (outer, inner): (Vec<Opening>, Vec<Opening>) = step
            .opened()
            .into_iter()
            .partition(|kind| matches!(kind, Opening::Layer | Opening::Transform | Opening::Group));
        for kind in outer {
            self.open(kind, &place, pass, out);
        }
        let (draw, fresh) = match kept {
            Some(draw) => {
                match &draw.recorded {
                    Recorded::Commands { boxed, content, .. } => {
                        out.replay(boxed);
                        for kind in inner {
                            self.open(kind, &place, pass, out);
                        }
                        out.replay(content);
                    }
                    Recorded::Live => {
                        self.draw_box(&place, pass, engine, out);
                        for kind in inner {
                            self.open(kind, &place, pass, out);
                        }
                        self.draw_content(&place, canvas, pass, engine, out);
                    }
                }
                (draw, false)
            }
            None => {
                out.begin_record();
                self.draw_box(&place, pass, engine, out);
                let boxed = out.end_record();
                for kind in inner {
                    self.open(kind, &place, pass, out);
                }
                out.begin_record();
                self.draw_content(&place, canvas, pass, engine, out);
                let content = out.end_record();
                let draw = Box::new(NodeDraw {
                    key: DrawKey::of(step, pass),
                    paint: Rc::clone(&paint),
                    recorded: Recorded::of(boxed, content),
                });
                (draw, true)
            }
        };
        self.keep_draw(id, draw, fresh, pass);
    }

    /// Check a kept drawing against one made now (`VERIFY_DRAWS`): its
    /// paint against the paint the node is drawn with now, and its box and
    /// content against the ones that paint and the node's boxes draw now,
    /// recorded in a scratch output. The check only reads: the frame, the
    /// kept drawing and the layers' versions are the replay's.
    fn verify_draw(
        &self,
        step: &Step,
        list: &PaintList,
        draw: &NodeDraw,
        pass: PaintPass,
        engine: &mut arvane::Engine,
    ) {
        let id = step.id;
        let paint = visual::paint(self, id);
        assert!(
            *draw.paint == paint,
            "the kept paint of node {id:?} is stale: a change it is drawn from did not forget it"
        );
        let Recorded::Commands { boxed, content, .. } = &draw.recorded else {
            return;
        };
        let (place, canvas) = self.place_at(step, list, &paint);
        let mut scratch = SceneOut::default();
        scratch.begin_record();
        self.draw_box(&place, pass, engine, &mut scratch);
        let now_boxed = scratch.end_record();
        scratch.begin_record();
        self.draw_content(&place, canvas, pass, engine, &mut scratch);
        let now_content = scratch.end_record();
        assert!(
            now_boxed.as_ref() == Some(boxed) && now_content.as_ref() == Some(content),
            "the kept drawing of node {id:?} is stale: a change it is drawn from did not forget it"
        );
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
                let opacity = if step.grouping == Grouping::Opacity {
                    place.paint.opacity
                } else {
                    1.0
                };
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
                plane: arvane_protocol::Plane::Flat,
                backface: kivact_types::paint::BackfaceVisibility::Visible,
                grid: arvane_protocol::RasterGrid::Target,
            });
        let context = self.painting(pass);
        let effects = place.paint.effects.as_deref();
        let layer_effects = LayerEffects {
            opacity: sanitize::unit(place.paint.opacity, 1.0),
            blend: effects.map_or(BlendMode::Normal, |e| e.blend),
            composite: kivact_types::Composite::SourceOver,
            filters: effects.map_or_else(Vec::new, |e| context.layer_filters(&e.filter)),
            clip: place.clip_path.clone(),
            masks: effects.map_or_else(Vec::new, |e| context.layer_masks(&e.masks, place.bounds)),
            backdrop: effects.and_then(|e| {
                let shape = BoxShape::of(place.bounds, &place.paint.radius).shape();
                context.layer_backdrop(&e.backdrop, shape)
            }),
        };
        // The context's failures are the frame's: a window keeps painting.
        // The layer effects that were dropped are reported under layer 2.
        let dropped = context.finish().dropped;
        self.note_dropped(place.step.id, 2, dropped);
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
        boxes: &Boxes,
        clip_path: bool,
        tint: &Tint,
        pass: PaintPass,
        out: &mut SceneOut,
    ) {
        let outline = &boxes.border;
        let knockout = !border::opaque(p.background.as_ref(), tint);
        border::shadows(out, outline, &p.shadows, false, knockout, tint);
        if let Some(fill) = &p.background {
            let area = outline.rect();
            let origin = Point {
                x: area.x,
                y: area.y,
            };
            self.fill(out, boxes, fill, tint, origin, pass);
        }
        border::shadows(out, outline, &p.shadows, true, false, tint);
        if let Some(image) = &p.border_image {
            let context = self.painting(pass);
            let widths = shape::border_widths(p.border.as_ref());
            lower_border_image(out, image, outline.rect(), widths, &context);
            context.finish();
        } else if let Some(b) = &p.border {
            border::border(out, outline, b, tint);
        }
        if let Some(ring) = &p.ring
            && !clip_path
        {
            border::focus_ring(out, outline, None, ring, tint);
        }
    }

    /// RasterPaint `step`'s node after its children: its `DrawOver` scene and its
    /// scrollbars, then close what it opened, innermost first.
    fn paint_close(
        &self,
        step: &Step,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut SceneOut,
    ) {
        match self.take_draw(step, pass) {
            Some(mut draw) => {
                self.close_kept(step, &mut draw, pass, engine, out);
                self.keep_draw(step.id, draw, false, pass);
            }
            None => {
                let p = visual::paint(self, step.id);
                self.draw_over(step, &p, pass, engine, out);
            }
        }
        for kind in step.opened().into_iter().rev() {
            match kind {
                Opening::Layer => out.close_layer(),
                Opening::Transform => out.push(D::PopTransform),
                Opening::Group | Opening::ClipGroup => out.push(D::PopGroup),
                Opening::Presence | Opening::Clip => out.push(D::PopClip),
            }
        }
    }

    /// Draw the kept node's draw-over part: replayed when recorded (checked
    /// first, `VERIFY_DRAWS`), drawn when live, else drawn and recorded.
    fn close_kept(
        &self,
        step: &Step,
        draw: &mut NodeDraw,
        pass: PaintPass,
        engine: &mut arvane::Engine,
        out: &mut SceneOut,
    ) {
        let paint = Rc::clone(&draw.paint);
        let spoiled = match &mut draw.recorded {
            Recorded::Commands {
                over: Some(over), ..
            } => {
                if VERIFY_DRAWS {
                    self.verify_over(step, over, pass, engine);
                }
                out.replay(over);
                false
            }
            Recorded::Live => {
                self.draw_over(step, &paint, pass, engine, out);
                false
            }
            Recorded::Commands { over, .. } => {
                out.begin_record();
                self.draw_over(step, &paint, pass, engine, out);
                let fresh = out.end_record();
                let spoiled = fresh.is_none();
                *over = fresh;
                spoiled
            }
        };
        if spoiled {
            draw.recorded = Recorded::Live;
        }
    }

    /// Check a kept draw-over part against the one the node's paint now
    /// draws, recorded in a scratch output (`VERIFY_DRAWS`).
    fn verify_over(&self, step: &Step, over: &[D], pass: PaintPass, engine: &mut arvane::Engine) {
        let paint = visual::paint(self, step.id);
        let mut scratch = SceneOut::default();
        scratch.begin_record();
        self.draw_over(step, &paint, pass, engine, &mut scratch);
        assert!(
            scratch.end_record().as_deref() == Some(over),
            "the kept draw-over of node {:?} is stale",
            step.id
        );
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
                    paint: RasterPaint::Solid(tint.color(color)),
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
