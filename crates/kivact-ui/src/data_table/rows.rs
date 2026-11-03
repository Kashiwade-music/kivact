//! Role: Render keyed header and body cells with the table's actual part states.
use super::types::{CellSlot, Key, RowColor, RowName, RowSelectable, RowSlot, Sort};
use crate::layer::with_recipe_style;
use crate::variants::Variants;
use crate::{DataColumn, DataTableCellState, DataTableCheckState, DataTableRowState, checkbox};
use kivact_core::control::{Node, NodeBuilder, column, icon, node, row, text};
use kivact_core::i18n::message_with;
use kivact_core::primitive::Part;
use kivact_core::style::{Cursor, Justify, Position, Style, pct, style};
use kivact_core::view::{
    ButtonPress, CheckedState, Handled, IntoView, MouseButton, PointerEvent, PointerPhase, Role,
    View, dynamic,
};
use kivact_core::{Binding, Callback, Value, binding, memo, signal};
use kivact_interaction::{
    DragCoordinates, DragState, DragUpdate, ListGesture, Table, TableCellState, TableFocus,
    TableRowState, use_disclosure, use_drag,
};
use std::hash::Hash;
use std::rc::Rc;

/// What the header and the rows of one table share.
pub(super) struct Grid<R: 'static, K: Clone + Eq + Hash + 'static> {
    pub(super) behavior: Table<K, usize>,
    /// The keys of the rows of the page.
    pub(super) keys: Value<Vec<K>>,
    pub(super) columns: Rc<Vec<DataColumn<R>>>,
    /// The positions of the columns that are shown.
    pub(super) visible: Value<Vec<usize>>,
    pub(super) selection: Option<Binding<Vec<K>>>,
    pub(super) row_selectable: Option<RowSelectable<R>>,
    pub(super) widths: Option<Binding<Vec<f32>>>,
    pub(super) actions: Option<(f32, RowSlot<R>)>,
    pub(super) row_label: RowName<R>,
    pub(super) cells: Option<CellSlot<R>>,
    pub(super) details: Option<RowSlot<R>>,
    pub(super) expanded_rows: Option<Binding<Vec<K>>>,
    pub(super) reorder: Option<Callback<(K, usize)>>,
    pub(super) source: Value<Vec<R>>,
    pub(super) key: Key<R, K>,
    pub(super) sort: Sort,
    pub(super) focus: TableFocus,
    pub(super) row_height: Option<Value<f32>>,
    pub(super) row_color: Option<RowColor<R>>,
    pub(super) variants: Variants,
}

impl<R: 'static, K: Clone + Eq + Hash + 'static> Clone for Grid<R, K> {
    fn clone(&self) -> Self {
        Self {
            behavior: self.behavior,
            keys: self.keys,
            columns: self.columns.clone(),
            visible: self.visible,
            selection: self.selection,
            row_selectable: self.row_selectable.clone(),
            widths: self.widths,
            actions: self.actions.clone(),
            row_label: self.row_label.clone(),
            cells: self.cells.clone(),
            details: self.details.clone(),
            expanded_rows: self.expanded_rows,
            reorder: self.reorder,
            source: self.source,
            key: self.key.clone(),
            sort: self.sort,
            focus: self.focus,
            row_height: self.row_height,
            row_color: self.row_color.clone(),
            variants: self.variants.clone(),
        }
    }
}

impl<R: 'static, K: Clone + Eq + Hash + 'static> Grid<R, K> {
    /// Whether rows have check boxes: the table has a selection and cells
    /// hold the focus (with rows, a press selects the row).
    fn checks(&self) -> Option<Binding<Vec<K>>> {
        self.selection
            .filter(|_| matches!(self.focus, TableFocus::Cells(_)))
    }

