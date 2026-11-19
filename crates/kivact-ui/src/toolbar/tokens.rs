//! Role: Hold the toolbar tokens that the theme's `UiTokens` carries.
use kivact_core::style::Color;

/// Passive toolbar colors and metrics supplied through [`crate::UiTokens`].
#[derive(Clone, Debug, PartialEq)]
pub struct ToolbarTokens {
    pub background: Color,
    pub border: Color,
    pub padding: f64,
    pub gap: f64,
    pub min_height: f64,
}

impl Default for ToolbarTokens {
    fn default() -> Self {
        Self {
            background: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            padding: 8.0,
            gap: 4.0,
            min_height: 32.0,
        }
    }
}
