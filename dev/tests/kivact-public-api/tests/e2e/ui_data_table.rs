//! Role: Tests the data table of `kivact::ui` on real windows.
//!
//! Its roles and names, sorting, filtering, paging, the selection of rows
//! and of the page, the grid keys, column resizing, reordering, the toolbar
//! and footer slots and the controls that stay mounted in rows, through the
//! public API.

use crate::support::mount_with;
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
