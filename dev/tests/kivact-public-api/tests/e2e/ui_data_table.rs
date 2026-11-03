//! Role: Tests the data table of `kivact::ui` on real windows.
//!
//! Its roles and names, sorting, filtering, paging, the selection of rows
//! and of the page, the grid keys, column resizing, reordering, the toolbar
//! and footer slots and the controls that stay mounted in rows, through the
//! public API.

use crate::support::mount_with;
use kivact::interaction::{ScalarRange, SortOrder, TableFocus};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::view::{A11yAction, Modifiers, MouseButton, SortDirection};

#[derive(Clone, Debug, PartialEq)]
struct Person {
    id: u32,
    name: &'static str,
    age: u32,
}

fn people() -> Vec<Person> {
    let person = |id, name, age| Person { id, name, age };
    vec![
        person(1, "Ada", 36),
        person(2, "Grace", 45),
        person(3, "Linus", 28),
        person(4, "Alan", 41),
        person(5, "Edsger", 72),
    ]
}

fn columns() -> Vec<DataColumn<Person>> {
    vec![
        DataColumn::new("Name", |p: &Person| p.name.to_string())
            .compare(|a, b| a.name.cmp(b.name))
            .filter(|p, query| p.name.to_lowercase().contains(&query.to_lowercase())),
        DataColumn::new("Age", |p: &Person| p.age.to_string())
            .compare(|a, b| a.age.cmp(&b.age))
            .numeric(true)
            .width(80.0),
    ]
}

fn options() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        reduced_motion: true,
        width: 720.0,
        height: 600.0,
        ..RenderOptions::from_env()
    }
}

/// The names shown by the rows, top to bottom.
fn names(screen: &Screen) -> Vec<String> {
    let mut found: Vec<(f32, String)> = people()
        .iter()
        .filter_map(|p| screen.query_by_role(Role::GridCell, p.name))
        .map(|cell| (cell.rect().expect("laid out").y, cell.name()))
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().map(|(_, name)| name).collect()
}

