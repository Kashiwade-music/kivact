//! Role: Geometry values: points, sizes, rectangles and corner radii.

use super::length::Length;
use kivact_types::CornerRadius as WireCorner;

pub use kivact_types::{IntrinsicSize, Point, Rect, Size};

/// A corner's radius: circular (`8`, `pct(50)`) or elliptical (`(20, 10)`:
/// horizontal, vertical). Percentages are against the box's width
/// (horizontal) and height (vertical), so `pct(50)` rounds a box into an
/// ellipse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CornerRadius {
    pub x: Length,
    pub y: Length,
}

impl From<Length> for CornerRadius {
    fn from(r: Length) -> Self {
        CornerRadius { x: r, y: r }
    }
}

macro_rules! radius_from {
    ($($t:ty),*) => {$(
        impl From<$t> for CornerRadius {
            fn from(r: $t) -> Self {
                CornerRadius::from(Length::from(r))
            }
        }
    )*};
}

radius_from!(i32, u32, f64, f64);

impl<X: Into<Length>, Y: Into<Length>> From<(X, Y)> for CornerRadius {
    fn from((x, y): (X, Y)) -> Self {
        CornerRadius {
            x: x.into(),
            y: y.into(),
        }
    }
}

pub(super) fn corner(r: Option<CornerRadius>) -> WireCorner {
    r.map_or_else(WireCorner::default, |r| WireCorner {
        x: r.x.to_len(),
        y: r.y.to_len(),
    })
}
