//! Role: Resolve Astryx Neutral elevation and inset shadows.
use kivact::draw::Shadow;

/// Elevation shadows in front-to-back drawing order.
#[derive(Clone, Debug, PartialEq)]
pub struct Shadows {
    /// Layered shadows for subtly raised controls and surfaces.
    pub low: Vec<Shadow>,
    /// Layered shadows for floating surfaces with medium elevation.
    pub med: Vec<Shadow>,
    /// Layered shadows for the most prominent floating surfaces.
    pub high: Vec<Shadow>,
    /// Inset edge shadow used while a surface is hovered.
    pub inset_hover: Shadow,
    /// Inset edge shadow marking a selected surface.
    pub inset_selected: Shadow,
    /// Inset edge shadow for a successful or valid state.
    pub inset_success: Shadow,
    /// Inset edge shadow for a warning state.
    pub inset_warning: Shadow,
    /// Inset edge shadow for an error or invalid state.
    pub inset_error: Shadow,
    /// Additional input outline drawn on hover.
    pub input_hover_ring: Shadow,
    /// Additional input outline drawn while focused.
    pub input_focus_ring: Shadow,
}
