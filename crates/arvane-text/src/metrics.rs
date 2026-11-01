//! Role: Validates text metrics and resolves line heights.

use crate::error::TextError;
use kivact_types::LineHeight;
use kivact_types::protocol::TextStyle;

/// Validated line height in logical pixels: `Normal` is the font size times
/// 1.3 rounded, `Scale` multiplies the font size, `Px` is the height.
pub(crate) fn line_height(style: &TextStyle, max_width: Option<f32>) -> Result<f32, TextError> {
    let valid = |v: f32| v.is_finite() && v > 0.0;
    let value = match style.line_height {
        LineHeight::Normal => None,
        LineHeight::Scale(v) | LineHeight::Px(v) => Some(v),
        // Variants added later fall back to normal line height.
        _ => None,
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
        // Variants added later fall back to normal line height.
        _ => (style.font_size * 1.3).round(),
    })
}
