//! Role: Lengths: logical px, percentages, calc sums and content sizes.
//!
//! As authored and as laid out.
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
/// `em(1.5)` is 1.5 times the font size this style sets, or the theme's
/// base size (`font::SIZE`) when it sets none. A sum with a theme variable
/// is resolved again when the theme changes. A sum the host cannot take
/// (`auto` or a content size in `+` or `-`, or two different variables) is
/// `Invalid` and reported as `Unsupported` where it is used; it is never
/// silently `Auto`.
///
/// ```
/// use kivact_types::{Length, pct};
/// assert_eq!(pct(100) - 20, Length::Calc { percent: 100.0, px: -20.0, em: 0.0, var: None });
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
        em: f64,
        var: Option<(&'static crate::ThemeVar<f64>, f64)>,
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
    /// `width(Length::MaxContent).min_width(pct(100))`, with `.flex_shrink(0)` in
    /// a row).
    MaxContent,
    /// The content's width, at most the available width (CSS
    /// `fit-content`). For `width`, `height` and `basis`, and grid tracks;
    /// elsewhere it is `auto` and reported as unsupported. In a column,
    /// `align_self(Align::Start)` sizes a child the same way (and measures a
    /// wrapping child's height).
    FitContent,
    /// A multiple of the font size (see [`em`]).
    Em(f64),
    /// A theme variable (`space::MD`), resolved where the style is compiled.
    Var(&'static crate::ThemeVar<f64>),
    /// A sum the host cannot take; reported as `Unsupported` where used.
    Invalid,
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

/// One term list of a sum: px, percent, em and one variable with its factor.
type Terms = (f64, f64, f64, Option<(&'static crate::ThemeVar<f64>, f64)>);

impl Length {
    /// The theme variable `v` as a length.
    pub const fn var(v: &'static crate::ThemeVar<f64>) -> Length {
        Length::Var(v)
    }

    /// A content size (`min_content()`, `max_content()`, `fit_content()`).
    pub fn is_content(self) -> bool {
        matches!(
            self,
            Length::MinContent | Length::MaxContent | Length::FitContent
        )
    }

    /// Whether the length needs a theme (a variable) or the font size (an
    /// `em`, also in a sum) to resolve, or is an invalid sum.
    pub fn needs_theme(self) -> bool {
        match self {
            Length::Var(_) | Length::Em(_) | Length::Invalid => true,
            Length::Calc { em, var, .. } => em != 0.0 || var.is_some(),
            _ => false,
        }
    }

    /// The terms of a sum, or `None` for `Auto`, the content sizes and `Invalid`.
    fn terms(self) -> Option<Terms> {
        match self {
            Length::Px(v) => Some((v, 0.0, 0.0, None)),
            Length::Percent(p) => Some((0.0, p, 0.0, None)),
            Length::Em(e) => Some((0.0, 0.0, e, None)),
            Length::Var(v) => Some((0.0, 0.0, 0.0, Some((v, 1.0)))),
            Length::Calc {
                percent,
                px,
                em,
                var,
            } => Some((px, percent, em, var)),
            Length::Auto
            | Length::MinContent
            | Length::MaxContent
            | Length::FitContent
            | Length::Invalid => None,
        }
    }

    fn from_terms((px, percent, em, var): Terms) -> Length {
        match (percent != 0.0, em != 0.0, var) {
            (false, false, None) => Length::Px(px),
            (true, false, None) if px == 0.0 => Length::Percent(percent),
            (false, true, None) if px == 0.0 => Length::Em(em),
            (false, false, Some((v, k))) if px == 0.0 && k == 1.0 => Length::Var(v),
            _ => Length::Calc {
                percent,
                px,
                em,
                var,
            },
        }
    }

    /// `self + sign * rhs`, or `Invalid` (see the type's doc).
    fn combine(self, rhs: Length, sign: f64) -> Length {
        let (Some((px1, p1, e1, v1)), Some((px2, p2, e2, v2))) = (self.terms(), rhs.terms()) else {
            return Length::Invalid;
        };
        let var = match (v1, v2) {
            (None, None) => None,
            (Some(a), None) => Some(a),
            (None, Some((v, k))) => Some((v, sign * k)),
            (Some((a, k1)), Some((b, k2))) if std::ptr::eq(a, b) => Some((a, k1 + sign * k2)),
            (Some(_), Some(_)) => return Length::Invalid,
        };
        Length::from_terms((px1 + sign * px2, p1 + sign * p2, e1 + sign * e2, var))
    }
}

impl<T: Into<Length>> Add<T> for Length {
    type Output = Length;

    /// The sum (CSS `calc(a + b)`); see the type's doc for what it keeps.
    fn add(self, rhs: T) -> Length {
        self.combine(rhs.into(), 1.0)
    }
}

impl<T: Into<Length>> Sub<T> for Length {
    type Output = Length;

    /// The difference (CSS `calc(a - b)`).
    fn sub(self, rhs: T) -> Length {
        self.combine(rhs.into(), -1.0)
    }
}

/// `v` times the font size (CSS `em`): the font size the style sets, or the
/// theme's base size when it sets none (resolved where the style is compiled).
pub fn em(v: impl Into<f64>) -> Length {
    Length::Em(v.into())
}

#[cfg(test)]
mod tests {
    //! Role: Tests of length sums.
    use super::*;

    static V: crate::ThemeVar<f64> = crate::ThemeVar::new("t.v", 8.0);
    static W: crate::ThemeVar<f64> = crate::ThemeVar::new("t.w", 4.0);

    #[test]
    fn a_sum_keeps_its_variable() {
        assert_eq!(
            Length::var(&V) + 2,
            Length::Calc {
                percent: 0.0,
                px: 2.0,
                em: 0.0,
                var: Some((&V, 1.0))
            }
        );
    }

    #[test]
    fn sums_of_like_terms_collapse() {
        assert_eq!(pct(50) - pct(50) + 3, Length::Px(3.0));
        assert_eq!(em(1) + em(0.5), Length::Em(1.5));
    }

    #[test]
    fn unsupported_sums_are_invalid() {
        assert_eq!(Length::Auto + 2, Length::Invalid);
        assert_eq!(Length::var(&V) - Length::var(&W), Length::Invalid);
        assert_eq!(Length::MaxContent - 1, Length::Invalid);
    }
}
