//! Role: The Pop looks of the collections: lists, data tables, item lists, tables, trees and action groups.
//!
//! A rounded surface with 40 px headers, 52 px rows and 160 px columns that
//! scroll sideways, muted on hover and selection, drawn by `kivact::ui`.

use crate::color;
use crate::variants::Hue;
use kivact::custom::PartState;
use kivact::interaction::{ListItemState, TableHeaderState};
use kivact::style::{Align, FlexDirection, Justify, Style, Theme, style};
use kivact::ui::theme::{
    ButtonGroupState, ButtonGroupTheme, DataTableCellState, DataTableRowState, DataTableTheme,
    ItemListTheme, ItemRowState, ListTheme, Look, TableTheme, ToggleGroupItemState,
    ToggleGroupState, ToggleGroupTheme, ToolbarState, ToolbarTheme, TreeRowRecipeState, TreeTheme,
};

/// Install the kit's recipes of the collections and action groups.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(
            ListTheme::default()
                .item(list_item)
                .description(list_description),
        )
        .with(
            DataTableTheme::default()
                .frame(table_frame)
                .toolbar(table_toolbar)
                .footer(table_footer)
                .header(table_header)
                .row(table_row)
                .cell(table_cell),
        )
        .with(
            ItemListTheme::default()
                .root(item_list_root)
                .item(item_list_item)
                .body(item_list_body)
                .description(item_list_description)
                .media(item_list_media),
        )
        .with(
            TableTheme::default()
                .root(plain_table)
                .header(plain_table_header)
                .head(plain_table_head)
                .cell(plain_table_cell),
        )
        .with(TreeTheme::default().item(tree_item))
        .with(ToolbarTheme::default().root(toolbar_root))
        .with(
            ButtonGroupTheme::default()
                .root(button_group_root)
                .separator(button_group_separator)
                .text(button_group_text),
        );
    theme.with(
        ToggleGroupTheme::default()
            .root(toggle_group_root)
            .item(toggle_group_item),
    )
}

fn list_item(_theme: &Theme, _: &Look<ListItemState>, base: Style) -> Style {
    base.padding_x(12)
        .min_height(40)
        .border_radius(10)
        .font_weight(500)
        .hover(style().background(color::SURFACE_MUTED))
        .selected(style().background(color::SURFACE_ACTIVE))
}

fn list_description(_theme: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style().font_size(13).color(color::TEXT_MUTED)
}

fn table_frame(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.background(color::SURFACE)
        .border_color(color::LINE)
        .border_radius(14)
}

fn table_toolbar(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.padding(16).gap(8).border_bottom(1, color::LINE)
}

fn table_footer(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.padding_x(16).padding_y(12).gap(4)
}

fn table_header(_theme: &Theme, _: &Look<TableHeaderState>, base: Style) -> Style {
    base.width(160)
        .min_width(160)
        .min_height(40)
        .padding_x(12)
        .font_size(13)
        .font_weight(500)
        .color(color::TEXT_MUTED)
        .border_color(color::LINE)
}

fn table_row(_theme: &Theme, s: &Look<DataTableRowState>, base: Style) -> Style {
    base.min_height(s.height.unwrap_or(52.0))
        .border_color(color::LINE)
        .hover(style().background(color::SURFACE_MUTED))
        .selected(style().background(color::SURFACE_MUTED))
}

fn table_cell(_theme: &Theme, _: &Look<DataTableCellState>, base: Style) -> Style {
    base.width(160).min_width(160).padding_x(12)
}

fn item_list_root(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.gap(0)
}

fn item_list_item(_theme: &Theme, _: &Look<ItemRowState>, base: Style) -> Style {
    base.padding_x(8)
        .padding_y(10)
        .gap(0)
        .border_radius(8)
        .hover(style().background(color::SURFACE_ACTIVE))
}

fn item_list_body(_theme: &Theme, _: &Look<ItemRowState>, base: Style) -> Style {
    base.gap(12).align_items(Align::Center)
}

fn item_list_description(_theme: &Theme, _: &Look<ItemRowState>, base: Style) -> Style {
    base.font_size(12).color(color::TEXT_MUTED)
}

/// The leading tile of an item: the hue's tint and text color, gray unless the row has a hue.
fn item_list_media(_theme: &Theme, s: &Look<ItemRowState>, _base: Style) -> Style {
    let hue = s.variant::<Hue>().unwrap_or(Hue::Gray);
    let (bg, fg) = hue.colors();
    style()
        .size(36, 36)
        .border_radius(8)
        .background(bg)
        .color(fg)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0)
}

fn plain_table(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.border_radius(14)
        .border(1, color::LINE)
        .background(color::SURFACE)
}

fn plain_table_header(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.background(color::SURFACE_MUTED)
}

fn plain_table_head(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.font_size(12).color(color::TEXT_MUTED)
}

fn plain_table_cell(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.font_size(13).padding_x(12).padding_y(10)
}

fn tree_item(_theme: &Theme, _: &Look<TreeRowRecipeState>, base: Style) -> Style {
    base.border_radius(8)
        .font_size(13)
        .hover(style().background(color::SURFACE_MUTED))
}

fn toolbar_root(_theme: &Theme, _: &Look<ToolbarState>, base: Style) -> Style {
    base.background(color::SURFACE).border_radius(8)
}

fn button_group_root(_theme: &Theme, _: &Look<ButtonGroupState>, base: Style) -> Style {
    base.border_radius(8)
}

fn button_group_separator(_theme: &Theme, _: &Look<ButtonGroupState>, base: Style) -> Style {
    base.background(color::LINE)
}

fn button_group_text(_theme: &Theme, _: &Look<ButtonGroupState>, base: Style) -> Style {
    base.background(color::SURFACE_MUTED).color(color::TEXT)
}

fn toggle_group_root(_theme: &Theme, _: &Look<ToggleGroupState>, base: Style) -> Style {
    base.padding(4)
        .border_radius(8)
        .background(color::SURFACE_MUTED)
}

fn toggle_group_item(_theme: &Theme, _: &Look<ToggleGroupItemState>, base: Style) -> Style {
    base.border_radius(6)
        .hover(style().background(color::SURFACE_ACTIVE))
}
