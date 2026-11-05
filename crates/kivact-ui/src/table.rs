//! Role: Compose static tables with themed rows and cells.
use crate::layer::with_base_recipe_style;
use crate::variants::{Variant, Variants};
use kivact_core::state::{IntoValue, Value};
use kivact_core::style::{Justify, Length, TextAlign, style};
use kivact_core::view::{IntoChildren, IntoView, Node, NodeBuilder, Role, View, node};

/// Theme overrides for static tables.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TableTokens {
    pub cell_padding: Option<f64>,
    pub header_height: Option<f64>,
    pub font_size: Option<f64>,
}
/// A structural part of a static table.
pub struct Table {
    root: Node,
    part: TablePart,
    variants: Variants,
}
#[derive(Clone, Copy)]
enum TablePart {
    Root,
    Header,
    Body,
    Footer,
    Row,
}
fn section(children: impl IntoChildren, role: Role, part: TablePart) -> Table {
    Table {
        root: node().role(role).child(children.into_views()),
        part,
        variants: Variants::default(),
    }
}
/// A static table; use `data_table` for interactive grids.
#[track_caller]
pub fn table(children: impl IntoChildren) -> Table {
    section(children, Role::Table, TablePart::Root)
}
/// A group of column-header rows.
#[track_caller]
pub fn table_header(children: impl IntoChildren) -> Table {
    section(children, Role::RowGroup, TablePart::Header)
}
/// A group of data rows.
#[track_caller]
pub fn table_body(children: impl IntoChildren) -> Table {
    section(children, Role::RowGroup, TablePart::Body)
}
/// A group of summary rows.
#[track_caller]
pub fn table_footer(children: impl IntoChildren) -> Table {
    section(children, Role::RowGroup, TablePart::Footer)
}
/// A row of header or data cells.
#[track_caller]
pub fn table_row(children: impl IntoChildren) -> Table {
    section(children, Role::Row, TablePart::Row)
}
impl Table {
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for Table {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Table {
    fn into_view(self) -> View {
        let pick: fn(&crate::UiRecipes) -> &crate::Recipe<kivact_core::custom::PartState> =
            match self.part {
                TablePart::Root => |r| r.table_recipes().root_recipe(),
                TablePart::Header => |r| r.table_recipes().header_recipe(),
                TablePart::Body => |r| r.table_recipes().body_recipe(),
                TablePart::Footer => |r| r.table_recipes().footer_recipe(),
                TablePart::Row => |r| r.table_recipes().row_recipe(),
            };
        let part = kivact_core::custom::part(|state| state);
        let state = part.state();
        with_base_recipe_style(self.root.part(part), pick, &self.variants, move || {
            state.get()
        })
        .into_view()
    }
}
/// A header or data cell with proportional or fixed width.
pub struct TableCell {
    root: Node,
    header: bool,
    align: Value<TextAlign>,
    width: Option<Value<Length>>,
    span: Value<usize>,
    variants: Variants,
}
/// A column header.
#[track_caller]
pub fn table_head(content: impl IntoView) -> TableCell {
    cell(content, true)
}
/// A data cell.
#[track_caller]
pub fn table_cell(content: impl IntoView) -> TableCell {
    cell(content, false)
}
fn cell(content: impl IntoView, header: bool) -> TableCell {
    TableCell {
        root: node()
            .role(if header {
                Role::ColumnHeader
            } else {
                Role::Cell
            })
            .child(content),
        header,
        align: (|| TextAlign::Start).into_value(),
        width: None,
        span: 1.into_value(),
        variants: Variants::default(),
    }
}
impl TableCell {
    pub fn align(mut self, align: impl IntoValue<TextAlign>) -> Self {
        self.align = align.into_value();
        self
    }
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        self.width = Some(width.into_value());
        self
    }
    /// Allocate the proportional width of this many columns, with a minimum of one.
    pub fn col_span(mut self, span: impl IntoValue<usize>) -> Self {
        self.span = span.into_value();
        self
    }
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for TableCell {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for TableCell {
    fn into_view(self) -> View {
        let Self {
            root,
            header,
            align,
            width,
            span,
            variants,
        } = self;
        let part = kivact_core::custom::part(|state| state);
        let state = part.state();
        let root = with_base_recipe_style(
            root.part(part),
            if header {
                |r| r.table_recipes().head_recipe()
            } else {
                |r| r.table_recipes().cell_recipe()
            },
            &variants,
            move || state.get(),
        );
        root.base_style(move || {
            let s = style()
                .text_align(align.get())
                .justify_content(match align.get() {
                    TextAlign::Center => Justify::Center,
                    TextAlign::End => Justify::End,
                    _ => Justify::Start,
                });
            match width {
                Some(width) => s.width(width.get()).shrink(0),
                None => s.grow(span.get().max(1) as f64).basis(0).shrink(1),
            }
        })
        .into_view()
    }
}
