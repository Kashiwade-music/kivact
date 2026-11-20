//! Role: Lengths: logical px, percentages, calc sums and content sizes, as authored and as laid out.
//!
//! Lengths: numbers are logical px; `pct(50)` and `auto()`; `pct(100) - 20`
//! is a calc length.

use std::ops::{Add, Sub};

/// A length: logical px, a percentage of a reference length, or both.
///
/// `pct(100) - 20` is CSS `calc(100% - 20px)`, accepted wherever a length
/// is: in layout (sizes, minimums and maximums, padding, margins, insets,
/// gaps, the flex basis, grid tracks; the percentage is of what a
/// percentage there is of, and with an indefinite reference the length is
/// `auto` like a percentage's) and in paint positions (background sizes
/// and positions, gradient centers and stops, corner radii, clip path
/// points, masks).
///
/// ```
/// use kivact_types::{Length, pct};
/// assert_eq!(pct(100) - 20, Length::Calc { percent: 100.0, px: -20.0 });
/// assert_eq!(Length::from(4) + 6, Length::Px(10.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Length {
    #[default]
    Auto,
    Px(f64),
    Percent(f64),
    /// `percent`% of the reference plus `px` (CSS `calc(p% + npx)`).
    Calc {
        percent: f64,
        px: f64,
    },
    /// As wide (or high) as the content laid out at its narrowest: every
    /// line broken (CSS `min-content`). For `width`, `height` and `basis`,
    /// and grid tracks; elsewhere it is `auto` and reported as unsupported.
    /// As the width of a column's child, the child's height is still
    /// measured from its unwrapped content (taffy): size such boxes in a row
    /// or a grid.
    MinContent,
    /// As wide (or high) as the content laid out without breaking lines
    /// (CSS `max-content`): `width(Length::MaxContent)`. For `width`,
    /// `height` and `basis`, and grid tracks; elsewhere it is `auto` and
    /// reported as unsupported (CSS `min-width: max-content` on a box that
    /// otherwise fills its parent is
    /// `width(Length::MaxContent).min_width(pct(100))`, with `.shrink(0)` in
    /// a row).
    MaxContent,
    /// The content's width, at most the available width (CSS
    /// `fit-content`). For `width`, `height` and `basis`, and grid tracks;
    /// elsewhere it is `auto` and reported as unsupported. In a column,
    /// `align_self(Align::Start)` sizes a child the same way (and measures a
    /// wrapping child's height).
    FitContent,
}

/// Percentage of the containing box.
pub fn pct(v: impl Into<f64>) -> Length {
    Length::Percent(v.into())
}

pub fn auto() -> Length {
    Length::Auto
}

impl From<i32> for Length {
    fn from(v: i32) -> Self {
        Length::Px(v as f64)
    }
}

impl From<u32> for Length {
    fn from(v: u32) -> Self {
        Length::Px(v as f64)
    }
}

impl From<f32> for Length {
    fn from(value: f32) -> Self {
        Self::Px(f64::from(value))
    }
}

impl From<f64> for Length {
    fn from(v: f64) -> Self {
        Length::Px(v)
    }
}

impl Length {
    /// `(percent, px)`, or `None` for `Auto` and the content sizes.
    fn parts(self) -> Option<(f64, f64)> {
        match self {
            Length::Auto | Length::MinContent | Length::MaxContent | Length::FitContent => None,
            Length::Px(v) => Some((0.0, v)),
            Length::Percent(p) => Some((p, 0.0)),
            Length::Calc { percent, px } => Some((percent, px)),
        }
    }

    fn from_parts(percent: f64, px: f64) -> Length {
        if percent == 0.0 {
            Length::Px(px)
        } else if px == 0.0 {
            Length::Percent(percent)
        } else {
            Length::Calc { percent, px }
        }
    }

    /// A content size (`min_content()`, `max_content()`, `fit_content()`).
    pub fn is_content(self) -> bool {
        matches!(
            self,
            Length::MinContent | Length::MaxContent | Length::FitContent
        )
    }

    /// The paint form (`Auto` and the content sizes are zero).
    pub fn to_len(self) -> crate::paint::Len {
        let (pct, px) = self.parts().unwrap_or((0.0, 0.0));
        crate::paint::Len { px, pct }
    }
}

impl<T: Into<Length>> Add<T> for Length {
    type Output = Length;

    /// The sum; `Auto` (or a content size) gives `Auto`.
    fn add(self, rhs: T) -> Length {
        match (self.parts(), rhs.into().parts()) {
            (Some((p1, x1)), Some((p2, x2))) => Length::from_parts(p1 + p2, x1 + x2),
            _ => Length::Auto,
        }
    }
}

impl<T: Into<Length>> Sub<T> for Length {
    type Output = Length;

    /// The difference; `Auto` (or a content size) gives `Auto`.
    fn sub(self, rhs: T) -> Length {
        match (self.parts(), rhs.into().parts()) {
            (Some((p1, x1)), Some((p2, x2))) => Length::from_parts(p1 - p2, x1 - x2),
            _ => Length::Auto,
        }
    }
}
