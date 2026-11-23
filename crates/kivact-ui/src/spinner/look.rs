//! Role: The spinner recipe state, recipe table and default looks.
use crate::look_common::side;
use crate::recipe::recipe_table;
use crate::theme::Look;

use crate::control_size::ControlSize;
use kivact_core::draw::{LineCap, Path, Stroke};
use kivact_core::geometry::Point;
use kivact_core::style::{Length, ThemeVar};
use kivact_core::style::{Style, Theme, style};

/// The period of a spinner's turn, in milliseconds.
pub static PERIOD_MS_VAR: ThemeVar<f64> = ThemeVar::new("spinner.period_ms", 1000.0);
/// [`PERIOD_MS_VAR`] as a length for a style.
pub const PERIOD_MS: Length = Length::var(&PERIOD_MS_VAR);

/// The gap between a spinner and its label, in logical px.
pub static GAP_VAR: ThemeVar<f64> = ThemeVar::new("spinner.gap", 6.0);
/// [`GAP_VAR`] as a length for a style.
pub const GAP: Length = Length::var(&GAP_VAR);

pub(crate) fn spinner(_: &Theme, s: &Look<SpinnerState>) -> Style {
    let side = side(s.size) * 0.6;
    let turn = s.turn;
    style()
        .size(side, side)
        .flex_shrink(0)
        .paint(move |d, size| {
            let center = Point::new(size.width / 2.0, size.height / 2.0);
            let arc = Path::new().arc(
                center,
                size.width.min(size.height) / 2.0 - 2.0,
                turn,
                turn + 270.0,
                false,
            );
            d.stroke_path(
                &arc,
                &Stroke::new(2.0).cap(LineCap::Round),
                kivact_core::style::color::TEXT,
            );
        })
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpinnerState {
    pub size: ControlSize,
    pub turn: f64,
}

recipe_table! {
    /// The recipes of a spinner.
    SpinnerTheme {
        root(root_recipe): SpinnerState = spinner,
    }
    settings {
        /// The turn period under reduced motion, in milliseconds. `None` keeps the spinner's decorative motion paused under reduced motion; `Some` keeps the essential turn running at this period.
        reduced_period(reduced_period_value): Option<f64> = None,
    }
}
