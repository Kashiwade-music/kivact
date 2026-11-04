//! Role: Layout units: pixels, percentages, fractions.
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
/// use kivact::prelude::*;
/// use kivact::style::{pct};
/// use kivact::style::Length;
/// assert_eq!(pct(100) - 20, Length::Calc { percent: 100.0, px: -20.0 });
/// assert_eq!(Length::from(4) + 6, Length::Px(10.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Length {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
    /// `percent`% of the reference plus `px` (CSS `calc(p% + npx)`).
    Calc {
        percent: f32,
        px: f32,
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
    Length::Percent(v.into() as f32)
}

pub fn auto() -> Length {
    Length::Auto
}

impl From<i32> for Length {
    fn from(v: i32) -> Self {
        Length::Px(v as f32)
    }
}

impl From<u32> for Length {
    fn from(v: u32) -> Self {
        Length::Px(v as f32)
    }
}

impl From<f32> for Length {
    fn from(v: f32) -> Self {
        Length::Px(v)
    }
}

impl From<f64> for Length {
    fn from(v: f64) -> Self {
        Length::Px(v as f32)
    }
}

impl Length {
    /// `(percent, px)`, or `None` for `Auto` and the content sizes.
    fn parts(self) -> Option<(f32, f32)> {
        match self {
            Length::Auto | Length::MinContent | Length::MaxContent | Length::FitContent => None,
            Length::Px(v) => Some((0.0, v)),
            Length::Percent(p) => Some((p, 0.0)),
            Length::Calc { percent, px } => Some((percent, px)),
        }
    }

    fn from_parts(percent: f32, px: f32) -> Length {
        if percent == 0.0 {
            Length::Px(px)
        } else if px == 0.0 {
            Length::Percent(percent)
        } else {
            Length::Calc { percent, px }
        }
    }

    /// The layout form.
    pub(crate) fn to_wire(self) -> kivact_types::Dimension {
        use kivact_types::Dimension as D;
        match self {
            Length::Auto => D::Auto,
            Length::Px(v) => D::Px(v),
            Length::Percent(v) => D::Percent(v),
            Length::Calc { percent, px } => D::Calc { percent, px },
            Length::MinContent => D::MinContent,
            Length::MaxContent => D::MaxContent,
            Length::FitContent => D::FitContent,
        }
    }

    /// A content size (`min_content()`, `max_content()`, `fit_content()`).
    pub(crate) fn is_content(self) -> bool {
        matches!(
            self,
            Length::MinContent | Length::MaxContent | Length::FitContent
        )
    }

    /// The paint form (`Auto` and the content sizes are zero).
    pub(crate) fn to_len(self) -> kivact_types::Len {
        let (pct, px) = self.parts().unwrap_or((0.0, 0.0));
        kivact_types::Len { px, pct }
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

kivact_reactive::constant_value!(Length);

// A number is a constant length in logical px wherever a reactive length
// is taken (`dialog(..).width(640)`), as it is in `style()`.
impl kivact_reactive::IntoValue<Length> for i32 {
    fn into_prop(self) -> kivact_reactive::Prop<Length> {
        kivact_reactive::Prop::constant(Length::from(self))
    }
}

impl kivact_reactive::IntoValue<Length> for u32 {
    fn into_prop(self) -> kivact_reactive::Prop<Length> {
        kivact_reactive::Prop::constant(Length::from(self))
    }
}

impl kivact_reactive::IntoValue<Length> for f32 {
    fn into_prop(self) -> kivact_reactive::Prop<Length> {
        kivact_reactive::Prop::constant(Length::from(self))
    }
}

impl kivact_reactive::IntoValue<Length> for f64 {
    fn into_prop(self) -> kivact_reactive::Prop<Length> {
        kivact_reactive::Prop::constant(Length::from(self))
    }
}
