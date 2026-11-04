//! Role: Table parts share grid focus, list selection, sort and resize behavior.

use crate::support::mount_with;
use kivact::interaction::{
    ColumnDef, ListRangeOptions, Selection, SortOrder, TableFocus, use_data_table,
};
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions};
use kivact::view::{Modifiers, for_each};

#[test]
fn cells_follow_pointer_navigation_policy_removal_and_scroll() {
    let (screen, (rows, selected)) = mount_with(
        RenderOptions::default().mirror(MirrorMode::Required),
        || {
            let rows = signal(vec![1u32, 2, 3, 4, 5, 6, 7, 8]);
            let selected = signal(Vec::new());
            let table = use_data_table(
                rows,
                |row| *row,
                vec![ColumnDef::new(0, "Name"), ColumnDef::new(1, "Value")],
                selected.into_selection(),
                TableFocus::Cells(ListRangeOptions::default()),
            )
            .row_label(|key| format!("Row {key}"));
            let body = for_each(
                rows,
                |row| *row,
                move |row_value| {
                    let key = row_value.with(|row| *row);
                    row((0..2)
                        .map(|col| {
                            node()
                                .part(table.cell(key, col))
                                .name(format!("{key}:{col}"))
                                .style(style().size(100, 40))
                        })
                        .collect::<Vec<_>>())
                    .part(table.row(key))
                },
            );
            (
                (rows, selected),
                scroll(column(body).part(table.grid())).style(style().size(220, 100)),
            )
        },
    );
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::GridCell, "1:0").is_focused());
    screen.press_key(Key::End);
    assert!(screen.get_by_role(Role::GridCell, "1:1").is_focused());
    screen.press_key_with(
        Key::End,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!(screen.get_by_role(Role::GridCell, "8:1").is_focused());
    screen.press_key_with(
        Key::Home,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!(screen.get_by_role(Role::GridCell, "1:0").is_focused());
    screen.get_by_role(Role::GridCell, "2:1").click();
    screen.press_key(Key::ArrowRight);
    assert!(
        screen.get_by_role(Role::GridCell, "2:1").is_focused(),
        "cell movement clamps even with wrapping row policy"
    );
    screen.press_key(Key::ArrowDown);
    assert!(
        screen.get_by_role(Role::GridCell, "3:1").is_focused(),
        "cell arrows ignore the list axis"
    );
    screen.press_key(Key::Space);
    assert_eq!(selected.get_untracked(), vec![3]);
    assert!(screen.query_by_role(Role::Row, "Row 3").is_some());
    screen.press_key_with(
        Key::ArrowDown,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_eq!(selected.get_untracked(), vec![3, 4]);
    screen.press_key(Key::ArrowUp);
    selected.set(vec![1, 3]);
    screen.run_until_idle();
    screen.press_key_with(
        Key::ArrowDown,
        Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        },
    );
    assert_eq!(
        selected.get_untracked(),
        vec![1, 3, 4],
        "command-shift uses the additive range policy"
    );

    screen.press_key(Key::ArrowUp);
    screen.press_key(Key::Space);

    for _ in 0..5 {
        screen.press_key(Key::ArrowDown);
    }
    assert!(screen.get_by_role(Role::GridCell, "8:1").is_focused());
    let rect = screen
        .get_by_role(Role::GridCell, "8:1")
        .rect()
        .expect("table cell laid out");
    assert!(
        rect.y < 100.0,
        "keyboard navigation scrolls its cell into view: {rect:?}"
    );
    rows.update(|rows| rows.retain(|row| *row != 8));
    screen.run_until_idle();
    assert!(
        screen.get_by_role(Role::GridCell, "3:0").is_focused(),
        "removing the focused cell restores focus to the selected row"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn headers_sort_and_resize_through_caller_bindings() {
    let (screen, (sort, widths)) = mount_with(
        RenderOptions::default().mirror(MirrorMode::Required),
        || {
            let sort = signal(None);
            let widths = signal(std::collections::HashMap::from([(0usize, 100.0)]));
            let table = use_data_table(
                vec![1],
                |row| *row,
                vec![ColumnDef::new(0, "Name").compare(|a: &i32, b: &i32| a.cmp(b))],
                Selection::Multiple(signal(Vec::<i32>::new()).into_binding()),
                TableFocus::default(),
            )
            .with_sort(sort)
            .with_widths(widths);
            (
                (sort, widths),
                row((
                    node().part(table.header(0)).style(style().size(100, 30)),
                    node()
                        .part(table.resize(0))
                        .name("Width")
                        .style(style().size(8, 30)),
                ))
                .part(table.grid()),
            )
        },
    );
    screen.get_by_role(Role::ColumnHeader, "Name").focus();
    screen.press_key(Key::Enter);
    assert_eq!(sort.get_untracked(), Some((0, SortOrder::Ascending)));
    screen.press_key(Key::Space);
    assert_eq!(sort.get_untracked(), Some((0, SortOrder::Descending)));
    screen.get_by_role(Role::Splitter, "Width").focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(widths.get_untracked().get(&0), Some(&110.0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
