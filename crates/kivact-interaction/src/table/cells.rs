//! Role: Cell-focus rows and cells move a roving tab stop with grid keys.

use super::data::{TableCellState, TableFocus, TableRowState};
use super::state::State;
use crate::ListSelection;
use crate::policy::grid::GridMove;
use crate::roving::Member;
use kivact_core::diagnostics::ErrorKind;
use kivact_core::primitive::{NodeRefExt, Owner, Part, part};
use kivact_core::view::{
    ButtonPress, Handled, Key, KeyEvent, MouseButton, PointerEvent, PointerPhase, Role,
};
use kivact_core::{Value, Var, memo, var};
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

fn row_name<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static>(
    state: &Rc<State<K, C>>,
    key: Var<K>,
) -> Value<String> {
    let label = state.label;
    memo(move || {
        let label = label.get();
        label
            .map(|label| key.with(|key| label(key)))
            .unwrap_or_default()
    })
}

/// A row whose cells hold the focus.
pub(super) fn row<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static>(
    state: Rc<State<K, C>>,
    key: K,
) -> Part<TableRowState> {
    let key = var(key);
    let selected = state.list.selected();
    let is_selected = memo(move || key.with(|key| selected.with(|s| s.contains(key))));
    let active = state.active;
    let is_active = memo(move || {
        key.with(|key| active.with(|active| active.as_ref().is_some_and(|(row, _)| row == key)))
    });
    let row = part(move |input| TableRowState {
        input,
        selected: is_selected.get(),
        active: is_active.get(),
    })
    .role(Role::Row)
    .name(row_name(&state, key));
    if state.focus == TableFocus::None {
        row
    } else {
        row.selected(is_selected)
    }
}

/// A cell: with cell focus it is a grid cell with the roving tab stop.
pub(super) fn cell<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static>(
    state: Rc<State<K, C>>,
    key: K,
    column: C,
    at: &'static Location<'static>,
) -> Part<TableCellState> {
    let id = var((key, column));
    let selected = state.list.selected();
    let is_selected = memo(move || id.with(|(key, _)| selected.with(|s| s.contains(key))));
    if !matches!(state.focus, TableFocus::Cells(_)) {
        let role = if state.focus == TableFocus::None {
            Role::Cell
        } else {
            Role::GridCell
        };
        return part(move |input| TableCellState {
            input,
            selected: is_selected.get(),
            active: false,
        })
        .role(role);
    }
    let active = state.active;
    let is_active = memo(move || id.with(|id| active.with(|active| active.as_ref() == Some(id))));
    let keys = state.clone();
    let pointers = state.clone();
    part(move |input| TableCellState {
        input,
        selected: is_selected.get(),
        active: is_active.get(),
    })
    .role(Role::GridCell)
    .selected(is_selected)
    .focusable(is_active)
    .on_key(move |event: KeyEvent| key_down(&keys, &id.get(), &event))
    .on_pointer(move |event: PointerEvent| {
        if matches!(
            event.phase(),
            PointerPhase::Down(ButtonPress {
                button: MouseButton::Primary,
                ..
            })
        ) {
            let (key, column) = id.get();
            pointers
                .list
                .select(key.clone(), pointers.list.gesture(event.modifiers()));
            let cell = (key, column);
            if pointers
                .target(&cell)
                .is_some_and(|node| node.holds_focus_within())
            {
                pointers.roving.requested.set(Some(cell));
            } else {
                pointers.focus_cell(cell);
            }
        }
        Handled::No
    })
    .on_bind(move |mount| {
        let owner = Owner::current("Table::cell");
        if Owner::root("Table::cell") != state.root {
            owner.report(ErrorKind::WrongApp, "Table cell belongs to another App", at);
            return;
        }
        let cell = id.get();
        if state.roving.contains(&cell) {
            owner.report(
                ErrorKind::Message,
                "Table: a cell with this row and column is already mounted",
                state.at,
            );
            return;
        }
        let node = mount.node;
        state.roving.insert(
            cell.clone(),
            Member {
                node,
                eligible: mount.eligible,
                disabled: mount.disabled,
                hidden: mount.hidden,
            },
        );
        let mounted = node.mounted_node();
        let weak = Rc::downgrade(&state);
        owner.on_cleanup(move || {
            let Some(state) = weak.upgrade() else {
                return;
            };
            let focused = mounted.is_some() && state.owner.focused_node() == mounted;
            if state.roving.remove(&cell, node) && focused {
                state.roving.restore_from.set(mounted);
            }
        });
    })
}

fn key_down<K: Clone + Eq + Hash + 'static, C: Clone + Eq + Hash + 'static>(
    state: &State<K, C>,
    (key, column): &(K, C),
    event: &KeyEvent,
) -> Handled {
    let Some(node) = state.target(&(key.clone(), column.clone())) else {
        return Handled::No;
    };
    if !node.holds_focus() {
        return Handled::No;
    }
    let modifiers = event.modifiers();
    let command = modifiers.ctrl || modifiers.meta;
    let multiple = matches!(state.selection, ListSelection::Multiple(_));
    match event.key() {
        Key::Space if !modifiers.alt => {
            state
                .list
                .select(key.clone(), state.list.gesture(modifiers));
            return Handled::Yes;
        }
        Key::Char(c) if c.eq_ignore_ascii_case(&'a') && command && !modifiers.shift && multiple => {
            state.list.select_all();
            return Handled::Yes;
        }
        _ => {}
    }
    let Some(movement) = GridMove::from_key(event.key(), modifiers) else {
        return Handled::No;
    };
    let keys = state.keys.get();
    let columns = state
        .columns
        .with(|columns| columns.iter().map(|c| c.key.clone()).collect::<Vec<_>>());
    let (Some(row), Some(col)) = (
        keys.iter().position(|k| k == key),
        columns.iter().position(|c| c == column),
    ) else {
        return Handled::No;
    };
    let Some((row, col)) = movement.apply((row, col), keys.len(), columns.len()) else {
        return Handled::No;
    };
    let next = (keys[row].clone(), columns[col].clone());
    if modifiers.shift && multiple && matches!(movement, GridMove::Up | GridMove::Down) {
        state
            .list
            .select(next.0.clone(), state.list.gesture(modifiers));
    }
    state.focus_cell(next);
    Handled::Yes
}
