//! Role: Validates text metrics and resolves line heights.

use crate::error::TextError;
use kivact_types::ComputedTextStyle;
use kivact_types::LineHeight;

/// Validated line height in logical pixels: `Normal` is the font size times
/// 1.3 rounded, `Scale` multiplies the font size, `Px` is the height.
pub(crate) fn line_height(
    style: &ComputedTextStyle,
    max_width: Option<f64>,
) -> Result<f64, TextError> {
    let valid = |v: f64| v.is_finite() && v > 0.0;
    let value = match style.line_height {
        LineHeight::Normal => None,
        LineHeight::Scale(v) | LineHeight::Px(v) => Some(v),
    };
    if !valid(style.font_size)
        || value.is_some_and(|v| !valid(v))
        || max_width.is_some_and(|v| !v.is_finite() || v < 0.0)
    {
        return Err(TextError::InvalidMetrics);
    }
    Ok(match style.line_height {
        LineHeight::Normal => (style.font_size * 1.3).round(),
        LineHeight::Scale(v) => style.font_size * v,
        LineHeight::Px(v) => v,
    })
}
