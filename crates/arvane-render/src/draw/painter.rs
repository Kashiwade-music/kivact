//! Role: Records a scene's commands in order through a Canvas, tracking open groups.

use super::canvas::{Canvas, GlyphRun};
use crate::convert;
use crate::derived::Derived;
use crate::outline::glyph_outline;
use crate::paint::{self, Brush, PatternFill};
use crate::text;
use crate::validate::Fonts;
use kivact_types::geometry::Transform;
use kivact_types::render::{FontSource, GlyphStroke, Group, Paint, RenderCmd, RenderScene};
use kivact_types::scene::Shape;
use kivact_types::{PaintOrder, Point, Rect as LogicalRect};
use std::collections::HashMap;
use std::sync::Arc;
use vello_common::filter_effects::{EdgeMode, Filter, FilterPrimitive};
use vello_cpu::kurbo::{self, Affine, BezPath, Rect, Shape as _, Stroke};
use vello_cpu::peniko::{self, Blob, FontData};
use vello_cpu::{Glyph, PaintType};

// The commands each replay category handles, as patterns: the dispatch and every
// category's match are exhaustive, so a new command fails to compile until one handles it.
macro_rules! primitive_commands {
    () => {
        RenderCmd::Rect { .. }
            | RenderCmd::RoundedRect { .. }
            | RenderCmd::RoundedRectCorners { .. }
            | RenderCmd::StrokeRect { .. }
            | RenderCmd::StrokeRoundedRectCorners { .. }
            | RenderCmd::Line { .. }
    };
}
macro_rules! path_commands {
    () => {
        RenderCmd::Shadow { .. } | RenderCmd::Path { .. } | RenderCmd::Glyphs { .. }
    };
}
macro_rules! image_commands {
    () => {
        RenderCmd::Image { .. } | RenderCmd::ImageSampled { .. }
    };
}
macro_rules! group_commands {
    () => {
        RenderCmd::FillShape { .. }
            | RenderCmd::StrokeShape { .. }
            | RenderCmd::BoxShadow(_)
            | RenderCmd::PushClip(_)
            | RenderCmd::PushTransform(_)
            | RenderCmd::PushGroup(_)
            | RenderCmd::PopClip
            | RenderCmd::PopTransform
            | RenderCmd::PopGroup
    };
}

/// Fonts by the identity of their source and face index; the `FontData`
/// keeps the source's `Arc`, so the identity is not reused while it lives.
#[derive(Default)]
pub(crate) struct FontCache(HashMap<(usize, u32), FontData>);

impl FontCache {
    fn get(&mut self, font: &FontSource, index: u32) -> &FontData {
        self.0
            .entry((font.id(), index))
            .or_insert_with(|| FontData::new(Blob::new(Arc::clone(font.source())), index))
    }
}

/// What a rasterizer keeps for [`draw`] from frame to frame.
#[derive(Default)]
pub(crate) struct Caches {
    /// Font validation metadata (`validate`).
    pub(crate) validation: Fonts,
    /// Font data handed to the rasterizer.
    pub(crate) fonts: FontCache,
    /// Pixels derived from the scene's values.
    pub(crate) derived: Derived,
}

/// The frame a scene is drawn into.
pub(crate) struct Frame {
    /// Device pixels per logical pixel.
    pub(crate) scale: f64,
    /// The logical point drawn at the frame's top left corner.
    pub(crate) origin: kivact_types::Point,
    /// The logical rectangle the frame shows, from `origin`.
    pub(crate) visible: LogicalRect,
}

/// Record `scene`'s commands into `canvas` for `frame`.
pub(crate) fn draw(
    canvas: &mut impl Canvas,
    caches: &mut Caches,
    scene: &RenderScene,
    frame: &Frame,
) {
    caches.derived.begin_frame();
    let root = root_transform(frame);
    canvas.set_transform(root);
    let mut painter = Painter {
        canvas,
        fonts: &mut caches.fonts,
        derived: &mut caches.derived,
        root,
        scale: frame.scale,
        origin: frame.origin,
        visible: frame.visible,
        transform: Transform::IDENTITY,
        device_scale: frame.scale,
        stack: Vec::new(),
        hidden: 0,
    };
    for command in &scene.commands {
        painter.command(command);
    }
    caches.derived.end_frame();
}

