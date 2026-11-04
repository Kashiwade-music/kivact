//! Role: ListApi state shares one registry, selection writer and focus authority.

use super::data::{ListGesture, ListItem, ListMode, ListOptions};
use super::order::Order;
use super::selection::selection_transition;
use super::typeahead::Typeahead;
use crate::Selection;
use crate::roving::{Observation, Roving};
use kivact_core::ErrorKind;
use kivact_core::custom::{NodeRefExt, NodeToken, Owned, Owner};
use kivact_core::state::{Binding, Callback, Signal, Value, effect, memo, signal};
use kivact_core::state::{IntoBinding, IntoValue};
use kivact_core::view::NodeRef;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type Leave<K> = Rc<dyn Fn(&K) -> Option<K>>;
type Scroll<K> = Callback<K>;

#[derive(Clone)]
struct Pending<K> {
    key: K,
    origin: Option<NodeToken>,
    origin_ref: Option<NodeRef>,
}

/// Losing the source through virtualization is distinct from a bound-source blur.
fn pending_kept<K>(pending: &Pending<K>, current: Option<NodeToken>) -> bool {
    if current == pending.origin {
        return true;
    }
    current.is_none()
        && pending.origin.is_some()
        && pending
            .origin_ref
            .is_some_and(|node| node.mounted_node().is_none())
}

type Selectable<K> = Rc<dyn Fn(&K) -> bool>;

