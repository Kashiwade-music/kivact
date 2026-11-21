//! Role: ListApi state shares one registry, selection writer and focus authority.

use super::data::{ListGesture, ListItem, ListMode, ListOptions};
use super::machine;
use super::selection::selection_transition;
use super::typeahead::Typeahead;
use crate::Selection;
use crate::collection::{Index, Membership};
use crate::options_cell::OptionsCell;
use crate::roving::{Observation, Roving};
use kivact_core::ErrorKind;
use kivact_core::custom::{NodeRefExt, Owned, Owner, Sent};
use kivact_core::state::IntoValue;
use kivact_core::state::{Callback, Selector, Signal, Value, effect, memo, selector, signal};
use kivact_core::state::{MachineHandle, machine};
use kivact_core::view::NodeRef;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

type Leave<K> = Rc<dyn Fn(&K) -> Option<K>>;
type Scroll<K> = Callback<K>;

type Selectable<K> = Rc<dyn Fn(&K) -> bool>;

#[derive(Clone)]
pub(super) struct ListSetupOptions<K: kivact_core::state::ItemKey> {
    pub(super) reset_on_exit: bool,
    pub(super) page: Value<usize>,
    pub(super) leave: Option<Leave<K>>,
    pub(super) scroll: Option<Scroll<K>>,
    pub(super) selectable: Option<Selectable<K>>,
    pub(super) anchor: Signal<Option<K>>,
    /// The list whose type-to-find prefix this one types into (a select's
    /// open list types into its closed trigger's); `None` is its own.
    pub(super) typeahead: Option<MachineHandle<machine::Machine<K>>>,
}

pub(super) struct State<K: kivact_core::state::ItemKey> {
    pub(super) owner: Owner,
    /// The App's root when the list was created: an item of another App has
    /// another root.
    pub(super) root: Owner,
    pub(super) items: Value<Vec<ListItem<K>>>,
    pub(super) index: Value<Index<K>>,
    /// The number of distinct items; one memo shared by every item.
    pub(super) set_size: Value<u32>,
    /// Which keys the selection holds, per key.
    pub(super) membership: Membership<K>,
    /// Whether a key is the active one, per key.
    pub(super) active_key: Selector<K>,
    pub(super) selection: Selection<K>,
    pub(super) selected: Value<Vec<K>>,
    pub(super) anchor_value: Value<Option<K>>,
    /// The mounted items, the requested key and the focus bookkeeping.
    pub(super) roving: Roving<K>,
    pub(super) active: Value<Option<K>>,
    /// What choosing an item does; fixed by `use_list`.
    pub(super) mode: ListMode,
    pub(super) policy: OptionsCell<ListOptions>,
    pub(super) options: OptionsCell<ListSetupOptions<K>>,
    /// Whether the items leave the focus on another node (a select's
    /// trigger keeps it); fixed at construction, reactive in its value.
    pub(super) retained_focus: Value<bool>,
    pub(super) pending: MachineHandle<machine::Machine<K>>,
}

pub(super) fn selected<K: kivact_core::state::PropValue>(selection: Selection<K>) -> Vec<K> {
    match selection {
        Selection::Single(binding) => binding.get().into_iter().collect(),
        Selection::Multiple(binding) => binding.get(),
    }
}

pub(super) fn create<K: kivact_core::state::ItemKey>(
    items: Value<Vec<ListItem<K>>>,
    selection: Selection<K>,
    policy: ListOptions,
    retained_focus: Value<bool>,
) -> Owned<Rc<State<K>>> {
    let owner = Owner::current();
    let root = Owner::root();
    let index = memo(move || items.with(|items| Index::of_items(items)));
    let set_size =
        memo(move || index.with(|index| u32::try_from(index.count()).unwrap_or(u32::MAX)));
    let selected_value = memo(move || selected(selection));
    let membership = Membership::new(selection);
    let roving = Roving::<K>::new();
    let requested = roving.requested;
    let options = default_setup_options::<K>();
    let option_values = options.value();
    let anchor_value = memo(move || option_values.get().anchor.get());
    let mode = policy.mode;
    let policy = OptionsCell::new(policy);
    let targets = roving.clone();
    let pending = machine(
        machine::State {
            pending: machine::Pending::Nothing,
            typed: Typeahead::Idle,
        },
        || machine::Machine::new(),
        |_, ()| {},
    );
    // Where a search for the tab stop starts: the requested row's ordinal in
    // the current order, else the first row. A pure projection; no history.
    let position = memo(move || {
        requested
            .get()
            .and_then(|key| index.with(|index| index.position(&key)))
            .unwrap_or(0)
    });
    let active = active_value(
        index,
        targets,
        membership.keys(),
        option_values,
        retained_focus,
        mode,
        position,
    );
    let active_key = selector(active);
    let state = Rc::new(State {
        owner,
        root,
        items,
        index,
        set_size,
        membership,
        active_key,
        selection,
        selected: selected_value,
        anchor_value,
        roving,
        active,
        mode,
        policy,
        options,
        retained_focus,
        pending,
    });
    let stored = owner.store("ListApi", state.clone());
    observe_list(state, stored);
    stored
}

