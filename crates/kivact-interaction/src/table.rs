//! Role: A table behavior attaches grid, header, row, cell and resize parts.
//!
//! [`use_table`] takes the caller's rows in display order, their key, the
//! [`ColumnDef`]s (key, reactive label, optional starting width, and a header
//! sorts exactly when the definition has a `compare`) and a [`ListSelection`]. The parts are `grid`,
//! `header(col)`, `row(key)`, `cell(key, col)` and `resize(col)`. A header
//! flips ascending and descending and publishes [`Table::sort_value`];
//! sharing the sort is [`Table::sort`]. The widths are
//! [`Table::widths_value`] and [`Table::widths`], and a resize part is a
//! splitter whose range and key step come from [`Table::resize_limits`].
//!
//! With [`TableFocus::Cells`] (the default) one grid cell is the tab stop and
//! [`GridMove`](crate::GridMove) moves it in two dimensions; the row
//! selection keys follow the list's rules, and the list policy's axis and
//! boundary do not change the cell moves (only its range policy is used).
//! With [`TableFocus::Rows`] the rows are list items that share roving focus,
//! typeahead and selection, and `row_label` names them. The Basic, Game and
//! Pop kits use cells and Creative uses rows; a kit adds only drawing and
//! domain data. A calendar is not a fixed grid of rows and columns, so
//! [`use_calendar_grid`](crate::use_calendar_grid) applies
//! [`GridMove::from_key`](crate::GridMove::from_key) to dates itself (month
//! edges, date wrapping, PageUp/PageDown) rather than use table parts.
//! Row and column count properties are not published yet.
//!
//! [`use_table_model`] is the headless row model (TanStack Table's) that
//! filters, sorts and pages the rows before they reach the table; give both
//! the same sort binding. Row and key types are generic, so kits need not
//! agree on a key type.

mod cells;
mod data;
mod model;
mod state;

use crate::policy::scalar::{ScalarRange, ScalarRangeError};
use crate::{ListGesture, ListPolicy, ListSelection, SplitterState, use_splitter};
use kivact_core::diagnostics::ErrorKind;
use kivact_core::primitive::{Owned, Part, PartState, part, require_setup};
use kivact_core::view::{IntoValue, Orientation, Role, SortDirection};
use kivact_core::{Binding, Value, binding, memo, var};
use state::State;
use std::collections::HashMap;
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

pub use data::{TableCellState, TableFocus, TableHeaderState, TableRowState};
pub use model::{ColumnDef, TableModel, use_table_model};

/// A Copy table handle over caller-owned rows, columns and selection.
///
/// The caller renders every element: the grid container, one header per
/// column, and rows and cells with `for_each`, attaching
/// [`grid`](Self::grid), [`header`](Self::header), [`row`](Self::row),
/// [`cell`](Self::cell) and [`resize`](Self::resize). The table keeps the
/// row order, the selection through a [`List`](crate::List), the sort state,
/// the column widths and the keyboard focus; it creates no visible node.
///
/// Sorting the data is the caller's job: read [`sort_value`](Self::sort_value) and order
/// the rows passed to [`use_table`]. A caller that orders the rows before
/// creating the table keeps the sort state in its own binding and passes it
/// with [`sort`](Self::sort). Activating a sortable header
/// sorts by its column ascending, and activating it again flips the
/// direction.
///
/// [`TableFocus`] chooses the keyboard pattern. With
/// [`Cells`](TableFocus::Cells) (the default, an ARIA data grid) one cell is
/// the tab stop and the arrows, Home, End, Ctrl+Home and Ctrl+End move it
/// ([`GridMove`](crate::GridMove)); Space selects its row, Ctrl+Space
/// toggles it, Shift+ArrowUp/Down extend the selection and Ctrl+A or Cmd+A
/// selects every row. With [`Rows`](TableFocus::Rows) the rows are the items
/// of a list: one row is the tab stop and the list keys, pointer selection
/// and typeahead apply to rows. Configure during setup, before creating any
/// part.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::{control, prelude, view};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{ColumnDef, ListSelection, use_table};
/// use kivact::prelude::*;
/// use kivact::view::for_each;
/// #[derive(Clone, PartialEq)]
/// struct Person { id: u32, name: String, age: u32 }
/// let people = signal(vec![Person { id: 1, name: "Ada".into(), age: 36 }]);
/// let selected = signal(Vec::<u32>::new());
/// let columns = vec![
///     ColumnDef::new("name", "Name").compare(|a: &Person, b: &Person| a.name.cmp(&b.name)),
///     ColumnDef::new("age", "Age").compare(|a: &Person, b: &Person| a.age.cmp(&b.age)),
/// ];
/// let t = use_table(people, |p: &Person| p.id, columns.clone(),
///     ListSelection::Multiple(selected.into()));
/// let head = row(columns.iter().map(|c| {
///     node().part(t.header(*c.key())).child(text(c.label())).into_view()
/// }).collect::<Vec<_>>());
/// let body = for_each(people, |p| p.id, move |id, person| {
///     row((
///         node().part(t.cell(id, "name")).child(text(move || person.get().name)),
///         node().part(t.cell(id, "age")).child(text(move || person.get().age.to_string())),
///     ))
///     .part(t.row(id))
/// });
/// let view = column((head, body)).part(t.grid());
/// ```
pub struct Table<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static> {
    state: Owned<State<K, C>>,
}
impl<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static> Copy for Table<K, C> {}
impl<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static> Clone for Table<K, C> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Create a setup-owned table without rendering any elements.
///
/// `rows` is the caller's data in display order, `key` its stable row
/// identity, `columns` the column definitions in display order (a
/// [`TableModel`]'s [`columns`](TableModel::columns) are the shown ones) and
/// `selection` the
/// caller's authoritative row selection. Rows whose key repeats are
/// reported, and the first one wins, as in a [`List`](crate::List).
#[track_caller]
pub fn use_table<R, K, C>(
    rows: impl IntoValue<Vec<R>>,
    key: impl Fn(&R) -> K + 'static,
    columns: impl IntoValue<Vec<ColumnDef<R, C>>>,
    selection: ListSelection<K>,
) -> Table<K, C>
where
    R: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
    C: Clone + Eq + Hash + 'static,
{
    let rows = rows.into_value();
    let keys = memo(move || rows.with(|rows| rows.iter().map(&key).collect::<Vec<_>>()));
    let definitions = columns.into_value();
    let headers = memo(move || {
        definitions.with(|definitions| {
            definitions
                .iter()
                .map(ColumnDef::header)
                .collect::<Vec<_>>()
        })
    });
    Table {
        state: state::create(keys, headers, selection, Location::caller()),
    }
}

