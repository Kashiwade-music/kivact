//! Role: Hold the combobox tokens that the theme's `UiTokens` carries.
use kivact_core::style::Color;

/// Visual measurements for [`crate::combobox`], configured through [`crate::UiTokens`].
#[derive(Clone, Debug, PartialEq)]
pub struct ComboboxTokens {
    pub foreground: Color,
    pub muted: Color,
    pub background: Color,
    pub border: Color,
    pub active_background: Color,
    pub chip_background: Color,
    pub focus: Color,
    pub field_radius: Option<f64>,
    pub focus_ring: Option<f64>,
    pub disabled_opacity: Option<f64>,
    pub empty_font_size: Option<f64>,
    pub field_height: f64,
    pub field_padding: f64,
    pub chip_height: f64,
    pub chip_radius: f64,
    pub option_height: f64,
    pub option_padding_x: f64,
    pub option_padding_y: f64,
    pub option_radius: f64,
    pub list_height: f64,
    pub popup_offset: f64,
}

impl Default for ComboboxTokens {
    fn default() -> Self {
        Self {
            foreground: kivact_core::style::color::TEXT,
            muted: kivact_core::style::color::TEXT_MUTED,
            background: kivact_core::style::color::SURFACE,
            border: kivact_core::style::color::BORDER,
            active_background: kivact_core::style::color::HOVER,
            chip_background: kivact_core::style::color::SELECTION,
            focus: kivact_core::style::color::FOCUS,
            field_radius: None,
            focus_ring: None,
            disabled_opacity: None,
            empty_font_size: None,
            field_height: 32.0,
            field_padding: 4.0,
            chip_height: 24.0,
            chip_radius: 4.0,
            option_height: 32.0,
            option_padding_x: 8.0,
            option_padding_y: 6.0,
            option_radius: 4.0,
            list_height: 292.0,
            popup_offset: 6.0,
        }
    }
}
