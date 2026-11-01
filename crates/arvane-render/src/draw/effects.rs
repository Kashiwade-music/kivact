//! Role: Box shadows, masks and backdrops drawn with what both rasterizers have.
//!
//! Box shadows, masks and backdrops: drawn with what both rasterizers
//! have (vello_hybrid has no mask layers, no destructive blending and no
//! color matrix filter, and nothing reads pixels back).
//!
//! - Masks: at the masked layer's pop, a group composited with
//!   destination-out holds `1 - mask`: opaque everywhere, minus the mask
//!   paint drawn with destination-out. Removing it from the layer leaves
//!   the content times the mask's alpha, for any paint.
//! - Inset shadows: inside a layer clipped to the box, the shadow color
//!   over the whole box minus the blurred hole (destination-out).
//! - Outer shadows: the blurred rounded rect of `fill_blurred_rounded_rect`
//!   (in quarters for different radii) when the corners are circular, else
//!   the shape through a blur filter layer; a knockout removes the box
//!   from an isolated layer holding the shadow.
//! - Backdrops: what is beneath the shape is removed (destination-out),
//!   and the commands before the backdrop that reach its area are drawn
//!   again inside a layer clipped to the shape, through a blur filter
//!   layer, with the colors through the backdrop's matrix. A backdrop sees
//!   the frame only: where the area the blur reads leaves the frame, the
//!   frame's content is drawn again mirrored across the edge
//!   (`mirror_edges`, CSS backdrop roots end at the viewport), so neither
//!   the transparent outside (which darkened the window's edges) nor
//!   content beyond the window is blurred into them.
//!
//! An outer shadow's box is moved by its offset and grown by its spread,
//! its corners grown as CSS grows them (`spread_radius`: a radius smaller
//! than the spread grows by less, a sharp corner stays sharp). A knockout
//! costs an isolated layer and destination-out drawing, so a host whose
//! box is opaque leaves it unset. An inset shadow's hole is the box shrunk
//! by the spread (its radii too), blurred; a standard deviation of 0 is a
//! hard edge. A backdrop's replay reads `SHADOW_REACH` standard deviations
//! around the shape's bounds (`backdrop.rs` indexes which commands reach
//! there); the groups open around the backdrop are not drawn again, only
//! their transforms followed.
//!
//! Limits:
//! - Destination-out drawing (masks, inset shadows, a backdrop's removal
//!   of what is beneath) reaches only the innermost layer's pixels.
//! - A backdrop does not see what another backdrop drew, only the commands
//!   beneath both. Its color matrix applies to each replayed command's
//!   colors: the same as applying it to their composite for matrices
//!   without offsets (saturate, hue rotation, grayscale, sepia,
//!   brightness), an approximation for the others (contrast, invert).
//! - A shadow with different corner radii is drawn in quarters, each with
//!   its corner's radius (`fill_blurred_rounded_rect` takes one radius):
//!   where the blur is larger than the box, the quarters' seams may not
//!   join smoothly.
//! - vello_hybrid panics on mask layers, destructive blending and chained
//!   filters; no scene draws them (this module draws their effects with
//!   destination-out instead).

use super::canvas::Canvas;
use super::painter::{Area, Group, Painter};
use crate::convert;
use kivact_types::{
    Backdrop, BoxShadow, DrawCommand, Mask, Radius, Rect as LogicalRect, Rgba,
    RoundedRect as Rounded, SHADOW_REACH, Shape, Transform,
};
use vello_common::filter_effects::{EdgeMode, Filter, FilterPrimitive};
use vello_cpu::PaintType;
use vello_cpu::color::palette::css::BLACK;
use vello_cpu::kurbo::Shape as _;

