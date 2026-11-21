//! Role: The core control metrics of a theme: control height, disabled opacity and overlay margin.

use kivact_types::{Length, ThemeVar};

use crate::space::length_vars;

length_vars! {
    /// The minimum height of an interactive control, in logical px.
    HEIGHT_VAR / HEIGHT = "control.height", 28.0;
    /// The viewport inset used to fit overlays, in logical px.
    OVERLAY_MARGIN_VAR / OVERLAY_MARGIN = "control.overlay_margin", 0.0;
}

/// The opacity a disabled focusable element is drawn with, unless its
/// style's disabled state sets one.
pub static DISABLED_OPACITY_VAR: ThemeVar<f64> = ThemeVar::new("control.disabled_opacity", 0.5);
