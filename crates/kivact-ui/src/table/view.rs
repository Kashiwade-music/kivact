//! Role: The table components: their sections, rows and cells.

use crate::layer::with_base_recipe_style;
use crate::recipe::Recipe;
use crate::table::look::TableTheme;
use kivact_core::custom::{Caller, PartState};
use kivact_core::state::Value;
use kivact_core::style::{Justify, Length, TextAlign, style};
use kivact_core::view::{Children, Node, Role, Variants, View, node};
use kivact_macros::component;

/// A static table; use `data_table` for interactive grids.
#[component]
pub fn table(children: Children, #[caller] caller: Caller) -> impl Element {
    section(
        children,
        Role::Table,
        |r: &TableTheme| r.root_recipe(),
        &caller.variants(),
    )
}

/// A group of column-header rows.
#[component]
pub fn table_header(children: Children, #[caller] caller: Caller) -> impl Element {
    section(
        children,
        Role::RowGroup,
        |r: &TableTheme| r.header_recipe(),
        &caller.variants(),
    )
}

/// A group of data rows.
#[component]
pub fn table_body(children: Children, #[caller] caller: Caller) -> impl Element {
    section(
        children,
        Role::RowGroup,
        |r: &TableTheme| r.body_recipe(),
        &caller.variants(),
    )
}

/// A group of summary rows.
#[component]
pub fn table_footer(children: Children, #[caller] caller: Caller) -> impl Element {
    section(
        children,
        Role::RowGroup,
        |r: &TableTheme| r.footer_recipe(),
        &caller.variants(),
    )
}

/// A row of header or data cells. `selected(v)` marks the row selected: the
/// row is `selected` in the accessibility tree, and the theme's row recipe
/// draws its selected look.
#[component]
pub fn table_row(
    children: Children,
    selected: Option<Value<bool>>,
    #[caller] caller: Caller,
) -> impl Element {
    let row = section(
        children,
        Role::Row,
        |r: &TableTheme| r.row_recipe(),
        &caller.variants(),
    );
    match selected {
        Some(selected) => row.selected(selected),
        None => row,
    }
}

/// A column header with proportional or fixed width, spanning `col_span`
/// columns (at least one).
#[component]
pub fn table_head(
    content: View,
    #[prop(default = TextAlign::Start)] align: Value<TextAlign>,
    width: Option<Value<Length>>,
    #[prop(default = 1)] col_span: Value<usize>,
    #[caller] caller: Caller,
) -> impl Element {
    cell(content, true, align, width, col_span, &caller.variants())
}

/// A data cell with proportional or fixed width, spanning `col_span` columns
/// (at least one).
#[component]
pub fn table_cell(
    content: View,
    #[prop(default = TextAlign::Start)] align: Value<TextAlign>,
    width: Option<Value<Length>>,
    #[prop(default = 1)] col_span: Value<usize>,
    #[caller] caller: Caller,
) -> impl Element {
    cell(content, false, align, width, col_span, &caller.variants())
}

/// One section node: its role, its children and its part, styled by the recipe `pick`.
fn section(
    children: Children,
    role: Role,
    pick: fn(&TableTheme) -> &Recipe<PartState>,
    variants: &Variants,
) -> Node {
    let part = kivact_core::custom::part(|state| state);
    let state = part.state();
    with_base_recipe_style(
        node().role(role).child(children.into_views()).part(part),
        pick,
        variants,
        move || state.get(),
    )
}

/// One header or data cell node, styled by the head or the cell recipe.
fn cell(
    content: View,
    header: bool,
    align: Value<TextAlign>,
    width: Option<Value<Length>>,
    span: Value<usize>,
    variants: &Variants,
) -> Node {
    let root = node()
        .role(if header {
            Role::ColumnHeader
        } else {
            Role::Cell
        })
        .child(content);
    let part = kivact_core::custom::part(|state| state);
    let state = part.state();
    let root = with_base_recipe_style(
        root.part(part),
        if header {
            |r: &TableTheme| r.head_recipe()
        } else {
            |r: &TableTheme| r.cell_recipe()
        },
        variants,
        move || state.get(),
    );
    root.column_span(move || u32::try_from(span.get().max(1)).unwrap_or(u32::MAX))
        .base_style(move || {
            let s = style()
                .text_align(align.get())
                .justify_content(match align.get() {
                    TextAlign::Center => Justify::Center,
                    TextAlign::End => Justify::End,
                    _ => Justify::Start,
                });
            match width {
                Some(width) => s.width(width.get()).flex_shrink(0),
                None => s
                    .flex_grow(span.get().max(1) as f64)
                    .flex_basis(0)
                    .flex_shrink(1),
            }
        })
}
