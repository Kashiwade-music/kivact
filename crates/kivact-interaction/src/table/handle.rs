//! Role: The `Table` handle: options, sort, widths, and its grid, row and cell parts.

use super::data::{SortOrder, TableCellState, TableFocus, TableHeaderState, TableRowState};
use super::model::ColumnDef;
use super::state::State;
use super::{cells, state};
use crate::axis::check_list_axis;
use crate::policy::scalar::{ScalarRange, ScalarRangeError};
use crate::{ListGesture, ListMode, ListSelection, SplitterState, use_splitter};
use kivact_core::ErrorKind;
use kivact_core::custom::{Owned, Part, PartState, part, raise, require_setup};
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Value, binding, memo, var};
use kivact_core::view::{
    Handled, Key, KeyEvent, Orientation, Role, SortDirection, current_modifiers,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

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
/// and typeahead apply to rows. The variant carries the policy and is the
/// fifth argument of [`use_table`]; the other setters may be called before or
/// after the parts are made.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::{control, prelude, view};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{ColumnDef, ListSelection, TableFocus, use_table};
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
///     ListSelection::Multiple(selected.into()), TableFocus::default());
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
/// [`TableModel`](super::TableModel)'s [`columns`](super::TableModel::columns) are the shown ones) and
/// `selection` the
/// caller's authoritative row selection. `focus` chooses the keyboard
/// pattern with the policy it uses ([`TableFocus`]); `TableFocus::Rows` with a
/// policy whose mode is not `ListMode::Select` is a usage error. Rows whose
/// key repeats are reported, and the first one wins, as in a
/// [`List`](crate::List). Setup only.
#[track_caller]
pub fn use_table<R, K, C>(
    rows: impl IntoValue<Vec<R>>,
    key: impl Fn(&R) -> K + 'static,
    columns: impl IntoValue<Vec<ColumnDef<R, C>>>,
    selection: ListSelection<K>,
    focus: TableFocus,
) -> Table<K, C>
where
    R: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
    C: Clone + Eq + Hash + 'static,
{
    let at = Location::caller();
    if let TableFocus::Rows(policy) = focus {
        if policy.mode != ListMode::Select {
            raise(
                ErrorKind::Message,
                "use_table: TableFocus::Rows needs ListMode::Select",
                at,
            );
        }
        check_list_axis(policy.axis, "use_table");
    }
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
        state: state::create(keys, headers, selection, focus, at),
    }
}

impl<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static> Table<K, C> {
    #[track_caller]
    fn state(self) -> Rc<State<K, C>> {
        self.state.get()
    }
    /// Name each row, for its accessible name and for typeahead.
    ///
    /// The function may read reactive data; names follow it. Without one,
    /// rows have no name. Setup only.
    #[track_caller]
    pub fn row_label(self, label: impl Fn(&K) -> String + 'static) -> Self {
        let state = self.state();
        require_setup("Table::row_label");
        let label: Rc<dyn Fn(&K) -> String> = Rc::new(label);
        state.label.update(|current| *current = Some(label));
        self
    }

    /// Keep the sort state in the caller's binding instead of the table's own.
    ///
    /// Header activation offers the next state to the binding's writer. Use it
    /// to start sorted, or to order the rows before creating the table. Setup
    /// only.
    #[track_caller]
    pub fn sort(self, sort: impl Into<Binding<Option<(C, SortOrder)>>>) -> Self {
        let state = self.state();
        require_setup("Table::sort");
        let sort = sort.into();
        state.sort.update(|current| *current = sort);
        state.multi_sort.update(|current| *current = None);
        self
    }

    /// Use ordered sort keys. Shift-activation updates one key; ordinary
    /// activation replaces the list. Each key cycles ascending, descending,
    /// and unsorted. Setup only.
    #[track_caller]
    pub fn multi_sort(self, sort: impl Into<Binding<Vec<(C, SortOrder)>>>) -> Self {
        require_setup("Table::multi_sort");
        self.state()
            .multi_sort
            .update(|current| *current = Some(sort.into()));
        self
    }

