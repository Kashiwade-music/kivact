//! Role: Render keyed header and body cells with the table's actual part states.
use super::sticky_paint::surface;
use super::types::{
    CellSlot, Key, RowColor, RowContextMenu, RowName, RowSelectable, RowSlot, RowStatusOf, Sort,
    StickyEdges,
};
use crate::layer::with_recipe_style;
use crate::variants::Variants;
use crate::{DataColumn, DataTableCellState, DataTableCheckState, DataTableRowState, checkbox};
use kivact_core::custom::Part;
use kivact_core::draw::linear_gradient;
use kivact_core::view::IntoView;
use kivact_core::view::NodeBuilder;

use kivact_core::geometry::Rect;
use kivact_core::i18n::message_with;
use kivact_core::state::{Binding, Callback, Value, binding, memo, signal};
use kivact_core::style::{
    Color, Cursor, Direction, Justify, Position, Style, TransitionProp, pct, style, use_theme,
};
use kivact_core::view::{
    ButtonPress, CheckedState, Handled, MouseButton, NodeRef, PointerEvent, PointerPhase,
    ReadingDirection, Role, View, dynamic, node_ref, use_direction,
};
use kivact_core::view::{Node, column, icon, node, row, text};
use kivact_interaction::{
    DataTableApi, DragCoordinates, DragState, DragUpdate, ListGesture, TableCellState, TableFocus,
    TableRowState, use_disclosure, use_drag,
};
use std::collections::HashMap;
use std::rc::Rc;

/// What the header and the rows of one table share.
pub(super) struct Grid<R: 'static, K: kivact_core::state::ItemKey> {
    pub(super) tree: Option<super::tree::Plan<K>>,
    pub(super) behavior: DataTableApi<K, usize>,
    pub(super) filters: Option<super::filters::Filters>,
    /// The keys of the rows of the page.
    pub(super) keys: Value<Vec<K>>,
    pub(super) indexed: Option<Value<HashMap<K, usize>>>,
    pub(super) columns: Rc<Vec<DataColumn<R>>>,
    /// The positions of the columns that are shown.
    pub(super) visible: Value<Vec<usize>>,
    pub(super) selection: Option<Binding<Vec<K>>>,
    pub(super) row_selectable: Option<RowSelectable<R>>,
    pub(super) sticky: Option<Rc<StickyColumns>>,
    pub(super) widths: Option<Binding<Vec<f64>>>,
    pub(super) actions: Option<(f64, RowSlot<R>)>,
    pub(super) row_context_menu: Option<RowContextMenu<R>>,
    pub(super) row_label: RowName<R>,
    pub(super) cells: Option<CellSlot<R>>,
    pub(super) details: Option<RowSlot<R>>,
    pub(super) expanded_rows: Option<Binding<Vec<K>>>,
    pub(super) reorder: Option<Callback<kivact_interaction::Reorder<K>>>,
    pub(super) source: Value<Vec<R>>,
    pub(super) key: Key<R, K>,
    pub(super) sort: Sort,
    pub(super) focus: TableFocus,
    pub(super) row_height: Option<Value<f64>>,
    pub(super) row_status: Option<RowStatusOf<R>>,
    pub(super) row_color: Option<RowColor<R>>,
    pub(super) variants: Variants,
}

/// Shared header measurements let in-flow pins follow flex and resize.
pub(super) struct StickyColumns {
    edges: StickyEdges,
    rtl: bool,
    headers: Vec<NodeRef>,
    widths: Vec<Value<f64>>,
    pub(super) viewport: NodeRef,
    pub(super) content: NodeRef,
    overflow: Value<(bool, bool)>,
}

