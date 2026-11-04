//! Role: List input enters the shared selection and navigation state.

use super::data::{ListActivation, ListGesture, ListMode, ListSelection};
use super::order::Movement;
use super::state::{State, selected};
use super::typeahead::typeahead_find;
use kivact_core::view::{
    ButtonPress, Handled, Key, KeyEvent, Modifiers, MouseButton, Orientation, PointerEvent,
    PointerPhase,
};
use std::hash::Hash;

pub(super) fn gesture<K: Clone + Eq + Hash + 'static>(
    state: &State<K>,
    modifiers: Modifiers,
) -> ListGesture {
    if matches!(state.selection, ListSelection::Single(_)) {
        return ListGesture::Replace;
    }
    ListGesture::from_modifiers(modifiers, &state.policy())
}

pub(super) fn pointer<K: Clone + Eq + Hash + 'static>(
    state: &State<K>,
    key: &K,
    event: PointerEvent,
) -> Handled {
    if state.mode != ListMode::Select
        || !matches!(
            event.phase(),
            PointerPhase::Down(ButtonPress {
                button: MouseButton::Primary,
                ..
            })
        )
        || state.target(key).is_none()
    {
        return Handled::No;
    }
    state.select(key.clone(), gesture(state, event.modifiers()));
    let node = state.target(key);
    if node.is_some_and(|node| {
        use kivact_core::custom::NodeRefExt;
        node.holds_focus_within()
    }) {
        state.roving.requested.set(Some(key.clone()));
    } else {
        state.focus(key.clone());
    }
    Handled::No
}

/// Where an arrow, Home or End moves. In a right-to-left subtree
/// ([`current_direction`](kivact_core::view::current_direction)) the rows of
/// a horizontal order run from the right, so ArrowLeft moves to the next
/// row and ArrowRight to the previous one.
fn movement(
    key: Key,
    modifiers: Modifiers,
    axis: Option<Orientation>,
    mode: ListMode,
) -> Option<Movement> {
    if mode == ListMode::Activate && (modifiers.ctrl || modifiers.alt || modifiers.meta) {
        return None;
    }
    let rtl = kivact_core::view::current_direction() == kivact_core::view::ReadingDirection::Rtl;
    let (right, left) = if rtl {
        (Movement::Previous, Movement::Next)
    } else {
        (Movement::Next, Movement::Previous)
    };
    match key {
        Key::ArrowRight if axis != Some(Orientation::Vertical) => Some(right),
        Key::ArrowLeft if axis != Some(Orientation::Vertical) => Some(left),
        Key::ArrowDown if axis != Some(Orientation::Horizontal) => Some(Movement::Next),
        Key::ArrowUp if axis != Some(Orientation::Horizontal) => Some(Movement::Previous),
        Key::Home => Some(Movement::First),
        Key::End => Some(Movement::Last),
        _ => None,
    }
}

pub(super) fn key<K: Clone + Eq + Hash + 'static>(
    state: &State<K>,
    from: &K,
    event: KeyEvent,
    command: Option<Key>,
) -> Handled {
    if (!state.retained_focus.get() && !state.holds_focus(from)) || state.target(from).is_none() {
        return Handled::No;
    }
    let command = command.unwrap_or_else(|| event.key());
    let policy = state.policy();
    let modifiers = event.modifiers();
    if state.mode == ListMode::Select {
        match command {
            Key::Space => {
                let gesture = gesture(state, modifiers);
                if gesture != ListGesture::Replace
                    || !kivact_core::untrack(|| selected(state.selection).contains(from))
                {
                    state.select(from.clone(), gesture);
                }
                return Handled::Yes;
            }
            Key::Char(c)
                if c.eq_ignore_ascii_case(&'a')
                    && !modifiers.shift
                    && gesture(state, modifiers) == ListGesture::Toggle =>
            {
                select_all(state);
                return Handled::Yes;
            }
            _ => {}
        }
    }
    let jump = movement(command, modifiers, policy.axis, state.mode);
    let page = if state.mode == ListMode::Select {
        match command {
            Key::PageUp => Some(false),
            Key::PageDown => Some(true),
            _ => None,
        }
    } else {
        None
    };
    if jump.is_some() || page.is_some() {
        let next = destination(state, Some(from), jump, page, false);
        let Some(next) = next else {
            return Handled::No;
        };
        // Select's clamped boundary is consumed without another selection write.
        if state.mode == ListMode::Select && &next == from {
            return Handled::Yes;
        }
        if policy.activation == ListActivation::Automatic {
            if state.mode == ListMode::Activate {
                state.choose(next);
            } else {
                let requested = gesture(state, modifiers);
                if requested != ListGesture::Toggle {
                    state.select(next.clone(), requested);
                }
                state.focus(next);
            }
        } else {
            state.focus(next);
        }
        return Handled::Yes;
    }
    key_typeahead(state, from, &event, false)
}

