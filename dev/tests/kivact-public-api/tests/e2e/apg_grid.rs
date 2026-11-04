//! Role: Verify APG grid navigation and table cell semantics through public parts.
use crate::support::mount;
use kivact::interaction::{ColumnDef, Selection, TableFocus, Tone, use_data_table};
use kivact::prelude::*;
use kivact::test::Element;
use kivact::ui::RowStatus;
use kivact::view::Modifiers;

fn interactive_table() -> Node {
    let table = use_data_table(
        (0..24).collect::<Vec<i32>>(),
        |row| *row,
        vec![
            ColumnDef::new(0, "Name").compare(|a: &i32, b: &i32| a.cmp(b)),
            ColumnDef::new(1, "Value").compare(|a: &i32, b: &i32| a.cmp(b)),
        ],
        Selection::Single(signal(None::<i32>).into_binding()),
        TableFocus::default(),
    );
    column((
        row((
            node().part(table.header(0)).style(style().size(80, 20)),
            node().part(table.header(1)).style(style().size(80, 20)),
        ))
        .role(Role::Row),
        column(
            (0..24)
                .map(|key| {
                    row((0..2)
                        .map(|column| {
                            node()
                                .part(table.cell(key, column))
                                .name(format!("{key}:{column}"))
                                .style(style().size(80, 20))
                        })
                        .collect::<Vec<_>>())
                    .part(table.row(key))
                })
                .collect::<Vec<_>>(),
        ),
    ))
    .part(table.grid())
}

fn descendant_roles(element: Element<'_>) -> Vec<Role> {
    let mut roles = Vec::new();
    for child in element.children() {
        if let Some(role) = child.role() {
            roles.push(role);
        }
        roles.extend(descendant_roles(child));
    }
    roles
}

#[test]
fn sortable_headers_share_the_grid_tab_stop_and_arrow_navigation() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                interactive_table(),
                node()
                    .role(Role::Button)
                    .name("After")
                    .focusable(true)
                    .style(style().size(80, 20)),
            )),
        )
    });
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::GridCell, "0:0").is_focused());
    screen.press_key(Key::ArrowUp);
    assert!(screen.get_by_role(Role::ColumnHeader, "Name").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::ColumnHeader, "Value").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::GridCell, "0:1").is_focused());
    screen.press_key_with(
        Key::Home,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!(screen.get_by_role(Role::ColumnHeader, "Name").is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn page_keys_move_by_rows_and_stop_at_grid_boundaries() {
    let (screen, ()) = mount(|| ((), interactive_table()));
    screen.press_tab(false);
    screen.press_key(Key::PageDown);
    assert!(screen.get_by_role(Role::GridCell, "10:0").is_focused());
    screen.press_key(Key::PageDown);
    screen.press_key(Key::PageDown);
    assert!(screen.get_by_role(Role::GridCell, "23:0").is_focused());
    screen.press_key(Key::PageDown);
    assert!(screen.get_by_role(Role::GridCell, "23:0").is_focused());
    screen.press_key(Key::PageUp);
    assert!(screen.get_by_role(Role::GridCell, "13:0").is_focused());
    screen.press_key(Key::PageUp);
    screen.press_key(Key::PageUp);
    assert!(screen.get_by_role(Role::GridCell, "0:0").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn passive_table_status_lane_uses_table_cells() {
    let (screen, ()) = mount(|| {
        (
            (),
            data_table(
                vec![1],
                |row| *row,
                vec![DataColumn::new("Value", |value: &i32| value.to_string())],
            )
            .focus(TableFocus::None)
            .row_status(|_| Some(RowStatus::new("Ready", Tone::Success))),
        )
    });
    screen.get_by_role(Role::Table, "Data table");
    screen.get_by_role(Role::Image, "Ready");
    let roles = descendant_roles(screen.get_by_role(Role::Row, "1"));
    assert!(!roles.contains(&Role::GridCell));
    assert_eq!(roles.iter().filter(|role| **role == Role::Cell).count(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn grid_selection_accessory_uses_grid_cell_semantics() {
    let (screen, ()) = mount(|| {
        let selected = signal(Vec::<i32>::new());
        (
            (),
            data_table(
                vec![1],
                |row| *row,
                vec![DataColumn::new("Value", |value: &i32| value.to_string())],
            )
            .selection(selected),
        )
    });
    screen.get_by_role(Role::CheckBox, "Select 1");
    let roles = descendant_roles(screen.get_by_role(Role::Row, "1"));
    assert!(!roles.contains(&Role::Cell));
    assert_eq!(
        roles.iter().filter(|role| **role == Role::GridCell).count(),
        2
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn accessory_controls_share_one_tab_stop_and_remain_keyboard_operable() {
    let (screen, selected) = mount(|| {
        let selected = signal(Vec::<i32>::new());
        let expanded = signal(Vec::<i32>::new());
        (
            selected,
            column((
                data_table(
                    vec![1, 2],
                    |row| *row,
                    vec![DataColumn::new("Value", |value: &i32| value.to_string())],
                )
                .selection(selected)
                .row_index(1)
                .row_status(|_| Some(RowStatus::new("Ready", Tone::Success)))
                .row_details(|_| text("Details"))
                .expanded_rows(expanded)
                .row_actions(80.0, |_| button("Run")),
                node()
                    .role(Role::Button)
                    .name("After")
                    .focusable(true)
                    .style(style().size(80, 20)),
            )),
        )
    });
    screen.press_tab(false);
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    screen.press_tab(true);
    screen.get_by_role(Role::CheckBox, "Select 1").focus();
    screen.press_key(Key::Escape);
    screen.press_key(Key::Home);
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::Enter);
    assert!(screen.get_by_role(Role::CheckBox, "Select 1").is_focused());
    screen.press_key(Key::Space);
    assert_eq!(selected.get_untracked(), vec![1]);
    screen.press_key(Key::Escape);
    screen.press_key(Key::End);
    screen.press_key(Key::F(2));
    assert!(
        screen
            .all_by_role(Role::Button, "Run")
            .into_iter()
            .any(|button| button.is_focused())
    );
    screen.press_key(Key::Escape);
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
