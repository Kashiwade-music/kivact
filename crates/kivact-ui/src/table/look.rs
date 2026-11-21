//! Role: The table recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, FlexDirection, Style, Theme, pct, style};
use kivact_core::style::{Length, ThemeVar, control, font, space};

/// The padding of a table cell, in logical px.
pub static CELL_PADDING_VAR: ThemeVar<f64> = ThemeVar::alias("table.cell_padding", &space::SM_VAR);
/// [`CELL_PADDING_VAR`] as a length for a style.
pub const CELL_PADDING: Length = Length::var(&CELL_PADDING_VAR);

/// The height of a table header, in logical px.
pub static HEADER_HEIGHT_VAR: ThemeVar<f64> =
    ThemeVar::alias("table.header_height", &control::HEIGHT_VAR);
/// [`HEADER_HEIGHT_VAR`] as a length for a style.
pub const HEADER_HEIGHT: Length = Length::var(&HEADER_HEIGHT_VAR);

/// The font size of a table, in logical px.
pub static FONT_SIZE_VAR: ThemeVar<f64> = ThemeVar::alias("table.font_size", &font::SIZE_SMALL_VAR);
/// [`FONT_SIZE_VAR`] as a length for a style.
pub const FONT_SIZE: Length = Length::var(&FONT_SIZE_VAR);

pub(crate) fn table(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .min_width(0)
        .font_size(FONT_SIZE)
}

pub(crate) fn group(_: &Theme, _: &Look<PartState>) -> Style {
    style().flex_direction(FlexDirection::Column)
}

pub(crate) fn footer(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .background(kivact_core::style::color::SURFACE_ALT)
}

pub(crate) fn row(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .width(pct(100))
        .align_items(Align::Center)
        .border_bottom(1, kivact_core::style::color::BORDER)
        .selected(style().background(kivact_core::style::color::SELECTION))
}

pub(crate) fn cell(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .min_width(0)
        .padding(CELL_PADDING)
        .color(kivact_core::style::color::TEXT)
}

pub(crate) fn head(t: &Theme, s: &Look<PartState>) -> Style {
    cell(t, s).min_height(HEADER_HEIGHT).font_weight(600)
}

recipe_table! {
    /// Static table part recipes.
    TableTheme {
        root(root_recipe): PartState = table,
        header(header_recipe): PartState = group,
        body(body_recipe): PartState = group,
        footer(footer_recipe): PartState = footer,
        row(row_recipe): PartState = row,
        head(head_recipe): PartState = head,
        cell(cell_recipe): PartState = cell,
    }
}
