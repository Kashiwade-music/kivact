//! Role: Hold the item list tokens that the theme's `UiTokens` carries.

/// Visual overrides for static lists and item rows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ItemListTokens {
    pub gap: Option<f64>,
    pub padding: Option<f64>,
    pub row_gap: Option<f64>,
    pub description_size: Option<f64>,
    pub title_lines: Option<u32>,
    pub description_lines: Option<u32>,
}
