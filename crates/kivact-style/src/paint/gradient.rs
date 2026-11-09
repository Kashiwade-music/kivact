//! Role: Gradients placed by the filled box: linear, radial and conic.

use crate::color::Color;
use crate::length::Length;
use crate::theme::value::ThemeColor;
use kivact_types::{ColorInterpolation, ColorSpace, HueInterpolation};

/// The shape of a radial gradient's end, when no radius is given (CSS
/// `closest-side` .. `farthest-corner`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadialExtent {
    ClosestSide,
    ClosestCorner,
    FarthestSide,
    /// The default.
    FarthestCorner,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GradientKind {
    Linear {
        angle: f64,
    },
    Radial {
        circle: bool,
        size: RadialSize,
        at: (Length, Length),
    },
    Conic {
        from: f64,
        at: (Length, Length),
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RadialSize {
    Extent(RadialExtent),
    Radii(Length, Length),
}

/// A CSS gradient, made by [`linear_gradient`], [`radial_gradient`] or
/// [`conic_gradient`]. Colors may be theme tokens (resolved again when the
/// theme changes). Use it as a [`Paint`] (`style().background(gradient)`)
/// or give it a size and position first (`.size(..)`, `.position(..)`).
///
/// ```
/// use kivact::prelude::*;
/// use kivact::draw::linear_gradient;
/// // CSS `repeating-linear-gradient(90deg, #2a2a2a 0 3px, #3e3e3e 3px 4px)`
/// let ruler = linear_gradient(90)
///     .repeating()
///     .stop(rgb(0x2a2a2a), 0)
///     .stop(rgb(0x2a2a2a), 3)
///     .stop(rgb(0x3e3e3e), 3)
///     .stop(rgb(0x3e3e3e), 4);
/// let s = style().background(ruler);
/// # let _ = s;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub(crate) kind: GradientKind,
    pub(crate) stops: Vec<(Color, Option<Length>)>,
    /// The hint after each stop (the last one is unused), as `hint` sets it.
    pub(crate) hints: Vec<Option<Length>>,
    pub(crate) repeating: bool,
    pub(crate) interpolation: ColorInterpolation,
}

/// A linear gradient along `angle` degrees as CSS measures it: 0 to top,
/// 90 to right, 180 to bottom (CSS `linear-gradient(<angle>deg, ..)`;
/// `to bottom` is 180).
pub fn linear_gradient(angle: impl Into<f64>) -> Gradient {
    Gradient::new(GradientKind::Linear {
        angle: angle.into(),
    })
}

/// An elliptical gradient from the center of the area to its farthest
/// corner (CSS `radial-gradient(..)`); `.circle()`, `.radius(r)`,
/// `.radii(rx, ry)`, `.extent(..)` and `.at(x, y)` change its shape and
/// center.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{Color, rgba};
/// use kivact::draw::radial_gradient;
/// // CSS `radial-gradient(circle at 1px 1px, #ffffff10 1px, transparent 0) 0 0 / 18px 18px`
/// let dots = radial_gradient()
///     .circle()
///     .at(1, 1)
///     .stop(rgba(0xffffff, 0.06), 1)
///     .stop(Color::TRANSPARENT, 0)
///     .size(18, 18);
/// # let _ = dots;
/// ```
pub fn radial_gradient() -> Gradient {
    Gradient::new(GradientKind::Radial {
        circle: false,
        size: RadialSize::Extent(RadialExtent::FarthestCorner),
        at: (Length::Percent(50.0), Length::Percent(50.0)),
    })
}

/// A gradient around the center of the area starting at `from` degrees
/// (0 is up, clockwise; CSS `conic-gradient(from <deg>, ..)`). Its stop
/// positions are degrees (`.stop(c, 90)`) or percentages of the turn.
pub fn conic_gradient(from: impl Into<f64>) -> Gradient {
    Gradient::new(GradientKind::Conic {
        from: from.into(),
        at: (Length::Percent(50.0), Length::Percent(50.0)),
    })
}

impl Gradient {
    fn new(kind: GradientKind) -> Gradient {
        Gradient {
            kind,
            stops: Vec::new(),
            hints: Vec::new(),
            repeating: false,
            interpolation: ColorInterpolation::DEFAULT,
        }
    }

    /// A color stop at `at` along the gradient (px or a percentage of the
    /// gradient line; a conic gradient's are degrees). Two stops at the
    /// same position make a hard edge; a stop before the previous one is
    /// moved up to it (CSS), so `.stop(c, 0)` after `.stop(d, 10)` is at
    /// 10.
    pub fn stop(mut self, color: impl ThemeColor, at: impl Into<Length>) -> Gradient {
        self.stops.push((Color::of(color), Some(at.into())));
        self.hints.push(None);
        self
    }

    /// Color stops without positions: spread evenly between their
    /// positioned neighbours (the first at 0%, the last at 100% when they
    /// have no position), as in CSS `linear-gradient(red, blue)`.
    pub fn colors<C: ThemeColor>(mut self, colors: impl IntoIterator<Item = C>) -> Gradient {
        for c in colors {
            self.stops.push((Color::of(c), None));
            self.hints.push(None);
        }
        self
    }

    /// The color space the stops are interpolated in (CSS `in <space>`;
    /// default Oklab).
    pub fn interpolate(mut self, space: ColorSpace) -> Gradient {
        self.interpolation.space = space;
        self
    }

    /// How hues are interpolated in a polar space (CSS `in <space> <hue>`).
    pub fn hue(mut self, hue: HueInterpolation) -> Gradient {
        self.interpolation.hue = hue;
        self
    }

    /// The midpoint between the stop before it and the next one: where
    /// their colors are halfway (CSS color hint, `.hint(30)` after a stop).
    pub fn hint(mut self, at: impl Into<Length>) -> Gradient {
        if let Some(h) = self.hints.last_mut() {
            *h = Some(at.into());
        }
        self
    }

    /// Repeat the stops beyond the last one (CSS `repeating-*-gradient`):
    /// the pattern from the first stop to the last repeats.
    pub fn repeating(mut self) -> Gradient {
        self.repeating = true;
        self
    }

    /// A radial gradient: a circle instead of an ellipse.
    pub fn circle(mut self) -> Gradient {
        if let GradientKind::Radial { circle, .. } = &mut self.kind {
            *circle = true;
        }
        self
    }

    /// A radial gradient: a circle of radius `r` (CSS `radial-gradient(40px
    /// at ..)`).
    pub fn radius(mut self, r: impl Into<Length>) -> Gradient {
        if let GradientKind::Radial { circle, size, .. } = &mut self.kind {
            let r = r.into();
            *circle = true;
            *size = RadialSize::Radii(r, r);
        }
        self
    }

    /// A radial gradient: an ellipse of radii `rx` × `ry` (CSS
    /// `radial-gradient(900px 380px at ..)`; percentages of the area's
    /// width and height).
    pub fn radii(mut self, rx: impl Into<Length>, ry: impl Into<Length>) -> Gradient {
        if let GradientKind::Radial { circle, size, .. } = &mut self.kind {
            *circle = false;
            *size = RadialSize::Radii(rx.into(), ry.into());
        }
        self
    }

    /// A radial gradient: how far its end reaches (CSS `closest-side` ..).
    pub fn extent(mut self, extent: RadialExtent) -> Gradient {
        if let GradientKind::Radial { size, .. } = &mut self.kind {
            *size = RadialSize::Extent(extent);
        }
        self
    }

    /// A radial or conic gradient: its center (CSS `at x y`, against the
    /// area: `pct(50)` is the middle).
    pub fn at(mut self, x: impl Into<Length>, y: impl Into<Length>) -> Gradient {
        match &mut self.kind {
            GradientKind::Radial { at, .. } | GradientKind::Conic { at, .. } => {
                *at = (x.into(), y.into());
            }
            GradientKind::Linear { .. } => {}
        }
        self
    }
}
