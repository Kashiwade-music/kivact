//! Role: List parts bind and unregister exact mounted focus targets.

use super::data::{ListGesture, ListItemState, ListSelection};
use super::state::{State, selected};
use crate::roving::Member;
use kivact_core::diagnostics::ErrorKind;
use kivact_core::primitive::{NodeRefExt, Owner, Part, PartState, part};
use kivact_core::view::{A11yAction, Orientation, Role};
use kivact_core::{memo, signal, var};
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

pub(super) fn container<K: Clone + Eq + Hash + 'static>(state: Rc<State<K>>) -> Part<PartState> {
    state.started.set(true);
    let result = part(|input| input)
        .role(Role::ListBox)
        .multiselectable(matches!(state.selection, ListSelection::Multiple(_)));
    match state.policy.get().axis {
        Some(Orientation::Horizontal) => result.orientation(Orientation::Horizontal),
        Some(Orientation::Vertical) => result.orientation(Orientation::Vertical),
        Some(Orientation::Unspecified) | None => result,
    }
}

pub(super) fn item<K: Clone + Eq + Hash + 'static>(
    state: Rc<State<K>>,
    key: K,
    at: &'static Location<'static>,
) -> Part<ListItemState> {
    state.started.set(true);
    let key = var(key);
    let admitted = signal(false);
    let rejected = signal(false);
    let selected_state = {
        let state = state.clone();
        memo(move || key.with(|key| selected(state.selection).contains(key)))
    };
    let active = {
        let active = state.active;
        memo(move || {
            admitted.get() && key.with(|key| active.with(|active| active.as_ref() == Some(key)))
        })
    };
    let disabled = {
        let order = state.order;
        memo(move || rejected.get() || key.with(|key| !order.with(|order| order.enabled(key))))
    };
    let name = {
        let items = state.items;
        memo(move || {
            key.with(|key| {
                items.with(|items| {
                    items
                        .iter()
                        .find(|item| &item.key == key)
                        .map(|item| item.label.get())
                        .unwrap_or_default()
                })
            })
        })
    };
    let attachment = part(move |input| ListItemState {
        input,
        selected: selected_state.get(),
        active: active.get(),
    });
    let node = attachment.node_ref();
    let keys = state.clone();
    let pointers = state.clone();
    let actions = state.clone();
    let policy = state.policy.get();
    let attachment = attachment
        .role(Role::Option)
        .name(name)
        .selected(selected_state)
        .disabled(disabled)
        .focusable(active)
        .on_key(move |event| super::input::key(&keys, &key.get(), event))
        .on_pointer(move |event| super::input::pointer(&pointers, &key.get(), event));
    let invoke = move || {
        if !admitted.get_untracked() || rejected.get_untracked() {
            return;
        }
        let key = key.get();
        if policy.action == A11yAction::Activate {
            actions.choose(key);
        } else if actions.target(&key).is_some() {
            actions.select(key, ListGesture::Replace);
        }
    };
    let attachment = if policy.action == A11yAction::Activate {
        attachment.on_press(invoke)
    } else {
        attachment.on_a11y(policy.action, invoke)
    };
    attachment.on_bind(move |mount| {
        let owner = Owner::current("List::item");
        if Owner::root("List::item") != state.root {
            rejected.set(true);
            owner.report(ErrorKind::WrongApp, "List item belongs to another App", at);
            return;
        }
        let item_key = key.get();
        if state.roving.members.borrow().contains_key(&item_key) {
            rejected.set(true);
            owner.report(
                ErrorKind::Message,
                "List: an item with this key is already mounted",
                state.at,
            );
            return;
        }
        state.roving.members.borrow_mut().insert(
            item_key.clone(),
            Member {
                node: mount.node,
                eligible: mount.eligible,
                disabled: mount.disabled,
                hidden: mount.hidden,
            },
        );
        admitted.set(true);
        state.revision.update(|revision| *revision += 1);
        let id = node.mounted_node();
        let weak = Rc::downgrade(&state);
        owner.on_cleanup(move || {
            let Some(state) = weak.upgrade() else {
                return;
            };
            if !state
                .roving
                .members
                .borrow()
                .get(&item_key)
                .is_some_and(|entry| entry.node.same_ref(node))
            {
                return;
            }
            let current = kivact_core::primitive::focused_node();
            if id.is_some() && current == id {
                state.roving.restore_from.set(current);
            }
            state.roving.members.borrow_mut().remove(&item_key);
            if state.alive.try_get().is_some() {
                if admitted.try_get().is_some() {
                    admitted.set(false);
                }
                state.revision.update(|revision| *revision += 1);
            }
        });
        state.pending_mounted(&key.get());
    })
}
