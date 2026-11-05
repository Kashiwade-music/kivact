//! Role: Default themed recipes for the combobox family.
use crate::UiTokens;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, Color, Style, Theme, style};
use kivact_interaction::ListItemState;
pub(crate) fn frame(t: &Theme, state: &PartState) -> Style {
    let tokens = &t.ext::<UiTokens>().combobox;
    let s = style()
        .wrap(true)
        .align_items(Align::Center)
        .gap(4)
        .min_height(tokens.field_height)
        .padding(tokens.field_padding)
        .border(1, tokens.border)
        .radius(tokens.field_radius.unwrap_or(t.radius().md))
        .background(tokens.background)
        .color(tokens.foreground)
        .disabled(style().opacity(tokens.disabled_opacity.unwrap_or(t.disabled_opacity())));
    if state.focused {
        s.border_color(tokens.focus)
            .ring(tokens.focus_ring.unwrap_or(t.focus_ring()), tokens.focus)
    } else {
        s
    }
}
pub(crate) fn input(_: &Theme, _: &PartState) -> Style {
    style()
        .grow(1)
        .min_width(64)
        .height(24)
        .padding(0)
        .border(0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .ring_none()
        .hover(style().ring_none())
        .focus(style().ring_none())
}
pub(crate) fn surface(t: &Theme, _: &PartState) -> Style {
    let tokens = &t.ext::<UiTokens>().combobox;
    style()
        .padding(4)
        .border(1, tokens.border)
        .radius(tokens.field_radius.unwrap_or(t.radius().md))
        .background(tokens.background)
        .max_height(t.ext::<UiTokens>().combobox.list_height)
}
pub(crate) fn option(t: &Theme, state: &ListItemState) -> Style {
    let tokens = &t.ext::<UiTokens>().combobox;
    let s = style()
        .min_height(tokens.option_height)
        .padding_x(tokens.option_padding_x)
        .padding_y(tokens.option_padding_y)
        .radius(tokens.option_radius)
        .color(tokens.foreground)
        .disabled(style().opacity(tokens.disabled_opacity.unwrap_or(t.disabled_opacity())));
    if state.active {
        s.background(tokens.active_background)
    } else {
        s
    }
}
pub(crate) fn chip(t: &Theme, _: &PartState) -> Style {
    let tokens = &t.ext::<UiTokens>().combobox;
    style()
        .height(tokens.chip_height)
        .padding_left(8)
        .padding_right(4)
        .gap(4)
        .align_items(Align::Center)
        .radius(tokens.chip_radius)
        .background(tokens.chip_background)
        .color(tokens.foreground)
}
pub(crate) fn remove(_: &Theme, _: &PartState) -> Style {
    style().size(16, 16).padding(0)
}
pub(crate) fn empty(t: &Theme, _: &PartState) -> Style {
    let tokens = &t.ext::<UiTokens>().combobox;
    style()
        .padding(8)
        .font_size(tokens.empty_font_size.unwrap_or(t.text().size_small))
        .color(tokens.muted)
}
