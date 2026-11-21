//! Role: The Pop kit's lengths as theme variables, and the values the theme sets them to.

use kivact::style::{Length, Theme, ThemeVar};

/// The padding of a panel, in logical px.
pub(crate) static PANEL_PADDING_VAR: ThemeVar<f64> = ThemeVar::new("pop.panel_padding", 24.0);
/// The padding of a panel.
pub(crate) const PANEL_PADDING: Length = Length::var(&PANEL_PADDING_VAR);
/// The height of a control, in logical px.
pub(crate) static CONTROL_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("pop.control_height", 36.0);
/// The height of a control.
pub(crate) const CONTROL_HEIGHT: Length = Length::var(&CONTROL_HEIGHT_VAR);
/// The corner radius of a control, in logical px.
pub(crate) static CONTROL_RADIUS_VAR: ThemeVar<f64> = ThemeVar::new("pop.control_radius", 8.0);
/// The corner radius of a control.
pub(crate) const CONTROL_RADIUS: Length = Length::var(&CONTROL_RADIUS_VAR);
/// The corner radius of a panel, in logical px.
pub(crate) static PANEL_RADIUS_VAR: ThemeVar<f64> = ThemeVar::new("pop.panel_radius", 14.0);
/// The corner radius of a panel.
pub(crate) const PANEL_RADIUS: Length = Length::var(&PANEL_RADIUS_VAR);

/// Set every length variable of the kit to its value (the same in both modes).
pub(crate) fn set_sizes(theme: Theme) -> Theme {
    theme
        .set(&PANEL_PADDING_VAR, 24.0)
        .set(&CONTROL_HEIGHT_VAR, 36.0)
        .set(&CONTROL_RADIUS_VAR, 8.0)
        .set(&PANEL_RADIUS_VAR, 14.0)
}
