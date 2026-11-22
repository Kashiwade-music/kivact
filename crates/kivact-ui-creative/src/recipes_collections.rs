//! Role: Creative recipes for the standard list, table, item list and toggle group.
//!
//! The dense table of the editors: 30 px headers on the header surface,
//! rows of the kit's row height with an edge in the row's own color,
//! 9 px cell padding and small type. Drawn by `kivact::ui::list` and
//! `kivact::ui::data_table`.

use crate::color;
use crate::recipes_controls::toggle_button_look;
use crate::size::{ROW, ROW_VAR};
use kivact::custom::PartState;
use kivact::interaction::{ListItemState, TableHeaderState};
use kivact::style::{Style, Theme, style};
use kivact::ui::theme::{
    DataTableCellState, DataTableRowState, DataTableTheme, ItemListTheme, ItemRowState, ListTheme,
    Look, TableTheme, ToggleGroupItemState, ToggleGroupTheme,
};

/// The theme of a list: dense rows with the selection tint.
pub(crate) fn list() -> ListTheme {
    ListTheme::default().item(list_item)
}

fn list_item(_t: &Theme, _s: &Look<ListItemState>, base: Style) -> Style {
    base.min_height(ROW)
        .padding_x(9)
        .font_size(11.5)
        .color(color::TEXT)
        .selected(style().background(color::SELECTION))
}

/// The theme of the editor's data table: a dense header and rows.
pub(crate) fn data_table() -> DataTableTheme {
    DataTableTheme::default()
        .frame(data_table_frame)
        .header(data_table_header)
        .row(data_table_row)
        .cell(data_table_cell)
}

fn data_table_frame(_t: &Theme, _s: &Look<PartState>, base: Style) -> Style {
    base.border_radius(0).border_color(color::HAIR)
}

fn data_table_header(_t: &Theme, _s: &Look<TableHeaderState>, base: Style) -> Style {
    base.min_height(30)
        .padding_x(9)
        .font_size(10.5)
        .color(color::DIM)
        .background(color::HEADER)
        .border_color(color::HAIR)
}

fn data_table_row(t: &Theme, s: &Look<DataTableRowState>, base: Style) -> Style {
    base.min_height(s.height.unwrap_or(t.get(&ROW_VAR) + 4.0))
        .color(color::TEXT)
        .border_color(color::HAIR)
        .border_left(3, s.color.unwrap_or(color::HAIR))
        .hover(style().background(kivact::style::color::HOVER))
        .selected(style().background(color::SELECTION))
}

fn data_table_cell(_t: &Theme, _s: &Look<DataTableCellState>, base: Style) -> Style {
    base.padding_x(9).font_size(11.5)
}

/// The theme of a static table: the editor table's dense header and cells.
pub(crate) fn table() -> TableTheme {
    TableTheme::default()
        .header(table_header)
        .head(table_head)
        .cell(table_cell)
        .row(table_row)
}

fn table_header(_t: &Theme, _s: &Look<PartState>, base: Style) -> Style {
    base.background(color::HEADER)
}

fn table_head(_t: &Theme, _s: &Look<PartState>, base: Style) -> Style {
    base.font_size(10.5).color(color::DIM)
}

fn table_cell(_t: &Theme, _s: &Look<PartState>, base: Style) -> Style {
    base.padding_x(9).font_size(11.5)
}

fn table_row(_t: &Theme, _s: &Look<PartState>, base: Style) -> Style {
    base.min_height(ROW)
        .hover(style().background(kivact::style::color::HOVER))
        .selected(style().background(color::SELECTION))
}

/// The theme of an item list: dense rows with small descriptions.
pub(crate) fn item_list() -> ItemListTheme {
    ItemListTheme::default()
        .item(item_list_item)
        .title(item_list_title)
        .description(item_list_description)
}

fn item_list_item(_t: &Theme, _s: &Look<ItemRowState>, base: Style) -> Style {
    base.min_height(ROW)
        .padding_x(9)
        .border_radius(5)
        .hover(style().background(kivact::style::color::HOVER))
        .selected(style().background(color::SELECTION))
}

fn item_list_title(_t: &Theme, _s: &Look<ItemRowState>, base: Style) -> Style {
    base.font_size(11.5).color(color::TEXT)
}

fn item_list_description(_t: &Theme, _s: &Look<ItemRowState>, base: Style) -> Style {
    base.font_size(10.5).color(color::MUTED)
}

/// The theme of a toggle group: each item draws as a toggle button.
pub(crate) fn toggle_group() -> ToggleGroupTheme {
    ToggleGroupTheme::default().item(toggle_group_item)
}

fn toggle_group_item(t: &Theme, s: &Look<ToggleGroupItemState>, _base: Style) -> Style {
    toggle_button_look(t, &Look::new(s.toggle))
}

/// Install the collections' recipes into `theme`.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(toggle_group())
        .with(table())
        .with(item_list())
        .with(list())
        .with(data_table())
}
