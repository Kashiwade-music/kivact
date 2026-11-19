//! Role: Hold the carousel tokens that the theme's `UiTokens` carries.

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CarouselTokens {
    /// Space between slides; None uses the standard recipe.
    pub gap: Option<f64>,
    /// Viewport inset; None uses the standard recipe.
    pub padding: Option<f64>,
    /// Fade distance at overflowing edges.
    pub edge_fade_width: Option<f64>,
}