impl StickyColumns {
    pub(super) fn new(edges: StickyEdges, count: usize) -> Self {
        let headers: Vec<_> = (0..count).map(|_| node_ref()).collect();
        let widths = headers
            .iter()
            .map(|header| {
                let rect = header.rect();
                memo(move || rect.get().map_or(0.0, |rect| rect.width))
            })
            .collect();
        let viewport = node_ref();
        let content = node_ref();
        let (port_rect, content_rect, offset) =
            (viewport.rect(), content.rect(), viewport.scroll_offset());
        let overflow = memo(move || {
            let width = port_rect.get().map_or(0.0, |rect| rect.width);
            let total = content_rect.get().map_or(0.0, |rect| rect.width);
            let x = offset.get().x;
            (x > 1.0, x + width < total - 1.0)
        });
        Self {
            edges,
            rtl: use_direction().get() == ReadingDirection::Rtl,
            headers,
            widths,
            viewport,
            content,
            overflow,
        }
    }

    fn edge(&self, index: usize) -> Option<bool> {
        if self.edges.0.contains(&index) {
            Some(true)
        } else if self.edges.1.contains(&index) {
            Some(false)
        } else {
            None
        }
    }
}

impl<R: 'static, K: kivact_core::state::ItemKey> Clone for Grid<R, K> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            behavior: self.behavior,
            filters: self.filters,
            keys: self.keys,
            indexed: self.indexed,
            columns: self.columns.clone(),
            visible: self.visible,
            selection: self.selection,
            row_selectable: self.row_selectable.clone(),
            sticky: self.sticky.clone(),
            widths: self.widths,
            actions: self.actions.clone(),
            row_context_menu: self.row_context_menu.clone(),
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
            row_status: self.row_status.clone(),
            row_color: self.row_color.clone(),
            variants: self.variants.clone(),
        }
    }
}

