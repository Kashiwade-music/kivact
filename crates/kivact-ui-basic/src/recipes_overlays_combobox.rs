//! Role: Lyra field, suggestion and chip recipes of the combobox.

use crate::color::{FOREGROUND, INPUT, MUTED, MUTED_FOREGROUND};
use crate::recipe_parts::{field_fill, focus_ring, row_style, surface_style};
use kivact::custom::PartState;
use kivact::interaction::ListItemState;
use kivact::style::{
    Align, Color, FlexDirection, Justify, Style, TextAlign, Theme, ThemeMode, control, pct, style,
};
use kivact::ui::theme::{ComboboxTheme, Look};

/// The field: the wrapping row of chips and the input, bordered like a
/// text field; disabled, it is dimmed.
fn frame(t: &Theme, s: &Look<PartState>, _base: Style) -> Style {
    let dark = t.mode() == ThemeMode::Dark;
    let base = style()
        .flex_direction(FlexDirection::Row)
        .flex_wrap(true)
        .align_items(Align::Center)
        .gap(4.0)
        .width(pct(100.0))
        .min_height(32.0)
        .padding(4.0)
        .border_radius(0.0)
        .background(field_fill(dark))
        .border(1.0, INPUT);
    let base = if s.is_disabled() {
        base.merge(&style().opacity(t.get(&control::DISABLED_OPACITY_VAR)))
            .background(INPUT.with_alpha(if dark { 0.8 } else { 0.5 }))
    } else {
        base
    };
    if s.is_focused() {
        base.merge(&focus_ring(t))
    } else {
        base
    }
}

/// The text input inside the field: no border, no ring.
fn input(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_grow(1.0)
        .min_width(64.0)
        .height(22.0)
        .padding_x(2.0)
        .padding_y(0.0)
        .border(0.0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .outline_none()
        .hover(style().outline_none())
        .focus(style().outline_none())
}

/// The suggestion list: [`surface_style`], 288 px high at most.
fn surface(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    surface_style().max_height(288.0)
}

fn option(_: &Theme, s: &Look<ListItemState>, _base: Style) -> Style {
    row_style(s)
}

/// A chip of the selected values: `h-[21px]` in `bg-muted`.
fn chip(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(4.0)
        .height(21.0)
        .padding_left(6.0)
        .padding_right(2.0)
        .border_radius(0.0)
        .background(MUTED)
        .color(FOREGROUND)
        .font_size(12.0)
        .font_weight(500)
        .text_wrap(false)
}

/// A chip's remove button: half dimmed, full while hovered.
fn remove(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .size(16.0, 16.0)
        .padding(0.0)
        .opacity(0.5)
        .hover(style().opacity(1.0))
}

fn empty(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .width(pct(100.0))
        .padding_y(8.0)
        .font_size(12.0)
        .text_align(TextAlign::Center)
        .color(MUTED_FOREGROUND)
}

/// The combobox recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        ComboboxTheme::default()
            .frame(frame)
            .input(input)
            .surface(surface)
            .option(option)
            .chip(chip)
            .remove(remove)
            .empty(empty),
    )
}
