//! Role: The Creative kit's length variables (the control row and the gap) and their density values.

use crate::options::Density;
use kivact::style::{Length, Theme, ThemeVar};

/// The height of a control row, in logical px.
pub(crate) static ROW_VAR: ThemeVar<f64> = ThemeVar::new("creative.row", 24.0);
/// The gap between the items of a row, in logical px.
pub(crate) static GAP_VAR: ThemeVar<f64> = ThemeVar::new("creative.gap", 8.0);

pub(crate) const ROW: Length = Length::var(&ROW_VAR);
pub(crate) const GAP: Length = Length::var(&GAP_VAR);

/// Set the kit's lengths to their values at `density`.
pub(crate) fn set_sizes(theme: Theme, density: Density) -> Theme {
    let (row, gap) = match density {
        Density::Comfortable => (24.0, 8.0),
        Density::Compact => (20.0, 6.0),
    };
    theme.set(&ROW_VAR, row).set(&GAP_VAR, gap)
}
