//! Role: Records a scene's commands in order through a Canvas, tracking open groups.

use super::canvas::{Canvas, GlyphRun};
use crate::backdrop::Index;
use crate::derived::Derived;
use crate::gradient;
use crate::paint::{self, Brush, PatternFill};
use crate::{convert, text};
use kivact_types::{
    ColorMatrix, DrawCommand, Mask, Paint, Rect as LogicalRect, Rgba, Scene, Shape, Size, Transform,
};
use std::collections::HashMap;
use std::sync::Arc;
use vello_common::filter_effects::{EdgeMode, Filter, FilterPrimitive};
use vello_cpu::kurbo::{self, Affine, BezPath, Rect, RoundedRect, Shape as _, Stroke};
use vello_cpu::peniko::{self, Blob, FontData};
use vello_cpu::{Glyph, PaintType};

/// Fonts by the address of their bytes and face index; the `FontData`
/// keeps the bytes' `Arc`, so the address is not reused while it lives.
#[derive(Default)]
pub(crate) struct Fonts(HashMap<(usize, u32), FontData>);

impl Fonts {
    fn get(&mut self, font: &Arc<[u8]>, index: u32) -> &FontData {
        self.0
            .entry((font.as_ptr() as usize, index))
            .or_insert_with(|| FontData::new(Blob::new(Arc::new(Arc::clone(font))), index))
    }
}

/// What a renderer keeps for [`draw`] from frame to frame.
#[derive(Default)]
pub(crate) struct Caches {
    pub(crate) validation: crate::validate::Fonts,
    pub(crate) fonts: Fonts,
    pub(crate) derived: Derived,
}

/// The frame a scene is drawn into.
pub(crate) struct Frame {
    /// Device pixels per logical pixel (the canvas's root transform).
    pub(crate) scale: f32,
    /// Logical extent.
    pub(crate) size: Size,
    /// Whether the scene has a backdrop (`validate::scene` tells).
    pub(crate) backdrops: bool,
}

/// Record `scene`'s commands into `canvas` for `frame`.
pub(crate) fn draw(canvas: &mut impl Canvas, caches: &mut Caches, scene: &Scene, frame: &Frame) {
    let index = frame.backdrops.then(|| Index::new(scene));
    caches.derived.begin_frame();
    canvas.set_transform(Affine::scale(f64::from(frame.scale)));
    let mut painter = Painter {
        canvas,
        fonts: &mut caches.fonts,
        derived: &mut caches.derived,
        scene,
        index: index.as_ref(),
        scale: frame.scale,
        frame: LogicalRect {
            x: 0.0,
            y: 0.0,
            width: frame.size.width,
            height: frame.size.height,
        },
        transform: Transform::IDENTITY,
        device_scale: frame.scale,
        stack: Vec::new(),
        hidden: 0,
        matrix: None,
        replaying: false,
    };
    for (i, command) in scene.commands.iter().enumerate() {
        painter.command(i, command);
    }
    caches.derived.end_frame();
}

/// An open group.
pub(super) enum Group {
    Clip,
    /// The transform before it.
    Transform(Transform),
    /// How many canvas layers it pushed, and the mask applied at its pop.
    Layer {
        pops: u8,
        mask: Option<Mask>,
    },
    /// Opened before a backdrop's replay began: its push is not drawn
    /// again, and its pop is not replayed.
    Ancestor,
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
    pub(super) fonts: &'a mut Fonts,
    pub(super) derived: &'a mut Derived,
    pub(super) scene: &'a Scene,
    pub(super) index: Option<&'a Index>,
    pub(super) scale: f32,
    /// The frame in root logical coordinates.
    pub(super) frame: LogicalRect,
    /// From the current coordinates to the root's (logical pixels).
    pub(super) transform: Transform,
    /// Device pixels per unit of the current coordinates (at most).
    pub(super) device_scale: f32,
    pub(super) stack: Vec<Group>,
    /// Groups open since drawing stopped (0: drawing).
    pub(super) hidden: usize,
    /// The colors of what is drawn go through it (a backdrop's replay).
    pub(super) matrix: Option<ColorMatrix>,
    pub(super) replaying: bool,
}

