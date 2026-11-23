//! Role: Default themed recipes for the combobox family.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, Color, Style, Theme, style};
use kivact_core::style::{Length, ThemeVar, color, control, font, radius};
use kivact_interaction::ListItemState;

/// The text color of the combobox.
pub static FOREGROUND_VAR: ThemeVar<Color> =
    ThemeVar::alias("combobox.foreground", &color::TEXT_VAR);
/// [`FOREGROUND_VAR`] as a color for a style.
pub const FOREGROUND: Color = Color::var(&FOREGROUND_VAR);
/// The muted text color of the combobox (placeholders and hints).
pub static MUTED_VAR: ThemeVar<Color> = ThemeVar::alias("combobox.muted", &color::TEXT_MUTED_VAR);
/// [`MUTED_VAR`] as a color for a style.
pub const MUTED: Color = Color::var(&MUTED_VAR);
/// The background of the combobox field and its list.
pub static BACKGROUND_VAR: ThemeVar<Color> =
    ThemeVar::alias("combobox.background", &color::SURFACE_VAR);
/// [`BACKGROUND_VAR`] as a color for a style.
pub const BACKGROUND: Color = Color::var(&BACKGROUND_VAR);
/// The border of the combobox field.
pub static BORDER_VAR: ThemeVar<Color> = ThemeVar::alias("combobox.border", &color::BORDER_VAR);
/// [`BORDER_VAR`] as a color for a style.
pub const BORDER: Color = Color::var(&BORDER_VAR);
/// The background of the highlighted option.
pub static ACTIVE_BACKGROUND_VAR: ThemeVar<Color> =
    ThemeVar::alias("combobox.active_background", &color::HOVER_VAR);
/// [`ACTIVE_BACKGROUND_VAR`] as a color for a style.
pub const ACTIVE_BACKGROUND: Color = Color::var(&ACTIVE_BACKGROUND_VAR);
/// The background of a selected chip.
pub static CHIP_BACKGROUND_VAR: ThemeVar<Color> =
    ThemeVar::alias("combobox.chip_background", &color::SELECTION_VAR);
/// [`CHIP_BACKGROUND_VAR`] as a color for a style.
pub const CHIP_BACKGROUND: Color = Color::var(&CHIP_BACKGROUND_VAR);
/// The focus color of the combobox.
pub static FOCUS_VAR: ThemeVar<Color> = ThemeVar::alias("combobox.focus", &color::FOCUS_VAR);
/// [`FOCUS_VAR`] as a color for a style.
pub const FOCUS: Color = Color::var(&FOCUS_VAR);
/// The corner radius of the field and the list, in logical px.
pub static FIELD_RADIUS_VAR: ThemeVar<f64> =
    ThemeVar::alias("combobox.field_radius", &radius::MD_VAR);
/// [`FIELD_RADIUS_VAR`] as a length for a style.
pub const FIELD_RADIUS: Length = Length::var(&FIELD_RADIUS_VAR);
/// The width of the focus ring, in logical px.
pub static FOCUS_RING_VAR: ThemeVar<f64> = ThemeVar::new("combobox.focus_ring", 2.0);
/// [`FOCUS_RING_VAR`] as a length for a style.
pub const FOCUS_RING: Length = Length::var(&FOCUS_RING_VAR);
/// The opacity of a disabled combobox.
pub static DISABLED_OPACITY_VAR: ThemeVar<f64> =
    ThemeVar::alias("combobox.disabled_opacity", &control::DISABLED_OPACITY_VAR);
/// The font size of the empty message, in logical px.
pub static EMPTY_FONT_SIZE_VAR: ThemeVar<f64> =
    ThemeVar::alias("combobox.empty_font_size", &font::SIZE_SMALL_VAR);
