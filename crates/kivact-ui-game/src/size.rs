//! Role: The game kit's length theme variables, which take the values of its density.
use crate::theme::Density;
use kivact::style::{Length, Theme, ThemeVar, control};

/// `--control-height`: the height of a control; the theme's control height by default.
pub(crate) static CONTROL_HEIGHT_VAR: ThemeVar<f64> =
    ThemeVar::alias("game.control_height", &control::HEIGHT_VAR);
/// The height of a control.
pub(crate) const CONTROL_HEIGHT: Length = Length::var(&CONTROL_HEIGHT_VAR);

/// `--row-height`: the height of a list or table row.
pub(crate) static ROW_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("game.row_height", 42.);
/// The height of a list or table row.
pub(crate) const ROW_HEIGHT: Length = Length::var(&ROW_HEIGHT_VAR);

/// `--padding`: the padding of a panel.
pub(crate) static PADDING_VAR: ThemeVar<f64> = ThemeVar::new("game.padding", 20.);
/// The padding of a panel.
pub(crate) const PADDING: Length = Length::var(&PADDING_VAR);

/// Set every length variable of the kit to its value in `density`.
pub(crate) fn set_sizes(theme: Theme, density: Density) -> Theme {
    let (control_height, row_height, padding) = match density {
        Density::Comfortable => (34., 42., 20.),
        Density::Compact => (28., 32., 14.),
    };
    theme
        .set(&CONTROL_HEIGHT_VAR, control_height)
        .set(&ROW_HEIGHT_VAR, row_height)
        .set(&PADDING_VAR, padding)
}