/// Share printable-key eligibility and search without exposing raw input publicly.
pub(super) fn key_typeahead<K: Clone + Eq + Hash + 'static>(
    state: &State<K>,
    from: &K,
    event: &KeyEvent,
    consume_unmatched: bool,
) -> Handled {
    if (!state.retained_focus.get() && !state.holds_focus(from)) || state.target(from).is_none() {
        return Handled::No;
    }
    let modifiers = event.modifiers();
    if let Key::Char(c) = event.key()
        && !c.is_control()
        && !modifiers.ctrl
        && !modifiers.alt
        && !modifiers.meta
        && state.policy().typeahead
    {
        let handled = typed(state, Some(from), c, false);
        return if consume_unmatched {
            Handled::Yes
        } else {
            handled
        };
    }
    Handled::No
}

fn destination<K: Clone + Eq + Hash + 'static>(
    state: &State<K>,
    from: Option<&K>,
    movement: Option<Movement>,
    page: Option<bool>,
    logical: bool,
) -> Option<K> {
    let available = |key: &K| logical || state.focus_available(key);
    state.order.with_untracked(|order| {
        if let Some(movement) = movement {
            return order.navigate(from, movement, state.policy().boundary, available);
        }
        let forward = page?;
        let keys = order
            .keys()
            .into_iter()
            .filter(|key| order.enabled(key) && available(key))
            .collect::<Vec<_>>();
        let last = keys.len().checked_sub(1)?;
        let at = from
            .and_then(|from| keys.iter().position(|key| key == from))
            .unwrap_or(0);
        let distance = state.page.get().get_untracked().max(1);
        let index = if forward {
            at.saturating_add(distance).min(last)
        } else {
            at.saturating_sub(distance)
        };
        Some(keys[index].clone())
    })
}

pub(super) fn typed<K: Clone + Eq + Hash + 'static>(
    state: &State<K>,
    from: Option<&K>,
    c: char,
    logical: bool,
) -> Handled {
    let query = state
        .typed
        .get()
        .update(|typed| typed.push(c, kivact_core::custom::clock_now()));
    let items = state.items.with_untracked(|items| {
        let mut seen = std::collections::HashSet::new();
        items
            .iter()
            .filter(|item| seen.insert(item.key.clone()))
            .map(|item| {
                let disabled = item.disabled || (!logical && !state.focus_available(&item.key));
                (item.key.clone(), item.label.get_untracked(), disabled)
            })
            .collect::<Vec<_>>()
    });
    let Some(next) = typeahead_find(&items, from, &query) else {
        return Handled::No;
    };
    if logical {
        state.select(next, ListGesture::Replace);
    } else {
        if state.policy().activation == ListActivation::Automatic {
            state.select(next.clone(), ListGesture::Replace);
        }
        state.focus(next);
    }
    Handled::Yes
}

pub(super) fn key_selection<K: Clone + Eq + Hash + 'static>(
    state: &State<K>,
    event: &KeyEvent,
) -> Handled {
    let current = kivact_core::untrack(|| selected(state.selection).into_iter().next());
    let policy = state.policy();
    if let Some(movement) = movement(event.key(), event.modifiers(), policy.axis, state.mode) {
        let Some(next) = destination(state, current.as_ref(), Some(movement), None, true) else {
            return Handled::No;
        };
        state.select(next, ListGesture::Replace);
        return Handled::Yes;
    }
    let m = event.modifiers();
    if let Key::Char(c) = event.key()
        && !c.is_control()
        && !m.ctrl
        && !m.alt
        && !m.meta
        && policy.typeahead
    {
        return typed(state, current.as_ref(), c, true);
    }
    Handled::No
}

pub(super) fn select_all<K: Clone + Eq + Hash + 'static>(state: &State<K>) {
    if let ListSelection::Multiple(binding) = state.selection {
        let keys = state.order.with_untracked(|order| {
            order
                .keys()
                .into_iter()
                .filter(|key| order.enabled(key) && state.can_select(key))
                .collect()
        });
        binding.set(keys);
    }
}
