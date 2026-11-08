//! Role: `Shadow`: a box shadow with offset, blur, spread, color and inset.

use crate::color::Color;
use crate::geometry::Point;
use crate::theme::value::ThemeColor;

/// A box shadow (CSS `box-shadow`); several stack, the first on top.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{rgba};
/// use kivact::draw::Shadow;
/// // CSS `box-shadow: 0 4px 14px -4px rgba(0,0,0,.5), inset 0 1px 0 rgba(255,255,255,.35)`
/// let s = style()
///     .shadow(Shadow::new(0, 4, 14, rgba(0x000000, 0.5)).spread(-4))
///     .shadow(Shadow::new(0, 1, 0, rgba(0xffffff, 0.35)).inset());
/// # let _ = s;
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub offset: Point,
    /// Blur radius (CSS: a gaussian of standard deviation `blur / 2`).
    pub blur: f64,
    /// Growth of the shadow's shape (negative shrinks it).
    pub spread: f64,
    pub color: Color,
    /// Inside the box (CSS `inset`).
    pub inset: bool,
}

impl Shadow {
    /// A shadow offset by `(x, y)` with blur radius `blur`.
    pub fn new(
        x: impl Into<f64>,
        y: impl Into<f64>,
        blur: impl Into<f64>,
        color: impl ThemeColor,
    ) -> Shadow {
        Shadow {
            offset: Point::new(x.into(), y.into()),
            blur: blur.into(),
            spread: 0.0,
            color: Color::of(color),
            inset: false,
        }
    }

    /// Grow (or, negative, shrink) the shadow's shape (CSS spread radius).
    pub fn spread(mut self, spread: impl Into<f64>) -> Shadow {
        self.spread = spread.into();
        self
    }

    /// Draw it inside the box (CSS `inset`).
    pub fn inset(mut self) -> Shadow {
        self.inset = true;
        self
    }
}
