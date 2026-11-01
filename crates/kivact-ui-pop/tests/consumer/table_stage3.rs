//! Role: Table consumers share keyboard movement, sorting and row selection.
use kivact::prelude::*;
use kivact::test::RenderOptions;

use kivact::view::SortDirection;
use kivact_ui_pop as ui;

/// A row of the tables below: its key and its two texts.
#[derive(Clone, PartialEq)]
struct Row {
    id: u64,
    name: &'static str,
    level: &'static str,
}

fn row(id: u64, name: &'static str, level: &'static str) -> Row {
    Row { id, name, level }
}

/// The name column, sorted by name.
fn name(title: &'static str) -> kivact::ui::DataColumn<Row> {
    kivact::ui::DataColumn::new(title, |r: &Row| r.name.to_owned())
        .width(160.0)
        .compare(|a, b| a.name.cmp(b.name))
}

type State = (
    Signal<Vec<u64>>,
    Signal<Option<(usize, SortDirection)>>,
    Signal<usize>,
);

fn state(sort: Option<(usize, SortDirection)>, page: usize) -> State {
    (signal(Vec::new()), signal(sort), signal(page))
}

#[test]
fn table_stage3_keyboard_sort_and_selection() {
    let (screen, selected) = kivact::test::render_with_handle(
        RenderOptions {
            theme: ui::theme(kivact::style::ThemeMode::Light),

            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let (selected, sort, page) = state(None, 0);
            let view = kivact::ui::data_table(
                vec![row(1, "Beta", ""), row(2, "Alpha", "")],
                |r: &Row| r.id,
                vec![name("Name")],
            )
            .selection(selected)
            .sort(sort)
            .page(page, 6);
            (selected, view)
        },
    );
    screen.get_by_role(Role::ColumnHeader, "Name").focus();
    screen.press_key(Key::Enter);
    let alpha = screen.get_by_role(Role::GridCell, "Alpha");
    let beta = screen.get_by_role(Role::GridCell, "Beta");
    assert!(
        alpha.rect().expect("table cell laid out").y < beta.rect().expect("table cell laid out").y
    );
    alpha.focus();
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::GridCell, "Beta").is_focused());
    screen.press_key(Key::ArrowUp);
    screen.press_key(Key::Space);
    assert_eq!(selected.get_untracked(), vec![2]);
    screen.get_by_role(Role::ColumnHeader, "Name").focus();
    screen.press_key(Key::Enter);
    assert!(
        screen
            .get_by_role(Role::GridCell, "Beta")
            .rect()
            .expect("table cell laid out")
            .y
            < screen
                .get_by_role(Role::GridCell, "Alpha")
                .rect()
                .unwrap()
                .y
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn table_external_sort_page_and_comparator_are_authoritative() {
    let (screen, (sort, page)) = kivact::test::render_with_handle(
        RenderOptions {
            theme: ui::theme(kivact::style::ThemeMode::Light),

            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let (selected, sort, page) = state(Some((0, SortDirection::Ascending)), 1);
            (
                (sort, page),
                kivact::ui::data_table(
                    vec![row(1, "long", ""), row(2, "x", ""), row(3, "mid", "")],
                    |r: &Row| r.id,
                    vec![
                        kivact::ui::DataColumn::new("Custom", |r: &Row| r.name.to_owned())
                            .width(160.0)
                            .numeric(true)
                            .compare(|a, b| a.name.len().cmp(&b.name.len())),
                    ],
                )
                .selection(selected)
                .sort(sort)
                .page(page, 1),
            )
        },
    );
    screen.get_by_role(Role::GridCell, "mid");
    screen.get_by_role(Role::Button, "Next").click();
    assert_eq!(page.get_untracked(), 2);
    screen.get_by_role(Role::GridCell, "long");
    screen.get_by_role(Role::ColumnHeader, "Custom").click();
    assert_eq!(sort.get_untracked(), Some((0, SortDirection::Descending)));
    assert_eq!(page.get_untracked(), 0);
    screen.get_by_role(Role::GridCell, "long");
    page.set(usize::MAX);
    screen.run_until_idle();
    assert_eq!(page.get_untracked(), usize::MAX);
    screen.get_by_role(Role::GridCell, "x");
    screen.get_by_role(Role::Button, "Previous").click();
    assert_eq!(page.get_untracked(), 1);
    screen.get_by_role(Role::GridCell, "mid");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn table_columns_sort_by_their_own_compare_and_without_one_keep_the_order() {
    let (screen, sort) = kivact::test::render_with_handle(
        RenderOptions {
            theme: ui::theme(kivact::style::ThemeMode::Light),

            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let (selected, sort, page) = state(None, 0);
            (
                sort,
                kivact::ui::data_table(
                    vec![
                        row(1, "2", "2"),
                        row(2, "10", "10"),
                        row(3, "invalid", "invalid"),
                    ],
                    |r: &Row| r.id,
                    vec![
                        name("Text"),
                        kivact::ui::DataColumn::new("Numeric", |r: &Row| r.level.to_owned())
                            .width(160.0)
                            .compare(|a, b| {
                                match (a.level.parse::<f64>(), b.level.parse::<f64>()) {
                                    (Ok(a), Ok(b)) => a.total_cmp(&b),
                                    (Ok(_), Err(_)) => std::cmp::Ordering::Less,
                                    (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
                                    _ => a.level.cmp(b.level),
                                }
                            }),
                        kivact::ui::DataColumn::new("Plain", |r: &Row| format!("plain {}", r.id))
                            .width(160.0),
                    ],
                )
                .selection(selected)
                .sort(sort)
                .page(page, 6),
            )
        },
    );
    let cell_y = |name| {
        screen.all_by_role(Role::GridCell, name)[0]
            .rect()
            .unwrap()
            .y
    };
    screen.get_by_role(Role::ColumnHeader, "Plain").click();
    assert_eq!(
        sort.get_untracked(),
        None,
        "a column without compare does not sort"
    );
    assert!(cell_y("2") < cell_y("10"), "the caller's order");
    screen.get_by_role(Role::ColumnHeader, "Text").click();
    assert!(cell_y("10") < cell_y("2"));
    screen.get_by_role(Role::ColumnHeader, "Numeric").click();
    assert!(cell_y("2") < cell_y("10"));
    assert!(cell_y("10") < cell_y("invalid"));
    screen.get_by_role(Role::ColumnHeader, "Numeric").click();
    assert!(cell_y("invalid") < cell_y("10"));
    assert!(cell_y("10") < cell_y("2"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
