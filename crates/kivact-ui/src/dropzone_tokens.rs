//! Role: Optional drop target visual overrides supplied by kit themes.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DropzoneTokens {
    /// Root padding; None uses the standard recipe.
    pub padding: Option<f64>,
    /// Minimum target height; None uses the standard recipe.
    pub min_height: Option<f64>,
}
