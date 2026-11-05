//! Role: Define passive preview recipe projections and overlay geometry.
use kivact_core::view::Placement;

/// The current input and requested placement exposed to preview recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoverCardSurfaceState {
    pub input: kivact_core::custom::PartState,
    pub open: bool,
    pub placement: Placement,
}

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