    /// The style that shares the width of column `index` between its header
    /// and its cells: the resized or own width, else what the recipe says.
    fn width(&self, index: usize, numeric: bool) -> impl Fn() -> Style + 'static {
        let column = &self.columns[index];
        let (widths, own) = (self.widths, column.width_value());
        let flex = column.flex_weight();
        let minimum = column.minimum_width();
        move || {
            let fixed = widths
                .and_then(|widths| widths.with(|widths| widths.get(index).copied()))
                .filter(|width| width.is_finite() && *width > 0.0)
                .or(own);
            let s = match fixed {
                Some(width) => {
                    let width = width.max(minimum.unwrap_or(0.0));
                    style().width(width).min_width(width).grow(0).shrink(0)
                }
                None if let Some(flex) = flex => style()
                    .basis(0)
                    .grow(flex)
                    .min_width(minimum.unwrap_or(0.0))
                    .shrink(1),
                None => minimum.map_or_else(style, |min| style().min_width(min)),
            };
            if numeric {
                s.justify_content(Justify::End)
            } else {
                s
            }
        }
    }
}

/// The header row: the check column, the shown columns' headers and the
/// actions column's.
pub(super) fn header<R: Clone + PartialEq + 'static, K: Clone + Eq + Hash + 'static>(
    grid: &Grid<R, K>,
) -> View {
    let check = grid
        .checks()
        .map(|selection| super::chrome::select_page(selection, grid.keys, &grid.variants));
    let actions = grid.actions.as_ref().map(|(width, _)| {
        node()
            .role(Role::ColumnHeader)
            .name(kivact_core::i18n::message("data_table.actions", "Actions"))
            .style(style().width(*width).shrink(0))
    });
    let heads = grid.clone();
    let columns = dynamic(grid.visible, move |indexes: Vec<usize>| {
        indexes
            .into_iter()
            .map(|index| head_cell(&heads, index))
            .collect::<Vec<_>>()
    });
    let disclosure = (grid.details.is_some() && grid.expanded_rows.is_some()).then(|| {
        expansion_cell()
            .role(Role::ColumnHeader)
            .name(kivact_core::i18n::message("data_table.details", "Details"))
    });
    row((check, disclosure, columns, actions))
        .role(Role::Row)
        .into_view()
}

/// The header of column `index`: its title, the sort mark and, when the
/// table is resizable, the resize handle.
fn head_cell<R: Clone + PartialEq + 'static, K: Clone + Eq + Hash + 'static>(
    grid: &Grid<R, K>,
    index: usize,
) -> View {
    let column = &grid.columns[index];
    let title = column.title();
    let part = grid.behavior.header(index);
    let state = part.state();
    let mark = with_recipe_style(
        node(),
        |r| r.data_table_recipes().sort_recipe(),
        &grid.variants,
        move || state.get(),
    );
    let handle = (grid.widths.is_some() && column.is_resizable()).then(|| {
        node()
            .part(grid.behavior.resize(index))
            .name(message_with(
                "data_table.resize",
                "Resize {title}",
                move || [("title", title.get())],
            ))
            .style(
                style()
                    .position(Position::Absolute)
                    .right(0)
                    .top(0)
                    .width(6)
                    .height(pct(100))
                    .cursor(Cursor::EwResize),
            )
    });
    let header = with_recipe_style(
        node().part(part).name(title),
        |r| r.data_table_recipes().header_recipe(),
        &grid.variants,
        move || state.get(),
    )
    .style(style().position(Position::Relative))
    .style(grid.width(index, column.is_numeric()))
    .child(text(title).style(crate::helpers::single_line_text()))
    .child(mark)
    .child(handle);
    column.render_header(header)
}

/// A primary press on a data cell must not replace the row selection when
/// rows are chosen with their check box.
fn keep_selection(event: PointerEvent) -> Handled {
    if matches!(
        event.phase,
        PointerPhase::Down(ButtonPress {
            button: MouseButton::Primary,
            ..
        })
    ) {
        Handled::Yes
    } else {
        Handled::No
    }
}

