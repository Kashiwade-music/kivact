//! Role: ListApi parts bind and unregister exact mounted focus targets.

use super::data::ListItemState;
use super::state::State;
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
    crate::part_slot::attach_part_state(state.container_state, part_state);
    Part::new(part_state, move |host| {
        host.part(input)
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
    let watch = state.membership.watch(key.clone());
    let key = var(key);
    let bound = signal(ItemMount::Unbound);
    let (membership, active_key, index, items) =
        (state.membership, state.active_key, state.index, state.items);
    // The selection and the active key are read per key: a change notifies
    // only the items whose answer changed.
    let selected = move || key.with(|key| membership.contains(key, watch));
    let active = move || bound.get() == ItemMount::Admitted && key.with(|key| active_key.is(key));
    let name = memo(move || {
        let at = key.with(|key| index.with(|index| index.source(key)));
        at.and_then(|at| items.with(|items| items.get(at).map(|item| item.label.get())))
            .unwrap_or_default()
    });
    let input = part(move |input| ListItemState {
        input,
        selected: selected(),
        active: active(),
    });
    let part_state = input.state();
    observe_item(&state, key, part_state);
    let set_size = state.set_size;
    let attachment = Part::new(part_state, move |host| {
        let attachment = host.part(input);
        let keys = state.clone();
        let pointers = state.clone();
        let retained = state.retained_focus;
        let attachment = attachment
            .role(Role::Option)
            .name(name)
            .selected(selected)
            .set_size(set_size)
            .pos_in_set(move || {
                key.with(|key| {
                    index.with(|index| {
                        index
                            .position(key)
                            .map_or(0, |at| u32::try_from(at + 1).unwrap_or(u32::MAX))
                    })
                })
            })
            .disabled(move || {
                bound.get() == ItemMount::Rejected
                    || key.with(|key| !index.with(|index| index.enabled(key)))
            })
            .focusable(move || !retained.get() && active())
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
    let retained_focus = state.retained_focus.get_untracked();
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

fn observe_item<K: kivact_core::state::ItemKey>(
    state: &State<K>,
    key: kivact_core::state::Var<K>,
    part_state: kivact_core::state::Value<ListItemState>,
) {
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
}
