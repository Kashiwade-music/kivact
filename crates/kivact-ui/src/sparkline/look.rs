//! Role: The sparkline recipe state, recipe table and default looks.

use crate::recipe::Look;
use crate::recipe::recipe_table;
use kivact_core::style::{Color, Length, ThemeVar, color};
use kivact_core::style::{Style, Theme, pct, style};

/// The color of a sparkline's line.
pub static LINE_VAR: ThemeVar<Color> = ThemeVar::alias("sparkline.line", &color::ACCENT_VAR);
/// [`LINE_VAR`] as a color for a style.
pub const LINE: Color = Color::var(&LINE_VAR);
/// The width of a sparkline's line, in logical px.
pub static LINE_WIDTH_VAR: ThemeVar<f64> = ThemeVar::new("sparkline.line_width", 1.5);
/// [`LINE_WIDTH_VAR`] as a length for a style.
pub const LINE_WIDTH: Length = Length::var(&LINE_WIDTH_VAR);
/// The inset of a sparkline's points from its box's edges, in logical px.
pub static INSET_VAR: ThemeVar<f64> = ThemeVar::new("sparkline.inset", 0.0);
/// [`INSET_VAR`] as a length for a style.
pub const INSET: Length = Length::var(&INSET_VAR);

/// What fills the area under a sparkline's line.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SparklineFill {
    /// No fill: the line alone.
    #[default]
    None,
    /// A solid fill of this color.
    Solid(Color),
    /// A fill of this color that fades toward the baseline.
    Fade(Color),
}

pub(crate) fn sparkline_root(_: &Theme, _: &Look<()>) -> Style {
    style().width(pct(100)).height(32).min_width(1)
}

recipe_table! {
    /// The recipes of a [`sparkline`](super::sparkline).
    SparklineTheme {
        /// The chart's box: its size. The sparkline's theme variables and settings draw the line and the area in it.
        root(root_recipe): () = sparkline_root,
    }
    settings {
        /// Whether the line's ends are round; see the line's width in `sparkline.line_width`.
        round(round_value): bool = false,
        /// What fills the area under the line; see [`SparklineFill`].
        fill(fill_value): SparklineFill = SparklineFill::None,
    }
}
