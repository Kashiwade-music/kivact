//! Role: Hold the table tokens that the theme's `UiTokens` carries.

/// Theme overrides for static tables.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TableTokens {
    pub cell_padding: Option<f64>,
    pub header_height: Option<f64>,
    pub font_size: Option<f64>,
}
