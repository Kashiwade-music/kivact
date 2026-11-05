//! Role: ListApi parts bind and unregister exact mounted focus targets.

use super::data::ListItemState;
use super::machine;
use super::state::{State, selected};
use crate::Selection;
use crate::roving::Member;
use kivact_core::ErrorKind;
use kivact_core::custom::{NodeRefExt, Owner, Part, PartState, part};
use kivact_core::state::IntoValue;
use kivact_core::state::{Signal, memo, signal, var};
use kivact_core::view::{Orientation, Role};
use std::rc::Rc;

pub(super) fn container<K: kivact_core::state::ItemKey>(state: Rc<State<K>>) -> Part<PartState> {
    let policy = state.policy.value();
    let input = part(|input| input);
    let part_state = input.state();
    state.container_state.set(Some(part_state));
    let target = kivact_core::view::node_ref();
    Part::new(part_state, move |host| {
        host.part(input)
            .node_ref(target)
            .role(Role::ListBox)
            .multiselectable(matches!(state.selection, Selection::Multiple(_)))
            .orientation(move || {
                policy
                    .with(|policy| policy.axis)
                    .unwrap_or(Orientation::Unspecified)
            })
    })
}

/// How a list item's part is bound.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ItemMount {
    Unbound,
    Admitted,
    Rejected,
}

pub(super) fn item<K: kivact_core::state::ItemKey>(
    state: Rc<State<K>>,
    key: K,
    retain_disabled_focus: bool,
) -> (Part<ListItemState>, Signal<ItemMount>) {
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
    let order = state.order;
    let set_size =
        memo(move || order.with(|order| u32::try_from(order.keys().len()).unwrap_or(u32::MAX)));
    let position = memo(move || {
        key.with(|key| {
            order.with(|order| {
                order
                    .position(key)
                    .map_or(0, |index| u32::try_from(index + 1).unwrap_or(u32::MAX))
            })
        })
    });
    let input = part(move |input| ListItemState {
        input,
        selected: selected_state.get(),
        active: active.get(),
    });
    let part_state = input.state();
    let observed = state.item_states;
    observed.update(|states| {
        states.insert(key.get(), part_state);
    });
    kivact_core::state::on_cleanup(move || {
        if observed.try_get_untracked().is_some() {
            observed.update(|states| {
                let item_key = key.get();
                if states.get(&item_key) == Some(&part_state) {
                    states.remove(&item_key);
                }
            });
        }
    });
    let attachment = Part::new(part_state, move |host| {
        let attachment = host.part(input);
        let keys = state.clone();
        let pointers = state.clone();
        let focus_mode = state.pending;
        let attachment = attachment
            .role(Role::Option)
            .name(name)
            .selected(selected_state)
            .set_size(set_size)
            .pos_in_set(position)
            .disabled(disabled)
            .focusable(move || {
                focus_mode.state().get().focus == machine::Focus::Physical && active.get()
            })
            .focusable_when_disabled(retain_disabled_focus)
            .on_key(move |event| super::input::key(&keys, &key.get(), event, None))
            .on_pointer(move |event| super::input::pointer(&pointers, &key.get(), event));
        attachment.on_attach(move |mount| {
            mount_item(state.clone(), key, bound, retain_disabled_focus, mount);
        })
    });
    (attachment, bound)
}

fn mount_item<K: kivact_core::state::ItemKey>(
    state: Rc<State<K>>,
    key: kivact_core::state::Var<K>,
    bound: Signal<ItemMount>,
    retain_disabled_focus: bool,
    mount: kivact_core::custom::PartMount,
) {
    // Registry removal requires the same reference identity used at insertion.
    let node = mount.node;
    let owner = Owner::current();
    // The bound state is reset when the owner goes, on every path below.
    owner.on_cleanup(move || bound.set(ItemMount::Unbound));
    if Owner::root() != state.root {
        bound.set(ItemMount::Rejected);
        owner.report(ErrorKind::WrongApp, "ListApi item belongs to another App");
        return;
    }
    let item_key = key.get();
    if state.roving.contains(&item_key) {
        bound.set(ItemMount::Rejected);
        owner.report(
            ErrorKind::Message,
            "ListApi: an item with this key is already mounted",
        );
        return;
    }
    // Resolve structural focus only after validating the mounting App.
    let retained_focus = state.options.get().retained_focus.get_untracked();
    let focus = if retained_focus {
        machine::Focus::Retained
    } else {
        machine::Focus::Physical
    };
    state.pending.send(machine::Event::MountFocus(focus));
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
}
