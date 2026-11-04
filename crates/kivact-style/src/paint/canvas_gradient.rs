//! Role: Gradients placed by canvas coordinates: linear, two-circle radial and conic.
//!
//! Gradients placed by coordinates (HTML Canvas `createLinearGradient`,
//! `createRadialGradient` and `createConicGradient`).
//!
//! `linear(p0, p1)`, `radial(c0, r0, c1, r1)` and `conic(start_deg,
//! center)` take local points, `stop(offset, color)` and `extend`, and
//! convert to a `Paint`. `resolve` lowers `PaintImage::Canvas` to
//! `FillImage::Coordinate` (linear, two-circle radial or conic, the
//! offset-RGBA list and a `GradientExtend`) with the tokens resolved; the
//! engine adds the scene origin to the points (the border box's top left for
//! a background or a mask) and uses no tile size, position or repeat.
//!
//! Invariants:
//! - Stops are sorted by offset, equal offsets in the order added (a hard
//!   edge). Stops that fail `sanitize::fill` (at least one, finite, in
//!   `0..=1`, ascending), points or angles that are not finite and a
//!   negative radius draw nothing.
//! - A conic gradient starts at 0° to the right and ends at start + 360,
//!   repeating. A two-circle radial is the two-point radial of the CPU and
//!   GPU renderers: outside the cone of non-nested circles it is
//!   transparent.
use crate::geometry::Point;
use crate::{Color, ThemeColor};

/// What a [`CanvasGradient`] paints beyond its offsets 0 and 1.
#[doc(inline)]
pub use kivact_types::Extend as GradientExtend;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CanvasShape {
    Linear {
        from: Point,
        to: Point,
    },
    Radial {
        c0: Point,
        r0: f64,
        c1: Point,
        r1: f64,
    },
    Conic {
        start_deg: f64,
        center: Point,
    },
}

/// A gradient placed by coordinates, as `createLinearGradient`,
/// `createRadialGradient` and `createConicGradient` of HTML Canvas make
/// one, then [`stop`](CanvasGradient::stop) for `addColorStop`. It is a
/// [`Paint`](super::Paint) (`draw.fill_path(&path, gradient)`).
///
/// The points are in the local coordinates of what it paints: a `Draw`'s
/// (the canvas origin, with the transforms of the layers the shape is drawn
/// in applying, as in Canvas) or a box's (its top left corner). The gradient
/// has no transform of its own and does not follow the shape it fills: two
/// shapes filled with the same gradient show the same colors where they
/// overlap. A [`Paint`](super::Paint)'s tile size, position and repeat do not apply to it.
///
/// Offsets are in `0..=1`, and stops are kept in order of offset (a stop
/// at the offset of an earlier one comes after it, so two make a hard
/// edge). With no stop, or an offset outside `0..=1` or not finite,
/// nothing is painted. Colors may be theme tokens.
///
/// ```
/// use kivact::draw::{Path};
/// use kivact::geometry::{Point, Rect};
/// use kivact::prelude::*;
/// use kivact::draw::Draw;
/// use kivact::draw::{CanvasGradient, GradientExtend};
/// fn knob(d: &mut Draw) {
///     // createRadialGradient(34, 34, 0, 40, 40, 30) with two color stops.
///     let ball = CanvasGradient::radial(Point::new(34.0, 34.0), 0.0, Point::new(40.0, 40.0), 30.0)
///         .stop(0.0, rgb(0xffffff))
///         .stop(1.0, ColorToken::Accent);
///     d.fill_path(&Path::new().arc(Point::new(40.0, 40.0), 30, 0, 360, false), ball);
///     // Stripes along x: createLinearGradient(0, 0, 8, 0), repeating.
///     let stripes = CanvasGradient::linear(Point::new(0.0, 0.0), Point::new(8.0, 0.0))
///         .stop(0.0, rgb(0x000000))
///         .stop(0.5, rgb(0x000000))
///         .stop(0.5, rgb(0xffffff))
///         .stop(1.0, rgb(0xffffff))
///         .extend(GradientExtend::Repeat);
///     d.fill_rect(Rect::new(80.0, 0.0, 64.0, 16.0), stripes);
/// }
/// # let _ = knob;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasGradient {
    pub(crate) shape: CanvasShape,
    pub(crate) stops: Vec<(f64, Color)>,
    pub(crate) extend: GradientExtend,
}

impl CanvasGradient {
    fn of(shape: CanvasShape) -> Self {
        CanvasGradient {
            shape,
            stops: Vec::new(),
            extend: GradientExtend::Pad,
        }
    }

    /// Offset 0 at `p0` and offset 1 at `p1`, constant across the line
    /// between them (`createLinearGradient(x0, y0, x1, y1)`).
    pub fn linear(p0: Point, p1: Point) -> Self {
        Self::of(CanvasShape::Linear { from: p0, to: p1 })
    }

    /// Offset 0 on the circle of radius `r0` around `c0`, offset 1 on the
    /// circle of radius `r1` around `c1`, and the circles between for the
    /// offsets between (`createRadialGradient(x0, y0, r0, x1, y1, r1)`).
    /// Radii are not negative.
    pub fn radial(c0: Point, r0: f64, c1: Point, r1: f64) -> Self {
        Self::of(CanvasShape::Radial { c0, r0, c1, r1 })
    }

    /// Offsets by angle around `center`, one turn clockwise on the screen
    /// from `start_deg` (0 is to the right, as in `Path::arc`;
    /// `createConicGradient(startAngle, x, y)` with the angle in degrees).
    /// Beyond one turn it wraps, so [`extend`](CanvasGradient::extend) does
    /// not change it.
    pub fn conic(start_deg: f64, center: Point) -> Self {
        Self::of(CanvasShape::Conic { start_deg, center })
    }

    /// A color at `offset` (`addColorStop`). A theme token is resolved with
    /// the theme in effect where the gradient is drawn.
    pub fn stop(mut self, offset: f64, color: impl ThemeColor) -> Self {
        let at = self.stops.partition_point(|(o, _)| *o <= offset);
        self.stops.insert(at, (offset, Color::of(color)));
        self
    }

    /// What is painted beyond offsets 0 and 1 (default
    /// [`GradientExtend::Pad`]).
    pub fn extend(mut self, extend: GradientExtend) -> Self {
        self.extend = extend;
        self
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for canvas gradients.
    use super::*;

    #[test]
    fn stops_are_kept_in_offset_order_and_equal_offsets_in_the_order_added() {
        let g = CanvasGradient::linear(Point::new(0.0, 0.0), Point::new(1.0, 0.0))
            .stop(1.0, Color::WHITE)
            .stop(0.5, Color::BLACK)
            .stop(0.5, Color::TRANSPARENT)
            .stop(0.0, Color::WHITE);
        let offsets: Vec<f64> = g.stops.iter().map(|s| s.0).collect();
        assert_eq!(offsets, [0.0, 0.5, 0.5, 1.0]);
        assert_eq!(g.stops[1].1, Color::BLACK);
        assert_eq!(g.stops[2].1, Color::TRANSPARENT);
    }
}
