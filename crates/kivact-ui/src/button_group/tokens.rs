//! Role: Hold the button group tokens that the theme's `UiTokens` carries.
use kivact_core::style::Color;

/// Passive button group metrics and colors.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonGroupTokens {
    pub gap: f64,
    pub joined: bool,
    pub radius: f64,
    pub divider: Color,
    pub solid_divider: Color,
    pub text_background: Color,
    pub text_color: Color,
}

impl Default for ButtonGroupTokens {
    fn default() -> Self {
        Self {
            gap: 0.0,
            joined: true,
            radius: 0.0,
            divider: Color::TRANSPARENT,
            solid_divider: Color::TRANSPARENT,
            text_background: Color::TRANSPARENT,
            text_color: Color::BLACK,
        }
    }
}
