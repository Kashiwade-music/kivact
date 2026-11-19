//! Role: The date parts recipe state, tokens, recipe table and default looks.

use crate::tokens::UiTokens;
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
