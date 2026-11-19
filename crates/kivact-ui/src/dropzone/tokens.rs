//! Role: Hold the dropzone tokens that the theme's `UiTokens` carries.

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DropzoneTokens {
    /// Root padding; None uses the standard recipe.
    pub padding: Option<f64>,
    /// Minimum target height; None uses the standard recipe.
    pub min_height: Option<f64>,
}