/// The check box of the row `id`, in its box: it selects or clears the row.
fn row_check<R, K>(grid: &Grid<R, K>, selection: Binding<Vec<K>>, id: K, item: Value<R>) -> View
where
    R: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
{
    let behavior = grid.behavior;
    let key = id.clone();
    let chosen = move || selection.with(|keys| keys.contains(&key));
    let state = {
        let chosen = chosen.clone();
        move || DataTableCheckState {
            header: false,
            checked: CheckedState::from_bool(chosen()),
        }
    };
    let label = grid.row_label.clone();
    let name = message_with("data_table.select_named", "Select {row}", move || {
        [("row", item.with(|row| label(row)))]
    });
    let toggle = kivact_core::view::callback(move |_: bool| {
        behavior.select(id.clone(), ListGesture::Toggle);
    });
    with_recipe_style(
        node().role(Role::Cell),
        |r| r.data_table_recipes().check_recipe(),
        &grid.variants,
        state,
    )
    .child({
        let selectable = grid.row_selectable.clone();
        checkbox(name, binding(chosen, move |on| toggle.try_call(on)))
            .hide_label(true)
            .disabled(move || selectable.as_ref().is_some_and(|test| !item.with(|row| test(row))))
    })
    .into_view()
}

/// The row for `id`: its check box, the shown cells and the row actions,
/// with the details under it.
pub(super) fn body<R, K>(grid: &Grid<R, K>, id: K, item: Value<R>) -> View
where
    R: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
{
    let part = grid.behavior.row(id.clone());
    let state = part.state();
    let check = grid
        .checks()
        .map(|selection| row_check(grid, selection, id.clone(), item));
    let checked = grid.checks().is_some();
    let cells = {
        let grid = grid.clone();
        let id = id.clone();
        dynamic(grid.visible, move |indexes: Vec<usize>| {
            indexes
                .into_iter()
                .map(|index| cell(&grid, &id, index, item, checked, state))
                .collect::<Vec<_>>()
        })
    };
    let actions = grid.actions.as_ref().map(|(width, render)| {
        node()
            .role(Role::Cell)
            .style(
                style()
                    .width(*width)
                    .shrink(0)
                    .direction(kivact_core::style::Direction::Row)
                    .align_items(kivact_core::style::Align::Center),
            )
            .child(render(item))
    });
    let (trigger, details) = match (&grid.details, grid.expanded_rows) {
        (Some(render), Some(expanded)) => {
            let (trigger, region) = row_disclosure(grid, id.clone(), item, expanded, render);
            (Some(trigger), Some(region.into_view()))
        }
        (Some(render), None) => (None, Some(render(item))),
        (None, _) => (None, None),
    };
    let (color, height) = (grid.row_color.clone(), grid.row_height);
    let line = with_recipe_style(
        row((check, trigger, cells, actions))
            .style(style().group())
            .part(part),
        |r| r.data_table_recipes().row_recipe(),
        &grid.variants,
        move || DataTableRowState {
            interaction: state.get(),
            color: color.as_ref().and_then(|color| item.with(|row| color(row))),
            height: height.map(|height| height.get().max(18.0)),
        },
    );
    let line = match grid.reorder {
        Some(reorder) => line.part(drag_source(grid, id, reorder)),
        None => line,
    };
    match details {
        Some(details) => column((line, details)).into_view(),
        None => line.into_view(),
    }
}

fn expansion_cell() -> Node {
    node().style(
        style()
            .width(32)
            .shrink(0)
            .direction(kivact_core::style::Direction::Row)
            .align_items(kivact_core::style::Align::Center)
            .justify_content(Justify::Center),
    )
}

