//! Role: Box shadows of a scene.
//! Plain data usable on any thread.

use crate::color::Rgba;
use crate::geometry::Point;
use crate::scene::RoundedRect;

/// A CSS `box-shadow` of a box with elliptical corners
/// ([`crate::render::RenderCmd::BoxShadow`]).
///
/// The shadow's shape is the box moved by `offset` and grown by `spread`
/// (inset: shrunk), its corner radii grown (inset: shrunk) as CSS says,
/// then blurred by a gaussian of `std_dev` (CSS blur radius / 2). An outer
/// shadow is drawn outside the box only when `knockout` is set (as CSS
/// draws it; unset, it is also drawn beneath the box, which costs less and
/// looks the same under an opaque background); an inset shadow is drawn
/// inside the box only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShadow {
    /// The box casting the shadow (its border box).
    pub shape: RoundedRect,
    /// Offset of the shadow.
    pub offset: Point,
    /// Growth of the shadow's shape (negative shrinks; inset shadows grow
    /// inwards).
    pub spread: f64,
    /// Standard deviation of the blur (0: a hard edge).
    pub std_dev: f64,
    /// Color.
    pub color: Rgba,
    /// Inside the box instead of outside.
    pub inset: bool,
    /// Outer shadows: leave the box itself out.
    pub knockout: bool,
}