#[test]
fn a_table_names_its_grid_headers_rows_and_cells() {
    let (screen, ()) = mount_with(options(), || {
        (
            (),
            data_table(people(), |p: &Person| p.id, columns()).name("People"),
        )
    });
    screen.get_by_role(Role::Group, "People");
    screen.get_by_role(Role::Grid, "Data table");
    screen.get_by_role(Role::ColumnHeader, "Name");
    screen.get_by_role(Role::ColumnHeader, "Age");
    screen.get_by_role(Role::GridCell, "Grace");
    screen.get_by_role(Role::GridCell, "45");
    // A row is named by its first column unless the caller names it.
    screen.get_by_role(Role::Row, "Ada");
    assert_eq!(names(&screen), ["Ada", "Grace", "Linus", "Alan", "Edsger"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout();
}

#[test]
fn a_column_context_menu_preserves_header_sorting() {
    let (screen, selected) = mount_with(options(), || {
        let selected = signal(false);
        let mut columns = columns();
        columns[0] = columns[0].clone().header_context_menu(move || {
            menu(
                "Name options",
                vec![MenuEntry::item("inspect", "Inspect names")],
                move |_| {
                    selected.set(true);
                },
            )
        });
        (selected, data_table(people(), |p: &Person| p.id, columns))
    });
    let name = screen.get_by_role(Role::ColumnHeader, "Name");
    name.click_with(MouseButton::Secondary);
    screen.get_by_role(Role::MenuItem, "Inspect names").click();
    assert!(selected.get());
    name.press_key(Key::ContextMenu);
    screen.get_by_role(Role::MenuItem, "Inspect names");
    screen.press_key(Key::Escape);
    screen
        .get_by_role(Role::ColumnHeader, "Age")
        .click_with(MouseButton::Secondary);
    assert!(
        screen
            .query_by_role(Role::MenuItem, "Inspect names")
            .is_none()
    );
    name.click();
    assert_eq!(names(&screen), ["Ada", "Alan", "Edsger", "Grace", "Linus"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn passive_data_table_uses_table_roles_and_a_caller_supplied_empty_state() {
    let (screen, rows) = mount_with(options(), || {
        let rows = signal(Vec::<Person>::new());
        (
            rows,
            data_table(rows, |p: &Person| p.id, columns())
                .focus(TableFocus::None)
                .empty(|| text("No people")),
        )
    });
    screen.get_by_role(Role::Table, "Data table");
    screen.get_by_text("No people");
    rows.set(people());
    screen.run_until_idle();
    screen.get_by_role(Role::Row, "Ada");
    assert!(
        !screen
            .get_by_role(Role::Cell, "Grace")
            .a11y_actions()
            .contains(&A11yAction::Focus)
    );
    assert!(screen.query_by_role(Role::Grid, "Data table").is_none());
    assert!(screen.query_by_role(Role::Text, "No people").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn column_resize_has_shift_steps_and_respects_columns_without_handles() {
    let (screen, widths) = mount_with(options(), || {
        let widths = signal(vec![120.0, 80.0]);
        let mut columns = columns();
        columns[1] = columns[1].clone().resizable(false);
        (
            widths,
            data_table(people(), |p: &Person| p.id, columns)
                .resizable(widths)
                .resize_range(ScalarRange::new(50.0, 400.0).expect("valid column range")),
        )
    });
    assert!(screen.query_by_role(Role::Splitter, "Resize Age").is_none());
    let handle = screen.get_by_role(Role::Splitter, "Resize Name");
    handle.press_key_with(
        Key::ArrowRight,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(widths.get_untracked(), [170.0, 80.0]);
    handle.press_key(Key::Home);
    assert_eq!(widths.get_untracked(), [50.0, 80.0]);
    handle.press_key(Key::End);
    assert_eq!(widths.get_untracked(), [400.0, 80.0]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_header_sorts_ascending_then_descending_and_the_sort_binding_follows() {
    let (screen, sort) = mount_with(options(), || {
        let sort = signal(None::<(usize, SortOrder)>);
        (
            sort,
            data_table(people(), |p: &Person| p.id, columns()).sort(sort),
        )
    });
    let name = screen.get_by_role(Role::ColumnHeader, "Name");
    name.click();
    assert_eq!(sort.get_untracked(), Some((0, SortOrder::Ascending)));
    assert_eq!(name.sort(), SortDirection::Ascending);
    assert_eq!(names(&screen), ["Ada", "Alan", "Edsger", "Grace", "Linus"]);
    name.click();
    assert_eq!(sort.get_untracked(), Some((0, SortOrder::Descending)));
    assert_eq!(names(&screen), ["Linus", "Grace", "Edsger", "Alan", "Ada"]);
    // The caller sorts too: the table reads the binding back.
    sort.set(Some((1, SortOrder::Ascending)));
    screen.run_until_idle();
    assert_eq!(names(&screen), ["Linus", "Ada", "Alan", "Grace", "Edsger"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_grid_keys_move_between_cells_and_space_selects_the_row() {
    let (screen, selected) = mount_with(options(), || {
        let selected = signal(Vec::<u32>::new());
        (
            selected,
            column((
                button("Before"),
                data_table(people(), |p: &Person| p.id, columns()).selection(selected),
            )),
        )
    });
    // Cells hold the grid's one tab stop.
    screen.get_by_role(Role::GridCell, "Ada").focus();
    assert!(screen.get_by_role(Role::GridCell, "Ada").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::GridCell, "Grace").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::GridCell, "45").is_focused());
    screen.press_key(Key::Space);
    assert_eq!(selected.get_untracked(), [2]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn proportional_columns_share_space_and_resizing_respects_the_column_minimum() {
    let (screen, widths) = mount_with(options(), || {
        let widths = signal(vec![0.0, 0.0]);
        let mut columns = columns();
        columns[0] = columns[0].clone().flex(1.0).min_width(130.0);
        columns[1] = columns[1].clone().flex(2.0).min_width(130.0);
        (
            widths,
            data_table(people(), |p: &Person| p.id, columns)
                .resizable(widths)
                .resize_range(ScalarRange::new(50.0, 600.0).expect("valid column range")),
        )
    });
    let first = screen.get_by_role(Role::ColumnHeader, "Name");
    let second = screen.get_by_role(Role::ColumnHeader, "Age");
    let first_width = first.rect().expect("laid out header").width;
    let second_width = second.rect().expect("laid out header").width;
    assert!(first_width >= 130.0);
    assert!(second_width > first_width * 1.5);
    let cell = screen.get_by_role(Role::GridCell, "Ada");
    assert!((cell.rect().expect("laid out cell").width - first_width).abs() < 1.0);

    let handle = screen.get_by_role(Role::Splitter, "Resize Name");
    handle.press_key(Key::Home);
    assert_eq!(widths.get_untracked(), [130.0, 0.0]);
    handle.press_key(Key::ArrowLeft);
    assert_eq!(widths.get_untracked(), [130.0, 0.0]);
    assert!((first.rect().expect("laid out resized header").width - 130.0).abs() < 1.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn multi_sort_keeps_stable_ties_and_shift_cycles_the_secondary_key() {
    let (screen, sort) = mount_with(options(), || {
        let sort = signal(vec![(1, SortOrder::Ascending)]);
        let mut columns = columns();
        columns[1] = columns[1]
            .clone()
            .compare(|a, b| (a.age / 10).cmp(&(b.age / 10)));
        (
            sort,
            data_table(people(), |p: &Person| p.id, columns).multi_sort(sort),
        )
    });
    assert_eq!(names(&screen), ["Linus", "Ada", "Grace", "Alan", "Edsger"]);

    let name = screen.get_by_role(Role::ColumnHeader, "Name");
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    name.press_key_with(Key::Enter, shift);
    assert_eq!(
        sort.get_untracked(),
        [(1, SortOrder::Ascending), (0, SortOrder::Ascending)]
    );
    assert_eq!(names(&screen), ["Linus", "Ada", "Alan", "Grace", "Edsger"]);
    assert_eq!(name.sort(), SortDirection::Ascending);

    name.press_key_with(Key::Enter, shift);
    assert_eq!(
        sort.get_untracked(),
        [(1, SortOrder::Ascending), (0, SortOrder::Descending)]
    );
    name.press_key_with(Key::Enter, shift);
    assert_eq!(sort.get_untracked(), [(1, SortOrder::Ascending)]);
    assert_eq!(name.sort(), SortDirection::None);

    name.click();
    assert_eq!(sort.get_untracked(), [(0, SortOrder::Ascending)]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn expanded_details_follow_row_keys_across_sorting() {
    let (screen, expanded) = mount_with(options(), || {
        let expanded = signal(Vec::<u32>::new());
        (
            expanded,
            data_table(people(), |p: &Person| p.id, columns())
                .row_label(|person| person.name.to_owned())
                .row_details(|person| {
                    text(move || person.with(|person| format!("Details for {}", person.name)))
                })
                .expanded_rows(expanded),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Grace details");
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(screen.query_by_text("Details for Grace").is_none());
    trigger.press_key(Key::Enter);
    assert_eq!(expanded.get_untracked(), [2]);
    assert_eq!(trigger.is_expanded(), Some(true));
    assert_eq!(trigger.controls().len(), 1);
    assert!(screen.query_by_text("Details for Grace").is_some());
    assert!(screen.query_by_text("Details for Ada").is_none());

    screen.get_by_role(Role::ColumnHeader, "Name").click();
    let trigger = screen.get_by_role(Role::Button, "Grace details");
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(screen.query_by_text("Details for Grace").is_some());
    trigger.press_key(Key::Enter);
    assert!(expanded.get_untracked().is_empty());
    assert!(screen.query_by_text("Details for Grace").is_none());
    trigger.click();
    assert_eq!(expanded.get_untracked(), [2]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
