//! Role: Cell-focus rows and cells move a roving tab stop with grid keys.

use super::data::{TableCellState, TableFocus, TableRowState};
use super::state::{State, Target};
use crate::Selection;
use crate::policy::grid::GridMove;
use crate::roving::Member;
use kivact_core::ErrorKind;
use kivact_core::custom::{NodeRefExt, Owner, Part, part};
use kivact_core::state::{Value, Var, memo, var};
use kivact_core::view::{
    ButtonPress, Handled, Key, KeyEvent, MouseButton, PointerEvent, PointerPhase, Role,
};
use std::rc::Rc;

fn row_name<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
    state: &Rc<State<K, C>>,
    key: Var<K>,
) -> Value<String> {
    let options = state.options.value();
    memo(move || {
        let label = options.with(|options| options.label.clone());
        label
            .map(|label| key.with(|key| label(key)))
            .unwrap_or_default()
    })
}

/// A row whose cells hold the focus.
pub(super) fn row<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
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
    let input = part(move |input| TableRowState {
        input,
        selected: is_selected.get(),
        active: is_active.get(),
    });
    let part_state = input.state();
    let observed = state.row_states;
    observed.update(|states| {
        states.insert(key.get(), part_state);
    });
    kivact_core::state::on_cleanup(move || {
        if observed.try_get_untracked().is_some() {
            observed.update(|states| {
                let key = key.get();
                if states.get(&key) == Some(&part_state) {
                    states.remove(&key);
                }
            });
        }
    });
    Part::new(part_state, move |host| {
        let row = host.part(input).role(Role::Row).name(row_name(&state, key));
        if state.focus == TableFocus::None {
            row
        } else {
            row.selected(is_selected)
        }
    })
}

/// A cell: with cell focus it is a grid cell with the roving tab stop.
pub(super) fn cell<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
    state: Rc<State<K, C>>,
    key: K,
    column: C,
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
        let input = part(move |input| TableCellState {
            input,
            selected: is_selected.get(),
            active: false,
        });
        let part_state = input.state();
        observe_cell(&state, id, part_state);
        return Part::new(part_state, move |host| host.part(input).role(role));
    }
    let active = state.active;
    let is_active = memo(move || id.with(|id| active.with(|active| active.as_ref() == Some(id))));
    let keys = state.clone();
    let pointers = state.clone();
    let input = part(move |input| TableCellState {
        input,
        selected: is_selected.get(),
        active: is_active.get(),
    });
    let part_state = input.state();
    observe_cell(&state, id, part_state);
    Part::new(part_state, move |host| {
        host.part(input)
            .role(Role::GridCell)
            .selected(is_selected)
            .focusable(true)
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
                        pointers
                            .roving
                            .requested
                            .set(Some(Target::Cell(cell.0, cell.1)));
                    } else {
                        pointers.focus_cell(cell);
                    }
                }
                Handled::No
            })
            .on_attach(move |mount| {
                mount_cell(state.clone(), id.get(), mount);
            })
    })
}

fn key_down<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
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
    let multiple = matches!(state.selection, Selection::Multiple(_));
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
    let movement = GridMove::from_key(event.key(), modifiers);
    if movement.is_none() && !matches!(event.key(), Key::PageUp | Key::PageDown) {
        return Handled::No;
    }
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
    if matches!(movement, Some(GridMove::Up)) && row == 0 && state.focus_header(column.clone()) {
        return Handled::Yes;
    }
    if matches!(movement, Some(GridMove::First))
        && columns
            .first()
            .is_some_and(|column| state.focus_header(column.clone()))
    {
        return Handled::Yes;
    }
    let destination = match event.key() {
        Key::PageUp => Some((row.saturating_sub(10), col)),
        Key::PageDown => Some(((row + 10).min(keys.len().saturating_sub(1)), col)),
        _ => movement.and_then(|movement| movement.apply((row, col), keys.len(), columns.len())),
    };
    let Some((row, col)) = destination else {
        return Handled::No;
    };
    let next = (keys[row].clone(), columns[col].clone());
    if modifiers.shift && multiple && matches!(movement, Some(GridMove::Up | GridMove::Down)) {
        state
            .list
            .select(next.0.clone(), state.list.gesture(modifiers));
    }
    state.focus_cell(next);
    Handled::Yes
}

fn mount_cell<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
    state: Rc<State<K, C>>,
    id: (K, C),
    mount: kivact_core::custom::PartMount,
) {
    let owner = Owner::current();
    if Owner::root() != state.root {
        owner.report(
            ErrorKind::WrongApp,
            "DataTableApi cell belongs to another App",
        );
        return;
    }
    let (key, column) = id;
    let cell = Target::Cell(key, column);
    if state.roving.contains(&cell) {
        owner.report(
            ErrorKind::Message,
            "DataTableApi: a cell with this row and column is already mounted",
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
}

fn observe_cell<K: kivact_core::state::ItemKey, C: kivact_core::state::ItemKey>(
    state: &State<K, C>,
    id: kivact_core::state::Var<(K, C)>,
    part_state: kivact_core::state::Value<TableCellState>,
) {
    let observed = state.cell_states;
    observed.update(|states| {
        states.insert(id.get(), part_state);
    });
    kivact_core::state::on_cleanup(move || {
        if observed.try_get_untracked().is_some() {
            observed.update(|states| {
                let key = id.get();
                if states.get(&key) == Some(&part_state) {
                    states.remove(&key);
                }
            });
        }
    });
}
