//! Role: Hold the tree tokens that the theme's `UiTokens` carries.

/// Theme overrides for tree rows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TreeTokens {
    pub marker_width: Option<f64>,
    pub indent: Option<f64>,
    pub padding: Option<f64>,
    pub gap: Option<f64>,
    pub row_height: Option<f64>,
    pub radius: Option<f64>,
}
