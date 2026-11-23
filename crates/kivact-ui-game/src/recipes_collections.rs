//! Role: Draw the console's lists, data tables, trees, item lists and action groups.
//!
//! The console's table: mono type, 34 px headers on the alternate surface,
//! 160 px columns that share the width beyond that, a cyan-tinted row on
//! hover and an accent edge on a selected row.
use crate::color::{LINE_STRONG, PRIMARY_WEAK};
use crate::fonts::MONO_FAMILY_VAR;
use crate::size::ROW_HEIGHT;
use kivact::custom::PartState;
use kivact::interaction::{ListItemState, TableHeaderState};
use kivact::style::color::{ACCENT, ACCENT_TEXT, BORDER, SURFACE, SURFACE_ALT, TEXT_MUTED};
use kivact::style::{Style, Theme, auto, style};
use kivact::ui::theme::{
    ButtonGroupState, ButtonGroupTheme, DataTableCellState, DataTableRowState, DataTableTheme,
    ItemListTheme, ItemRowState, ListTheme, Look, TableTheme, ToggleGroupItemState,
    ToggleGroupState, ToggleGroupTheme, ToolbarState, ToolbarTheme, TreeRowRecipeState, TreeTheme,
};

/// Install the list, data table, tree, table, item list, toolbar and group recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(ListTheme::default().item(list_item))
        .with(
            DataTableTheme::default()
                .frame(table_frame)
                .header(table_header)
                .row(table_row)
                .cell(table_cell),
        )
        .with(
            TreeTheme::default()
                .root(tree_root)
                .item(tree_item)
                .marker(tree_marker),
        )
        .with(
            TableTheme::default()
                .root(plain_table_root)
                .header(plain_table_header)
                .row(plain_table_row)
                .head(plain_table_head)
                .cell(plain_table_cell),
        )
        .with(
            ItemListTheme::default()
                .item(item_row)
                .title(item_title)
                .description(item_description),
        )
        .with(ToolbarTheme::default().root(toolbar_root))
        .with(
            ButtonGroupTheme::default()
                .root(button_group_root)
                .joined(false),
        )
        .with(
            ToggleGroupTheme::default()
                .root(toggle_group_root)
                .item(toggle_group_item),
        )
}

fn list_item(_: &Theme, _: &Look<ListItemState>, base: Style) -> Style {
    base.min_height(ROW_HEIGHT)
        .padding_x(12)
        .border_radius(0)
        .hover(style().background(PRIMARY_WEAK))
        .selected(style().background(PRIMARY_WEAK).border_left(3, ACCENT))
}

fn table_frame(_: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.border_radius(0).background(SURFACE)
}

fn table_header(t: &Theme, _: &Look<TableHeaderState>, base: Style) -> Style {
    base.width(160)
        .min_width(160)
        .min_height(34)
        .padding_x(12)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(11.5)
        .background(SURFACE_ALT)
}

fn table_row(_: &Theme, s: &Look<DataTableRowState>, base: Style) -> Style {
    base.group()
        .min_height(s.height.map_or(ROW_HEIGHT, kivact::style::Length::from))
        .hover(style().background(PRIMARY_WEAK))
        .selected(style().background(PRIMARY_WEAK).border_left(3, ACCENT))
}

fn table_cell(t: &Theme, _: &Look<DataTableCellState>, base: Style) -> Style {
    base.width(160)
        .min_width(160)
        .padding_x(12)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(11.5)
}

fn tree_root(_: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.padding_y(4)
}

fn tree_item(_: &Theme, _: &Look<TreeRowRecipeState>, base: Style) -> Style {
    base.height(26)
        .min_height(26)
        .hover(style().background(SURFACE_ALT))
        .selected(style().background(ACCENT).color(ACCENT_TEXT))
}

fn tree_marker(_: &Theme, _: &Look<TreeRowRecipeState>, base: Style) -> Style {
    base.font_size(11).color(TEXT_MUTED)
}

fn plain_table_root(t: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
}

fn plain_table_header(_: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.background(SURFACE_ALT)
}

fn plain_table_row(_: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.min_height(ROW_HEIGHT)
        .hover(style().background(PRIMARY_WEAK))
}

fn plain_table_head(_: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.padding_x(12).font_size(11.5)
}

fn plain_table_cell(_: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.padding_x(12).font_size(11.5)
}

fn item_row(_: &Theme, _: &Look<ItemRowState>, base: Style) -> Style {
    base.min_height(ROW_HEIGHT)
        .border_bottom(1, LINE_STRONG)
        .hover(style().background(PRIMARY_WEAK))
}

fn item_title(_: &Theme, _: &Look<ItemRowState>, base: Style) -> Style {
    base.font_size(12.5).font_weight(700)
}

fn item_description(_: &Theme, _: &Look<ItemRowState>, base: Style) -> Style {
    base.font_size(11.5)
}

fn toolbar_root(_: &Theme, _: &Look<ToolbarState>, base: Style) -> Style {
    base.width(auto()).border(1, BORDER)
}

fn button_group_root(_: &Theme, _: &Look<ButtonGroupState>, base: Style) -> Style {
    base.align_items(kivact::style::Align::Center)
}

fn toggle_group_root(_: &Theme, _: &Look<ToggleGroupState>, base: Style) -> Style {
    base.align_items(kivact::style::Align::Center)
}

fn toggle_group_item(_: &Theme, _: &Look<ToggleGroupItemState>, base: Style) -> Style {
    base.border_radius(0).border_color(LINE_STRONG)
}