    /// Keep the column widths in the caller's binding instead of the table's own.
    ///
    /// The table's own widths start from each [`ColumnDef::width`]. Resize parts
    /// offer the whole next map to the binding's writer. Setup only.
    #[track_caller]
    pub fn widths(self, widths: impl Into<Binding<HashMap<C, f64>>>) -> Self {
        let state = self.state();
        require_setup("Table::widths");
        let widths = widths.into();
        state.widths.update(|current| *current = widths);
        self
    }

    /// Bound the widths a resize part writes and set its key increment.
    ///
    /// The default is 16 to 4096 logical pixels in steps of 10. A non-finite
    /// or non-positive step is rejected as [`ScalarRange::with_step`] does.
    /// Setup only.
    #[track_caller]
    pub fn resize_limits(
        self,
        range: ScalarRange,
        key_step: f64,
    ) -> Result<Self, ScalarRangeError> {
        if !key_step.is_finite() || key_step <= 0.0 {
            return Err(ScalarRangeError::InvalidStep);
        }
        let state = self.state();
        require_setup("Table::resize_limits");
        state.resize.set((range, key_step));
        Ok(self)
    }

    /// Read the sort column and direction; `None` is the caller's own order.
    #[track_caller]
    pub fn sort_value(self) -> Value<Option<(C, SortOrder)>> {
        let sort = self.state().sort;
        (move || sort.with(|sort| sort.get())).into_value()
    }

    /// Read the column widths, in the caller's unit, by column key.
    #[track_caller]
    pub fn widths_value(self) -> Value<HashMap<C, f64>> {
        let widths = self.state().widths;
        (move || widths.with(|widths| widths.get())).into_value()
    }

    /// Offer a selection gesture for a row, without moving the focus.
    #[track_caller]
    pub fn select(self, key: K, gesture: ListGesture) -> bool {
        self.state().list.select(key, gesture)
    }

    /// Limit which row keys can be selected without changing focus. Setup only.
    pub fn row_selectable(self, test: impl Fn(&K) -> bool + 'static) -> Self {
        self.state().list.selectable(test);
        self
    }

    /// Table or grid semantics, without appearance or children.
    #[track_caller]
    pub fn grid(self) -> Part<PartState> {
        let state = self.state();
        require_setup("Table::grid");
        match state.focus {
            TableFocus::None => part(|input| input).role(Role::Table),
            TableFocus::Rows(_) => state.list.list().role(Role::Grid),
            TableFocus::Cells(_) => part(|input| input)
                .role(Role::Grid)
                .multiselectable(matches!(state.selection, ListSelection::Multiple(_))),
        }
    }

