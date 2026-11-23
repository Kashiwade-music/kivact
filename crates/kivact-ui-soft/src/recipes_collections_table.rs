//! Role: Install Astryx Neutral's recipes of the table group: the static table's rows and
//! cells, and the data table's frame, header, rows, cells and sort mark.
use crate::color as c;
use crate::metrics as m;
use crate::recipe_parts::{body_text, fast, ring};
use crate::space as s;
use crate::type_scale as t;
use crate::variants::{
    ItemDensity, TableCellAlign, TableCellDivider, TableRowDividers, TableRowHover, TableRowStriped,
};
use kivact::custom::PartState;
use kivact::draw::{LineCap, Stroke};
use kivact::geometry::Point;
use kivact::interaction::{HeaderSort, SortOrder, TableHeaderState};
use kivact::style::{Align, Cursor, FlexDirection, Justify, Style, Theme, pct, style};
use kivact::ui::DataTableDensity;
use kivact::ui::theme::{
    DataTableCellState, DataTableCheckState, DataTableGroupState, DataTableRowState,
    DataTableTheme, Look, TableTheme,
};

/// The column of a static table: full width, its header, body and footer stacked.
fn table_root(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .min_width(0)
}

/// A header or body group of a static table: a column of rows.
fn table_group(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().flex_direction(FlexDirection::Column)
}

/// The footer of a static table: a column on the muted background.
fn table_footer(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .background(c::BG_MUTED)
}

/// A row of a static table: its divider, its hover fill and its stripe, as its variants ask.
fn static_row(_: &Theme, look: &Look<PartState>, _base: Style) -> Style {
    let mut row = style()
        .flex_direction(FlexDirection::Row)
        .width(pct(100))
        .min_width(0)
        .selected(style().background(c::ACCENT_MUTED));
    if look.variant::<TableRowDividers>().is_none_or(|v| v.0) {
        row = row.border_bottom(m::BORDER_WIDTH, c::BORDER);
    }
    if look.variant::<TableRowHover>().is_some_and(|v| v.0) {
        row = row.hover(style().background(c::OVERLAY_HOVER));
    }
    if look.variant::<TableRowStriped>().is_some_and(|v| v.0) {
        row = row.background(c::BG_MUTED);
    }
    row
}

/// A cell of a static table: the body role, its density, its vertical alignment and its divider.
fn static_cell(_: &Theme, look: &Look<PartState>, _base: Style) -> Style {
    let padding = look.variant::<ItemDensity>().unwrap_or_default().padding();
    let align = look
        .variant::<TableCellAlign>()
        .map_or(Align::Center, |v| v.0);
    let cell = body_text(style())
        .flex_direction(FlexDirection::Row)
        .align_items(align)
        .min_width(0)
        .padding_y(padding)
        .padding_x(padding + 4);
    if look.variant::<TableCellDivider>().is_some_and(|v| v.0) {
        cell.border_right(m::BORDER_WIDTH, c::BORDER)
    } else {
        cell
    }
}

/// A header cell of a static table: the cell in the semibold weight.
fn static_head(theme: &Theme, look: &Look<PartState>, base: Style) -> Style {
    static_cell(theme, look, base).font_weight(t::WEIGHT_SEMIBOLD)
}

/// The frame of a data table: a full-width column.
fn data_frame(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .min_width(0)
        .flex_shrink(0.0)
}

/// The toolbar of a data table: its items with the spacing step between them.
fn data_toolbar(_: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.gap(s::SPACE_S2).padding(0).padding_bottom(s::SPACE_S2)
}

/// The padding of a data table's cell for its density: the inline and block steps.
fn density_cell(density: DataTableDensity) -> Style {
    let (inline, block) = match density {
        DataTableDensity::Compact => (s::SPACE_S2, s::SPACE_S1),
        DataTableDensity::Balanced => (s::SPACE_S3, s::SPACE_S2),
        DataTableDensity::Spacious => (s::SPACE_S4, s::SPACE_S3),
    };
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .flex_grow(1.0)
        .flex_basis(0.0)
        .flex_shrink(0.0)
        .min_width(0)
        .padding_x(inline)
        .padding_y(block)
        .font_size(t::BODY.size_px)
        .line_height(1.5)
}