impl<C: Canvas> Painter<'_, C> {
    /// Make the current transform the canvas's.
    pub(super) fn set_transform(&mut self) {
        self.canvas
            .set_transform(Affine::scale(f64::from(self.scale)) * convert::affine(self.transform));
        self.device_scale = self.scale * self.transform.max_scale();
    }

    /// `color` through the replay's matrix, as the rasterizer takes it.
    fn color(&self, color: Rgba) -> vello_cpu::color::AlphaColor<vello_cpu::color::Srgb> {
        convert::color(gradient::color(color, self.matrix.as_ref()))
    }

    /// Hide everything up to the pop of a group being opened.
    fn hide(&mut self) {
        self.stack.push(Group::Hidden);
        self.hidden = 1;
    }

    pub(super) fn command(&mut self, i: usize, command: &DrawCommand) {
        if self.hidden > 0 {
            self.hidden_command(command);
            return;
        }
        match command {
            DrawCommand::Rect { rect, fill } => {
                self.fill(fill, *rect, Area::Rect(convert::rect(*rect)));
            }
            DrawCommand::RoundedRect { rect, radius, fill } => {
                let path =
                    RoundedRect::from_rect(convert::rect(*rect), f64::from(*radius)).to_path(0.1);
                self.fill(fill, *rect, Area::Path(&path));
            }
            DrawCommand::RoundedRectCorners { rect, radii, fill } => {
                let path = convert::rounded_rect(*rect, *radii).to_path(0.1);
                self.fill(fill, *rect, Area::Path(&path));
            }
            DrawCommand::StrokeRect {
                rect,
                width,
                color,
                radius,
            } => {
                self.canvas.set_paint(PaintType::Solid(self.color(*color)));
                self.canvas.set_stroke(Stroke::new(f64::from(*width)));
                self.canvas.stroke_path(
                    &RoundedRect::from_rect(convert::rect(*rect), f64::from(*radius)).to_path(0.1),
                );
            }
            DrawCommand::StrokeRoundedRectCorners {
                rect,
                radii,
                width,
                color,
            } => {
                self.canvas.set_paint(PaintType::Solid(self.color(*color)));
                self.canvas.set_stroke(Stroke::new(f64::from(*width)));
                self.canvas
                    .stroke_path(&convert::rounded_rect(*rect, *radii).to_path(0.1));
            }
            DrawCommand::Line {
                from,
                to,
                width,
                color,
            } => {
                // Snapping to the pixel grid assumes the root transform.
                let snap = self.transform.is_identity().then_some(self.scale);
                let path = convert::line(*from, *to, *width, snap);
                self.canvas.set_paint(PaintType::Solid(self.color(*color)));
                self.canvas.set_stroke(Stroke::new(f64::from(*width)));
                self.canvas.stroke_path(&path);
            }
            DrawCommand::Shadow {
                rect,
                radii,
                std_dev,
                color,
            } => {
                self.canvas.set_paint(PaintType::Solid(self.color(*color)));
                let whole = convert::rect(*rect);
                let blur = convert::blur_param(*std_dev) as f32;
                match convert::shadow_quarters(*rect, *radii, *std_dev) {
                    convert::ShadowParts::Whole(radius) => {
                        self.canvas
                            .fill_blurred_rounded_rect(&whole, radius as f32, blur);
                    }
                    convert::ShadowParts::Quarters(quarters) => {
                        for (quarter, radius) in quarters {
                            self.canvas.push_clip_layer(&quarter.to_path(0.1));
                            self.canvas
                                .fill_blurred_rounded_rect(&whole, radius as f32, blur);
                            self.canvas.pop_layer();
                        }
                    }
                }
            }
            DrawCommand::Path { cmds, fill, stroke } => {
                let path = convert::path(cmds);
                if let Some(color) = fill {
                    self.canvas.set_paint(PaintType::Solid(self.color(*color)));
                    self.canvas.fill_path(&path);
                }
                if let Some((width, color)) = stroke {
                    self.canvas.set_paint(PaintType::Solid(self.color(*color)));
                    self.canvas.set_stroke(Stroke::new(f64::from(*width)));
                    self.canvas.stroke_path(&path);
                }
            }
            DrawCommand::Glyphs { run, origin, color } => self.glyphs(run, *origin, *color),
            DrawCommand::Image {
                image,
                rect,
                opacity,
                color_matrix,
            }
            | DrawCommand::ImageSampled {
                image,
                rect,
                opacity,
                color_matrix,
                ..
            } => {
                let (source, pixelated) = match command {
                    DrawCommand::ImageSampled {
                        source, pixelated, ..
                    } => (*source, *pixelated),
                    _ => (None, false),
                };
                if rect.width == 0.0
                    || rect.height == 0.0
                    || *opacity == 0.0
                    || source.is_some_and(|r| r.width <= 0.0 || r.height <= 0.0)
                {
                    return;
                }
                let matrix = match (color_matrix, &self.matrix) {
                    (Some(own), Some(m)) => Some(own.then(*m)),
                    (own, m) => own.or(*m),
                };
                let filtered;
                let image = match matrix {
                    Some(m) => {
                        filtered = self.derived.filtered(image, &m);
                        &filtered
                    }
                    None => image,
                };
                let size = (image.width, image.height);
                self.sampled(*rect, source, *opacity, size, |canvas, placed| {
                    canvas.draw_image(image, placed, pixelated);
                });
            }
            DrawCommand::External {
                frame,
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
                // Drawn only where the renderer has the frame ready.
                let Some(size) = self.canvas.external_size(frame) else {
                    return;
                };
                self.sampled(*rect, *source, *opacity, size, |canvas, placed| {
                    canvas.draw_external(frame, placed, *pixelated);
                });
            }
            DrawCommand::PushClip(rect) => {
                self.canvas
                    .push_clip_layer(&convert::rect(*rect).to_path(0.1));
                self.stack.push(Group::Clip);
            }
            DrawCommand::FillShape { shape, paint } => {
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
            DrawCommand::StrokeShape {
                shape,
                stroke,
                paint,
            } => self.stroke(shape, stroke, paint),
            DrawCommand::BoxShadow(shadow) => self.box_shadow(shadow),
            DrawCommand::PushTransform(t) => {
                let outer = self.transform;
                let inner = t.then(outer);
                let det = inner.determinant();
                if det == 0.0 || !det.is_finite() {
                    self.hide();
                    return;
                }
                self.stack.push(Group::Transform(outer));
                self.transform = inner;
                self.set_transform();
            }
            DrawCommand::PushLayer(layer) => {
                if layer.opacity == 0.0 {
                    self.hide();
                    return;
                }
                let clip = layer
                    .clip
                    .as_ref()
                    .map(|shape| (convert::shape(shape), shape.rule()));
                let blend = convert::blend(layer.blend);
                let opacity = (layer.opacity < 1.0).then_some(layer.opacity);
                let filter = layer.filter.map(|f| self.filter(f));
                let mut pops = 0;
                if clip.is_some()
                    || blend.is_some()
                    || opacity.is_some()
                    || layer.mask.is_some()
                    || filter.is_none()
                {
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
                    // Inside the clip: the filtered result is clipped, as CSS
                    // orders filter, clip-path, mask and opacity.
                    self.canvas.push_layer(None, None, None, Some(filter));
                    pops += 1;
                }
                self.stack.push(Group::Layer {
                    pops,
                    mask: layer.mask.clone(),
                });
            }
            DrawCommand::Backdrop(backdrop) => {
                if !self.replaying {
                    self.backdrop(i, backdrop);
                }
            }
            DrawCommand::PopClip | DrawCommand::PopTransform | DrawCommand::PopLayer => self.pop(),
        }
    }

    /// A command while drawing nothing: only the groups are followed.
    /// Draw a `width` × `height` pixel source (an image, an external
    /// frame) into `rect`: its `source` region (`None`: all of it) scaled
    /// onto `rect`, faded by `opacity`. `draw` draws the whole source
    /// stretched over the rectangle it is given.
    fn sampled(
        &mut self,
        rect: LogicalRect,
        source: Option<LogicalRect>,
        opacity: f32,
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
                f64::from(rect.x - source.x * sx),
                f64::from(rect.y - source.y * sy),
                f64::from(rect.x - source.x * sx + width as f32 * sx),
                f64::from(rect.y - source.y * sy + height as f32 * sy),
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

    fn hidden_command(&mut self, command: &DrawCommand) {
        match command {
            DrawCommand::PushClip(_)
            | DrawCommand::PushTransform(_)
            | DrawCommand::PushLayer(_) => {
                self.hidden += 1;
            }
            DrawCommand::PopClip | DrawCommand::PopTransform | DrawCommand::PopLayer => {
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
            Some(Group::Clip) => self.canvas.pop_layer(),
            Some(Group::Transform(outer)) => {
                self.transform = outer;
                self.set_transform();
            }
            Some(Group::Layer { mut pops, mask }) => {
                if let Some(mask) = mask {
                    if pops == 2 {
                        // The filter's layer first: the mask applies to the
                        // filtered content.
                        self.canvas.pop_layer();
                        pops = 1;
                    }
                    self.mask(&mask);
                }
                for _ in 0..pops {
                    self.canvas.pop_layer();
                }
            }
            Some(Group::Hidden | Group::Ancestor) | None => {}
        }
    }

    /// The raster filter of a layer's.
    fn filter(&self, filter: kivact_types::LayerFilter) -> Filter {
        Filter::from_primitive(match filter {
            kivact_types::LayerFilter::Blur { std_dev } => FilterPrimitive::GaussianBlur {
                std_deviation: std_dev,
                edge_mode: EdgeMode::None,
            },
            kivact_types::LayerFilter::DropShadow {
                offset,
                std_dev,
                color,
            } => FilterPrimitive::DropShadow {
                dx: offset.x,
                dy: offset.y,
                std_deviation: std_dev,
                color: self.color(color),
                edge_mode: EdgeMode::None,
            },
        })
    }

    /// Fill `area` (with `bounds`) with `paint`.
    pub(super) fn fill(&mut self, paint: &Paint, bounds: LogicalRect, area: Area<'_>) {
        let device_scale = self.device_scale;
        match paint::brush(
            paint,
            bounds,
            self.matrix.as_ref(),
            self.derived,
            device_scale,
        ) {
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

    fn stroke(&mut self, shape: &Shape, stroke: &kivact_types::Stroke, paint: &Paint) {
        let mut path = convert::shape(shape);
        if stroke.cap != kivact_types::LineCap::Butt {
            path = convert::dots(path);
        }
        let style = convert::stroke(stroke);
        let bounds = shape.bounds();
        if let Paint::Pattern(_) = paint {
            // The stroke's outline filled with the pattern.
            let outline = kurbo::stroke(
                path.iter(),
                &style,
                &kurbo::StrokeOpts::default(),
                0.1 / f64::from(self.device_scale.max(1.0)),
            );
            let w = stroke.width * stroke.miter_limit.max(1.5);
            self.fill(paint, bounds.inflate(w, w), Area::Path(&outline));
            return;
        }
        let device_scale = self.device_scale;
        let Brush::Paint(brush, transform) = paint::brush(
            paint,
            bounds,
            self.matrix.as_ref(),
            self.derived,
            device_scale,
        ) else {
            unreachable!("invariant: only patterns make pattern brushes");
        };
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

    fn glyphs(
        &mut self,
        run: &kivact_types::GlyphRun,
        origin: kivact_types::Point,
        color: Option<kivact_types::Rgba>,
    ) {
        let coords: Vec<i16> = run
            .normalized_coords
            .iter()
            .map(|v| (*v * 16384.0).round() as i16)
            .collect();
        let color = self.color(color.unwrap_or(run.color));
        self.canvas.set_paint(PaintType::Solid(color));
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
                    cached: true,
                },
                run.glyphs.iter().map(|g| Glyph {
                    id: g.id,
                    x: text::subpixel(g.x + origin.x, scale),
                    y: g.y + origin.y,
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
                    cached: true,
                },
                run.glyphs.iter().map(|g| Glyph {
                    id: g.id,
                    x: (text::subpixel((g.x + origin.x) * s + t.e, scale) - t.e) / s,
                    y: g.y + origin.y,
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
                    x: g.x + origin.x,
                    y: g.y + origin.y,
                }),
            );
        }
    }
}
