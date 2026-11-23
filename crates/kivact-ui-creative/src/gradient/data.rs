//! Role: Public gradient stop data, color transform and position normalization.
use kivact::style::Color;
use std::rc::Rc;

/// One application-owned gradient stop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    /// Normalized ramp position. External invalid values are only cleaned for display.
    pub position: f64,
    /// Color in the application's storage color space.
    pub color: Color,
}

impl GradientStop {
    /// Create a stop, clamping position to 0..1. NaN/-infinity use 0; +infinity uses 1.
    pub fn new(position: f64, color: Color) -> Self {
        Self {
            position: position_value(position),
            color,
        }
    }
}

/// How a gradient editor shows the stored colors: `to_display` maps a stored
/// color to the color shown, and `from_display` maps a shown color back to
/// storage. Capturing closures are supported. Opening and cancellation
/// preserve raw storage values, including when the two are not exact inverses.
#[derive(Clone)]
pub struct ColorTransform {
    pub(super) to_display: Rc<dyn Fn(Color) -> Color>,
    pub(super) from_display: Rc<dyn Fn(Color) -> Color>,
}

impl ColorTransform {
    /// Create a transform from the storage-to-display and display-to-storage functions.
    pub fn new(
        to_display: impl Fn(Color) -> Color + 'static,
        from_display: impl Fn(Color) -> Color + 'static,
    ) -> Self {
        Self {
            to_display: Rc::new(to_display),
            from_display: Rc::new(from_display),
        }
    }
}

pub(super) fn position_value(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}