impl<R: 'static, K: kivact_core::state::ItemKey> Grid<R, K> {
    /// Whether rows have check boxes: the table has a selection and cells
    /// hold the focus (with rows, a press selects the row).
    fn checks(&self) -> Option<Binding<Vec<K>>> {
        self.selection
            .filter(|_| matches!(self.focus, TableFocus::Cells(_)))
    }

    fn direction(&self) -> Style {
        if self.sticky.as_ref().is_some_and(|sticky| sticky.rtl)
            || (self.row_status.is_some() && use_direction().get() == ReadingDirection::Rtl)
        {
            style().direction(Direction::RowReverse)
        } else {
            style()
        }
    }

    fn sticky_style(
        &self,
        index: usize,
        header: bool,
        state: Option<Value<TableCellState>>,
        color: Option<Value<Option<Color>>>,
    ) -> impl Fn() -> Style + 'static {
        let sticky = self.sticky.clone();
        let visible = self.visible;
        move || {
            let Some(sticky) = &sticky else {
                return style();
            };
            let Some(start) = sticky.edge(index) else {
                return style();
            };
            let offset: f64 = visible.with(|visible| {
                visible
                    .iter()
                    .copied()
                    .filter(|&other| if start { other < index } else { other > index })
                    .filter(|&other| sticky.edge(other) == Some(start))
                    .map(|other| sticky.widths[other].get())
                    .sum()
            });
            let base = surface(
                header,
                state.is_some_and(|state| state.get().selected),
                color.and_then(|color| color.get()),
            )
            .position(Position::Sticky)
            .z_index(if header { 3 } else { 1 });
            if start != sticky.rtl {
                base.left(offset)
            } else {
                base.right(offset)
            }
        }
    }

    fn sticky_shadow(&self, index: usize) -> Option<Node> {
        let sticky = self.sticky.clone()?;
        let start = sticky.edge(index)?;
        let visible = self.visible;
        let on_left = start != sticky.rtl;
        Some(node().hit_testable(false).style(move || {
            let paint = use_theme()
                .get()
                .ext::<crate::UiTokens>()
                .data_table
                .sticky
                .clone();
            let width = paint.shadow_width;
            let boundary = visible.with(|columns| {
                let mut pinned = columns
                    .iter()
                    .copied()
                    .filter(|&index| sticky.edge(index) == Some(start));
                if start {
                    pinned.next_back()
                } else {
                    pinned.next()
                }
            });
            let (before, after) = sticky.overflow.get();
            let show = boundary == Some(index) && if on_left { before } else { after };
            let shadow = paint.shadow;
            let base = style()
                .position(Position::Absolute)
                .top(0)
                .bottom(0)
                .width(width)
                .opacity(if show { 1.0 } else { 0.0 })
                .paint(move |draw, size| {
                    let gradient = linear_gradient(if on_left { 90 } else { 270 })
                        .stop(shadow, 0)
                        .stop(Color::TRANSPARENT, pct(100));
                    draw.fill_rect(
                        Rect {
                            x: 0.0,
                            y: 0.0,
                            width: size.width,
                            height: size.height,
                        },
                        gradient,
                    );
                });
            let base = if on_left {
                base.left(pct(100))
            } else {
                base.right(pct(100))
            };
            if let Some(transition) = paint.shadow_transition.clone() {
                base.transition(TransitionProp::Paint, transition)
            } else {
                base
            }
        }))
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
pub(super) fn header<R: kivact_core::state::PropValue, K: kivact_core::state::ItemKey>(
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
    let index = grid
        .indexed
        .map(|_| index_cell(Role::ColumnHeader, text("#")).name("#"));
    let status = grid
        .row_status
        .as_ref()
        .map(|_| super::row_status::header());
    row((status, index, check, disclosure, columns, actions))
        .style(grid.direction())
        .role(Role::Row)
        .into_view()
}

/// The header of column `index`: its title, the sort mark and, when the
/// table is resizable, the resize handle.
fn head_cell<R: kivact_core::state::PropValue, K: kivact_core::state::ItemKey>(
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
    let field = grid.filters.and_then(|_| column.filter_editor());
    let inline = grid
        .filters
        .is_some_and(|filters| filters.ui != crate::FilterUi::Popover);
    let filter_anchor = (!inline && field.is_some()).then(kivact_core::view::node_ref);
    let filter = grid
        .filters
        .map(|filters| filters.control(index, field, title, filter_anchor));
    let mut header = node().part(part).name(title);
    if let Some(anchor) = filter_anchor {
        header = header.node_ref(anchor);
    }
    if let Some(sticky) = &grid.sticky {
        header = header.node_ref(sticky.headers[index]);
    }
    let header = with_recipe_style(
        header,
        |r| r.data_table_recipes().header_recipe(),
        &grid.variants,
        move || state.get(),
    )
    .style(style().position(Position::Relative))
    .style(grid.width(index, column.is_numeric()))
    .style(grid.sticky_style(index, true, None, None))
    .child(grid.sticky_shadow(index))
    .child(
        grid.tree
            .filter(|tree| tree.column == index && tree.show_all)
            .map(|tree| tree.header()),
    );
    let title = text(title).style(crate::helpers::single_line_text());
    let header = if inline {
        use kivact_core::style::{Align, Direction};
        header
            .style(
                style()
                    .direction(Direction::Column)
                    .align_items(Align::Stretch)
                    .gap(4),
            )
            .child(row((title, mark)).style(style().gap(4)))
            .child(filter)
    } else {
        header.child(title).child(mark).child(filter)
    };
    column.render_header(header.child(handle))
}

/// A primary press on a data cell must not replace the row selection when
/// rows are chosen with their check box.
fn keep_selection(event: PointerEvent) -> Handled {
    if matches!(
        event.phase(),
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
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
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
    let toggle = kivact_core::state::callback(move |_: bool| {
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
            .disabled(move || {
                selectable
                    .as_ref()
                    .is_some_and(|test| !item.with(|row| test(row)))
            })
    })
    .into_view()
}

/// The row for `id`: its check box, the shown cells and the row actions,
/// with the details under it.
pub(super) fn body<R, K>(grid: &Grid<R, K>, id: K, item: Value<R>) -> View
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    if let Some(tree) = grid.tree {
        let grid = grid.clone();
        dynamic(tree.is_parent(id.clone()), move |branch| {
            body_row(&grid, id.clone(), item, branch)
        })
    } else {
        body_row(grid, id, item, false)
    }
}

fn body_row<R, K>(grid: &Grid<R, K>, id: K, item: Value<R>, branch: bool) -> View
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let mut part = grid.behavior.row(id.clone());
    if let Some(tree) = grid.tree {
        let level = tree.level(id.clone());
        let state = part.state();
        part = Part::new(state, move |host| host.part(part).level(level));
        if branch {
            let expanded = tree.open(id.clone());
            let state = part.state();
            part = Part::new(state, move |host| host.part(part).expanded(expanded));
        }
    }
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
            .child(render.call(item))
    });
    let (trigger, details) = match (&grid.details, grid.expanded_rows) {
        (Some(render), Some(expanded)) => {
            let (trigger, region) = row_disclosure(grid, id.clone(), item, expanded, render);
            (Some(trigger), Some(region.into_view()))
        }
        (Some(render), None) => (None, Some(render.call(item))),
        (None, _) => (None, None),
    };
    let index = grid.indexed.map(|positions| {
        let id = id.clone();
        let role = if grid.focus == TableFocus::None {
            Role::Cell
        } else {
            Role::GridCell
        };
        index_cell(
            role,
            text(move || {
                positions
                    .with(|positions| positions.get(&id).map(usize::to_string).unwrap_or_default())
            }),
        )
    });
    let (color, height) = (grid.row_color.clone(), grid.row_height);
    let status = grid.row_status.as_ref().map(|status| {
        let status = status.clone();
        super::row_status::cell(memo(move || item.with(|row| status(row))))
    });
    let line = with_recipe_style(
        row((status, index, check, trigger, cells, actions))
            .style(style().group())
            .part(part),
        |r| r.data_table_recipes().row_recipe(),
        &grid.variants,
        move || DataTableRowState {
            interaction: state.get(),
            color: color.as_ref().and_then(|color| item.with(|row| color(row))),
            height: height.map(|height| height.get().max(18.0)),
        },
    )
    .style(grid.direction());
    let line = match grid.reorder {
        Some(reorder) => line.part(drag_source(grid, id, reorder)),
        None => line,
    };
    let line = match &grid.row_context_menu {
        Some(menu) => menu.call((item, line)),
        None => line.into_view(),
    };
    match details {
        Some(details) => column((line, details)).into_view(),
        None => line,
    }
}

