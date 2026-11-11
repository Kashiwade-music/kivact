//! Role: Table options, sorting, widths and grid, row and cell parts.

use super::data::{
    HeaderSort, SortOrder, TableCellState, TableFocus, TableHeaderState, TableRowState,
};
use super::model::ColumnDef;
use super::state::{State, Target};
use super::{cells, sort as sorting, state};
use crate::Selection;
use crate::axis::check_list_axis;
use crate::policy::scalar::{ScalarRange, ScalarRangeError};
use crate::roving::Member;
use crate::{ListGesture, ListMode, SplitterState, use_splitter};
use kivact_core::ErrorKind;
use kivact_core::custom::{NodeRefExt, Owner};
use kivact_core::custom::{Owned, Part, PartState, part, raise, require_setup};
use kivact_core::state::IntoValue;
use kivact_core::state::{Value, binding, memo, var};
use kivact_core::view::{Handled, Key, KeyEvent, Orientation, Role, SortDirection, modifiers};
use std::collections::HashMap;
use std::rc::Rc;

/// A Copy table handle over caller-owned rows, columns and selection.
///
/// The caller renders every element: the grid container, one header per
/// column, and rows and cells with `for_each`, attaching
/// [`grid`](Self::grid), [`header`](Self::header), [`row`](Self::row),
/// [`cell`](Self::cell) and [`resize`](Self::resize). The table keeps the
/// row order, the selection through a [`ListApi`](crate::ListApi), the sort state,
/// the column widths and the keyboard focus; it creates no visible node.
///
/// Sorting the data is the caller's job: read [`sort_value`](Self::sort) and order
/// the rows passed to [`use_data_table`]. A caller that orders the rows before
/// creating the table keeps the sort state in its own binding and passes it
/// with [`sort`](Self::sort). Activating a sortable header
/// sorts by its column ascending, and activating it again flips the
/// direction.
///
/// [`TableFocus`] chooses the keyboard pattern. With
/// [`Cells`](TableFocus::Cells) (the default, an ARIA data grid) one cell is
/// the tab stop and the arrows, PageUp, PageDown, Home, End, Ctrl+Home and Ctrl+End move it
/// ([`GridMove`](crate::GridMove)); Space selects its row, Ctrl+Space
/// toggles it, Shift+ArrowUp/Down extend the selection and Ctrl+A or Cmd+A
/// selects every row. With [`Rows`](TableFocus::Rows) the rows are the items
/// of a list: one row is the tab stop and the list keys, pointer selection
/// and typeahead apply to rows. The variant carries the policy and is the
/// fifth argument of [`use_data_table`]; the other setters may be called before or
/// after the parts are made.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// # pub use kivact_core::{view, state, style};
/// # pub mod prelude {
/// # //! Role: Supply the canonical imports used by this documentation example.
/// # pub use kivact_core::state::{Callback, IntoBinding, IntoText, IntoValue, Signal, Value, binding, effect, memo, on_cleanup, on_mount, signal};
/// # pub use kivact_core::view::{EditPhase, IntoView, Element, Orientation, Role, column, for_each, node, node_ref, row, show, text};
/// # pub use kivact_core::style::{ColorToken, Cursor, Style, style};
/// # }
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{ColumnDef, TableFocus, use_data_table};
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
/// let t = use_data_table(people, |p: &Person| p.id, columns.clone(),
///     selected, TableFocus::default());
/// let head = row(columns.iter().map(|c| {
///     node().part(t.header(*c.key())).child(text(c.label())).into_view()
/// }).collect::<Vec<_>>());
/// let body = for_each(people, |p| p.id, move |person| {
///     let id = person.with(|p| p.id);
///     row((
///         node().part(t.cell(id, "name")).child(text(move || person.get().name)),
///         node().part(t.cell(id, "age")).child(text(move || person.get().age.to_string())),
///     ))
///     .part(t.row(id))
/// });
/// let view = column((head, body)).part(t.grid());
/// ```
pub struct DataTableApi<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey> {
    state: Owned<Rc<State<K, C>>>,
}

