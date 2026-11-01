//! Role: Table state shares the row list, sort, widths and the cell registry.

use super::data::{Header, TableFocus};
use crate::policy::scalar::ScalarRange;
use crate::roving::{Observation, Roving};
use crate::{List, ListItem, ListSelection, use_list};
use kivact_core::primitive::{Owned, Owner};
use kivact_core::view::NodeRef;
use kivact_core::{Binding, Signal, Value, effect, memo, signal};
use std::cell::{Cell, RefCell};
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
    pub(super) focus: Cell<TableFocus>,
    pub(super) started: Cell<bool>,
    pub(super) label: Rc<RefCell<Option<Label<K>>>>,
    pub(super) labels: Signal<u64>,
    pub(super) sort: Cell<Binding<Option<(C, kivact_core::view::SortDirection)>>>,
    pub(super) widths: Cell<Binding<HashMap<C, f64>>>,
    pub(super) resize: Cell<(ScalarRange, f64)>,
    headers: RefCell<HashMap<C, NodeRef>>,
    /// The mounted cells, the cell the keyboard or pointer last moved to and
    /// the focus bookkeeping.
    pub(super) roving: Roving<(K, C)>,
    /// The cell holding the tab stop.
    pub(super) active: Value<Option<(K, C)>>,
    pub(super) revision: Signal<u64>,
    pub(super) at: &'static Location<'static>,
}

pub(super) fn create<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static>(
    keys: Value<Vec<K>>,
    columns: Value<Vec<Header<C>>>,
    selection: ListSelection<K>,
    at: &'static Location<'static>,
) -> Owned<State<K, C>> {
    let owner = Owner::current("table");
    let root = Owner::root("table");
    let alive = owner.store("Table", Rc::new(()));
    let label = Rc::new(RefCell::new(None::<Label<K>>));
    let labels = signal(0u64);
    let names = label.clone();
    let items = memo(move || {
        labels.get();
        let label = names.borrow().clone();
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
    let rows = use_list(items, selection);
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
    let revision = signal(0u64);
    let registry = roving.members.clone();
    let active = memo(move || {
        alive.try_get()?;
        revision.get();
        let cells = registry.borrow();
        let available = |cell: &(K, C)| cells.get(cell).is_some_and(|entry| entry.available());
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
    let sort = signal(None);
    let state = Rc::new(State {
        owner,
        root,
        alive,
        list: rows,
        keys,
        columns,
        selection,
        focus: Cell::new(TableFocus::default()),
        started: Cell::new(false),
        label,
        labels,
        sort: Cell::new(Binding::from(sort)),
        widths: Cell::new(Binding::from(widths)),
        resize: Cell::new((
            ScalarRange::new(16.0, 4096.0).expect("invariant: finite column bounds"),
            10.0,
        )),
        headers: RefCell::new(HashMap::new()),
        roving,
        active,
        revision,
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