fn index_cell(role: Role, text: Node) -> Node {
    row(text.style(
        style()
            .tabular_nums()
            .font_family("monospace")
            .font_size(12)
            .color(kivact_core::style::ColorToken::TextMuted),
    ))
    .role(role)
    .style(
        style()
            .width(48)
            .min_width(48)
            .shrink(0)
            .justify_content(Justify::End),
    )
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

fn row_disclosure<R: 'static, K: kivact_core::state::ItemKey>(
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
    let shown = disclosure.is_open();
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
    let region = column(render.call(item))
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
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let def = grid.columns[index].clone();
    let content = match &grid.cells {
        Some(render) => render.call((item, index)),
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
    let content = match grid.tree {
        Some(tree) if tree.column == index => tree.cell(id.clone(), content),
        _ => content,
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
    let tint = grid
        .sticky
        .as_ref()
        .and(grid.row_color.clone())
        .map(|color| memo(move || item.with(|row| color(row))));
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
    .style(grid.sticky_style(index, false, Some(state), tint))
    .child(grid.sticky_shadow(index))
    .child(content);
    if checked {
        cell.on_pointer(keep_selection).into_view()
    } else {
        cell.into_view()
    }
}

/// The drag part of a row that reports moves of one row in the caller's
/// order and clears the sort.
fn drag_source<R, K>(
    grid: &Grid<R, K>,
    id: K,
    reorder: Callback<kivact_interaction::Reorder<K>>,
) -> Part<DragState>
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
{
    let consumed = signal(0.0_f64);
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
                reorder.call(kivact_interaction::Reorder {
                    key: id.clone(),
                    to: destination,
                });
                sort.set(None);
                consumed.set(update.delta.y);
            }
        });
    drag.source(())
}
