//! Role: Table state shares the row list, sort, widths and the cell registry.

use super::data::{Header, SortOrder, TableFocus};
use crate::policy::scalar::ScalarRange;
use crate::roving::{Observation, Roving};
use crate::{List, ListItem, ListPolicy, ListSelection, use_list};
use kivact_core::primitive::{Owned, Owner};
use kivact_core::view::NodeRef;
use kivact_core::{Binding, Signal, Value, effect, memo, signal};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

type Label<K> = Rc<dyn Fn(&K) -> String>;

pub(super) struct State<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static> {
    pub(super) owner: Owner,
    /// The App's root when the table was created: a cell of another App has
    /// another root.
    pub(super) root: Owner,
    /// Resolves while the table's owner is alive and not being disposed.
    pub(super) alive: Owned<()>,
    pub(super) list: List<K>,
    pub(super) keys: Value<Vec<K>>,
    pub(super) columns: Value<Vec<Header<C>>>,
    pub(super) selection: ListSelection<K>,
    /// The keyboard pattern; fixed by `use_table`.
    pub(super) focus: TableFocus,
    pub(super) label: Signal<Option<Label<K>>>,
    pub(super) sort: Signal<Binding<Option<(C, SortOrder)>>>,
    pub(super) multi_sort: Signal<Option<Binding<Vec<(C, SortOrder)>>>>,
    pub(super) widths: Signal<Binding<HashMap<C, f64>>>,
    pub(super) resize: Signal<(ScalarRange, f64)>,
    headers: RefCell<HashMap<C, NodeRef>>,
    /// The mounted cells, the cell the keyboard or pointer last moved to and
    /// the focus bookkeeping.
    pub(super) roving: Roving<(K, C)>,
    /// The cell holding the tab stop.
    pub(super) active: Value<Option<(K, C)>>,
    pub(super) at: &'static Location<'static>,
}

pub(super) fn create<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static>(
    keys: Value<Vec<K>>,
    columns: Value<Vec<Header<C>>>,
    selection: ListSelection<K>,
    focus: TableFocus,
    at: &'static Location<'static>,
) -> Owned<State<K, C>> {
    let owner = Owner::current("table");
    let root = Owner::root("table");
    let alive = owner.store("Table", Rc::new(()));
    let label = signal(None::<Label<K>>);
    let items = memo(move || {
        let label = label.get();
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
        TableFocus::None => ListPolicy::default(),
        TableFocus::Rows(policy) => policy,
        TableFocus::Cells(range) => ListPolicy {
            range,
            ..ListPolicy::default()
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
    let roving = Roving::<(K, C)>::new();
    let requested = roving.requested;
    let registry = roving.clone();
    let active = memo(move || {
        alive.try_get()?;
        registry.track();
        let available = |cell: &(K, C)| {
            registry.with_members(|cells| cells.get(cell).is_some_and(|entry| entry.available()))
        };
        if let Some(cell) = requested.get().filter(|cell| available(cell)) {
            return Some(cell);
        }
        let first_in = |key: &K| {
            columns.with(|columns| {
                columns
                    .iter()
                    .map(|column| (key.clone(), column.key.clone()))
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
        })
    });
    let sort = signal(Binding::from(signal(None::<(C, SortOrder)>)));
    let state = Rc::new(State {
        owner,
        root,
        alive,
        list: rows,
        keys,
        columns,
        selection,
        focus,
        label,
        sort,
        multi_sort: signal(None),
        widths: signal(Binding::from(widths)),
        resize: signal((
            ScalarRange::new(16.0, 4096.0).expect("invariant: finite column bounds"),
            10.0,
        )),
        headers: RefCell::new(HashMap::new()),
        roving,
        active,
        at,
    });
    let stored = owner.store("Table", state.clone());
    effect(move || reconcile(&state));
    stored
}

/// Follow focus that reaches a cell by other means, and give the focus of a
/// removed cell to the tab stop.
fn reconcile<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static>(state: &State<K, C>) {
    if state.alive.try_get().is_none() {
        return;
    }
    let active = state.active.get();
    let observation = Observation {
        current: state.owner.focused_node(),
        active: active.as_ref(),
        adopt: true,
        // A focused cell counts as the cell's focus whatever its constraints.
        reject_refused: false,
    };
    if let Some(node) = state.roving.reconcile(observation, |_| true) {
        node.scroll_into_view();
    }
}

impl<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static> State<K, C> {
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
        self.roving.target(cell, |_| true)
    }

    /// Move the tab stop to a cell and focus it.
    pub(super) fn focus_cell(&self, cell: (K, C)) -> bool {
        let Some(node) = self.target(&cell) else {
            return false;
        };
        self.roving.requested.set(Some(cell));
        node.focus();
        node.scroll_into_view();
        true
    }
}