/// [`EMPTY_FONT_SIZE_VAR`] as a length for a style.
pub const EMPTY_FONT_SIZE: Length = Length::var(&EMPTY_FONT_SIZE_VAR);
/// The height of the field, in logical px.
pub static FIELD_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("combobox.field_height", 32.0);
/// [`FIELD_HEIGHT_VAR`] as a length for a style.
pub const FIELD_HEIGHT: Length = Length::var(&FIELD_HEIGHT_VAR);
/// The padding of the field, in logical px.
pub static FIELD_PADDING_VAR: ThemeVar<f64> = ThemeVar::new("combobox.field_padding", 4.0);
/// [`FIELD_PADDING_VAR`] as a length for a style.
pub const FIELD_PADDING: Length = Length::var(&FIELD_PADDING_VAR);
/// The height of a chip, in logical px.
pub static CHIP_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("combobox.chip_height", 24.0);
/// [`CHIP_HEIGHT_VAR`] as a length for a style.
pub const CHIP_HEIGHT: Length = Length::var(&CHIP_HEIGHT_VAR);
/// The corner radius of a chip, in logical px.
pub static CHIP_RADIUS_VAR: ThemeVar<f64> = ThemeVar::new("combobox.chip_radius", 4.0);
/// [`CHIP_RADIUS_VAR`] as a length for a style.
pub const CHIP_RADIUS: Length = Length::var(&CHIP_RADIUS_VAR);
/// The height of an option, in logical px.
pub static OPTION_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("combobox.option_height", 32.0);
/// [`OPTION_HEIGHT_VAR`] as a length for a style.
pub const OPTION_HEIGHT: Length = Length::var(&OPTION_HEIGHT_VAR);
/// The horizontal padding of an option, in logical px.
pub static OPTION_PADDING_X_VAR: ThemeVar<f64> = ThemeVar::new("combobox.option_padding_x", 8.0);
/// [`OPTION_PADDING_X_VAR`] as a length for a style.
pub const OPTION_PADDING_X: Length = Length::var(&OPTION_PADDING_X_VAR);
/// The vertical padding of an option, in logical px.
pub static OPTION_PADDING_Y_VAR: ThemeVar<f64> = ThemeVar::new("combobox.option_padding_y", 6.0);
/// [`OPTION_PADDING_Y_VAR`] as a length for a style.
pub const OPTION_PADDING_Y: Length = Length::var(&OPTION_PADDING_Y_VAR);
/// The corner radius of an option, in logical px.
pub static OPTION_RADIUS_VAR: ThemeVar<f64> = ThemeVar::new("combobox.option_radius", 4.0);
/// [`OPTION_RADIUS_VAR`] as a length for a style.
pub const OPTION_RADIUS: Length = Length::var(&OPTION_RADIUS_VAR);
/// The maximum height of the list, in logical px.
pub static LIST_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("combobox.list_height", 292.0);
/// [`LIST_HEIGHT_VAR`] as a length for a style.
pub const LIST_HEIGHT: Length = Length::var(&LIST_HEIGHT_VAR);
/// The distance between the field and the list, in logical px.
pub static POPUP_OFFSET_VAR: ThemeVar<f64> = ThemeVar::new("combobox.popup_offset", 6.0);
/// [`POPUP_OFFSET_VAR`] as a length for a style.
pub const POPUP_OFFSET: Length = Length::var(&POPUP_OFFSET_VAR);

pub(crate) fn frame(t: &Theme, state: &Look<PartState>) -> Style {
    let s = style()
        .flex_wrap(true)
        .align_items(Align::Center)
        .gap(4)
        .min_height(FIELD_HEIGHT)
        .padding(FIELD_PADDING)
        .border(1, BORDER)
        .border_radius(FIELD_RADIUS)
        .background(BACKGROUND)
        .color(FOREGROUND)
        .disabled(style().opacity(t.get(&DISABLED_OPACITY_VAR)));
    if state.is_focused() {
        s.border_color(FOCUS).outline(t.get(&FOCUS_RING_VAR), FOCUS)
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
pub(crate) fn surface(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding(4)
        .border(1, BORDER)
        .border_radius(FIELD_RADIUS)
        .background(BACKGROUND)
        .max_height(LIST_HEIGHT)
}
pub(crate) fn option(t: &Theme, state: &Look<ListItemState>) -> Style {
    let s = style()
        .min_height(OPTION_HEIGHT)
        .padding_x(OPTION_PADDING_X)
        .padding_y(OPTION_PADDING_Y)
        .border_radius(OPTION_RADIUS)
        .color(FOREGROUND)
        .disabled(style().opacity(t.get(&DISABLED_OPACITY_VAR)));
    if state.active {
        s.background(ACTIVE_BACKGROUND)
    } else {
        s
    }
}
pub(crate) fn chip(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .height(CHIP_HEIGHT)
        .padding_left(8)
        .padding_right(4)
        .gap(4)
        .align_items(Align::Center)
        .border_radius(CHIP_RADIUS)
        .background(CHIP_BACKGROUND)
        .color(FOREGROUND)
}
pub(crate) fn remove(_: &Theme, _: &Look<PartState>) -> Style {
    style().size(16, 16).padding(0)
}
pub(crate) fn empty(_: &Theme, _: &Look<PartState>) -> Style {
    style().padding(8).font_size(EMPTY_FONT_SIZE).color(MUTED)
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
