//! Role: List parts bind and unregister exact mounted focus targets.

use super::data::{ListGesture, ListItemState, ListMode, ListSelection};
use super::state::{State, selected};
use crate::roving::Member;
use kivact_core::ErrorKind;
use kivact_core::custom::{NodeRefExt, Owner, Part, PartState, part};
use kivact_core::state::IntoValue;
use kivact_core::state::{memo, signal, var};
use kivact_core::view::{A11yAction, Orientation, Role};
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

pub(super) fn container<K: Clone + Eq + Hash + 'static>(state: Rc<State<K>>) -> Part<PartState> {
    let policy = state.policy;
    part(|input| input)
        .role(Role::ListBox)
        .multiselectable(matches!(state.selection, ListSelection::Multiple(_)))
        .orientation(move || {
            policy
                .with(|policy| policy.axis)
                .unwrap_or(Orientation::Unspecified)
        })
}

/// How a list item's part is bound.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ItemMount {
    Unbound,
    Admitted,
    Rejected,
}

pub(super) fn item<K: Clone + Eq + Hash + 'static>(
    state: Rc<State<K>>,
    key: K,
    at: &'static Location<'static>,
    retain_disabled_focus: bool,
) -> Part<ListItemState> {
    let key = var(key);
    let bound = signal(ItemMount::Unbound);
    let selected_state = {
        let state = state.clone();
        memo(move || key.with(|key| selected(state.selection).contains(key)))
    };
    let active = {
        let active = state.active;
        memo(move || {
            bound.get() == ItemMount::Admitted
                && key.with(|key| active.with(|active| active.as_ref() == Some(key)))
        })
    };
    let disabled = {
        let order = state.order;
        memo(move || {
            bound.get() == ItemMount::Rejected
                || key.with(|key| !order.with(|order| order.enabled(key)))
        })
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
    let mode = state.mode;
    let retained_focus = state.retained_focus.get();
    let attachment = attachment
        .role(Role::Option)
        .name(name)
        .selected(selected_state)
        .disabled(disabled)
        .focusable(move || !retained_focus && active.get())
        .focusable_when_disabled(retain_disabled_focus)
        .on_key(move |event| super::input::key(&keys, &key.get(), event, None))
        .on_pointer(move |event| super::input::pointer(&pointers, &key.get(), event));
    let invoke = move || {
        if bound.get_untracked() != ItemMount::Admitted {
            return;
        }
        let key = key.get();
        if mode == ListMode::Activate {
            actions.choose(key);
        } else if actions.target(&key).is_some() {
            actions.select(key, ListGesture::Replace);
        }
    };
    let attachment = match mode {
        ListMode::Activate => attachment.on_press(invoke),
        ListMode::Select => attachment.on_a11y(A11yAction::Select, invoke),
    };
    attachment.on_bind(move |mount| {
        let owner = Owner::current("List::item");
        // The bound state is reset when the owner goes, on every path below.
        owner.on_cleanup(move || bound.set(ItemMount::Unbound));
        if Owner::root("List::item") != state.root {
            bound.set(ItemMount::Rejected);
            owner.report(ErrorKind::WrongApp, "List item belongs to another App", at);
            return;
        }
        let item_key = key.get();
        if state.roving.contains(&item_key) {
            bound.set(ItemMount::Rejected);
            owner.report(
                ErrorKind::Message,
                "List: an item with this key is already mounted",
                state.at,
            );
            return;
        }
        state.roving.insert(
            item_key.clone(),
            Member {
                node: mount.node,
                eligible: if retained_focus {
                    true.into_value()
                } else {
                    mount.eligible
                },
                disabled: if retain_disabled_focus {
                    false.into_value()
                } else {
                    mount.disabled
                },
                hidden: mount.hidden,
            },
        );
        bound.set(ItemMount::Admitted);
        let id = node.mounted_node();
        let weak = Rc::downgrade(&state);
        owner.on_cleanup(move || {
            let Some(state) = weak.upgrade() else {
                return;
            };
            if !state.roving.remove(&item_key, node) {
                return;
            }
            let current = kivact_core::custom::focused_node();
            if id.is_some() && current == id {
                state.roving.restore_from.set(current);
            }
        });
        state.pending_mounted(&key.get());
    })
}
