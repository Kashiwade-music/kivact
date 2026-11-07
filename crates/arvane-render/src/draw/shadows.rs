//! Role: Box shadows drawn with what both rasterizers have.
//!
//! Box shadows: drawn with what both rasterizers have (vello_hybrid has no
//! destructive blending except through destination-out drawing inside a
//! layer, which both use here, and nothing reads pixels back).
//!
//! - Inset shadows: inside a layer clipped to the box, the shadow color
//!   over the whole box minus the blurred hole (destination-out).
//! - Outer shadows: the blurred rounded rect of `fill_blurred_rounded_rect`
//!   (in quarters for different radii) when the corners are circular, else
//!   the shape through a blur filter layer; a knockout removes the box
//!   from an isolated layer holding the shadow.
//!
//! An outer shadow's box is moved by its offset and grown by its spread,
//! its corners grown as CSS grows them (`spread_radius`: a radius smaller
//! than the spread grows by less, a sharp corner stays sharp). A knockout
//! costs an isolated layer and destination-out drawing, so a host whose
//! box is opaque leaves it unset. An inset shadow's hole is the box shrunk
//! by the spread (its radii too), blurred; a standard deviation of 0 is a
//! hard edge.
//!
//! Limits:
//! - Destination-out drawing reaches only the innermost layer's pixels.
//! - A shadow with different corner radii is drawn in quarters, each with
//!   its corner's radius (`fill_blurred_rounded_rect` takes one radius):
//!   where the blur is larger than the box, the quarters' seams may not
//!   join smoothly.

use super::canvas::Canvas;
use super::painter::Painter;
use crate::convert;
use kivact_types::render::BoxShadow;
use kivact_types::scene::RoundedRect as Rounded;
use kivact_types::scene::{Radius, Shape};
use kivact_types::{Rect as LogicalRect, Rgba};
use vello_common::filter_effects::{EdgeMode, Filter, FilterPrimitive};
use vello_cpu::PaintType;
use vello_cpu::color::palette::css::BLACK;
use vello_cpu::kurbo::Shape as _;

impl<C: Canvas> Painter<'_, C> {
    /// Draw a CSS box shadow, outer or inset.
    pub(super) fn box_shadow(&mut self, shadow: &BoxShadow) {
        let color = shadow.color;
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
    fn blurred(&mut self, shape: &Rounded, std_dev: f64, color: Rgba, remove: bool) {
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
            let blur = convert::blur_param(std_dev);
            if remove {
                self.canvas.set_blend_mode(dest_out);
            }
            self.canvas.set_paint(paint);
            match convert::shadow_quarters(shape.rect, radii, std_dev) {
                convert::ShadowParts::Whole(radius) => {
                    self.canvas.fill_blurred_rounded_rect(&whole, radius, blur);
                }
                convert::ShadowParts::Quarters(quarters) => {
                    for (quarter, radius) in quarters {
                        // Clip paths, not layers: destination-out drawing
                        // must reach the enclosing layer.
                        self.canvas.push_clip_path(&quarter.to_path(0.1));
                        self.canvas.fill_blurred_rounded_rect(&whole, radius, blur);
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
            std_deviation: std_dev as f32,
            edge_mode: EdgeMode::None,
        });
        self.canvas
            .push_layer(None, remove.then_some(dest_out), None, Some(blur));
        self.canvas.set_paint(paint);
        self.canvas
            .fill_path(&convert::shape(&Shape::RoundedRect(*shape)));
        self.canvas.pop_layer();
    }
}

/// A corner radius of an outer shadow grown by `spread` (CSS Backgrounds
/// 3: a radius smaller than the spread grows by less, so sharp corners stay
/// sharp).
fn spread_radius(r: f64, spread: f64) -> f64 {
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