impl<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static> Table<K, C> {
    #[track_caller]
    fn state(self) -> Rc<State<K, C>> {
        self.state.get()
    }
    #[track_caller]
    fn configure(self, name: &'static str) -> Option<Rc<State<K, C>>> {
        let state = self.state();
        require_setup(name);
        if state.started.get() {
            state.owner.report(
                ErrorKind::Message,
                "Table must be configured before creating parts",
                Location::caller(),
            );
            None
        } else {
            Some(state)
        }
    }

    /// Choose the keyboard pattern; [`TableFocus::Cells`] is the default.
    /// Setup only, before creating any Part.
    #[track_caller]
    pub fn focus(self, focus: TableFocus) -> Self {
        if let Some(state) = self.configure("Table::focus") {
            state.focus.set(focus);
        }
        self
    }

    /// Configure the row list's selection range and typeahead policy.
    ///
    /// With [`TableFocus::Rows`] the whole policy applies to the rows. With
    /// [`TableFocus::Cells`] only the range policy applies, and the
    /// cell keys replace the policy's navigation. Setup only, before creating
    /// any Part.
    #[track_caller]
    pub fn policy(self, policy: ListPolicy) -> Self {
        if let Some(state) = self.configure("Table::policy") {
            state.list.policy(policy);
        }
        self
    }

    /// Name each row, for its accessible name and for typeahead.
    ///
    /// The function may read reactive data; names follow it. Without one,
    /// rows have no name. Setup only, before creating any Part.
    #[track_caller]
    pub fn row_label(self, label: impl Fn(&K) -> String + 'static) -> Self {
        if let Some(state) = self.configure("Table::row_label") {
            *state.label.borrow_mut() = Some(Rc::new(label));
            state.labels.update(|revision| *revision += 1);
        }
        self
    }

    /// Keep the sort state in the caller's binding instead of the table's own.
    ///
    /// Header activation offers the next state to the binding's writer. Use it
    /// to start sorted, or to order the rows before creating the table. Setup
    /// only, before creating any Part.
    #[track_caller]
    pub fn sort(self, sort: impl Into<Binding<Option<(C, SortDirection)>>>) -> Self {
        if let Some(state) = self.configure("Table::sort") {
            state.sort.set(sort.into());
        }
        self
    }

    /// Keep the column widths in the caller's binding instead of the table's own.
    ///
    /// The table's own widths start from each [`ColumnDef::width`]. Resize parts
    /// offer the whole next map to the binding's writer. Setup only, before
    /// creating any Part.
    #[track_caller]
    pub fn widths(self, widths: impl Into<Binding<HashMap<C, f64>>>) -> Self {
        if let Some(state) = self.configure("Table::widths") {
            state.widths.set(widths.into());
        }
        self
    }

    /// Bound the widths a resize part writes and set its key increment.
    ///
    /// The default is 16 to 4096 logical pixels in steps of 10. A non-finite
    /// or non-positive step is rejected as [`ScalarRange::with_step`] does.
    /// Setup only, before creating any Part.
    #[track_caller]
    pub fn resize_limits(
        self,
        range: ScalarRange,
        key_step: f64,
    ) -> Result<Self, ScalarRangeError> {
        if !key_step.is_finite() || key_step <= 0.0 {
            return Err(ScalarRangeError::InvalidStep);
        }
        if let Some(state) = self.configure("Table::resize_limits") {
            state.resize.set((range, key_step));
        }
        Ok(self)
    }

    /// Read the sort column and direction; `None` is the caller's own order.
    #[track_caller]
    pub fn sort_value(self) -> Value<Option<(C, SortDirection)>> {
        self.state().sort.get().into_value()
    }

