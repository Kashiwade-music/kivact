//! Role: ListApi state shares one registry, selection writer and focus authority.

use super::data::{ListGesture, ListItem, ListMode, ListOptions};
use super::machine;
use super::order::Order;
use super::selection::selection_transition;
use super::typeahead::Typeahead;
use crate::Selection;
use crate::options_cell::OptionsCell;
use crate::roving::{Observation, Roving};
use kivact_core::ErrorKind;
use kivact_core::custom::{NodeRefExt, Owned, Owner};
use kivact_core::state::{Binding, Callback, Signal, Value, effect, memo, signal};
use kivact_core::state::{IntoBinding, IntoValue};
use kivact_core::state::{MachineHandle, machine};
use kivact_core::view::NodeRef;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type Leave<K> = Rc<dyn Fn(&K) -> Option<K>>;
type Scroll<K> = Callback<K>;

type Selectable<K> = Rc<dyn Fn(&K) -> bool>;

#[derive(Clone)]
pub(super) struct ListSetupOptions<K: kivact_core::state::ItemKey> {
    pub(super) retained_focus: Value<bool>,
    pub(super) reset_on_exit: bool,
    pub(super) page: Value<usize>,
    pub(super) leave: Option<Leave<K>>,
    pub(super) scroll: Option<Scroll<K>>,
    pub(super) selectable: Option<Selectable<K>>,
    pub(super) anchor: Binding<Option<K>>,
}

pub(super) struct State<K: kivact_core::state::ItemKey> {
    pub(super) item_states: Signal<std::collections::HashMap<K, Value<super::data::ListItemState>>>,
    // xtask-allow(state-fields): Mounted container registry metadata, independent of focus transitions.
    pub(super) container_state: Signal<Option<Value<kivact_core::custom::PartState>>>,
    pub(super) owner: Owner,
    /// The App's root when the list was created: an item of another App has
    /// another root.
    pub(super) root: Owner,
    pub(super) items: Value<Vec<ListItem<K>>>,
    pub(super) order: Value<Order<K>>,
    pub(super) selection: Selection<K>,
    pub(super) selected: Value<Vec<K>>,
    // xtask-allow(state-fields): Read-only projection of the caller-owned range anchor.
    pub(super) anchor_value: Value<Option<K>>,
    /// The mounted items, the requested key and the focus bookkeeping.
    pub(super) roving: Roving<K>,
    // xtask-allow(state-fields): Pure derived tab stop, never independently written.
    pub(super) active: Value<Option<K>>,
    /// What choosing an item does; fixed by `use_list`.
    pub(super) mode: ListMode,
    pub(super) policy: OptionsCell<ListOptions>,
    pub(super) options: OptionsCell<ListSetupOptions<K>>,
    pub(super) typed: Cell<Binding<Typeahead>>,
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
) -> Owned<Rc<State<K>>> {
    let owner = Owner::current();
    let root = Owner::root();
    let order = memo(move || Order::new(items.get()));
    let selected_value = memo(move || selected(selection));
    let roving = Roving::<K>::new();
    let requested = roving.requested;
    let options = OptionsCell::new(ListSetupOptions {
        retained_focus: false.into_value(),
        reset_on_exit: true,
        page: 10.into_value(),
        leave: None,
        scroll: None,
        selectable: None,
        anchor: signal(None::<K>).into_binding(),
    });
    let option_values = options.value();
    let anchor_value = memo(move || option_values.get().anchor.get());
    let mode = policy.mode;
    let policy = OptionsCell::new(policy);
    let targets = roving.clone();
    let pending = machine(
        machine::State {
            pending: machine::Pending::Nothing,
            position: 0,
            focus: machine::Focus::Physical,
        },
        || machine::Machine::new(),
        |_, ()| {},
    );
    let position = memo(move || {
        requested
            .get()
            .and_then(|key| order.with(|order| order.position(&key)))
            .unwrap_or_else(|| pending.state().get().position)
    });
    let active = active_value(
        order,
        targets,
        selected_value,
        option_values,
        pending,
        mode,
        position,
    );
    let state = Rc::new(State {
        item_states: signal(std::collections::HashMap::new()),
        container_state: signal(None),
        owner,
        root,
        items,
        order,
        selection,
        selected: selected_value,
        anchor_value,
        roving,
        active,
        mode,
        policy,
        options,
        typed: Cell::new(signal(Typeahead::default()).into_binding()),
        pending,
    });
    let stored = owner.store("ListApi", state.clone());
    let duplicates = RefCell::new(Vec::<K>::new());
    effect(move || {
        if stored.try_get().is_none() {
            return;
        }
        // History follows the derived tab stop even before an explicit focus request.
        if let Some(at) = state
            .active
            .get()
            .and_then(|key| state.order.with(|order| order.position(&key)))
        {
            pending.send(machine::Event::Position(at));
        }
        let next = state.order.with(|order| order.duplicates().to_vec());
        if *duplicates.borrow() != next {
            if !next.is_empty() {
                owner.report(
                    ErrorKind::Message,
                    format!(
                        "ListApi: {} duplicate logical key(s); first occurrence wins",
                        next.len()
                    ),
                );
            }
            *duplicates.borrow_mut() = next;
        }
        if state.pending.state().get().focus == machine::Focus::Physical {
            reconcile(&state);
        }
    });
    stored
}

impl<K: kivact_core::state::ItemKey> State<K> {
    pub(super) fn retains_focus(&self) -> bool {
        self.pending.state().get_untracked().focus == machine::Focus::Retained
    }

    /// The policy as set now (untracked).
    pub(super) fn policy(&self) -> ListOptions {
        self.policy.get()
    }
    /// Whether the order lets `key` take the focus or a selection.
    fn enabled(&self, key: &K) -> bool {
        self.order.with_untracked(|order| order.enabled(key))
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
        if !self.order.with_untracked(|order| order.enabled(&key)) || !self.can_select(&key) {
            return false;
        }
        let anchor = self.options.get().anchor;
        match self.selection {
            Selection::Single(binding) => {
                binding.set(Some(key.clone()));
                anchor.set(Some(key));
            }
            Selection::Multiple(binding) => {
                let visible = self.order.with_untracked(|order| {
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
        if !self.order.with_untracked(|order| order.enabled(&key)) {
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
                let origin = self.owner.focused_node();
                let origin_ref = self.roving.with_members(|members| {
                    members
                        .values()
                        .find(|entry| origin.is_some() && entry.node.mounted_node() == origin)
                        .map(|entry| entry.node)
                });
                self.pending.send(machine::Event::Activate {
                    key: key.clone(),
                    origin: machine::Origin {
                        token: origin,
                        node: origin_ref,
                    },
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
    order: Value<Order<K>>,
    targets: Roving<K>,
    selected_value: Value<Vec<K>>,
    option_values: Value<ListSetupOptions<K>>,
    pending: MachineHandle<machine::Machine<K>>,
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
                    && pending.state().get().focus == machine::Focus::Physical
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
                .find(|key| selected.contains(key) && order.enabled(key) && available(key));
            preference.cloned().or_else(|| {
                // Keep the last tab stop's ordinal when a selected item disappears.
                // An empty selection still starts a fresh entry at the first item.
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
