//! Role: DataTableApi state shares the row list, sort, widths and the cell registry.

use super::data::{Header, SortOrder, TableFocus};
use crate::Selection;
use crate::options_cell::OptionsCell;
use crate::policy::scalar::ScalarRange;
use crate::roving::{Observation, Roving};
use crate::{ListApi, ListItem, ListOptions, use_list};
use kivact_core::custom::{NodeRefExt, Owned, Owner};
use kivact_core::state::{Binding, IntoBinding, Signal, Value, effect, memo, signal};
use kivact_core::view::NodeRef;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type Label<K> = Rc<dyn Fn(&K) -> String>;
type MultiSort<C> = Binding<Vec<(C, SortOrder)>>;
type SortOption<K, C> = OptionValue<DataTableOptions<K, C>, Binding<Option<(C, SortOrder)>>>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) enum Target<K, C> {
    Header(C),
    Cell(K, C),
}

#[derive(Clone)]
pub(super) struct DataTableOptions<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey> {
    pub(super) label: Option<Label<K>>,
    pub(super) sort: Binding<Option<(C, SortOrder)>>,
    pub(super) multi_sort: Option<MultiSort<C>>,
    pub(super) widths: Binding<HashMap<C, f64>>,
    pub(super) resize: (ScalarRange, f64),
}

pub(super) struct State<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey> {
    // xtask-allow(state-fields): Optional mounted-part drawing observation, independent of behavior phase.
    pub(super) grid_state: Signal<Option<Value<kivact_core::custom::PartState>>>,
    pub(super) header_states:
        Signal<std::collections::HashMap<C, Value<super::data::TableHeaderState>>>,
    pub(super) row_states: Signal<std::collections::HashMap<K, Value<super::data::TableRowState>>>,
    pub(super) cell_states:
        Signal<std::collections::HashMap<(K, C), Value<super::data::TableCellState>>>,

    pub(super) owner: Owner,
    /// The App's root when the table was created: a cell of another App has
    /// another root.
    pub(super) root: Owner,
    pub(super) list: ListApi<K>,
    pub(super) keys: Value<Vec<K>>,
    pub(super) columns: Value<Vec<Header<C>>>,
    pub(super) selection: Selection<K>,
    /// The keyboard pattern; fixed by `use_data_table`.
    pub(super) focus: TableFocus,
    pub(super) options: OptionsCell<DataTableOptions<K, C>>,
    headers: RefCell<HashMap<C, NodeRef>>,
    /// The mounted cells, the cell the keyboard or pointer last moved to and
    /// the focus bookkeeping.
    pub(super) roving: Roving<Target<K, C>>,
    // xtask-allow(state-fields): Pure projection of canonical order and registry, not a stored focus request.
    pub(super) tab_stop: Value<Option<Target<K, C>>>,
    /// The cell holding the tab stop.
    // xtask-allow(state-fields): Pure projection of the canonical order, selection, and mounted registry.
    pub(super) active: Value<Option<(K, C)>>,
}

pub(super) fn create<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
    keys: Value<Vec<K>>,
    columns: Value<Vec<Header<C>>>,
    selection: Selection<K>,
    focus: TableFocus,
) -> Owned<Rc<State<K, C>>> {
    let owner = Owner::current();
    let root = Owner::root();
    let options = OptionsCell::new(DataTableOptions {
        label: None::<Label<K>>,
        sort: signal(None::<(C, SortOrder)>).into_binding(),
        multi_sort: None,
        widths: signal(HashMap::new()).into_binding(),
        resize: (
            ScalarRange::new(16.0, 4096.0).expect("invariant: finite column bounds"),
            10.0,
        ),
    });
    let settings = options.value();
    let items = memo(move || {
        let label = settings.with(|options| options.label.clone());
        keys.with(|keys| {
            keys.iter()
                .map(|key| match &label {
                    Some(label) => {
                        let (label, key) = (label.clone(), key.clone());
                        ListItem::new(key.clone(), move || label(&key))
                    }
                    None => ListItem::new(key.clone(), ""),
                })
                .collect::<Vec<_>>()
        })
    });
    let policy = match focus {
        TableFocus::None => ListOptions::default(),
        TableFocus::Rows(policy) => policy,
        TableFocus::Cells(range) => ListOptions {
            range,
            ..ListOptions::default()
        },
    };
    let rows = use_list(items, selection, policy);
    let selected = rows.selected();
    let starting = memo(move || {
        columns.with(|columns| {
            columns
                .iter()
                .filter_map(|column| column.width.map(|width| (column.key.clone(), width)))
                .collect::<HashMap<_, _>>()
        })
    });
    let widths = signal(starting.get_untracked());
    let roving = Roving::<Target<K, C>>::new();
    let requested = roving.requested;
    let registry = roving.clone();
    let tab_stop = memo(move || {
        keys.try_get()?;
        registry.track();
        let available = |cell: &Target<K, C>| {
            registry.with_members(|cells| cells.get(cell).is_some_and(|entry| entry.available()))
        };
        if let Some(cell) = requested.get().filter(|cell| available(cell)) {
            return Some(cell);
        }
        let first_in = |key: &K| {
            columns.with(|columns| {
                columns
                    .iter()
                    .map(|column| Target::Cell(key.clone(), column.key.clone()))
                    .find(|cell| available(cell))
            })
        };
        keys.with(|keys| {
            selected
                .with(|selected| {
                    keys.iter()
                        .filter(|key| selected.contains(key))
                        .find_map(&first_in)
                })
                .or_else(|| keys.iter().find_map(first_in))
                .or_else(|| {
                    columns.with(|columns| {
                        columns
                            .iter()
                            .map(|column| Target::Header(column.key.clone()))
                            .find(|header| available(header))
                    })
                })
        })
    });
    let active = memo(move || {
        tab_stop.get().and_then(|target| match target {
            Target::Cell(key, column) => Some((key, column)),
            Target::Header(_) => None,
        })
    });
    options.update(|options| options.widths = widths.into_binding());
    let state = Rc::new(State {
        grid_state: signal(None),
        header_states: signal(std::collections::HashMap::new()),
        row_states: signal(std::collections::HashMap::new()),
        cell_states: signal(std::collections::HashMap::new()),
        owner,
        root,
        list: rows,
        keys,
        columns,
        selection,
        focus,
        options,
        headers: RefCell::new(HashMap::new()),
        roving,
        active,
        tab_stop,
    });
    let stored = owner.store("DataTableApi", state.clone());
    effect(move || {
        if let Some(state) = stored.try_get() {
            reconcile(&state);
        }
    });
    stored
}