fn row_disclosure<R: 'static, K: Clone + Eq + Hash + 'static>(
    grid: &Grid<R, K>,
    key: K,
    item: Value<R>,
    expanded: Binding<Vec<K>>,
    render: &RowSlot<R>,
) -> (Node, Node) {
    let current_key = key.clone();
    let open = binding(
        move || expanded.with(|keys| keys.contains(&current_key)),
        move |open| {
            let mut keys = expanded.get_untracked();
            if open {
                if keys.contains(&key) {
                    return;
                }
                keys.push(key.clone());
            } else {
                keys.retain(|candidate| candidate != &key);
            }
            expanded.set(keys);
        },
    );
    let disclosure = use_disclosure(open);
    let shown = disclosure.shown();
    let row_label = grid.row_label.clone();
    let name = message_with("data_table.row_details", "{row} details", move || {
        [("row", item.with(|row| row_label(row)))]
    });
    let role = if matches!(grid.focus, TableFocus::None) {
        Role::Cell
    } else {
        Role::GridCell
    };
    let trigger = node()
        .part(disclosure.trigger())
        .name(name)
        .on_pointer(keep_selection)
        .style(style().size(28, 28).cursor(Cursor::Pointer))
        .child(icon(move || {
            kivact_core::style::Icon::named(if shown.get() {
                "chevron-down"
            } else {
                "chevron-right"
            })
        }));
    let region = column(render(item))
        .role(Role::Group)
        .name(name)
        .part(disclosure.content());
    (expansion_cell().role(role).child(trigger), region)
}

/// One body cell: the cell controls, the column's cell view or its text.
fn cell<R, K>(
    grid: &Grid<R, K>,
    id: &K,
    index: usize,
    item: Value<R>,
    checked: bool,
    row: Value<TableRowState>,
) -> View
where
    R: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
{
    let def = grid.columns[index].clone();
    let content = match &grid.cells {
        Some(render) => render(item, index),
        None if def.has_cell() => {
            let def = def.clone();
            dynamic(item, move |row: R| def.cell_of(&row)).into_view()
        }
        None => {
            let def = def.clone();
            text(move || item.with(|row| def.text_of(row)))
                .style(crate::helpers::single_line_text())
                .into_view()
        }
    };
    let numeric = def.is_numeric();
    let named = def.clone();
    // With row focus a press reaches the row, which selects: the cell is a
    // passive grid cell and takes its state from the row.
    let (node, state) = if matches!(grid.focus, TableFocus::Rows(_)) {
        let state = memo(move || {
            let row = row.get();
            TableCellState {
                input: row.input,
                selected: row.selected,
                active: false,
            }
        });
        (node().role(Role::GridCell), state)
    } else {
        let part = grid.behavior.cell(id.clone(), index);
        let state = part.state();
        (node().part(part), state)
    };
    let cell = with_recipe_style(
        node,
        |r| r.data_table_recipes().cell_recipe(),
        &grid.variants,
        move || DataTableCellState {
            interaction: state.get(),
            numeric,
        },
    )
    .name(move || item.with(|row| named.text_of(row)))
    .style(grid.width(index, numeric))
    .child(content);
    if checked {
        cell.on_pointer(keep_selection).into_view()
    } else {
        cell.into_view()
    }
}

/// The drag part of a row that reports moves of one row in the caller's
/// order and clears the sort.
fn drag_source<R, K>(grid: &Grid<R, K>, id: K, reorder: Callback<(K, usize)>) -> Part<DragState>
where
    R: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
{
    let consumed = signal(0.0_f32);
    let (source, key, sort) = (grid.source, grid.key.clone(), grid.sort);
    let drag = use_drag::<()>(DragCoordinates::Window)
        .on_start(move |_| consumed.set(0.0))
        .on_update(move |update: DragUpdate<()>| {
            let dy = update.delta.y - consumed.get_untracked();
            if dy.abs() <= 28.0 {
                return;
            }
            let destination = source.with(|rows| {
                let at = rows.iter().position(|row| key(row) == id)?;
                Some(if dy > 0.0 {
                    (at + 1).min(rows.len().saturating_sub(1))
                } else {
                    at.saturating_sub(1)
                })
            });
            if let Some(destination) = destination {
                reorder.call((id.clone(), destination));
                sort.set(None);
                consumed.set(update.delta.y);
            }
        });
    drag.source(())
}
