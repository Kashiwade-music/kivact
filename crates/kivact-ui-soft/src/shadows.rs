//! Role: Resolve Astryx Neutral elevation and inset shadows.
use kivact::draw::Shadow;

/// Elevation shadows in front-to-back drawing order.
#[derive(Clone, Debug, PartialEq)]
pub struct Shadows {
    pub low: Vec<Shadow>,
    pub med: Vec<Shadow>,
    pub high: Vec<Shadow>,
    pub inset_hover: Shadow,
    pub inset_selected: Shadow,
    pub inset_success: Shadow,
    pub inset_warning: Shadow,
    pub inset_error: Shadow,
    pub input_hover_ring: Shadow,
    pub input_focus_ring: Shadow,
}
