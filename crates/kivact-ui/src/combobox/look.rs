//! Role: Default themed recipes for the combobox family.

use crate::UiTokens;
use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, Color, ColorToken, Style, Theme, style};
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
    if state.is_focused() {
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

/// Visual measurements for [`crate::combobox`], configured through [`crate::UiTokens`].
#[derive(Clone, Debug, PartialEq)]
pub struct ComboboxTokens {
    pub foreground: Color,
    pub muted: Color,
    pub background: Color,
    pub border: Color,
    pub active_background: Color,
    pub chip_background: Color,
    pub focus: Color,
    pub field_radius: Option<f64>,
    pub focus_ring: Option<f64>,
    pub disabled_opacity: Option<f64>,
    pub empty_font_size: Option<f64>,
    pub field_height: f64,
    pub field_padding: f64,
    pub chip_height: f64,
    pub chip_radius: f64,
    pub option_height: f64,
    pub option_padding_x: f64,
    pub option_padding_y: f64,
    pub option_radius: f64,
    pub list_height: f64,
    pub popup_offset: f64,
}

impl Default for ComboboxTokens {
    fn default() -> Self {
        Self {
            foreground: Color::token(ColorToken::Text),
            muted: Color::token(ColorToken::TextMuted),
            background: Color::token(ColorToken::Surface),
            border: Color::token(ColorToken::Border),
            active_background: Color::token(ColorToken::Hover),
            chip_background: Color::token(ColorToken::Selection),
            focus: Color::token(ColorToken::Focus),
            field_radius: None,
            focus_ring: None,
            disabled_opacity: None,
            empty_font_size: None,
            field_height: 32.0,
            field_padding: 4.0,
            chip_height: 24.0,
            chip_radius: 4.0,
            option_height: 32.0,
            option_padding_x: 8.0,
            option_padding_y: 6.0,
            option_radius: 4.0,
            list_height: 292.0,
            popup_offset: 6.0,
        }
    }
}

recipe_table! {
    /// Recipes of the standard combobox family.
    ComboboxRecipes {
        frame(frame_recipe): PartState = frame,
        input(input_recipe): PartState = input,
        surface(surface_recipe): PartState = surface,
        option(option_recipe): ListItemState = option,
        chip(chip_recipe): PartState = chip,
        remove(remove_recipe): PartState = remove,
        empty(empty_recipe): PartState = empty,
    }
}