pub(super) struct State<K: kivact_core::state::ItemKey> {
    pub(super) item_states: Signal<std::collections::HashMap<K, Value<super::data::ListItemState>>>,
    pub(super) container_state: Signal<Option<Value<kivact_core::custom::PartState>>>,
    pub(super) owner: Owner,
    /// The App's root when the list was created: an item of another App has
    /// another root.
    pub(super) root: Owner,
    pub(super) items: Value<Vec<ListItem<K>>>,
    pub(super) order: Value<Order<K>>,
    pub(super) selection: Selection<K>,
    pub(super) selectable: Signal<Option<Selectable<K>>>,
    pub(super) selected: Value<Vec<K>>,
    pub(super) anchor_value: Value<Option<K>>,
    pub(super) anchor: Signal<Binding<Option<K>>>,
    /// The mounted items, the requested key and the focus bookkeeping.
    pub(super) roving: Roving<K>,
    pub(super) active: Value<Option<K>>,
    /// What choosing an item does; fixed by `use_list`.
    pub(super) mode: ListMode,
    pub(super) retained_focus: Signal<bool>,
    pub(super) policy: Signal<ListOptions>,
    pub(super) page: Cell<Value<usize>>,
    pub(super) leave: Rc<RefCell<Option<Leave<K>>>>,
    pub(super) scroll: RefCell<Option<Scroll<K>>>,
    pub(super) typed: Cell<Binding<Typeahead>>,
    pending: RefCell<Option<Pending<K>>>,
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
    let alive = owner.store("ListApi", ());
    let order = memo(move || Order::new(items.get()));
    let selected_value = memo(move || selected(selection));
    let roving = Roving::<K>::new();
    let requested = roving.requested;
    let anchor = signal(signal(None::<K>).into_binding());
    let anchor_value = memo(move || anchor.get().get());
    let mode = policy.mode;
    let policy = signal(policy);
    let targets = roving.clone();
    let leave = Rc::new(RefCell::new(None::<Leave<K>>));
    let leave_active = leave.clone();
    let position = Cell::new(0usize);
    let active = memo(move || {
        alive.try_get()?;
        targets.track();
        let requested = requested.get();
        let selected = selected_value.get();
        order.with(|order| {
            let available = |key: &K| {
                targets
                    .with_members(|targets| targets.get(key).is_some_and(|entry| entry.available()))
            };
            if let Some(key) = requested.as_ref() {
                if order.enabled(key) && available(key) {
                    if let Some(at) = order.position(key) {
                        position.set(at);
                    }
                    return Some(key.clone());
                }
                if mode == ListMode::Select {
                    let moved = leave_active.borrow().as_ref().and_then(|leave| leave(key));
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
            let preference = if mode == ListMode::Select {
                selected
                    .iter()
                    .rev()
                    .find(|key| order.enabled(key) && available(key))
            } else {
                selected
                    .iter()
                    .find(|key| order.enabled(key) && available(key))
            };
            preference.cloned().or_else(|| {
                order
                    .keys()
                    .into_iter()
                    .find(|key| order.enabled(key) && available(key))
            })
        })
    });
    let state = Rc::new(State {
        item_states: signal(std::collections::HashMap::new()),
        container_state: signal(None),
        owner,
        root,
        items,
        order,
        selection,
        selectable: signal(None),
        selected: selected_value,
        anchor,
        anchor_value,
        roving,
        active,
        mode,
        retained_focus: signal(false),
        policy,
        page: Cell::new((10).into_value()),
        leave,
        scroll: RefCell::new(None),
        typed: Cell::new(signal(Typeahead::default()).into_binding()),
        pending: RefCell::new(None),
    });
    let stored = owner.store("ListApi", state.clone());
    let duplicates = RefCell::new(Vec::<K>::new());
    effect(move || {
        if alive.try_get().is_none() {
            return;
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
        if !state.retained_focus.get() {
            reconcile(&state);
        }
    });
    stored
}

impl<K: kivact_core::state::ItemKey> State<K> {
    /// The policy as set now (untracked).
    pub(super) fn policy(&self) -> ListOptions {
        self.policy.get_untracked()
    }
    /// Whether the order lets `key` take the focus or a selection.
    fn enabled(&self, key: &K) -> bool {
        self.order.with_untracked(|order| order.enabled(key))
    }
    pub(super) fn target(&self, key: &K) -> Option<NodeRef> {
        self.roving.target(key, |key| self.enabled(key))
    }
    pub(super) fn can_select(&self, key: &K) -> bool {
        self.selectable
            .with_untracked(|test| test.as_ref().is_none_or(|test| test(key)))
    }

    pub(super) fn select(&self, key: K, gesture: ListGesture) -> bool {
        if !self.order.with_untracked(|order| order.enabled(&key)) || !self.can_select(&key) {
            return false;
        }
        let anchor = self.anchor.get_untracked();
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
            || (!self.retained_focus.get()
                && self.scroll.borrow().is_some()
                && !self.roving.contains(key))
    }
    pub(super) fn focus(&self, key: K) -> bool {
        if !self.order.with_untracked(|order| order.enabled(&key)) {
            return false;
        }
        let scroll = *self.scroll.borrow();
        let target = self.target(&key);
        if !self.focus_available(&key) {
            return false;
        }
        self.roving.requested.set(Some(key.clone()));
        *self.pending.borrow_mut() = None;
        match target {
            Some(node) => {
                if !self.retained_focus.get() {
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
                *self.pending.borrow_mut() = Some(Pending {
                    key: key.clone(),
                    origin,
                    origin_ref,
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
        let pending = self.pending.borrow().clone();
        let Some(pending) = pending else {
            return;
        };
        if &pending.key != key {
            return;
        }
        let current = self.owner.focused_node();
        if !pending_kept(&pending, current) {
            self.pending.borrow_mut().take();
            return;
        }
        if let Some(node) = self.target(key) {
            self.pending.borrow_mut().take();
            self.roving.requested.set(Some(key.clone()));
            node.focus();
            node.scroll_into_view();
        }
    }
}

fn reconcile<K: kivact_core::state::ItemKey>(state: &State<K>) {
    let active = state.active.get();
    let current = state.owner.focused_node();
    if state
        .pending
        .borrow()
        .as_ref()
        .is_some_and(|pending| !pending_kept(pending, current))
    {
        state.pending.borrow_mut().take();
    }
    if state.pending.borrow().is_none() && state.roving.requested.get_untracked() != active {
        state.roving.requested.set(active.clone());
    }
    // A pending request of the list's own keeps the tab stop where it asked.
    // A focus on an item the list refuses goes back to the tab stop.
    let observation = Observation {
        current,
        active: active.as_ref(),
        adopt: state.pending.borrow().is_none(),
        reject_refused: true,
    };
    state
        .roving
        .reconcile(observation, |key| state.enabled(key));
}