/// The map from the scene's logical coordinates to device pixels.
pub(crate) fn root_transform(frame: &Frame) -> Affine {
    Affine::scale(frame.scale) * Affine::translate((-frame.origin.x, -frame.origin.y))
}

/// An open group.
pub(super) enum Open {
    /// A clip layer (`PushClip`).
    Clip,
    /// The transform before it.
    Transform(Transform),
    /// How many canvas layers a group pushed.
    Group { pops: u8 },
    /// Opened while drawing nothing.
    Hidden,
}

/// The area a paint fills.
pub(super) enum Area<'a> {
    Rect(Rect),
    Path(&'a BezPath),
}

pub(super) struct Painter<'a, C: Canvas> {
    pub(super) canvas: &'a mut C,
    pub(super) fonts: &'a mut FontCache,
    pub(super) derived: &'a mut Derived,
    /// The map from logical root coordinates to device pixels.
    pub(super) root: Affine,
    /// Device pixels per logical pixel.
    pub(super) scale: f64,
    /// The logical point at the frame's top left corner.
    pub(super) origin: Point,
    /// The frame in root logical coordinates.
    pub(super) visible: LogicalRect,
    /// From the current coordinates to the root's (logical pixels).
    pub(super) transform: Transform,
    /// Device pixels per unit of the current coordinates (at most).
    pub(super) device_scale: f64,
    pub(super) stack: Vec<Open>,
    /// Groups open since drawing stopped (0: drawing).
    pub(super) hidden: usize,
}

