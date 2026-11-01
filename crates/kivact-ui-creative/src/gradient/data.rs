//! Role: Public gradient stop data and position normalization.
use kivact::style::Color;
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
pub(super) fn position_value(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}