    /// A column header with its label, sort semantics and sort activation.
    #[track_caller]
    pub fn header(self, column: C) -> Part<TableHeaderState> {
        let state = self.state();
        require_setup("Table::header");
        let reference = state.header_ref(&column);
        let (columns, sort, multi_sort) = (state.columns, state.sort, state.multi_sort);
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
                if let Some(multi_sort) = multi_sort.get() {
                    return multi_sort.with(|items| {
                        items
                            .iter()
                            .find(|(column, _)| column == key)
                            .map(|(_, order)| *order)
                    });
                }
                sort.with(|sort| {
                    sort.with(|sort| match sort {
                        Some((column, order)) if column == key => Some(*order),
                        _ => None,
                    })
                })
            })
        });
        let rank = memo(move || {
            multi_sort.get().and_then(|sort| {
                sort.with(|items| {
                    if items.len() < 2 {
                        return None;
                    }
                    key.with(|key| {
                        items
                            .iter()
                            .position(|(column, _)| column == key)
                            .map(|index| (index + 1, items.len()))
                    })
                })
            })
        });
        let priority = kivact_core::i18n::message_with(
            "data_table.sort_priority",
            "Priority {rank} of {total}",
            move || {
                let (rank, total) = rank.get().unwrap_or_default();
                [("rank", rank.to_string()), ("total", total.to_string())]
            },
        );
        let activate = move |extend: bool| {
            if !sortable.get() {
                return;
            }
            if let Some(binding) = multi_sort.get_untracked() {
                let column = key.get();
                let mut keys = binding.get_untracked();
                let index = keys.iter().position(|(key, _)| *key == column);
                let next = match index.map(|index| keys[index].1) {
                    None => Some(SortOrder::Ascending),
                    Some(SortOrder::Ascending) => Some(SortOrder::Descending),
                    Some(SortOrder::Descending) => None,
                };
                if !extend {
                    keys.clear();
                    keys.extend(next.map(|next| (column, next)));
                } else if let Some(index) = index {
                    if let Some(next) = next {
                        keys[index].1 = next;
                    } else {
                        keys.remove(index);
                    }
                } else if let Some(next) = next {
                    keys.push((column, next));
                }
                binding.set(keys);
                return;
            }
            let next = match direction.get() {
                Some(SortOrder::Ascending) => SortOrder::Descending,
                _ => SortOrder::Ascending,
            };
            sort.get_untracked().set(Some((key.get(), next)));
        };
        let focused = reference.focused();
        part(move |input| TableHeaderState {
            input,
            sort: direction.get(),
            sortable: sortable.get(),
            sort_rank: rank.get().map(|(rank, _)| rank),
        })
        .role(Role::ColumnHeader)
        .name(move || {
            info.with(|info| info.as_ref().map(|info| info.0.clone()))
                .unwrap_or_default()
        })
        .sort(move || {
            direction
                .get()
                .map_or(SortDirection::None, SortDirection::from)
        })
        .focusable(sortable)
        .description(move || {
            if rank.get().is_some() {
                priority.get()
            } else {
                String::new()
            }
        })
        .on_press(move || activate(current_modifiers().shift))
        .on_key(move |event: KeyEvent| {
            let modifiers = event.modifiers();
            if !event.repeat()
                && modifiers.shift
                && !modifiers.ctrl
                && !modifiers.alt
                && !modifiers.meta
                && matches!(event.key(), Key::Enter | Key::Space)
                && focused.get_untracked()
                && multi_sort.get_untracked().is_some()
            {
                activate(true);
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .reference(reference)
    }

    /// A row with its name, selection and, with [`TableFocus::Rows`], the
    /// list's roving focus and keys.
    #[track_caller]
    pub fn row(self, key: K) -> Part<TableRowState> {
        let state = self.state();
        require_setup("Table::row");
        if matches!(state.focus, TableFocus::Rows(_)) {
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

    /// A cell; with [`TableFocus::None`] it has the native `Cell` role. With
    /// [`TableFocus::Cells`] it takes the roving focus and the
    /// grid keys. With [`TableFocus::Rows`], callers normally render passive
    /// cells with `Role::GridCell` and attach only [`row`](Self::row) as the
    /// input target, so pointer selection and dragging reach the row.
    #[track_caller]
    pub fn cell(self, key: K, column: C) -> Part<TableCellState> {
        let state = self.state();
        require_setup("Table::cell");
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
        let rect = state.header_ref(&column).rect();
        let (columns, widths) = (state.columns, state.widths);
        let resize = state.resize;
        let key = var(column);
        let value = memo(move || {
            key.with(|key| {
                widths
                    .with(|widths| widths.with(|widths| widths.get(key).copied()))
                    .or_else(|| {
                        columns.with(|columns| {
                            columns
                                .iter()
                                .find(|column| &column.key == key)
                                .and_then(|column| column.width)
                        })
                    })
                    .filter(|width| width.is_finite() && *width > 0.0)
                    .unwrap_or_else(|| rect.get().map_or(100.0, |rect| rect.width))
            })
        });
        let write = move |width: f64| {
            if width.is_finite() {
                let widths = widths.get_untracked();
                let mut next = widths.get_untracked();
                next.insert(key.get(), width);
                widths.set(next);
            }
        };
        let range = memo(move || {
            let range = resize.with(|limits| limits.0);
            let minimum = key.with(|key| {
                columns.with(|columns| {
                    columns
                        .iter()
                        .find(|column| &column.key == key)
                        .and_then(|column| column.min_width)
                })
            });
            let Some(minimum) = minimum.filter(|minimum| *minimum > range.min()) else {
                return range;
            };
            let bounded = ScalarRange::new(minimum, range.max().max(minimum))
                .expect("invariant: finite ordered column bounds");
            range
                .step()
                .map_or(bounded, |step| bounded.with_step(step).unwrap_or(bounded))
        });
        let step = memo(move || resize.with(|limits| limits.1));
        use_splitter(binding(value, write), range, Orientation::Horizontal)
            .key_step_value(step)
            .shift_step_factor(5.0)
            .handle()
    }
}
