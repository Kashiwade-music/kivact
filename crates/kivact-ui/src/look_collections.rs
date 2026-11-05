//! Role: Default recipes for the list and the data table.
use super::values::{
    DataTableCellState, DataTableCheckState, DataTablePageState, DataTableRowState,
};
use crate::data_table::group_state::DataTableGroupState;
use kivact_core::custom::PartState;
use kivact_core::draw::{LineCap, LineJoin, Stroke};
use kivact_core::geometry::Point;
use kivact_core::style::{
    Align, ColorToken, Direction, Justify, Overflow, Style, Theme, pct, style,
};
use kivact_core::view::{ReadingDirection, use_direction};
use kivact_interaction::{ListItemState, TableHeaderState, TreeItemState};

pub(super) fn group_header(_: &Theme, _: &DataTableGroupState) -> Style {
    style()
        .direction(Direction::Row)
        .background(ColorToken::SurfaceAlt)
        .border_bottom(1, ColorToken::Border)
}

/// The items in a column.
pub(super) fn list(t: &Theme, _: &PartState) -> Style {
    style().direction(Direction::Column).gap(t.space().xs)
}

pub(super) fn tree_item(t: &Theme, row: &TreeItemState) -> Style {
    let rtl = use_direction().get() == ReadingDirection::Rtl;
    let tokens = t.ext::<crate::UiTokens>();
    let tree = &tokens.tree;
    let padding = tree.padding.unwrap_or(t.space().md);
    let indent = padding + row.level.saturating_sub(1) as f64 * tree.indent.unwrap_or(16.0);
    style()
        .enter_size(kivact_core::view::Orientation::Vertical)
        .exit_size(kivact_core::view::Orientation::Vertical)
        .direction(if rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Center)
        .min_height(tree.row_height.unwrap_or(t.control_height()))
        .padding_left(if rtl { padding } else { indent })
        .padding_right(if rtl { indent } else { padding })
        .gap(tree.gap.unwrap_or(t.space().xs))
        .radius(tree.radius.unwrap_or(t.radius().sm))
        .hover(style().background(ColorToken::Hover))
        .selected(style().background(ColorToken::Selection))
}

/// An item: its icon beside its label, the whole row marked while it is
/// selected.
pub(super) fn list_item(t: &Theme, _: &ListItemState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .min_height(t.control_height())
        .padding_x(t.space().md)
        .radius(t.radius().sm)
        .color(ColorToken::Text)
        .hover(style().background(ColorToken::Hover))
        .selected(style().background(ColorToken::Selection))
}

/// The secondary line of an item with a description.
pub(super) fn list_description(t: &Theme, _: &ListItemState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

/// The frame around the toolbar, the grid and the footer.
pub(super) fn frame(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Column)
        .width(pct(100))
        .min_width(0)
        // The table keeps the height of its rows in a bounded parent; the
        // parent's own scroll view carries them.
        .shrink(0)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .overflow(Overflow::Hidden)
}

/// The row above the grid.
pub(super) fn toolbar(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .padding(t.space().md)
}

/// The row under the grid.
pub(super) fn footer(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .padding(t.space().md)
}

/// A column header: its title and sort mark, sharing the width with the
/// column's cells.
pub(super) fn header(t: &Theme, state: &TableHeaderState) -> Style {
    let s = style()
        .direction(Direction::Row)
        .grow(1)
        .basis(0)
        .shrink(0)
        .min_width(0)
        .min_height(t.control_height())
        .padding_x(t.space().md)
        .align_items(Align::Center)
        .gap(t.space().sm)
        .weight(t.text().weight_strong)
        .color(ColorToken::TextMuted)
        .border_bottom(1, ColorToken::Border);
    if state.sortable {
        s.hover(style().background(ColorToken::Hover))
    } else {
        s
    }
}

/// A body row, marked while it is hovered or selected; its main row
/// height and accent are the caller's when given.
pub(super) fn row(t: &Theme, state: &DataTableRowState) -> Style {
    let s = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_height(state.height.unwrap_or(t.control_height()))
        .border_bottom(1, ColorToken::Border)
        .hover(style().background(ColorToken::Hover))
        .selected(style().background(ColorToken::Selection));
    let s = match state.height {
        Some(height) => s.height(height),
        None => s,
    };
    match state.color {
        Some(color) => s.border_left(3, color),
        None => s,
    }
}

/// A body cell, sharing the width with the column's header.
pub(super) fn cell(t: &Theme, state: &DataTableCellState) -> Style {
    let s = style()
        .direction(Direction::Row)
        .grow(1)
        .basis(0)
        .shrink(0)
        .min_width(0)
        .padding_x(t.space().md)
        .align_items(Align::Center);
    if state.numeric {
        s.justify_content(Justify::End)
    } else {
        s
    }
}

/// The box around a check box, as wide as the check column.
pub(super) fn check(t: &Theme, _: &DataTableCheckState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .shrink(0)
        .width(40)
        .padding_x(t.space().md)
}

/// The sort mark, drawn in the box: an arrow up or down by the direction
/// the column is sorted in, and nothing for a column that is not sorted or
/// does not sort.
pub(super) fn sort(_: &Theme, state: &TableHeaderState) -> Style {
    let order = state.sort;
    let s = style().size(12, 12).shrink(0);
    if !state.sortable {
        return style().size(0, 0).shrink(0);
    }
    s.paint(move |d, size| {
        use kivact_interaction::SortOrder;
        let (w, h) = (size.width, size.height);
        let at = |x: f64, y: f64| Point::new(w * x, h * y);
        let stroke = Stroke::new(1.5).cap(LineCap::Round).join(LineJoin::Round);
        let up = [at(0.2, 0.62), at(0.5, 0.3), at(0.8, 0.62)];
        let down = [at(0.2, 0.38), at(0.5, 0.7), at(0.8, 0.38)];
        match order {
            Some(SortOrder::Ascending) => d.polyline(up, &stroke, ColorToken::Text),
            Some(SortOrder::Descending) => d.polyline(down, &stroke, ColorToken::Text),
            _ => {
                let faint = kivact_core::style::Color::token(ColorToken::TextMuted).with_alpha(0.6);
                d.polyline(
                    [at(0.2, 0.42), at(0.5, 0.15), at(0.8, 0.42)],
                    &stroke,
                    faint,
                );
                d.polyline(
                    [at(0.2, 0.58), at(0.5, 0.85), at(0.8, 0.58)],
                    &stroke,
                    faint,
                );
            }
        }
    })
}

/// The pager: its summary and buttons in a row.
pub(super) fn page(t: &Theme, _: &DataTablePageState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().sm)
}

pub(super) fn tree_marker(t: &Theme, _: &TreeItemState) -> Style {
    style()
        .width(t.ext::<crate::UiTokens>().tree.marker_width.unwrap_or(16.0))
        .shrink(0)
}