impl<C: Canvas> Painter<'_, C> {
    /// Make the current transform the canvas's.
    pub(super) fn set_transform(&mut self) {
        self.canvas
            .set_transform(self.root * convert::affine(self.transform));
        self.device_scale = self.scale * self.transform.max_scale();
    }

    /// Hide everything up to the pop of a group being opened.
    fn hide(&mut self) {
        self.stack.push(Open::Hidden);
        self.hidden = 1;
    }

    /// Replay one command.
    pub(super) fn command(&mut self, command: &RenderCmd) {
        if self.hidden > 0 {
            self.hidden_command(command);
            return;
        }
        match command {
            primitive_commands!() => self.primitive(command),
            path_commands!() => self.path_command(command),
            image_commands!() => self.image_command(command),
            group_commands!() => self.group_command(command),
        }
    }

    /// Replay primitive drawing operations.
    fn primitive(&mut self, command: &RenderCmd) {
        match command {
            RenderCmd::Rect { rect, fill } => {
                self.fill(fill, *rect, Area::Rect(convert::rect(*rect)));
            }
            RenderCmd::RoundedRect { rect, radius, fill } => {
                let path =
                    kurbo::RoundedRect::from_rect(convert::rect(*rect), *radius).to_path(0.1);
                self.fill(fill, *rect, Area::Path(&path));
            }
            RenderCmd::RoundedRectCorners { rect, radii, fill } => {
                let path = convert::rounded_rect(*rect, *radii).to_path(0.1);
                self.fill(fill, *rect, Area::Path(&path));
            }
            RenderCmd::StrokeRect {
                rect,
                width,
                color,
                radius,
            } => {
                self.canvas
                    .set_paint(PaintType::Solid(convert::color(*color)));
                self.canvas.set_stroke(Stroke::new(*width));
                self.canvas.stroke_path(
                    &kurbo::RoundedRect::from_rect(convert::rect(*rect), *radius).to_path(0.1),
                );
            }
            RenderCmd::StrokeRoundedRectCorners {
                rect,
                radii,
                width,
                color,
            } => {
                self.canvas
                    .set_paint(PaintType::Solid(convert::color(*color)));
                self.canvas.set_stroke(Stroke::new(*width));
                self.canvas
                    .stroke_path(&convert::rounded_rect(*rect, *radii).to_path(0.1));
            }
            RenderCmd::Line {
                from,
                to,
                width,
                color,
            } => {
                // Snapping to the pixel grid assumes the root transform.
                let snap = self
                    .transform
                    .is_identity()
                    .then_some((self.scale, self.origin));
                let path = convert::line(*from, *to, *width, snap);
                self.canvas
                    .set_paint(PaintType::Solid(convert::color(*color)));
                self.canvas.set_stroke(Stroke::new(*width));
                self.canvas.stroke_path(&path);
            }
            path_commands!() | image_commands!() | group_commands!() => {
                unreachable!("invariant: the dispatch selects primitive commands")
            }
        }
    }

    /// Replay path command drawing operations.
    fn path_command(&mut self, command: &RenderCmd) {
        match command {
            RenderCmd::Shadow {
                rect,
                radii,
                std_dev,
                color,
            } => {
                self.canvas
                    .set_paint(PaintType::Solid(convert::color(*color)));
                let whole = convert::rect(*rect);
                let blur = convert::blur_param(*std_dev);
                match convert::shadow_quarters(*rect, *radii, *std_dev) {
                    convert::ShadowParts::Whole(radius) => {
                        self.canvas.fill_blurred_rounded_rect(&whole, radius, blur);
                    }
                    convert::ShadowParts::Quarters(quarters) => {
                        for (quarter, radius) in quarters {
                            self.canvas.push_clip_layer(&quarter.to_path(0.1));
                            self.canvas.fill_blurred_rounded_rect(&whole, radius, blur);
                            self.canvas.pop_layer();
                        }
                    }
                }
            }
            RenderCmd::Path { cmds, fill, stroke } => {
                let path = convert::path(cmds);
                if let Some(color) = fill {
                    self.canvas
                        .set_paint(PaintType::Solid(convert::color(*color)));
                    self.canvas.fill_path(&path);
                }
                if let Some((width, color)) = stroke {
                    self.canvas
                        .set_paint(PaintType::Solid(convert::color(*color)));
                    self.canvas.set_stroke(Stroke::new(*width));
                    self.canvas.stroke_path(&path);
                }
            }
            RenderCmd::Glyphs {
                run,
                origin,
                fill,
                stroke,
                order,
            } => self.glyphs(run, *origin, fill.as_ref(), stroke.as_ref(), *order),
            primitive_commands!() | image_commands!() | group_commands!() => {
                unreachable!("invariant: the dispatch selects path commands")
            }
        }
    }

    /// Replay image command drawing operations.
    fn image_command(&mut self, command: &RenderCmd) {
        match command {
            RenderCmd::Image {
                image,
                rect,
                opacity,
            } => {
                if rect.width == 0.0 || rect.height == 0.0 || *opacity == 0.0 {
                    return;
                }
                let size = (image.width, image.height);
                self.sampled(*rect, None, *opacity, size, |canvas, placed| {
                    canvas.draw_image(image, placed, false);
                });
            }
            RenderCmd::ImageSampled {
                image,
                rect,
                source,
                opacity,
                pixelated,
            } => {
                if rect.width == 0.0
                    || rect.height == 0.0
                    || *opacity == 0.0
                    || source.is_some_and(|r| r.width <= 0.0 || r.height <= 0.0)
                {
                    return;
                }
                let size = (image.width, image.height);
                self.sampled(*rect, *source, *opacity, size, |canvas, placed| {
                    canvas.draw_image(image, placed, *pixelated);
                });
            }
            primitive_commands!() | path_commands!() | group_commands!() => {
                unreachable!("invariant: the dispatch selects image commands")
            }
        }
    }

    /// Replay group command drawing operations.
    fn group_command(&mut self, command: &RenderCmd) {
        match command {
            RenderCmd::PushClip(rect) => {
                self.canvas
                    .push_clip_layer(&convert::rect(*rect).to_path(0.1));
                self.stack.push(Open::Clip);
            }
            RenderCmd::FillShape { shape, paint } => {
                if let Shape::Rect(r) = shape {
                    self.fill(paint, *r, Area::Rect(convert::rect(*r)));
                    return;
                }
                let path = convert::shape(shape);
                let rule = convert::fill_rule(shape.rule());
                let odd = rule == peniko::Fill::EvenOdd;
                if odd {
                    self.canvas.set_fill_rule(rule);
                }
                self.fill(paint, shape.bounds(), Area::Path(&path));
                if odd {
                    self.canvas.set_fill_rule(peniko::Fill::NonZero);
                }
            }
            RenderCmd::StrokeShape {
                shape,
                stroke,
                paint,
            } => self.stroke(shape, stroke, paint),
            RenderCmd::BoxShadow(shadow) => self.box_shadow(shadow),
            RenderCmd::PushTransform(t) => {
                let outer = self.transform;
                let inner = t.then(outer);
                let det = inner.determinant();
                if det == 0.0 || !det.is_finite() {
                    self.hide();
                    return;
                }
                self.stack.push(Open::Transform(outer));
                self.transform = inner;
                self.set_transform();
            }
            RenderCmd::PushGroup(group) => self.push_group(group),
            RenderCmd::PopClip | RenderCmd::PopTransform | RenderCmd::PopGroup => self.pop(),
            primitive_commands!() | path_commands!() | image_commands!() => {
                unreachable!("invariant: the dispatch selects group commands")
            }
        }
    }

    /// Open a group: its clip, opacity and blend on one layer, and its
    /// blur on a filter layer inside it (the filtered content is clipped,
    /// as CSS orders filter, clip-path and opacity).
    fn push_group(&mut self, group: &Group) {
        if group.opacity == 0.0 {
            self.hide();
            return;
        }
        let clip = group
            .clip
            .as_ref()
            .map(|shape| (convert::shape(shape), shape.rule()));
        let blend = convert::blend(group.blend);
        let opacity = (group.opacity < 1.0).then_some(group.opacity);
        let filter = group.blur.map(|std_dev| {
            Filter::from_primitive(FilterPrimitive::GaussianBlur {
                std_deviation: std_dev as f32,
                edge_mode: EdgeMode::None,
            })
        });
        let mut pops = 0;
        if clip.is_some() || blend.is_some() || opacity.is_some() || filter.is_none() {
            let odd = clip
                .as_ref()
                .is_some_and(|c| c.1 != kivact_types::FillRule::NonZero);
            if odd {
                self.canvas.set_fill_rule(peniko::Fill::EvenOdd);
            }
            self.canvas
                .push_layer(clip.as_ref().map(|c| &c.0), blend, opacity, None);
            if odd {
                self.canvas.set_fill_rule(peniko::Fill::NonZero);
            }
            pops += 1;
        }
        if let Some(filter) = filter {
            self.canvas.push_layer(None, None, None, Some(filter));
            pops += 1;
        }
        self.stack.push(Open::Group { pops });
    }

    /// Draw a `width` × `height` pixel source (an image) into `rect`: its
    /// `source` region (`None`: all of it) scaled onto `rect`, faded by
    /// `opacity`. `draw` draws the whole source stretched over the
    /// rectangle it is given.
    fn sampled(
        &mut self,
        rect: LogicalRect,
        source: Option<LogicalRect>,
        opacity: f64,
        (width, height): (u32, u32),
        draw: impl FnOnce(&mut C, Rect),
    ) {
        // Neither rasterizer applies a sampler's alpha; a layer clipped to
        // the image applies the opacity without copying pixels.
        let bounds = convert::rect(rect);
        let faded = opacity < 1.0;
        if faded {
            self.canvas
                .push_opacity_layer(&bounds.to_path(0.1), opacity);
        }
        if let Some(source) = source {
            self.canvas.push_clip_layer(&bounds.to_path(0.1));
            let sx = rect.width / source.width;
            let sy = rect.height / source.height;
            let full = Rect::new(
                rect.x - source.x * sx,
                rect.y - source.y * sy,
                rect.x - source.x * sx + width as f64 * sx,
                rect.y - source.y * sy + height as f64 * sy,
            );
            draw(self.canvas, full);
            self.canvas.pop_layer();
        } else {
            draw(self.canvas, bounds);
        }
        if faded {
            self.canvas.pop_layer();
        }
    }

    /// A command while drawing nothing: only the groups are followed.
    fn hidden_command(&mut self, command: &RenderCmd) {
        match command {
            RenderCmd::PushClip(_) | RenderCmd::PushTransform(_) | RenderCmd::PushGroup(_) => {
                self.hidden += 1;
            }
            RenderCmd::PopClip | RenderCmd::PopTransform | RenderCmd::PopGroup => {
                self.hidden -= 1;
                if self.hidden == 0 {
                    self.pop();
                }
            }
            _ => {}
        }
    }

    /// Close the innermost group.
    fn pop(&mut self) {
        match self.stack.pop() {
            Some(Open::Clip) => self.canvas.pop_layer(),
            Some(Open::Transform(outer)) => {
                self.transform = outer;
                self.set_transform();
            }
            Some(Open::Group { pops }) => {
                for _ in 0..pops {
                    self.canvas.pop_layer();
                }
            }
            Some(Open::Hidden) | None => {}
        }
    }

    /// The frame in the current coordinates (bounds of it).
    pub(super) fn local_frame(&self) -> LogicalRect {
        match self.transform.invert() {
            Some(inverse) => inverse.bounds(self.visible),
            None => LogicalRect::default(),
        }
    }

    /// Fill `area` (with `bounds`) with `paint`.
    pub(super) fn fill(&mut self, paint: &Paint, bounds: LogicalRect, area: Area<'_>) {
        match paint::brush(paint, self.derived, self.device_scale) {
            Brush::Paint(brush, transform) => {
                self.canvas.set_paint(brush);
                if let Some(t) = transform {
                    self.canvas.set_paint_transform(t);
                }
                match area {
                    Area::Rect(r) => self.canvas.fill_rect(&r),
                    Area::Path(p) => self.canvas.fill_path(p),
                }
                if transform.is_some() {
                    self.canvas.reset_paint_transform();
                }
            }
            Brush::Pattern(pattern) => {
                let rect_path;
                let path = match area {
                    Area::Rect(r) => {
                        rect_path = r.to_path(0.1);
                        &rect_path
                    }
                    Area::Path(p) => p,
                };
                self.fill_pattern(path, bounds, &pattern);
            }
        }
    }

    /// Fill `path` (within `bounds`) with a pattern, transparent beyond its
    /// copies along an axis it does not repeat along.
    fn fill_pattern(&mut self, path: &BezPath, bounds: LogicalRect, pattern: &PatternFill) {
        let visible = convert::rect(self.local_frame());
        if pattern.repeat_x && pattern.repeat_y {
            self.canvas.fill_pattern(path, pattern, visible);
            return;
        }
        let tile = pattern.tile;
        let mut band = bounds;
        if !pattern.repeat_x {
            band.x = tile.x;
            band.width = tile.width;
        }
        if !pattern.repeat_y {
            band.y = tile.y;
            band.height = tile.height;
        }
        self.canvas
            .push_clip_path(&convert::rect(band).to_path(0.1));
        self.canvas.fill_pattern(path, pattern, visible);
        self.canvas.pop_clip_path();
    }

    /// Stroke a shape's outline with `paint`.
    fn stroke(&mut self, shape: &Shape, stroke: &kivact_types::Stroke, paint: &Paint) {
        let mut path = convert::shape(shape);
        if stroke.cap != kivact_types::LineCap::Butt {
            path = convert::dots(path);
        }
        let style = convert::stroke(stroke);
        let grow = stroke.width * stroke.miter_limit.max(1.5);
        match paint::brush(paint, self.derived, self.device_scale) {
            Brush::Pattern(pattern) => {
                // The stroke's outline filled with the pattern.
                let outline = self.outline_of(&path, &style);
                let bounds = logical(path.bounding_box()).inflate(grow, grow);
                self.fill_pattern(&outline, bounds, &pattern);
            }
            Brush::Paint(brush, transform) => {
                self.canvas.set_paint(brush);
                if let Some(t) = transform {
                    self.canvas.set_paint_transform(t);
                }
                self.canvas.set_stroke(style);
                self.canvas.stroke_path(&path);
                if transform.is_some() {
                    self.canvas.reset_paint_transform();
                }
            }
        }
    }

    /// The outline of `path` stroked with `style`, as a filled path.
    fn outline_of(&self, path: &BezPath, style: &Stroke) -> BezPath {
        kurbo::stroke(
            path.iter(),
            style,
            &kurbo::StrokeOpts::default(),
            0.1 / self.device_scale.max(1.0),
        )
    }

    /// The outlines of `run`'s glyphs at `origin`, in the current coordinates.
    fn glyph_path(&self, run: &kivact_types::render::GlyphRun, origin: Point) -> BezPath {
        let mut path = convert::path(&glyph_outline(run));
        path.apply_affine(Affine::translate((origin.x, origin.y)));
        path
    }

    /// Replay a glyph run: its fill, and its outline stroke, in `order`.
    fn glyphs(
        &mut self,
        run: &kivact_types::render::GlyphRun,
        origin: Point,
        fill: Option<&Paint>,
        outline: Option<&GlyphStroke>,
        order: PaintOrder,
    ) {
        let shaped = Paint::Solid(run.color);
        let fill = fill.unwrap_or(&shaped);
        match order {
            PaintOrder::FillStroke => {
                self.glyph_fill(run, origin, fill);
                if let Some(stroke) = outline {
                    self.glyph_stroke(run, origin, stroke);
                }
            }
            PaintOrder::StrokeFill => {
                if let Some(stroke) = outline {
                    self.glyph_stroke(run, origin, stroke);
                }
                self.glyph_fill(run, origin, fill);
            }
        }
    }

    /// Fill a glyph run with `paint`.
    fn glyph_fill(&mut self, run: &kivact_types::render::GlyphRun, origin: Point, paint: &Paint) {
        match paint::brush(paint, self.derived, self.device_scale) {
            Brush::Pattern(pattern) => {
                let path = self.glyph_path(run, origin);
                let bounds = logical(path.bounding_box());
                self.fill_pattern(&path, bounds, &pattern);
            }
            Brush::Paint(brush, transform) => {
                let solid = matches!(paint, Paint::Solid(_));
                self.canvas.set_paint(brush);
                if let Some(t) = transform {
                    self.canvas.set_paint_transform(t);
                }
                self.fill_glyphs(run, origin, solid);
                if transform.is_some() {
                    self.canvas.reset_paint_transform();
                }
            }
        }
    }

    /// Stroke a glyph run's outlines with `stroke`.
    fn glyph_stroke(
        &mut self,
        run: &kivact_types::render::GlyphRun,
        origin: Point,
        stroke: &GlyphStroke,
    ) {
        let style = convert::stroke(&stroke.stroke);
        match paint::brush(&stroke.paint, self.derived, self.device_scale) {
            Brush::Pattern(pattern) => {
                let path = self.glyph_path(run, origin);
                let outline = self.outline_of(&path, &style);
                let grow = stroke.stroke.width * stroke.stroke.miter_limit.max(1.5);
                let bounds = logical(path.bounding_box()).inflate(grow, grow);
                self.fill_pattern(&outline, bounds, &pattern);
            }
            Brush::Paint(brush, transform) => {
                self.canvas.set_paint(brush);
                if let Some(t) = transform {
                    self.canvas.set_paint_transform(t);
                }
                self.canvas.set_stroke(style);
                let coords = glyph_coords(run);
                let font = self.fonts.get(&run.font, run.font_index);
                // Outlines are stroked as they are: unhinted, no darkening,
                // never from the atlas.
                self.canvas.stroke_glyphs(
                    GlyphRun {
                        font,
                        font_size: run.font_size,
                        coords: &coords,
                        embolden: glifo::FontEmbolden::new(vello_cpu::kurbo::Diagonal2::new(
                            0.0, 0.0,
                        )),
                        hint: false,
                        cached: false,
                    },
                    run.glyphs.iter().map(|g| Glyph {
                        id: g.id,
                        x: (g.x + origin.x) as f32,
                        y: (g.y + origin.y) as f32,
                    }),
                );
                if transform.is_some() {
                    self.canvas.reset_paint_transform();
                }
            }
        }
    }

    /// Fill a glyph run with the current paint, placed as the transform says
    /// (`solid`: the glyphs may come from the atlas).
    fn fill_glyphs(&mut self, run: &kivact_types::render::GlyphRun, origin: Point, solid: bool) {
        let coords = glyph_coords(run);
        let font = self.fonts.get(&run.font, run.font_index);
        let scale = self.scale;
        let t = self.transform;
        if t.is_identity() {
            self.canvas.fill_glyphs(
                GlyphRun {
                    font,
                    font_size: run.font_size,
                    coords: &coords,
                    embolden: glifo::FontEmbolden::new(text::darkening(run.font_size * scale)),
                    hint: true,
                    cached: solid,
                },
                run.glyphs.iter().map(|g| Glyph {
                    id: g.id,
                    x: text::subpixel(g.x + origin.x, scale) as f32,
                    y: (g.y + origin.y) as f32,
                }),
            );
        } else if let Some(s) = t.uniform_scale() {
            // Hinted and on quarter device pixels, through the transform.
            self.canvas.fill_glyphs(
                GlyphRun {
                    font,
                    font_size: run.font_size,
                    coords: &coords,
                    embolden: glifo::FontEmbolden::new(text::darkening(run.font_size * scale * s)),
                    hint: true,
                    cached: solid,
                },
                run.glyphs.iter().map(|g| Glyph {
                    id: g.id,
                    x: ((text::subpixel((g.x + origin.x) * s + t.e, scale) - t.e) / s) as f32,
                    y: (g.y + origin.y) as f32,
                }),
            );
        } else {
            // Rotated, skewed or stretched: unhinted outlines where the
            // shaping put them.
            let size = run.font_size * scale * t.determinant().abs().sqrt();
            self.canvas.fill_glyphs(
                GlyphRun {
                    font,
                    font_size: run.font_size,
                    coords: &coords,
                    embolden: glifo::FontEmbolden::new(text::darkening(size)),
                    hint: false,
                    cached: false,
                },
                run.glyphs.iter().map(|g| Glyph {
                    id: g.id,
                    x: (g.x + origin.x) as f32,
                    y: (g.y + origin.y) as f32,
                }),
            );
        }
    }
}

/// A run's normalized variation coordinates in the rasterizers' 2.14 fixed point.
fn glyph_coords(run: &kivact_types::render::GlyphRun) -> Vec<i16> {
    run.normalized_coords
        .iter()
        .map(|v| (*v * 16384.0).round() as i16)
        .collect()
}

/// A raster rectangle as a logical one.
fn logical(rect: Rect) -> LogicalRect {
    LogicalRect {
        x: rect.x0,
        y: rect.y0,
        width: rect.width(),
        height: rect.height(),
    }
}