impl<C: Canvas> Painter<'_, C> {
    /// The frame in the current coordinates (bounds of it).
    pub(super) fn local_frame(&self) -> LogicalRect {
        match self.transform.invert() {
            Some(inverse) => inverse.bounds(self.frame),
            None => LogicalRect::default(),
        }
    }

    /// Apply `mask` to the content of the innermost layer.
    pub(super) fn mask(&mut self, mask: &Mask) {
        let dest_out = convert::dest_out();
        self.canvas.push_layer(None, Some(dest_out), None, None);
        let everywhere = self.local_frame().union(mask.rect);
        self.canvas.set_paint(PaintType::Solid(BLACK));
        self.canvas.fill_rect(&convert::rect(everywhere));
        self.canvas.set_blend_mode(dest_out);
        // The mask's alpha only: the replay's colors do not apply.
        let matrix = self.matrix.take();
        self.fill(&mask.paint, mask.rect, Area::Rect(convert::rect(mask.rect)));
        self.matrix = matrix;
        self.canvas.set_blend_mode(Default::default());
        self.canvas.pop_layer();
    }

    pub(super) fn box_shadow(&mut self, shadow: &BoxShadow) {
        let color = crate::gradient::color(shadow.color, self.matrix.as_ref());
        if color.a <= 0.0 {
            return;
        }
        let shape = shadow.shape;
        let moved = |r: LogicalRect| LogicalRect {
            x: r.x + shadow.offset.x,
            y: r.y + shadow.offset.y,
            ..r
        };
        if shadow.inset {
            let hole = Rounded {
                rect: moved(shape.rect).inflate(-shadow.spread, -shadow.spread),
                radii: shape.fitted_radii().map(|r| Radius {
                    x: (r.x - shadow.spread).max(0.0),
                    y: (r.y - shadow.spread).max(0.0),
                }),
            };
            let clip = convert::shape(&Shape::RoundedRect(shape));
            self.canvas.push_layer(Some(&clip), None, None, None);
            self.canvas
                .set_paint(PaintType::Solid(convert::color(color)));
            self.canvas.fill_rect(&convert::rect(shape.rect));
            self.blurred(
                &hole,
                shadow.std_dev,
                Rgba {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                true,
            );
            self.canvas.pop_layer();
            return;
        }
        let spread = shadow.spread;
        let cast = Rounded {
            rect: moved(shape.rect).inflate(spread, spread),
            radii: shape.fitted_radii().map(|r| Radius {
                x: spread_radius(r.x, spread),
                y: spread_radius(r.y, spread),
            }),
        };
        if shadow.knockout {
            self.canvas.push_layer(None, None, None, None);
        }
        self.blurred(&cast, shadow.std_dev, color, false);
        if shadow.knockout {
            let dest_out = convert::dest_out();
            self.canvas.set_blend_mode(dest_out);
            self.canvas.set_paint(PaintType::Solid(BLACK));
            self.canvas
                .fill_path(&convert::shape(&Shape::RoundedRect(shape)));
            self.canvas.set_blend_mode(Default::default());
            self.canvas.pop_layer();
        }
    }

    /// `shape` blurred by a gaussian of `std_dev`, in `color`, removed from
    /// what is beneath when `remove`.
    fn blurred(&mut self, shape: &Rounded, std_dev: f32, color: Rgba, remove: bool) {
        let dest_out = convert::dest_out();
        let paint = PaintType::Solid(convert::color(color));
        if shape.rect.width <= 0.0 || shape.rect.height <= 0.0 {
            return;
        }
        if std_dev <= 0.0 {
            if remove {
                self.canvas.set_blend_mode(dest_out);
            }
            self.canvas.set_paint(paint);
            self.canvas
                .fill_path(&convert::shape(&Shape::RoundedRect(*shape)));
            if remove {
                self.canvas.set_blend_mode(Default::default());
            }
            return;
        }
        if shape.is_circular() {
            let radii = shape.fitted_radii().map(|r| r.x);
            let whole = convert::rect(shape.rect);
            let blur = convert::blur_param(std_dev) as f32;
            if remove {
                self.canvas.set_blend_mode(dest_out);
            }
            self.canvas.set_paint(paint);
            match convert::shadow_quarters(shape.rect, radii, std_dev) {
                convert::ShadowParts::Whole(radius) => {
                    self.canvas
                        .fill_blurred_rounded_rect(&whole, radius as f32, blur);
                }
                convert::ShadowParts::Quarters(quarters) => {
                    for (quarter, radius) in quarters {
                        // Clip paths, not layers: destination-out drawing
                        // must reach the enclosing layer.
                        self.canvas.push_clip_path(&quarter.to_path(0.1));
                        self.canvas
                            .fill_blurred_rounded_rect(&whole, radius as f32, blur);
                        self.canvas.pop_clip_path();
                    }
                }
            }
            if remove {
                self.canvas.set_blend_mode(Default::default());
            }
            return;
        }
        // Elliptical corners: the shape through a blur.
        let blur = Filter::from_primitive(FilterPrimitive::GaussianBlur {
            std_deviation: std_dev,
            edge_mode: EdgeMode::None,
        });
        self.canvas
            .push_layer(None, remove.then_some(dest_out), None, Some(blur));
        self.canvas.set_paint(paint);
        self.canvas
            .fill_path(&convert::shape(&Shape::RoundedRect(*shape)));
        self.canvas.pop_layer();
    }

    /// Draw the backdrop at command `at`.
    pub(super) fn backdrop(&mut self, at: usize, backdrop: &Backdrop) {
        let shape = convert::shape(&backdrop.shape);
        let rule = convert::fill_rule(backdrop.shape.rule());
        // Root coordinates of the area the blur reads.
        let reach = SHADOW_REACH * backdrop.std_dev * self.transform.max_scale();
        let region = self
            .transform
            .bounds(backdrop.shape.bounds())
            .inflate(reach + 1.0, reach + 1.0);
        self.canvas.set_fill_rule(rule);
        // The filtered backdrop replaces what is beneath (as CSS draws the
        // filtered backdrop root image): where the replayed content is
        // translucent, the crisp original must not show through.
        let dest_out = convert::dest_out();
        self.canvas.set_blend_mode(dest_out);
        self.canvas.set_paint(PaintType::Solid(BLACK));
        self.canvas.fill_path(&shape);
        self.canvas.set_blend_mode(Default::default());
        self.canvas.push_layer(Some(&shape), None, None, None);
        self.canvas.set_fill_rule(Default::default());
        let blurred = backdrop.std_dev > 0.0;
        if blurred {
            let blur = Filter::from_primitive(FilterPrimitive::GaussianBlur {
                std_deviation: backdrop.std_dev,
                edge_mode: EdgeMode::None,
            });
            self.canvas.push_layer(None, None, None, Some(blur));
        }
        // The blurred area stays within the region the blur reads.
        let outer = self.transform;
        self.transform = Transform::IDENTITY;
        self.set_transform();
        // The backdrop sees the frame only: where the blur reads outside
        // it, it reads the frame mirrored, never what lies beyond.
        if let Some(seen) = crate::backdrop::intersect(region, self.frame) {
            self.canvas
                .push_clip_layer(&convert::rect(seen).to_path(0.1));
            self.replay(at, seen, backdrop.color_matrix);
            self.canvas.pop_layer();
            if blurred {
                self.mirror_edges(at, region, backdrop.color_matrix);
            }
        }
        if blurred {
            self.canvas.pop_layer();
        }
        self.canvas.pop_layer();
        self.transform = outer;
        self.set_transform();
    }

    /// Draw the frame's content mirrored across its edges where `region`
    /// (root coordinates) leaves the frame, so the blur reads the content
    /// reflected there (like `EdgeMode::Mirror` at the window's edges)
    /// instead of the transparent outside, which would darken the edges
    /// of a backdrop that covers them. Each side and corner outside the
    /// frame is the replay of what lies across the edge, reflected, in a
    /// layer clipped to that part; the filter's own edge mode cannot do it,
    /// as its image is the content's bounds and the blur's margin.
    fn mirror_edges(
        &mut self,
        at: usize,
        region: LogicalRect,
        matrix: Option<kivact_types::ColorMatrix>,
    ) {
        let frame = self.frame;
        // The parts of one axis: before the frame (reflected about its
        // start), across it, after it (about its end), each at most the
        // frame long (one reflection), with its reflection.
        let parts = |start: f32, end: f32, low: f32, high: f32| {
            let length = high - low;
            [
                (start.max(low - length), end.min(low), Some(low)),
                (start.max(low), end.min(high), None),
                (start.max(high), end.min(high + length), Some(high)),
            ]
        };
        let xs = parts(region.x, region.right(), frame.x, frame.right());
        let ys = parts(region.y, region.bottom(), frame.y, frame.bottom());
        for &(x0, x1, about_x) in &xs {
            for &(y0, y1, about_y) in &ys {
                if (about_x.is_none() && about_y.is_none()) || x1 <= x0 || y1 <= y0 {
                    continue;
                }
                let part = LogicalRect {
                    x: x0,
                    y: y0,
                    width: x1 - x0,
                    height: y1 - y0,
                };
                // x' = 2 * about - x on each reflected axis.
                let reflect = Transform {
                    a: if about_x.is_some() { -1.0 } else { 1.0 },
                    d: if about_y.is_some() { -1.0 } else { 1.0 },
                    e: about_x.map_or(0.0, |x| 2.0 * x),
                    f: about_y.map_or(0.0, |y| 2.0 * y),
                    ..Transform::IDENTITY
                };
                self.canvas
                    .push_clip_layer(&convert::rect(part).to_path(0.1));
                self.transform = reflect;
                self.set_transform();
                self.replay(at, reflect.bounds(part), matrix);
                self.canvas.pop_layer();
            }
        }
    }

    /// Draw again the commands before `at` that reach `region` (root
    /// coordinates), skipping groups that do not, with colors through
    /// `matrix`. Starts and ends at the root transform.
    fn replay(
        &mut self,
        at: usize,
        region: LogicalRect,
        matrix: Option<kivact_types::ColorMatrix>,
    ) {
        let Some(index) = self.index else {
            return;
        };
        let stack = std::mem::take(&mut self.stack);
        let previous = std::mem::replace(&mut self.matrix, matrix);
        self.replaying = true;
        let scene = self.scene;
        let reaches = |i: usize| index.bounds[i].is_some_and(|b| b.intersects(region));
        let mut i = 0;
        while i < at {
            let command = &scene.commands[i];
            match command {
                DrawCommand::PushClip(_)
                | DrawCommand::PushLayer(_)
                | DrawCommand::PushTransform(_) => {
                    let close = index.close[i];
                    if close < at {
                        if !reaches(i) {
                            i = close + 1;
                            continue;
                        }
                        self.command(i, command);
                    } else if let DrawCommand::PushTransform(_) = command {
                        // Open around the backdrop: its transform applies
                        // to what it holds.
                        self.command(i, command);
                    } else {
                        // Open around the backdrop: already in place.
                        self.stack.push(Group::Ancestor);
                    }
                }
                DrawCommand::PopClip | DrawCommand::PopTransform | DrawCommand::PopLayer => {
                    self.command(i, command);
                }
                _ => {
                    if reaches(i) {
                        self.command(i, command);
                    }
                }
            }
            i += 1;
        }
        // Close what is still open (ancestors' groups): only transforms
        // touched the canvas, and the caller restores the transform.
        self.hidden = 0;
        self.stack = stack;
        self.matrix = previous;
        self.replaying = false;
        self.transform = Transform::IDENTITY;
        self.set_transform();
    }
}

/// A corner radius of an outer shadow grown by `spread` (CSS Backgrounds
/// 3: a radius smaller than the spread grows by less, so sharp corners stay
/// sharp).
fn spread_radius(r: f32, spread: f32) -> f32 {
    if r <= 0.0 {
        return 0.0;
    }
    if spread < 0.0 {
        return (r + spread).max(0.0);
    }
    let ratio = r / spread;
    let factor = if ratio < 1.0 {
        1.0 + (ratio - 1.0).powi(3)
    } else {
        1.0
    };
    r + spread * factor
}
