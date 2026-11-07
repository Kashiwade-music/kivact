//! Role: What scene fills and strokes are painted with.
//! Plain data usable on any thread.

use crate::color::Rgba;
use crate::geometry::{Point, Rect};
use crate::paint::Extend;
use crate::pixel::Pixels;

/// A paint for fills and strokes.
///
/// Coordinates are in the drawing command's coordinates (so they follow
/// [`crate::render::RenderCmd::PushTransform`] like the geometry).
///
/// ```
/// use kivact_types::{Extend, Point, Rgba};
/// use kivact_types::render::{Gradient, GradientShape, Paint};
/// let black = Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
/// let clear = Rgba { a: 0.0, ..black };
/// // CSS `repeating-linear-gradient(90deg, black 0 1px, transparent 1px 24px)`
/// // from x = 0: a 1 px line every 24 px, two stops at 1/24 making a hard edge.
/// let lines = Paint::Gradient(Gradient {
///     shape: GradientShape::Linear {
///         start: Point { x: 0.0, y: 0.0 },
///         end: Point { x: 24.0, y: 0.0 },
///     },
///     stops: vec![(0.0, black), (1.0 / 24.0, black), (1.0 / 24.0, clear), (1.0, clear)],
///     extend: Extend::Repeat,
/// });
/// # let _ = lines;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    /// One color.
    Solid(Rgba),
    /// A linear, radial or conic gradient with its own geometry.
    Gradient(Gradient),
    /// A tiled image or gradient tile.
    Pattern(Pattern),
}

/// Where a gradient's offsets 0 and 1 lie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientShape {
    /// Offset 0 at `start`, 1 at `end`, constant across the line between
    /// them.
    Linear {
        /// Where offset 0 is.
        start: Point,
        /// Where offset 1 is.
        end: Point,
    },
    /// Offset 0 at `center`, 1 on the ellipse of radii `radius_x` ×
    /// `radius_y` around it (a circle when they are equal). A zero radius
    /// paints the last stop's color.
    Radial {
        /// Center.
        center: Point,
        /// Horizontal radius.
        radius_x: f64,
        /// Vertical radius.
        radius_y: f64,
    },
    /// The gradient of two circles, as canvas `createRadialGradient`:
    /// offset 0 on the circle of radius `start_radius` around `start_center`,
    /// offset 1 on the circle of `end_radius` around `end_center`, and the
    /// circles between them (interpolated centers and radii) for the offsets
    /// between. A point on no circle of the offsets the gradient covers
    /// (outside the cone, when the circles are not nested) is transparent.
    /// Radii are not negative; the same circle twice paints nothing sensible
    /// (the first stop's color).
    RadialTwoCircle {
        /// Center of the offset 0 circle.
        start_center: Point,
        /// Radius of the offset 0 circle.
        start_radius: f64,
        /// Center of the offset 1 circle.
        end_center: Point,
        /// Radius of the offset 1 circle.
        end_radius: f64,
    },
    /// Offsets by angle around `center`: 0 at `start_deg`, 1 at `end_deg`
    /// (degrees clockwise from the positive x axis; CSS `conic-gradient(from
    /// θ)` is `start_deg = θ - 90`, `end_deg = start_deg + 360`; a
    /// repeating conic gradient ends sooner and repeats).
    Conic {
        /// Center.
        center: Point,
        /// Angle of offset 0.
        start_deg: f64,
        /// Angle of offset 1 (more than `start_deg`).
        end_deg: f64,
    },
}

impl GradientShape {
    /// The linear geometry of a CSS angle (degrees clockwise from left to
    /// right) over `rect`: the gradient line passes through the bounds'
    /// center and is as long as the bounds are across in its direction.
    pub fn linear_angle(rect: Rect, angle_deg: f64) -> GradientShape {
        let angle = angle_deg.to_radians();
        let (dy, dx) = angle.sin_cos();
        let span = rect.width * dx.abs() + rect.height * dy.abs();
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        GradientShape::Linear {
            start: Point {
                x: cx - dx * span / 2.0,
                y: cy - dy * span / 2.0,
            },
            end: Point {
                x: cx + dx * span / 2.0,
                y: cy + dy * span / 2.0,
            },
        }
    }
}

/// A gradient: geometry, color stops and how it extends.
///
/// Two stops at the same offset make a hard edge. The stops are sRGB: the
/// engine expands the authored gradient into them before it reaches here.
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    /// Geometry.
    pub shape: GradientShape,
    /// Stops: offsets in `0.0..=1.0`, not decreasing, at least one.
    pub stops: Vec<(f64, Rgba)>,
    /// Beyond the stops.
    pub extend: Extend,
}

/// An image or a gradient tile repeated over what it fills (a CSS
/// background layer with `background-size`, `background-position` and
/// `background-repeat`), in a constant number of commands whatever the
/// area.
///
/// One copy covers `tile`; copies repeat from there along each axis whose
/// `repeat_*` is set, and the paint is transparent beyond the copies
/// otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct Pattern {
    /// What one tile shows.
    pub source: PatternSource,
    /// Where one copy is drawn (the source is stretched to it).
    pub tile: Rect,
    /// Repeat along x.
    pub repeat_x: bool,
    /// Repeat along y.
    pub repeat_y: bool,
}

/// What a [`Pattern`]'s tile shows.
#[derive(Clone, Debug, PartialEq)]
pub enum PatternSource {
    /// An image.
    Image {
        /// Pixels.
        image: Pixels,
    },
    /// A gradient in the tile's own coordinates (`(0, 0)` is the tile's
    /// top-left corner), for 2D tiles a gradient's own period cannot make
    /// (CSS `radial-gradient(..) 0 0 / 24px 24px` dots). The renderer
    /// rasterizes it once at the tile's device size and repeats the
    /// pixels; a 1D period is better drawn as an [`Extend::Repeat`]
    /// gradient.
    Gradient(Gradient),
}
