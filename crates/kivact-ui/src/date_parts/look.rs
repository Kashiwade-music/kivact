//! Role: The date parts recipe state, tokens, recipe table and default looks.

use crate::UiTokens;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, style};
pub(crate) fn input(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().xs)
}

pub(crate) fn wheel(t: &Theme, _: &PartState) -> Style {
    style()
        .width(t.ext::<UiTokens>().date_input.wheel_width)
        .padding(t.space().sm)
        .gap(t.space().xs)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
}

/// Measurements of the rows visible around a wheel's selected value.
#[derive(Clone, Debug, PartialEq)]
pub struct DateInputTokens {
    pub wheel_row_height: f64,
    pub wheel_width: f64,
}

impl Default for DateInputTokens {
    fn default() -> Self {
        Self {
            wheel_row_height: 40.0,
            wheel_width: 80.0,
        }
    }
}