    /// Read the column widths, in the caller's unit, by column key.
    #[track_caller]
    pub fn widths_value(self) -> Value<HashMap<C, f64>> {
        self.state().widths.get().into_value()
    }

    /// Offer a selection gesture for a row, without moving the focus.
    #[track_caller]
    pub fn select(self, key: K, gesture: ListGesture) -> bool {
        self.state().list.select(key, gesture)
    }

    /// Grid semantics and multiselection, without appearance or children.
    #[track_caller]
    pub fn grid(self) -> Part<PartState> {
        let state = self.state();
        require_setup("Table::grid");
        state.started.set(true);
        match state.focus.get() {
            TableFocus::Rows => state.list.list().role(Role::Grid),
            TableFocus::Cells => part(|input| input)
                .role(Role::Grid)
                .multiselectable(matches!(state.selection, ListSelection::Multiple(_))),
        }
    }

    /// A column header with its label, sort semantics and sort activation.
    #[track_caller]
    pub fn header(self, column: C) -> Part<TableHeaderState> {
        let state = self.state();
        require_setup("Table::header");
        state.started.set(true);
        let reference = state.header_ref(&column);
        let (columns, sort) = (state.columns, state.sort.get());
        let key = var(column);
        let info = memo(move || {
            key.with(|key| {
                columns.with(|columns| {
                    columns
                        .iter()
                        .find(|column| &column.key == key)
                        .map(|column| (column.label.get(), column.sortable))
                })
            })
        });
        let sortable = memo(move || info.with(|info| info.as_ref().is_some_and(|info| info.1)));
        let direction = memo(move || {
            key.with(|key| {
                sort.with(|sort| match sort {
                    Some((column, direction)) if column == key => *direction,
                    _ => SortDirection::None,
                })
            })
        });
        part(move |input| TableHeaderState {
            input,
            sort: direction.get(),
            sortable: sortable.get(),
        })
        .role(Role::ColumnHeader)
        .name(move || {
            info.with(|info| info.as_ref().map(|info| info.0.clone()))
                .unwrap_or_default()
        })
        .sort(direction)
        .focusable(sortable)
        .on_press(move || {
            if !sortable.get() {
                return;
            }
            let next = match direction.get() {
                SortDirection::Ascending => SortDirection::Descending,
                _ => SortDirection::Ascending,
            };
            sort.set(Some((key.get(), next)));
        })
        .reference(reference)
    }

    /// A row with its name, selection and, with [`TableFocus::Rows`], the
    /// list's roving focus and keys.
    #[track_caller]
    pub fn row(self, key: K) -> Part<TableRowState> {
        let state = self.state();
        require_setup("Table::row");
        state.started.set(true);
        if state.focus.get() == TableFocus::Rows {
            return state
                .list
                .item(key)
                .map_state(|item| TableRowState {
                    input: item.input,
                    selected: item.selected,
                    active: item.active,
                })
                .role(Role::Row);
        }
        cells::row(state, key)
    }

    /// A cell; with [`TableFocus::Cells`] it takes the roving focus and the
    /// grid keys. With [`TableFocus::Rows`], callers normally render passive
    /// cells with `Role::GridCell` and attach only [`row`](Self::row) as the
    /// input target, so pointer selection and dragging reach the row.
    #[track_caller]
    pub fn cell(self, key: K, column: C) -> Part<TableCellState> {
        let state = self.state();
        require_setup("Table::cell");
        state.started.set(true);
        cells::cell(state, key, column, Location::caller())
    }

    /// A splitter on the trailing boundary of a column that edits its width.
    ///
    /// The width is the column's entry in [`widths_value`](Self::widths_value), else its
    /// [`ColumnDef::width`], else the measured width of its header. Writes go
    /// through [`resize_limits`](Self::resize_limits) to the widths binding.
    /// The caller names the handle and draws it. Setup only.
    #[track_caller]
    pub fn resize(self, column: C) -> Part<SplitterState> {
        let state = self.state();
        require_setup("Table::resize");
        state.started.set(true);
        let rect = state.header_ref(&column).rect();
        let (columns, widths) = (state.columns, state.widths.get());
        let key = var(column);
        let value = memo(move || {
            key.with(|key| {
                widths
                    .with(|widths| widths.get(key).copied())
                    .or_else(|| {
                        columns.with(|columns| {
                            columns
                                .iter()
                                .find(|column| &column.key == key)
                                .and_then(|column| column.width)
                        })
                    })
                    .filter(|width| width.is_finite() && *width > 0.0)
                    .unwrap_or_else(|| rect.get().map_or(100.0, |rect| f64::from(rect.width)))
            })
        });
        let write = move |width: f64| {
            if width.is_finite() {
                let mut next = widths.get_untracked();
                next.insert(key.get(), width);
                widths.set(next);
            }
        };
        let (range, step) = state.resize.get();
        use_splitter(binding(value, write), range, Orientation::Horizontal)
            .key_step(step)
            .expect("invariant: Table::resize_limits validated the step")
            .handle()
    }
}
