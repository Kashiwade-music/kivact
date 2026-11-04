//! Role: The Lyra table: equal-width cells, rules, header, body and footer.

use crate::Color;
use crate::theme_access::tokens;
use kivact::state::Value;
use kivact::view::{Node, NodeBuilder, column, node, row, text};

use kivact::state::{IntoText, IntoValue};
use kivact::style::{Align, Direction, Justify, Length, Overflow, pct, style};
use kivact::view::{IntoView, Role, View};

/// The horizontal alignment of a [`TableCell`]'s content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TableAlign {
    /// Against the leading edge.
    #[default]
    Start,
    /// Against the trailing edge (numbers, amounts).
    End,
}

/// A table (table.tsx): `Role::Table` with 12 px text.
///
/// Build it from [`table_header`], [`table_body`], [`table_footer`] and
/// [`table_caption`] (put the caption last: it is drawn where it is
/// placed). Rows divide the width equally unless a cell has `.width(..)`;
/// the cells of a column should therefore use the same width in every row.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn invoices() -> impl IntoView {
///     basic::table((
///         basic::table_header(basic::table_row((
///             basic::table_head("Invoice"),
///             basic::table_head("Amount").align(basic::TableAlign::End),
///         ))),
///         basic::table_body(basic::table_row((
///             basic::table_cell("INV001"),
///             basic::table_cell("$250.00").align(basic::TableAlign::End),
///         ))),
///     ))
/// }
/// ```
#[track_caller]
pub fn table(children: impl IntoView) -> impl kivact::view::Component {
    column(children)
        .with_node(|root| root.role(Role::Table))
        .style(
            style()
                .width(pct(100))
                .font_size(12)
                .overflow(Overflow::Hidden),
        )
}

/// A caption under the table, muted, with 16 px above it.
#[track_caller]
pub fn table_caption(text_value: impl IntoText) -> impl kivact::view::Component {
    text(text_value).style(
        style()
            .margin_top(16)
            .color(Color::MutedForeground)
            .text_align(kivact::style::TextAlign::Center),
    )
}

/// The header rows of a [`table`]: a `Role::RowGroup`.
#[track_caller]
pub fn table_header(children: impl IntoView) -> impl kivact::view::Component {
    column(children).with_node(|root| root.role(Role::RowGroup))
}

/// The body rows of a [`table`], a `Role::RowGroup`; the last row has no
/// bottom rule.
#[track_caller]
pub fn table_body(children: impl IntoView) -> impl kivact::view::Component {
    column(children)
        .with_node(|root| root.role(Role::RowGroup))
        .style(style().margin_bottom(-1))
}

/// The footer rows of a [`table`], a `Role::RowGroup`: a rule above, a muted
/// fill and medium text.
#[track_caller]
pub fn table_footer(children: impl IntoView) -> impl kivact::view::Component {
    column(children)
        .with_node(|root| root.role(Role::RowGroup))
        .style(move || {
            let p = tokens();
            style()
                .margin_bottom(-1)
                .border_top(1, p.border)
                .background(p.muted.with_alpha(0.5))
                .weight(500)
        })
}

/// A row of cells (table.tsx `TableRow`). Created by [`table_row`].
pub struct TableRow {
    root: Node,
}

/// A row of [`table_head`] or [`table_cell`] cells.
///
/// It has a rule under it, turns muted on hover and, when
/// [`TableRow::selected`], stays muted.
#[track_caller]
pub fn table_row(children: impl IntoView) -> TableRow {
    TableRow {
        root: row(children)
            .with_node(|root| root.role(Role::Row))
            .style(move || {
                let p = tokens();
                style()
                    .align_items(Align::Center)
                    .width(pct(100))
                    .border_bottom(1, p.border)
                    .hover(style().background(p.muted.with_alpha(0.5)))
                    .selected(style().background(p.muted))
            }),
    }
}

impl TableRow {
    /// Mark the row selected: a `selected` row in the accessibility tree and a muted fill.
    pub fn selected(mut self, selected: impl IntoValue<bool>) -> Self {
        self.root = self.root.selected(selected);
        self
    }
}

impl NodeBuilder for TableRow {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TableRow {
    fn into_view(self) -> View {
        self.root.into_view()
    }
}

/// A header or data cell. Created by [`table_head`] and [`table_cell`].
pub struct TableCell {
    root: Node,
    header: bool,
    content: View,
    width: Option<Value<Length>>,
    align: Value<TableAlign>,
}

/// The shared root of header and data cells.
#[track_caller]
fn cell(role: Role, header: bool, content: View) -> TableCell {
    TableCell {
        root: node().with_node(|root| root.role(role)),
        header,
        content,
        width: None,
        align: TableAlign::Start.into_value(),
    }
}

/// A column header cell: 40 px high, medium text, `Role::ColumnHeader`.
#[track_caller]
pub fn table_head(content: impl IntoView) -> TableCell {
    cell(Role::ColumnHeader, true, content.into_view())
}

/// A data cell with 8 px padding, `Role::Cell`.
#[track_caller]
pub fn table_cell(content: impl IntoView) -> TableCell {
    cell(Role::Cell, false, content.into_view())
}

impl TableCell {
    /// Give the cell a fixed `width` (constant or reactive) instead of an
    /// equal share.
    #[track_caller]
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        self.width = Some(width.into_value());
        self
    }

    /// Align the content.
    pub fn align(mut self, align: impl IntoValue<TableAlign>) -> Self {
        self.align = align.into_value();
        self
    }
}

impl NodeBuilder for TableCell {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TableCell {
    fn into_view(self) -> View {
        let (header, width, align) = (self.header, self.width, self.align);
        self.root
            .base_style(move || {
                let p = tokens();
                let s = style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .justify_content(match align.get() {
                        TableAlign::Start => Justify::Start,
                        TableAlign::End => Justify::End,
                    })
                    .padding_x(8)
                    .min_width(0)
                    .text_wrap(false)
                    .color(p.foreground);
                let s = if header {
                    s.height(40).weight(500)
                } else {
                    s.padding_y(8)
                };
                match width {
                    Some(w) => s.width(w.get()).shrink(0),
                    None => s.grow(1).basis(0).shrink(1),
                }
            })
            .child(self.content)
            .into_view()
    }
}

kivact::state::constant_value!(TableAlign);
