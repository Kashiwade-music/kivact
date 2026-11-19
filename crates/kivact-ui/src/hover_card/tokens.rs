//! Role: Hold the hover card tokens that the theme's `UiTokens` carries.

/// Per-theme geometry used by the preview overlay.
#[derive(Clone, Debug, PartialEq)]
pub struct HoverCardTokens {
    /// Logical pixel gap from the trigger.
    pub offset: f64,
    /// Logical pixel inset from the viewport edges.
    pub viewport_margin: f64,
}

impl Default for HoverCardTokens {
    fn default() -> Self {
        Self {
            offset: 4.0,
            viewport_margin: 8.0,
        }
    }
}
