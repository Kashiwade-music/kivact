//! Role: The elevation and inset shadows of Astryx Neutral, in front-to-back drawing order (crate-private).
use crate::color::{
    input_focus_color, input_hover_color, shadow_black, shadow_error, shadow_info, shadow_success,
    shadow_warning, shadow_white,
};
use kivact::style::Color;
use kivact::style::Shadow;

fn outside(y: i32, blur: i32, alpha: f64) -> Shadow {
    Shadow::new(0, y, blur, shadow_black(alpha))
}

fn inset(spread: i32, color: Color) -> Shadow {
    Shadow::new(0, 0, 0, color).spread(spread).inset()
}

/// Layered shadows for subtly raised controls and surfaces.
pub(crate) fn low(dark: bool) -> Vec<Shadow> {
    if dark {
        vec![
            outside(2, 4, 64.0 / 255.0),
            outside(4, 8, 102.0 / 255.0),
            inset(1, shadow_white(20.0 / 255.0)),
        ]
    } else {
        vec![outside(2, 4, 13.0 / 255.0), outside(4, 8, 26.0 / 255.0)]
    }
}

/// Layered shadows for floating surfaces with medium elevation.
pub(crate) fn med(dark: bool) -> Vec<Shadow> {
    if dark {
        vec![
            outside(2, 4, 89.0 / 255.0),
            outside(4, 12, 128.0 / 255.0),
            inset(1, shadow_white(31.0 / 255.0)),
        ]
    } else {
        vec![outside(2, 4, 13.0 / 255.0), outside(4, 12, 26.0 / 255.0)]
    }
}

/// Layered shadows for the most prominent floating surfaces.
pub(crate) fn high(dark: bool) -> Vec<Shadow> {
    if dark {
        vec![
            outside(4, 6, 128.0 / 255.0),
            outside(12, 24, 179.0 / 255.0),
            inset(1, shadow_white(38.0 / 255.0)),
        ]
    } else {
        vec![outside(4, 6, 26.0 / 255.0), outside(12, 24, 38.0 / 255.0)]
    }
}

/// Inset edge shadow used while a surface is hovered.
pub(crate) fn inset_hover() -> Shadow {
    inset(2, shadow_info(77.0 / 255.0))
}

/// Inset edge shadow marking a selected surface.
pub(crate) fn inset_selected() -> Shadow {
    inset(2, shadow_info(128.0 / 255.0))
}

/// Inset edge shadow for a successful or valid state.
pub(crate) fn inset_success() -> Shadow {
    inset(2, shadow_success(77.0 / 255.0))
}

/// Inset edge shadow for a warning state.
pub(crate) fn inset_warning() -> Shadow {
    inset(2, shadow_warning(77.0 / 255.0))
}

/// Inset edge shadow for an error or invalid state.
pub(crate) fn inset_error() -> Shadow {
    inset(2, shadow_error(77.0 / 255.0))
}

/// Additional input outline drawn on hover.
pub(crate) fn input_hover_ring(dark: bool) -> Shadow {
    inset(2, input_hover_color(dark))
}

/// Additional input outline drawn while focused.
pub(crate) fn input_focus_ring(dark: bool) -> Shadow {
    inset(2, input_focus_color(dark))
}
