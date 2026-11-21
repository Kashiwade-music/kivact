//! Role: The theme colors of a plot: five series, the grid and the axes.
//!
//! A kit sets them with `Theme::set`; without a kit they follow the core
//! colors of the theme.

use kivact::style::{Color, ThemeVar, color};

macro_rules! plot_color {
    ($var:ident, $name:ident, $init:expr, $doc:literal) => {
        #[doc = $doc]
        pub static $var: ThemeVar<Color> = $init;
        #[doc = $doc]
        pub const $name: Color = Color::var(&$var);
    };
}

plot_color!(
    SERIES_1_VAR,
    SERIES_1,
    ThemeVar::alias("plot.series_1", &color::ACCENT_VAR),
    "The first series without its own color."
);
plot_color!(
    SERIES_2_VAR,
    SERIES_2,
    ThemeVar::alias("plot.series_2", &color::SUCCESS_VAR),
    "The second series."
);
plot_color!(
    SERIES_3_VAR,
    SERIES_3,
    ThemeVar::alias("plot.series_3", &color::WARNING_VAR),
    "The third series."
);
plot_color!(
    SERIES_4_VAR,
    SERIES_4,
    ThemeVar::alias("plot.series_4", &color::DANGER_VAR),
    "The fourth series."
);
plot_color!(
    SERIES_5_VAR,
    SERIES_5,
    ThemeVar::alias("plot.series_5", &color::TEXT_MUTED_VAR),
    "The fifth series."
);
plot_color!(
    GRID_VAR,
    GRID,
    ThemeVar::alias("plot.grid", &color::BORDER_VAR),
    "Grid lines and the plot frame."
);
plot_color!(
    AXIS_VAR,
    AXIS,
    ThemeVar::alias("plot.axis", &color::TEXT_MUTED_VAR),
    "Axis labels, ticks and titles."
);

/// The series colors in order; series `i` without a color uses entry `i % 5`.
pub(crate) const SERIES: [Color; 5] = [SERIES_1, SERIES_2, SERIES_3, SERIES_4, SERIES_5];
