//! Role: The typing indicator recipe state, recipe table and default looks.

use crate::recipe::Look;
use crate::recipe::recipe_table;
use kivact_core::style::{Length, ThemeVar};
use kivact_core::style::{Style, Theme, style};

/// The period of the typing dots' cycle, in milliseconds.
pub static PERIOD_MS_VAR: ThemeVar<f64> = ThemeVar::new("typing_indicator.period_ms", 1000.0);
/// [`PERIOD_MS_VAR`] as a length for a style.
pub const PERIOD_MS: Length = Length::var(&PERIOD_MS_VAR);

pub(crate) fn typing_dots(_: &Theme, s: &Look<TypingIndicatorState>) -> Style {
    let phase = s.phase;
    style().size(48, 24).paint(move |d, _| {
        for i in 0..3 {
            let y = 12.0 - (phase - i as f64 * 0.95).sin().max(0.0) * 3.0;
            d.rounded_rect(
                kivact_core::geometry::Rect::new(7.0 + i as f64 * 12.0, y, 5.0, 5.0),
                2.5,
                kivact_core::style::color::TEXT_MUTED,
            );
        }
    })
}

/// The repeating typing phase supplied to recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct TypingIndicatorState {
    pub phase: f64,
}

recipe_table! {
    /// Styles for the persistent parts of TypingIndicator.
    TypingIndicatorTheme {
        root(root_recipe): TypingIndicatorState = crate::look_common::row,
        avatar(avatar_recipe): TypingIndicatorState = crate::look_common::slot,
        dots(dots_recipe): TypingIndicatorState = typing_dots,
    }
}