/// Follow focus that reaches a cell by other means, and give the focus of a
/// removed cell to the tab stop.
fn reconcile<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(state: &State<K, C>) {
    let active = state.tab_stop.get();
    let observation = Observation {
        current: {
            let focused = state.owner.focused_node();
            state
                .roving
                .with_members(|members| {
                    members
                        .values()
                        .find(|member| {
                            member.node.holds_focus() || member.node.holds_focus_within()
                        })
                        .and_then(|member| member.node.mounted_node())
                })
                .or(focused)
        },
        active: active.as_ref(),
        adopt: true,
        // A focused cell counts as the cell's focus whatever its constraints.
        reject_refused: false,
    };
    if let Some(node) = state.roving.reconcile(observation, |_| true) {
        node.scroll_into_view();
    }
}

impl<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey> State<K, C> {
    pub(super) fn sort(&self) -> SortOption<K, C> {
        OptionValue::new(self.options, |options| options.sort)
    }
    pub(super) fn multi_sort(&self) -> OptionValue<DataTableOptions<K, C>, Option<MultiSort<C>>> {
        OptionValue::new(self.options, |options| options.multi_sort)
    }
    pub(super) fn widths(&self) -> OptionValue<DataTableOptions<K, C>, Binding<HashMap<C, f64>>> {
        OptionValue::new(self.options, |options| options.widths)
    }
    pub(super) fn resize(&self) -> OptionValue<DataTableOptions<K, C>, (ScalarRange, f64)> {
        OptionValue::new(self.options, |options| options.resize)
    }
    /// The shared reference of a column's header, for widths and relations.
    #[track_caller]
    pub(super) fn header_ref(&self, column: &C) -> NodeRef {
        if let Some(reference) = self.headers.borrow().get(column) {
            return *reference;
        }
        let reference = self.owner.node_ref();
        self.headers.borrow_mut().insert(column.clone(), reference);
        reference
    }

    /// The mounted, available node of a cell.
    pub(super) fn target(&self, cell: &(K, C)) -> Option<NodeRef> {
        self.roving
            .target(&Target::Cell(cell.0.clone(), cell.1.clone()), |_| true)
    }

    pub(super) fn focus_header(&self, column: C) -> bool {
        let target = Target::Header(column);
        let Some(node) = self.roving.target(&target, |_| true) else {
            return false;
        };
        self.roving.requested.set(Some(target));
        node.focus();
        node.scroll_into_view();
        true
    }
    /// Move the tab stop to a cell and focus it.
    pub(super) fn focus_cell(&self, cell: (K, C)) -> bool {
        let Some(node) = self.target(&cell) else {
            return false;
        };
        self.roving
            .requested
            .set(Some(Target::Cell(cell.0, cell.1)));
        node.focus();
        node.scroll_into_view();
        true
    }
}

/// A projection of setup options that does not introduce another stored value.
pub(super) struct OptionValue<S: Clone + 'static, T> {
    options: OptionsCell<S>,
    select: fn(&S) -> T,
}
impl<S: Clone + 'static, T> Copy for OptionValue<S, T> {}
impl<S: Clone + 'static, T> Clone for OptionValue<S, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<S: Clone + 'static, T> OptionValue<S, T> {
    fn new(options: OptionsCell<S>, select: fn(&S) -> T) -> Self {
        Self { options, select }
    }
    pub(super) fn get(self) -> T {
        self.options.value().with(self.select)
    }
    pub(super) fn get_untracked(self) -> T {
        (self.select)(&self.options.get())
    }
    pub(super) fn with<R>(self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.get())
    }
}
