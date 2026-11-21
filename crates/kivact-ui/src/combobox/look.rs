//! Role: Default themed recipes for the combobox family.

use crate::UiTokens;
use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, Color, Style, Theme, style};
use kivact_interaction::ListItemState;
pub(crate) fn frame(t: &Theme, state: &Look<PartState>) -> Style {
    let tokens = &t.ext::<UiTokens>().combobox;
    let s = style()
        .flex_wrap(true)
        .align_items(Align::Center)
        .gap(4)
        .min_height(tokens.field_height)
        .padding(tokens.field_padding)
        .border(1, tokens.border)
        .border_radius(
            tokens
                .field_radius
                .unwrap_or(t.get(&kivact_core::style::radius::MD_VAR)),
        )
        .background(tokens.background)
        .color(tokens.foreground)
        .disabled(
            style().opacity(
                tokens
                    .disabled_opacity
                    .unwrap_or(t.get(&kivact_core::style::control::DISABLED_OPACITY_VAR)),
            ),
        );
    if state.is_focused() {
        s.border_color(tokens.focus)
            .outline(tokens.focus_ring.unwrap_or(t.focus_ring()), tokens.focus)
    } else {
        s
    }
}
pub(crate) fn input(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_grow(1)
        .min_width(64)
        .height(24)
        .padding(0)
        .border(0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .outline_none()
        .hover(style().outline_none())
        .focus(style().outline_none())
}
pub(crate) fn surface(t: &Theme, _: &Look<PartState>) -> Style {
    let tokens = &t.ext::<UiTokens>().combobox;
    style()
        .padding(4)
        .border(1, tokens.border)
        .border_radius(
            tokens
                .field_radius
                .unwrap_or(t.get(&kivact_core::style::radius::MD_VAR)),
        )
        .background(tokens.background)
        .max_height(t.ext::<UiTokens>().combobox.list_height)
}
pub(crate) fn option(t: &Theme, state: &Look<ListItemState>) -> Style {
    let tokens = &t.ext::<UiTokens>().combobox;
    let s = style()
        .min_height(tokens.option_height)
        .padding_x(tokens.option_padding_x)
        .padding_y(tokens.option_padding_y)
        .border_radius(tokens.option_radius)
        .color(tokens.foreground)
        .disabled(
            style().opacity(
                tokens
                    .disabled_opacity
                    .unwrap_or(t.get(&kivact_core::style::control::DISABLED_OPACITY_VAR)),
            ),
        );
    if state.active {
        s.background(tokens.active_background)
    } else {
        s
    }
}
pub(crate) fn chip(t: &Theme, _: &Look<PartState>) -> Style {
    let tokens = &t.ext::<UiTokens>().combobox;
    style()
        .height(tokens.chip_height)
        .padding_left(8)
        .padding_right(4)
        .gap(4)
        .align_items(Align::Center)
        .border_radius(tokens.chip_radius)
        .background(tokens.chip_background)
        .color(tokens.foreground)
}
pub(crate) fn remove(_: &Theme, _: &Look<PartState>) -> Style {
    style().size(16, 16).padding(0)
}
pub(crate) fn empty(t: &Theme, _: &Look<PartState>) -> Style {
    let tokens = &t.ext::<UiTokens>().combobox;
    style()
        .padding(8)
        .font_size(
            tokens
                .empty_font_size
                .unwrap_or(t.get(&kivact_core::style::font::SIZE_SMALL_VAR)),
        )
        .color(tokens.muted)
}

recipe_table! {
    /// Recipes of the standard combobox family.
    ComboboxTheme {
        frame(frame_recipe): PartState = frame,
        input(input_recipe): PartState = input,
        surface(surface_recipe): PartState = surface,
        option(option_recipe): ListItemState = option,
        chip(chip_recipe): PartState = chip,
        remove(remove_recipe): PartState = remove,
        empty(empty_recipe): PartState = empty,
    }
}