fn default_setup_options<K: kivact_core::state::ItemKey>() -> OptionsCell<ListSetupOptions<K>> {
    OptionsCell::new(ListSetupOptions {
        reset_on_exit: true,
        page: 10.into_value(),
        leave: None,
        scroll: None,
        selectable: None,
        anchor: signal(None::<K>),
        typeahead: None,
    })
}

impl<K: kivact_core::state::ItemKey> State<K> {
    pub(super) fn retains_focus(&self) -> bool {
        self.retained_focus.get_untracked()
    }

    /// The machine that holds the type-to-find prefix this list types
    /// into: its own, or the one given to `ListApi::typeahead_from`.
    pub(super) fn typeahead_owner(&self) -> MachineHandle<machine::Machine<K>> {
        self.options.get().typeahead.unwrap_or(self.pending)
    }

    /// The policy as set now (untracked).
    pub(super) fn policy(&self) -> ListOptions {
        self.policy.get()
    }
    /// Whether the order lets `key` take the focus or a selection.
    fn enabled(&self, key: &K) -> bool {
        self.index.with_untracked(|order| order.enabled(key))
    }
    pub(super) fn target(&self, key: &K) -> Option<NodeRef> {
        self.roving.target(key, |key| self.enabled(key))
    }
    pub(super) fn can_select(&self, key: &K) -> bool {
        self.options
            .get()
            .selectable
            .as_ref()
            .is_none_or(|test| test(key))
    }

    pub(super) fn select(&self, key: K, gesture: ListGesture) -> bool {
        if !self.index.with_untracked(|order| order.enabled(&key)) || !self.can_select(&key) {
            return false;
        }
        let anchor = self.options.get().anchor;
        match self.selection {
            Selection::Single(binding) => {
                binding.set(Some(key.clone()));
                anchor.set(Some(key));
            }
            Selection::Multiple(binding) => {
                let visible = self.index.with_untracked(|order| {
                    order
                        .keys()
                        .into_iter()
                        .filter(|key| order.enabled(key) && self.can_select(key))
                        .collect::<Vec<_>>()
                });
                let next = kivact_core::state::untrack(|| {
                    selection_transition(
                        &selected(self.selection),
                        anchor.get_untracked().as_ref(),
                        &visible,
                        &key,
                        gesture,
                        self.policy().range.missing_anchor,
                    )
                });
                binding.set(next.selected);
                anchor.set(next.anchor);
            }
        }
        true
    }
    pub(super) fn focus_available(&self, key: &K) -> bool {
        self.target(key).is_some()
            || (!self.retains_focus()
                && self.options.get().scroll.is_some()
                && !self.roving.contains(key))
    }
    pub(super) fn focus(&self, key: K) -> bool {
        if !self.index.with_untracked(|order| order.enabled(&key)) {
            return false;
        }
        let scroll = self.options.get().scroll;
        let target = self.target(&key);
        if !self.focus_available(&key) {
            return false;
        }
        self.roving.requested.set(Some(key.clone()));
        self.pending.send(machine::Event::Cancel);
        match target {
            Some(node) => {
                if !self.retains_focus() {
                    node.focus();
                }
                match scroll {
                    Some(scroll) => scroll.call(key),
                    None => node.scroll_into_view(),
                }
            }
            None => {
                let origin = match self.owner.focused_node() {
                    None => machine::Origin::Nowhere,
                    Some(token) => self
                        .roving
                        .with_members(|members| {
                            members
                                .values()
                                .find(|entry| entry.node.mounted_node() == Some(token))
                                .map(|entry| entry.node)
                        })
                        .map_or(machine::Origin::Outside(token), |node| {
                            machine::Origin::Member { token, node }
                        }),
                };
                self.pending.send(machine::Event::Activate {
                    key: key.clone(),
                    origin,
                });
                if let Some(scroll) = scroll {
                    scroll.call(key);
                }
            }
        }
        true
    }
    pub(super) fn choose(&self, key: K) -> bool {
        if self.target(&key).is_none() {
            return false;
        }
        self.select(key.clone(), ListGesture::Replace);
        self.focus(key)
    }
    pub(super) fn holds_focus(&self, key: &K) -> bool {
        self.roving.with_members(|members| {
            members
                .get(key)
                .is_some_and(|entry| entry.node.holds_focus())
        })
    }
    pub(super) fn pending_mounted(&self, key: &K) {
        let pending = self.pending.state().get_untracked().pending;
        let machine::Pending::Activating {
            key: pending_key,
            origin,
        } = pending
        else {
            return;
        };
        if &pending_key != key {
            return;
        }
        let current = self.owner.focused_node();
        if !origin.kept(current) {
            self.pending.send(machine::Event::Cancel);
            return;
        }
        if let Some(node) = self.target(key) {
            self.pending.send(machine::Event::Cancel);
            self.roving.requested.set(Some(key.clone()));
            node.focus();
            node.scroll_into_view();
        }
    }
}

