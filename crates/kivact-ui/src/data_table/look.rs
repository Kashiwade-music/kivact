//! Role: The data table recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::draw::{LineCap, LineJoin, Stroke};
use kivact_core::geometry::Point;
use kivact_core::style::{
    Align, ColorToken, Direction, Justify, Overflow, Style, Theme, pct, style,
};
use kivact_core::view::CheckedState;
use kivact_interaction::{HeaderSort, TableHeaderState};
pub(crate) fn group_header(_: &Theme, _: &DataTableGroupState) -> Style {
    style()
        .direction(Direction::Row)
        .background(ColorToken::SurfaceAlt)
        .border_bottom(1, ColorToken::Border)
}

/// The frame around the toolbar, the grid and the footer.
pub(crate) fn frame(t: &Theme, _: &PartState) -> Style {
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
pub(crate) fn toolbar(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .padding(t.space().md)
}

/// The row under the grid.
pub(crate) fn footer(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .padding(t.space().md)
}

/// A column header: its title and sort mark, sharing the width with the
/// column's cells.
pub(crate) fn header(t: &Theme, state: &TableHeaderState) -> Style {
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
    if matches!(state.sorting, HeaderSort::Unsortable) {
        s
    } else {
        s.hover(style().background(ColorToken::Hover))
    }
}

/// A body row, marked while it is hovered or selected; its main row
/// height and accent are the caller's when given.
pub(crate) fn row(t: &Theme, state: &DataTableRowState) -> Style {
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
pub(crate) fn cell(t: &Theme, state: &DataTableCellState) -> Style {
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
pub(crate) fn check(t: &Theme, _: &DataTableCheckState) -> Style {
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
pub(crate) fn sort(_: &Theme, state: &TableHeaderState) -> Style {
    let order = match state.sorting {
        HeaderSort::Sorted { direction, .. } => Some(direction),
        _ => None,
    };
    let s = style().size(12, 12).shrink(0);
    if matches!(state.sorting, HeaderSort::Unsortable) {
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
pub(crate) fn page(t: &Theme, _: &DataTablePageState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().sm)
}

/// The state of a main row of a [`data_table`](super::data_table) for its
/// recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataTableRowState {
    /// The actual state of the row's attached table part: its input state,
    /// whether it is selected and whether it holds the tab stop.
    pub interaction: kivact_interaction::TableRowState,
    /// The accent the caller gave the row with
    /// [`DataTable::row_color`](super::DataTable::row_color); the recipe
    /// resolves it in the theme in effect.
    pub color: Option<kivact_core::style::Color>,
    /// The height the caller gave the rows with
    /// [`DataTable::row_height`](super::DataTable::row_height), at least 18.
    pub height: Option<f64>,
}

/// The state of a cell of a [`data_table`](super::data_table) for its
/// recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataTableCellState {
    /// The actual state of the cell's attached table part.
    pub interaction: kivact_interaction::TableCellState,
    /// Whether the column is [`numeric`](super::DataColumn::numeric).
    pub numeric: bool,
}

/// The state of the box around a check box of a
/// [`data_table`](super::data_table), for its recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataTableCheckState {
    /// The header's box, which checks the page's rows; Indeterminate while
    /// some of them are checked.
    Header(CheckedState),
    /// A row's box.
    Row { checked: bool },
}

impl DataTableCheckState {
    /// Whether this is the header's box, which checks the page's rows.
    pub fn is_header(&self) -> bool {
        matches!(self, Self::Header(_))
    }
    /// The box's checked state; only the header's is ever Indeterminate.
    pub fn checked(&self) -> CheckedState {
        match *self {
            Self::Header(checked) => checked,
            Self::Row { checked } => CheckedState::from_bool(checked),
        }
    }
}

/// The page a [`data_table`](super::data_table) shows, for the recipe of its
/// pager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataTablePageState {
    /// The page shown, from 0.
    pub index: usize,
    /// How many pages the filtered rows fill. Zero for a remote page with unknown total.
    pub pages: usize,
}

/// The state a group-header recipe draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataTableGroupState {
    /// Matching rows in this group on the current page, including collapsed rows.
    pub count: usize,
    /// Whether the group is collapsed.
    pub collapsed: bool,
}

recipe_table! {
    /// The recipes of a [`data_table`](super::data_table). What a header, a
    /// cell or the toolbar shows is the table's own or a slot the caller
    /// fills ([`DataTable::toolbar`](super::DataTable::toolbar),
    /// [`footer`](super::DataTable::footer),
    /// [`cell_controls`](super::DataTable::cell_controls)).
    DataTableRecipes {
        /// The frame around the toolbar, the grid and the footer.
        frame(frame_recipe): PartState = frame,
        /// The row above the grid, holding the filter field or the
        /// toolbar slot.
        toolbar(toolbar_recipe): PartState = toolbar,
        /// The row under the grid, holding the summary and the pager or the
        /// footer slot.
        footer(footer_recipe): PartState = footer,
        /// A column header; it knows whether its column sorts and in which
        /// direction. The column's width is the table's.
        header(header_recipe): TableHeaderState = header,
        /// A body row; it knows whether it is selected and the caller's row
        /// height and accent.
        row(row_recipe): DataTableRowState = row,
        /// A group header spanning the visible columns.
        group_header(group_header_recipe): DataTableGroupState = group_header,
        /// A body cell.
        cell(cell_recipe): DataTableCellState = cell,
        /// The box around a check box, in a header or a row.
        check(check_recipe): DataTableCheckState = check,
        /// The sort mark of a header, drawn by the recipe in its own box. A
        /// look that gives the box no size shows no mark.
        sort(sort_recipe): TableHeaderState = sort,
        /// The pager under the grid: Previous, Next and the page shown.
        page(page_recipe): DataTablePageState = page,
    }
}