impl<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey> Copy for DataTableApi<K, C> {}

impl<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey> Clone for DataTableApi<K, C> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Create a setup-owned table without rendering any elements.
///
/// `rows` is the caller's data in display order, `key` its stable row
/// identity, `columns` the column definitions in display order (a
/// [`DataTableModelApi`](super::DataTableModelApi)'s [`columns`](super::DataTableModelApi::columns) are the shown ones) and
/// `selection` the
/// caller's authoritative row selection. `focus` chooses the keyboard
/// pattern with the policy it uses ([`TableFocus`]); `TableFocus::Rows` with a
/// policy whose mode is not `ListMode::Select` is a usage error. Rows whose
/// key repeats are reported, and the first one wins, as in a
/// [`ListApi`](crate::ListApi). Setup only.
#[track_caller]
pub fn use_data_table<R, K, C>(
    rows: impl IntoValue<Vec<R>>,
    key: impl Fn(&R) -> K + 'static,
    columns: impl IntoValue<Vec<ColumnDef<R, C>>>,
    selection: impl crate::IntoSelection<K>,
    focus: TableFocus,
) -> DataTableApi<K, C>
where
    R: kivact_core::state::PropValue,
    K: kivact_core::state::ItemKey,
    C: kivact_core::state::ItemKey,
{
    let selection = selection.into_selection();
    if let TableFocus::Rows(policy) = focus {
        if policy.mode != ListMode::Select {
            raise(
                ErrorKind::Message,
                "use_data_table: TableFocus::Rows needs ListMode::Select",
            );
        }
        check_list_axis(policy.axis, "use_data_table");
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
    DataTableApi {
        state: state::create(keys, headers, selection, focus),
    }
}

impl<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey> DataTableApi<K, C> {
    /// Drawing state of the mounted table container.
    pub fn grid_state(self) -> Value<PartState> {
        let observed = self.state().grid_state;
        memo(move || observed.get().map(|state| state.get()).unwrap_or_default())
    }
    /// Drawing state of the mounted column header.
    pub fn header_state(self, column: C) -> Value<TableHeaderState> {
        let observed = self.state().header_states;
        memo(move || {
            observed
                .with(|states| states.get(&column).copied())
                .map(|state| state.get())
                .unwrap_or(TableHeaderState {
                    input: PartState::default(),
                    sorting: HeaderSort::Unsortable,
                })
        })
    }
    /// Drawing state of the mounted row.
    pub fn row_state(self, key: K) -> Value<TableRowState> {
        let shared = self.state();
        let row = shared.list.item_state(key.clone());
        memo(move || {
            shared
                .row_states
                .with(|states| states.get(&key).copied())
                .map(|state| state.get())
                .unwrap_or_else(|| {
                    let row = row.get();
                    TableRowState {
                        input: row.input,
                        selected: row.selected,
                        active: row.active,
                    }
                })
        })
    }
    /// Drawing state of the mounted cell.
    pub fn cell_state(self, key: K, column: C) -> Value<TableCellState> {
        let shared = self.state();
        let row = shared.list.item_state(key.clone());
        memo(move || {
            shared
                .cell_states
                .with(|states| states.get(&(key.clone(), column.clone())).copied())
                .map(|state| state.get())
                .unwrap_or_else(|| {
                    let row = row.get();
                    TableCellState {
                        input: PartState::default(),
                        selected: row.selected,
                        active: false,
                    }
                })
        })
    }

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
        require_setup("DataTableApi::row_label");
        let label: Rc<dyn Fn(&K) -> String> = Rc::new(label);
        state.options.update(|options| options.label = Some(label));
        self
    }

    /// Keep the sort state in the caller's binding instead of the table's own.
    ///
    /// Header activation offers the next state to the binding's writer. Use it
    /// to start sorted, or to order the rows before creating the table. Setup
    /// only.
    #[track_caller]
    pub fn with_sort(
        self,
        sort: impl kivact_core::state::IntoBinding<Option<(C, SortOrder)>>,
    ) -> Self {
        let state = self.state();
        require_setup("DataTableApi::sort");
        let sort = sort.into_binding();
        state.options.update(|options| {
            options.sort = sort;
            options.multi_sort = None;
        });
        self
    }

    /// Use ordered sort keys. Shift-activation updates one key; ordinary
    /// activation replaces the list. Each key cycles ascending, descending,
    /// and unsorted. Setup only.
    #[track_caller]
    pub fn multi_sort(
        self,
        sort: impl kivact_core::state::IntoBinding<Vec<(C, SortOrder)>>,
    ) -> Self {
        require_setup("DataTableApi::multi_sort");
        let sort = sort.into_binding();
        self.state()
            .options
            .update(|options| options.multi_sort = Some(sort));
        self
    }

    /// Keep the column widths in the caller's binding instead of the table's own.
    ///
    /// The table's own widths start from each [`ColumnDef::width`]. Resize parts
    /// offer the whole next map to the binding's writer. Setup only.
    #[track_caller]
    pub fn with_widths(
        self,
        widths: impl kivact_core::state::IntoBinding<HashMap<C, f64>>,
    ) -> Self {
        let state = self.state();
        require_setup("DataTableApi::widths");
        let widths = widths.into_binding();
        state.options.update(|options| options.widths = widths);
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
        key_step: impl Into<f64>,
    ) -> Result<Self, ScalarRangeError> {
        let key_step = key_step.into();
        if !key_step.is_finite() || key_step <= 0.0 {
            return Err(ScalarRangeError::InvalidStep);
        }
        let state = self.state();
        require_setup("DataTableApi::resize_limits");
        state
            .options
            .update(|options| options.resize = (range, key_step));
        Ok(self)
    }

    /// Read the sort column and direction; `None` is the caller's own order.
    #[track_caller]
    pub fn sort(self) -> Value<Option<(C, SortOrder)>> {
        let sort = self.state().sort();
        (move || sort.with(|sort| sort.get())).into_value()
    }

    /// Read the column widths, in the caller's unit, by column key.
    #[track_caller]
    pub fn widths(self) -> Value<HashMap<C, f64>> {
        let widths = self.state().widths();
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

    /// DataTableApi or grid semantics, without appearance or children.
    #[track_caller]
    pub fn grid(self) -> Part<PartState> {
        let state = self.state();
        require_setup("DataTableApi::grid");
        let input = if matches!(state.focus, TableFocus::Rows(_)) {
            state.list.list()
        } else {
            part(|input| input)
        };
        let part_state = input.state();
        crate::part_slot::attach_part_state(state.grid_state, part_state);
        Part::new(part_state, move |host| {
            let host = host.part(input).role(if state.focus == TableFocus::None {
                Role::Table
            } else {
                Role::Grid
            });
            if matches!(state.focus, TableFocus::Cells(_)) {
                host.roving_focus(Orientation::Horizontal)
                    .multiselectable(matches!(state.selection, Selection::Multiple(_)))
            } else {
                host
            }
        })
    }

    /// A column header with its label, sort semantics and sort activation.
    #[track_caller]
    pub fn header(self, column: C) -> Part<TableHeaderState> {
        let state = self.state();
        require_setup("DataTableApi::header");
        let reference = state.header_ref(&column);
        let columns = state.columns;
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
        let (direction, rank) = header_sort(&state, key);
        let priority = kivact_core::i18n::message_with(
            "data_table.sort_priority",
            "Priority {rank} of {total}",
            move || {
                let (rank, total) = rank.get().unwrap_or_default();
                [("rank", rank.to_string()), ("total", total.to_string())]
            },
        );
        let state_for_sort = state.clone();
        let activate = move |extend: bool| {
            activate_header(&state_for_sort, &key.get(), sortable.get(), extend);
        };
        let focused = reference.focused();
        let focus_mode = state.focus;
        let focusable = memo(move || sortable.get() || matches!(focus_mode, TableFocus::Cells(_)));
        let keyboard = state.clone();
        let mounted = state.clone();
        let input = part(move |input| TableHeaderState {
            input,
            sorting: header_sorting(
                sortable.get(),
                direction.get(),
                rank.get().map(|(rank, _)| rank),
            ),
        });
        let part_state = input.state();
        observe_header(&state, key, part_state);
        Part::new(part_state, move |host| {
            host.part(input)
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
                .focusable(focusable)
                .description(move || {
                    if rank.get().is_some() {
                        priority.get()
                    } else {
                        String::new()
                    }
                })
                .on_press(move || activate(modifiers().get().shift))
                .on_attach(move |mount| {
                    mount_header(mounted.clone(), key.get(), mount);
                })
                .on_key(move |event: KeyEvent| {
                    header_key(
                        &keyboard,
                        &key.get(),
                        &event,
                        focused.get_untracked(),
                        sortable.get(),
                    )
                })
                .node_ref(reference)
        })
    }

    /// A row with its name, selection and, with [`TableFocus::Rows`], the
    /// list's roving focus and keys.
    #[track_caller]
    pub fn row(self, key: K) -> Part<TableRowState> {
        let state = self.state();
        require_setup("DataTableApi::row");
        if matches!(state.focus, TableFocus::Rows(_)) {
            let base = state.list.item(key).map_state(|item| TableRowState {
                input: item.input,
                selected: item.selected,
                active: item.active,
            });
            return Part::new(base.state(), move |host| host.part(base).role(Role::Row));
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
        require_setup("DataTableApi::cell");
        cells::cell(state, key, column)
    }

    /// A splitter on the trailing boundary of a column that edits its width.
    ///
    /// The width is the column's entry in [`widths_value`](Self::widths), else its
    /// [`ColumnDef::width`], else the measured width of its header. Writes go
    /// through [`resize_limits`](Self::resize_limits) to the widths binding.
    /// The caller names the handle and draws it. Setup only.
    #[track_caller]
    pub fn resize(self, column: C) -> Part<SplitterState> {
        let state = self.state();
        require_setup("DataTableApi::resize");
        let rect = state.header_ref(&column).rect();
        let (columns, widths) = (state.columns, state.widths());
        let resize = state.resize();
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

type HeaderSortBindings = (Value<Option<SortOrder>>, Value<Option<(usize, usize)>>);

fn header_sort<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
    state: &State<K, C>,
    key: kivact_core::state::Var<C>,
) -> HeaderSortBindings {
    let sort = state.sort();
    let multi_sort = state.multi_sort();
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
    (direction, rank)
}

/// The drawing state of a header from whether it can sort and its sort.
fn header_sorting(sortable: bool, direction: Option<SortOrder>, rank: Option<usize>) -> HeaderSort {
    match (sortable, direction) {
        (_, Some(direction)) => HeaderSort::Sorted { direction, rank },
        (true, None) => HeaderSort::Sortable,
        (false, None) => HeaderSort::Unsortable,
    }
}

fn activate_header<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
    state: &State<K, C>,
    key: &C,
    sortable: bool,
    extend: bool,
) {
    let sort = state.sort();
    let multi_sort = state.multi_sort();
    if !sortable {
        return;
    }
    let multiple = multi_sort.get_untracked();
    let keys = multiple.map_or_else(
        || sort.get_untracked().get_untracked().into_iter().collect(),
        |binding| binding.get_untracked(),
    );
    let next = sorting::Sort::activate(sorting::Event {
        current: sorting::Sort::from_keys(keys),
        column: key.clone(),
        extend,
        multiple: multiple.is_some(),
    })
    .keys();
    if let Some(binding) = multiple {
        binding.set(next);
    } else {
        sort.get_untracked().set(next.into_iter().next());
    }
}

fn mount_header<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
    mounted: Rc<State<K, C>>,
    key: C,
    mount: kivact_core::custom::PartMount,
) {
    if !matches!(mounted.focus, TableFocus::Cells(_)) {
        return;
    }
    let owner = Owner::current();
    if Owner::root() != mounted.root {
        owner.report(
            ErrorKind::WrongApp,
            "DataTableApi header belongs to another App",
        );
        return;
    }
    let target = Target::Header(key);
    let node = mount.node;
    mounted.roving.insert(
        target.clone(),
        Member {
            node,
            eligible: mount.eligible,
            disabled: mount.disabled,
            hidden: mount.hidden,
        },
    );
    let token = node.mounted_node();
    let weak = Rc::downgrade(&mounted);
    owner.on_cleanup(move || {
        if let Some(state) = weak.upgrade() {
            let focused = token.is_some() && state.owner.focused_node() == token;
            if state.roving.remove(&target, node) && focused {
                state.roving.restore_from.set(token);
            }
        }
    });
}

fn header_key<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
    keyboard: &State<K, C>,
    key: &C,
    event: &KeyEvent,
    focused: bool,
    sortable: bool,
) -> Handled {
    let multi_sort = keyboard.multi_sort();
    if matches!(keyboard.focus, TableFocus::Cells(_)) && focused {
        let column = key.clone();
        let columns = keyboard.columns.with(|columns| {
            columns
                .iter()
                .map(|column| column.key.clone())
                .collect::<Vec<_>>()
        });
        if let Some(index) = columns.iter().position(|key| *key == column) {
            let destination = match event.key() {
                Key::ArrowLeft => Some(index.saturating_sub(1)),
                Key::ArrowRight => Some((index + 1).min(columns.len() - 1)),
                Key::Home => Some(0),
                Key::End if !event.modifiers().ctrl && !event.modifiers().meta => {
                    Some(columns.len() - 1)
                }
                _ => None,
            };
            if let Some(destination) = destination {
                keyboard.focus_header(columns[destination].clone());
                return Handled::Yes;
            }
        }
        let rows = keyboard.keys.get_untracked();
        let row = match event.key() {
            Key::ArrowDown => rows.first(),
            Key::PageDown => rows.get(9).or_else(|| rows.last()),
            Key::End if event.modifiers().ctrl || event.modifiers().meta => rows.last(),
            _ => None,
        };
        if let Some(row) = row {
            let column = if event.key() == Key::End {
                keyboard
                    .columns
                    .with(|columns| columns.last().map(|column| column.key.clone()))
                    .unwrap_or(column)
            } else {
                column
            };
            keyboard.focus_cell((row.clone(), column));
            return Handled::Yes;
        }
    }
    let modifiers = event.modifiers();
    if !event.repeat()
        && modifiers.shift
        && !modifiers.ctrl
        && !modifiers.alt
        && !modifiers.meta
        && matches!(event.key(), Key::Enter | Key::Space)
        && focused
        && multi_sort.get_untracked().is_some()
    {
        activate_header(keyboard, key, sortable, true);
        Handled::Yes
    } else {
        Handled::No
    }
}

fn observe_header<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
    state: &State<K, C>,
    key: kivact_core::state::Var<C>,
    part_state: Value<TableHeaderState>,
) {
    let observed = state.header_states;
    observed.update(|states| {
        states.insert(key.get(), part_state);
    });
    kivact_core::state::on_cleanup(move || {
        if observed.try_get_untracked().is_some() {
            observed.update(|states| {
                let key = key.get();
                if states.get(&key) == Some(&part_state) {
                    states.remove(&key);
                }
            });
        }
    });
}