fn reconcile<K: kivact_core::state::ItemKey>(state: &State<K>) {
    let active = state.active.get();
    let current = state.owner.focused_node();
    state.pending.send(machine::Event::Observe(current));
    let pending = state.pending.state().get_untracked().pending;
    // Once focus leaves, the next Tab entry starts at the selected item.
    // Pending virtualization and retained trigger focus keep their own authority.
    if state.options.get().reset_on_exit
        && matches!(pending, machine::Pending::Nothing)
        && state.roving.left_members(current)
    {
        state.roving.requested.set(None);
    }
    // A pending request of the list's own keeps the tab stop where it asked.
    // A focus on an item the list refuses goes back to the tab stop.
    let observation = Observation {
        current,
        active: active.as_ref(),
        adopt: matches!(pending, machine::Pending::Nothing),
        reject_refused: true,
    };
    state
        .roving
        .reconcile(observation, |key| state.enabled(key));
}

fn active_value<K: kivact_core::state::ItemKey>(
    order: Value<Index<K>>,
    targets: Roving<K>,
    selected_value: Value<HashSet<K>>,
    option_values: Value<ListSetupOptions<K>>,
    retained: Value<bool>,
    mode: ListMode,
    position: Value<usize>,
) -> Value<Option<K>> {
    let requested = targets.requested;
    memo(move || {
        order.try_get()?;
        targets.track();
        let requested = requested.get();
        let selected = selected_value.get();
        order.with(|order| {
            let available = |key: &K| {
                targets
                    .with_members(|targets| targets.get(key).is_some_and(|entry| entry.available()))
            };
            if let Some(key) = requested.as_ref() {
                // A caller scroll callback can materialize a requested logical row.
                // Keep that destination authoritative until its node mounts.
                let virtual_request = option_values.get().scroll.is_some()
                    && !retained.get()
                    && !targets.contains(key);
                if order.enabled(key) && (available(key) || virtual_request) {
                    return Some(key.clone());
                }
                if mode == ListMode::Select {
                    let moved = option_values
                        .get()
                        .leave
                        .as_ref()
                        .and_then(|leave| leave(key));
                    if let Some(moved) = moved.filter(|key| order.enabled(key) && available(key)) {
                        return Some(moved);
                    }
                    let keys = order.keys();
                    return keys
                        .iter()
                        .skip(position.get())
                        .find(|key| order.enabled(key) && available(key))
                        .cloned()
                        .or_else(|| {
                            keys.iter()
                                .rev()
                                .find(|key| order.enabled(key) && available(key))
                                .cloned()
                        });
                }
            }
            let keys = order.keys();
            let preference = keys
                .iter()
                .find(|key| selected.contains(*key) && order.enabled(key) && available(key));
            preference.cloned().or_else(|| {
                // Start at the requested row's ordinal while that row is listed; otherwise at the first row.
                let start = if mode == ListMode::Select && !selected.is_empty() {
                    position.get()
                } else {
                    0
                };
                keys.iter()
                    .skip(start)
                    .find(|key| order.enabled(key) && available(key))
                    .or_else(|| {
                        keys.iter()
                            .rev()
                            .find(|key| order.enabled(key) && available(key))
                    })
                    .cloned()
            })
        })
    })
}

fn observe_list<K: kivact_core::state::ItemKey>(state: Rc<State<K>>, stored: Owned<Rc<State<K>>>) {
    let owner = state.owner;
    let duplicates = RefCell::new(Sent::<Vec<K>>::new());
    effect(move || {
        if stored.try_get().is_none() {
            return;
        }
        let next = state.index.with(|order| order.duplicates().to_vec());
        let changed = duplicates.borrow_mut().update(next).map(Vec::len);
        if let Some(count) = changed
            && count > 0
        {
            owner.report(
                ErrorKind::Message,
                format!("ListApi: {count} duplicate logical key(s); first occurrence wins"),
            );
        }
        if !state.retained_focus.get() {
            reconcile(&state);
        }
    });
}
