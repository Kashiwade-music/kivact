//! Role: Tests the data table of `kivact::ui` on real windows.
//!
//! Its roles and names, sorting, filtering, paging, the selection of rows
//! and of the page, the grid keys, column resizing, reordering, the toolbar
//! and footer slots and the controls that stay mounted in rows, through the
//! public API.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::interaction::TableFocus;
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::view::SortDirection;

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
fn a_header_sorts_ascending_then_descending_and_the_sort_binding_follows() {
    let (screen, sort) = mount_with(options(), || {
        let sort = signal(None::<(usize, SortDirection)>);
        (
            sort,
            data_table(people(), |p: &Person| p.id, columns()).sort(sort),
        )
    });
    let name = screen.get_by_role(Role::ColumnHeader, "Name");
    name.click();
    assert_eq!(sort.get_untracked(), Some((0, SortDirection::Ascending)));
    assert_eq!(name.sort(), SortDirection::Ascending);
    assert_eq!(names(&screen), ["Ada", "Alan", "Edsger", "Grace", "Linus"]);
    name.click();
    assert_eq!(sort.get_untracked(), Some((0, SortDirection::Descending)));
    assert_eq!(names(&screen), ["Linus", "Grace", "Edsger", "Alan", "Ada"]);
    // The caller sorts too: the table reads the binding back.
    sort.set(Some((1, SortDirection::Ascending)));
    screen.run_until_idle();
    assert_eq!(names(&screen), ["Linus", "Ada", "Alan", "Grace", "Edsger"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn headers_sort_in_the_tables_own_state_without_a_sort_binding() {
    let (screen, ()) = mount_with(options(), || {
        ((), data_table(people(), |p: &Person| p.id, columns()))
    });
    screen.get_by_role(Role::ColumnHeader, "Age").click();
    assert_eq!(names(&screen), ["Linus", "Ada", "Alan", "Grace", "Edsger"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_filter_field_keeps_matching_rows_and_shows_the_first_page() {
    let (screen, (query, page)) = mount_with(options(), || {
        let query = signal(String::new());
        let page = signal(1_usize);
        (
            (query, page),
            data_table(people(), |p: &Person| p.id, columns())
                .filter(query)
                .page(page, 2),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Filter…");
    field.focus();
    field.type_text("a");
    assert_eq!(query.get_untracked(), "a");
    assert_eq!(page.get_untracked(), 0, "a new text shows the first page");
    assert_eq!(names(&screen), ["Ada", "Grace"]);
    // The query is the caller's too.
    query.set("lin".to_string());
    screen.run_until_idle();
    assert_eq!(names(&screen), ["Linus"]);
    query.set("zzz".to_string());
    screen.run_until_idle();
    screen.get_by_text("No results found.");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pages_turn_with_previous_and_next_and_a_page_past_the_end_shows_the_last() {
    let (screen, page) = mount_with(options(), || {
        let page = signal(0_usize);
        (
            page,
            data_table(people(), |p: &Person| p.id, columns()).page(page, 2),
        )
    });
    assert_eq!(names(&screen), ["Ada", "Grace"]);
    screen.get_by_text("Page 1 of 3");
    assert!(screen.get_by_role(Role::Button, "Previous").is_disabled());
    screen.get_by_role(Role::Button, "Next").click();
    assert_eq!(page.get_untracked(), 1);
    assert_eq!(names(&screen), ["Linus", "Alan"]);
    screen.get_by_role(Role::Button, "Next").click();
    assert_eq!(names(&screen), ["Edsger"]);
    assert!(screen.get_by_role(Role::Button, "Next").is_disabled());
    page.set(9);
    screen.run_until_idle();
    assert_eq!(names(&screen), ["Edsger"], "the last page shows");
    assert_eq!(page.get_untracked(), 9, "the binding is left as it is");
    screen.get_by_role(Role::Button, "Previous").click();
    assert_eq!(page.get_untracked(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn check_boxes_select_rows_and_the_header_one_selects_only_the_page() {
    let (screen, selected) = mount_with(options(), || {
        let selected = signal(vec![5_u32]);
        let page = signal(0_usize);
        (
            selected,
            data_table(people(), |p: &Person| p.id, columns())
                .selection(selected)
                .page(page, 2),
        )
    });
    screen.get_by_text("1 of 5 row(s) selected.");
    screen.get_by_role(Role::CheckBox, "Select Grace").click();
    assert_eq!(selected.get_untracked(), [5, 2]);
    assert_eq!(
        screen
            .get_by_role(Role::CheckBox, "Select Grace")
            .is_checked(),
        Some(true)
    );
    // A press on a data cell does not replace the check boxes' selection.
    screen.get_by_role(Role::GridCell, "Ada").click();
    assert_eq!(selected.get_untracked(), [5, 2]);
    let all = screen.get_by_role(Role::CheckBox, "Select all");
    assert!(all.is_indeterminate());
    all.click();
    assert_eq!(selected.get_untracked(), [5, 1, 2], "the page was added");
    screen.get_by_text("3 of 5 row(s) selected.");
    screen.get_by_role(Role::CheckBox, "Select all").click();
    assert_eq!(
        selected.get_untracked(),
        [5],
        "clearing the page keeps the rows of other pages"
    );
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
fn row_focus_selects_a_row_with_a_press_and_has_no_check_boxes() {
    let (screen, selected) = mount_with(options(), || {
        let selected = signal(Vec::<u32>::new());
        (
            selected,
            data_table(people(), |p: &Person| p.id, columns())
                .selection(selected)
                .focus(TableFocus::Rows)
                .row_label(|p| p.name.to_string()),
        )
    });
    assert!(screen.query_by_role(Role::CheckBox, "Select all").is_none());
    screen.get_by_role(Role::Row, "Linus").click();
    assert_eq!(selected.get_untracked(), [3]);
    assert!(screen.get_by_role(Role::Row, "Linus").is_selected());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_resize_handle_edits_the_width_binding_from_the_keyboard() {
    let (screen, widths) = mount_with(options(), || {
        let widths = signal(vec![0.0_f32, 0.0]);
        (
            widths,
            data_table(people(), |p: &Person| p.id, columns()).resizable(widths),
        )
    });
    let handle = screen.get_by_role(Role::Splitter, "Resize Age");
    let before = screen
        .get_by_role(Role::ColumnHeader, "Age")
        .rect()
        .expect("laid out")
        .width;
    assert_eq!(before, 80.0, "the column's own width");
    handle.focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(widths.get_untracked()[1], 90.0);
    let after = screen
        .get_by_role(Role::ColumnHeader, "Age")
        .rect()
        .expect("laid out")
        .width;
    assert_eq!(after, 90.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_reorder_keeps_the_callers_order_so_headers_do_not_sort_without_a_sort_binding() {
    let (screen, ()) = mount_with(options(), || {
        (
            (),
            data_table(people(), |p: &Person| p.id, columns()).on_reorder(|_| {}),
        )
    });
    screen.get_by_role(Role::ColumnHeader, "Name").click();
    assert_eq!(names(&screen), ["Ada", "Grace", "Linus", "Alan", "Edsger"]);
    assert_eq!(
        screen.get_by_role(Role::ColumnHeader, "Name").sort(),
        SortDirection::None
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn toolbar_and_footer_slots_get_the_live_state_and_replace_the_default_regions() {
    let (screen, (selected, query)) = mount_with(options(), || {
        let selected = signal(vec![1_u32, 2]);
        let query = signal(String::new());
        let view = data_table(people(), |p: &Person| p.id, columns())
            .selection(selected)
            .filter(query)
            .toolbar(|context| {
                let selection = context.selection.expect("the table has a selection");
                button("Clear selection").on_press(move || selection.set(Vec::new()))
            })
            .footer(|context| {
                let count = context.model.row_count();
                text(move || format!("{} people", count.get()))
            });
        ((selected, query), view)
    });
    assert!(screen.query_by_role(Role::TextBox, "Filter…").is_none());
    assert!(screen.query_by_text("1 of 5 row(s) selected.").is_none());
    screen.get_by_text("5 people");
    query.set("a".to_string());
    screen.run_until_idle();
    screen.get_by_text("3 people");
    screen.get_by_role(Role::Button, "Clear selection").click();
    assert!(selected.get_untracked().is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn cell_controls_and_details_stay_mounted_while_rows_sort_and_edit() {
    let built = Rc::new(RefCell::new(0_u32));
    let count = built.clone();
    let (screen, (rows, sort)) = mount_with(options(), move || {
        let rows = signal(people());
        let sort = signal(None::<(usize, SortDirection)>);
        let view = data_table(rows, |p: &Person| p.id, columns())
            .sort(sort)
            .cell_controls(move |row: Value<Person>, column: usize| {
                *count.borrow_mut() += 1;
                let note = signal(String::new());
                if column == 0 {
                    text_input("Note", note).test_id(format!("note-{}", row.get().id))
                } else {
                    text_input("Extra", signal(String::new()))
                }
            })
            .row_details(|row: Value<Person>| {
                text(move || format!("Details of {}", row.get().name))
            });
        ((rows, sort), view)
    });
    let built_at_start = *built.borrow();
    assert_eq!(built_at_start, 10, "one control per cell");
    screen.get_by_text("Details of Ada");
    let note = screen.get_by_test_id("note-1");
    note.focus();
    sort.set(Some((0, SortDirection::Descending)));
    screen.run_until_idle();
    rows.update(|rows| rows[0].age = 37);
    screen.run_until_idle();
    assert_eq!(*built.borrow(), built_at_start, "no control was rebuilt");
    assert!(
        screen.get_by_test_id("note-1").is_focused(),
        "the same control still holds the focus"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn row_actions_are_controls_after_the_cells_with_a_named_header() {
    let (screen, clicked) = mount_with(options(), || {
        let clicked = signal(None::<u32>);
        let view = data_table(people(), |p: &Person| p.id, columns()).row_actions(
            60.0,
            move |row: Value<Person>| {
                let id = row.get().id;
                button(format!("Open {id}")).on_press(move || clicked.set(Some(id)))
            },
        );
        (clicked, view)
    });
    screen.get_by_role(Role::ColumnHeader, "Actions");
    screen.get_by_role(Role::Button, "Open 3").click();
    assert_eq!(clicked.get_untracked(), Some(3));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_row_height_and_color_reach_the_row_recipe() {
    let (screen, ()) = mount_with(options(), || {
        (
            (),
            data_table(people(), |p: &Person| p.id, columns())
                .row_height(40.0)
                .row_color(|p| (p.id == 1).then(|| rgb(0xff0000)))
                .row_label(|p| p.name.to_string()),
        )
    });
    let row = screen
        .get_by_role(Role::Row, "Ada")
        .rect()
        .expect("laid out");
    assert_eq!(row.height, 40.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_callers_style_wins_over_the_table_recipes() {
    let (screen, ()) = mount_with(options(), || {
        (
            (),
            data_table(people(), |p: &Person| p.id, columns())
                .style(style().width(400))
                .test_id("table"),
        )
    });
    assert_eq!(
        screen
            .get_by_test_id("table")
            .rect()
            .expect("laid out")
            .width,
        400.0
    );
}
