//! Role: The kbd recipe state, tokens, recipe table and default looks.

use kivact_core::style::{Align, ColorToken, Style, Theme, style};
pub(crate) fn kbd(_: &Theme, _: &()) -> Style {
    style().align_items(Align::Center).gap(4).shrink(0)
}

pub(crate) fn kbd_key(t: &Theme, _: &KbdKeyState) -> Style {
    style()
        .min_height(20)
        .min_width(20)
        .padding_x(t.space().sm)
        .radius(t.radius().sm)
        .background(ColorToken::Surface)
        .border_bottom(2, ColorToken::Border)
        .color(ColorToken::Text)
        .font_size(12)
        .text_wrap(false)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KbdKeyState {
    pub glyph: String,
    pub label: String,
}
