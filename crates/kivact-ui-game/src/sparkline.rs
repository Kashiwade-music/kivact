//! Role: The sparkline theme of the metrics: a solid primary fill.

use crate::color as kit;
use kivact::style::Color;
use kivact::ui::theme::{SparklineFill, SparklineTheme};

/// The sparkline's fill: the primary tint, solid.
pub(crate) fn sparkline_table() -> SparklineTheme {
    SparklineTheme::default().fill(SparklineFill::Solid(Color::var(&kit::PRIMARY_WEAK_VAR)))
}