/// A column header of a data table: the density's cell, sorted or not, with the focus ring.
fn data_header(_: &Theme, look: &Look<TableHeaderState>, _base: Style) -> Style {
    let sortable = !matches!(look.sorting, HeaderSort::Unsortable);
    density_cell(look.variant::<DataTableDensity>().unwrap_or_default())
        .gap(s::SPACE_S1)
        .font_weight(t::WEIGHT_SEMIBOLD)
        .color(c::TEXT_SECONDARY)
        .border_bottom(m::BORDER_WIDTH, c::BORDER)
        .cursor(if sortable {
            Cursor::Pointer
        } else {
            Cursor::Default
        })
        .focus_visible(ring(c::FOCUS_RING))
}

/// The sort mark of a column: two arrows when unsorted, one when sorted, drawn in the box.
fn sort_mark(_: &Theme, look: &Look<TableHeaderState>, _base: Style) -> Style {
    if matches!(look.sorting, HeaderSort::Unsortable) {
        return style().size(0, 0).flex_shrink(0.0);
    }
    let order = match look.sorting {
        HeaderSort::Sorted { direction, .. } => Some(direction),
        HeaderSort::Unsortable | HeaderSort::Sortable => None,
    };
    let color = if order.is_some() {
        c::ICON_ACCENT
    } else {
        c::ICON_SECONDARY.with_alpha(if look.input.is_hovered() || look.input.is_focused() {
            1.0
        } else {
            0.35
        })
    };
    style()
        .size(s::SPACE_S3, s::SPACE_S3)
        .flex_shrink(0.0)
        .paint(move |d, size| {
            let at = |x, y| Point::new(size.width * x, size.height * y);
            let stroke = Stroke::new(size.width * 0.08).cap(LineCap::Round);
            let mut arrow = |x, tip, tail, wing| {
                d.polyline([at(x, tail), at(x, tip)], &stroke, color);
                d.polyline(
                    [at(x - 0.2, wing), at(x, tip), at(x + 0.2, wing)],
                    &stroke,
                    color,
                );
            };
            match order {
                Some(SortOrder::Ascending) => arrow(0.5, 0.15, 0.85, 0.4),
                Some(SortOrder::Descending) => arrow(0.5, 0.85, 0.15, 0.6),
                None => {
                    arrow(0.25, 0.15, 0.85, 0.4);
                    arrow(0.75, 0.85, 0.15, 0.6);
                }
            }
        })
}

/// A body row of a data table: its minimum height, its divider, its hover and selected fills, and its transition.
fn data_row(_: &Theme, look: &Look<DataTableRowState>, base: Style) -> Style {
    fast(
        base.min_height(look.height.unwrap_or(0.0))
            .border_bottom(m::BORDER_WIDTH, c::BORDER)
            .hover(style().background(c::OVERLAY_HOVER))
            .selected(style().background(c::ACCENT_MUTED)),
    )
}

/// A group header of a data table: the muted row with its divider and the body size.
fn group_header(_: &Theme, _: &Look<DataTableGroupState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .background(c::BG_MUTED)
        .border_bottom(m::BORDER_WIDTH, c::BORDER)
        .font_size(t::BODY.size_px)
}

/// A body cell of a data table: the density's cell, aligned to the end when it is numeric.
fn data_cell(_: &Theme, look: &Look<DataTableCellState>, _base: Style) -> Style {
    density_cell(look.variant::<DataTableDensity>().unwrap_or_default())
        .justify_content(if look.numeric {
            Justify::End
        } else {
            Justify::Start
        })
        .color(c::TEXT_PRIMARY)
        .focus_visible(ring(c::FOCUS_RING))
}

/// The box around a data table's check box: a fixed-width, centered column.
fn data_check(_: &Theme, _: &Look<DataTableCheckState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .width(36)
        .flex_shrink(0.0)
}

/// Install the kit's recipes of the table group: the static table and the data table.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme.with(
        TableTheme::default()
            .root(table_root)
            .header(table_group)
            .body(table_group)
            .footer(table_footer)
            .row(static_row)
            .head(static_head)
            .cell(static_cell),
    );
    theme.with(
        DataTableTheme::default()
            .frame(data_frame)
            .toolbar(data_toolbar)
            .header(data_header)
            .sort(sort_mark)
            .row(data_row)
            .group_header(group_header)
            .cell(data_cell)
            .check(data_check),
    )
}
