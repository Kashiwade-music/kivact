//! Role: The headless table model filters, sorts and pages rows for a table.
//!
//! `use_table_model` derives the rows of the current page from the caller's
//! rows, each column's compare and filter functions, the sort and page
//! bindings and the filter text; `use_table` given its rows and columns and
//! the same sort binding sorts the model from a header press.

use crate::support::mount_with;
use kivact::interaction::{ColumnDef, ListSelection, use_table, use_table_model};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::view::{SortDirection, for_each};

#[derive(Clone, Debug, PartialEq)]
struct Track {
    id: String,
    title: String,
    seconds: u32,
}

fn track(id: &str, title: &str, seconds: u32) -> Track {
    Track {
        id: id.into(),
        title: title.into(),
        seconds,
    }
}

type Sort = Option<(&'static str, SortDirection)>;

/// The titles of the rows shown, top to bottom.
fn shown(screen: &Screen) -> Vec<String> {
    screen
        .all_by_test_id("row")
        .iter()
        .map(|row| row.name())
        .collect()
}

#[test]
fn a_model_filters_sorts_and_pages_rows_and_a_header_press_sorts_it() {
    let (screen, (rows, sort, query, page, show_length, counts)) = mount_with(
        RenderOptions {
            window: WindowMode::Real,
            ..Default::default()
        },
        || {
            let rows = signal(vec![
                track("a", "Bloom", 200),
                track("b", "Arcade", 180),
                track("c", "Dune", 240),
                track("d", "Comet", 150),
                track("e", "Bay", 210),
            ]);
            let sort = signal(None::<(&'static str, SortDirection)>);
            let query = signal(String::new());
            let page = signal(0_usize);
            let show_length = signal(true);
            let model = use_table_model(
                rows,
                |t: &Track| t.id.clone(),
                vec![
                    ColumnDef::new("title", "Title")
                        .compare(|a: &Track, b: &Track| a.title.cmp(&b.title))
                        .filter(|t: &Track, q: &str| {
                            t.title.to_lowercase().contains(&q.to_lowercase())
                        }),
                    ColumnDef::new("length", "Length")
                        .compare(|a: &Track, b: &Track| a.seconds.cmp(&b.seconds))
                        .visible(show_length),
                    ColumnDef::new("id", "Id"),
                ],
            )
            .sort(sort)
            .filter(query)
            .page(page, 2_usize);
            let selected = signal(Vec::<String>::new());
            let table = use_table(
                model.rows(),
                |t: &Track| t.id.clone(),
                model.columns(),
                ListSelection::Multiple(selected.into()),
            )
            .sort(sort)
            .row_label(move |id: &String| {
                rows.with(|rows| {
                    rows.iter()
                        .find(|t| &t.id == id)
                        .map(|t| t.title.clone())
                        .unwrap_or_default()
                })
            });
            let headers = for_each(
                model.columns(),
                |c| *c.key(),
                move |key, column| {
                    node()
                        .part(table.header(key))
                        .child(text(move || column.get().label()))
                        .style(style().size(80, 20))
                },
            );
            let body = for_each(
                model.rows(),
                |t| t.id.clone(),
                move |id, _| {
                    node().part(table.row(id.clone())).test_id("row").child(
                        node()
                            .part(table.cell(id, "title"))
                            .style(style().size(80, 20)),
                    )
                },
            );
            let counts = (model.row_count(), model.page_count(), model.keys());
            (
                (rows, sort, query, page, show_length, counts),
                column((row(headers), column(body))).part(table.grid()),
            )
        },
    );
    let (count, pages, keys) = counts;
    assert_eq!(
        shown(&screen),
        ["Bloom", "Arcade"],
        "the caller's order, page 1"
    );
    assert_eq!((count.get_untracked(), pages.get_untracked()), (5, 3));
    page.set(2);
    screen.run_until_idle();
    assert_eq!(shown(&screen), ["Bay"]);

    // A header press writes the shared sort binding, which sorts the model.
    page.set(0);
    screen.get_by_role(Role::ColumnHeader, "Title").click();
    assert_eq!(
        sort.get_untracked(),
        Some(("title", SortDirection::Ascending))
    );
    assert_eq!(shown(&screen), ["Arcade", "Bay"]);
    screen.get_by_role(Role::ColumnHeader, "Title").click();
    assert_eq!(shown(&screen), ["Dune", "Comet"]);
    assert_eq!(keys.get_untracked(), ["c", "d"]);
    let by_length: Sort = Some(("length", SortDirection::Ascending));
    sort.set(by_length);
    screen.run_until_idle();
    assert_eq!(shown(&screen), ["Comet", "Arcade"]);

    // The filter keeps the rows a shown column matches; past the last page
    // the model shows the last one and leaves the page binding alone.
    page.set(2);
    query.set("  E ".into());
    screen.run_until_idle();
    assert_eq!((count.get_untracked(), pages.get_untracked()), (3, 2));
    assert_eq!(
        page.get_untracked(),
        2,
        "the page binding is not written back"
    );
    assert_eq!(shown(&screen), ["Dune"], "the last page is shown");
    query.set("zz".into());
    screen.run_until_idle();
    assert_eq!((count.get_untracked(), pages.get_untracked()), (0, 1));
    assert_eq!(page.get_untracked(), 2);
    assert!(shown(&screen).is_empty());
    query.set(String::new());
    screen.run_until_idle();
    assert_eq!(
        (page.get_untracked(), shown(&screen)),
        (2, vec!["Dune".to_string()]),
        "the page number comes back with the rows"
    );
    page.set(0);

    // A hidden column leaves the columns, and its sort keeps the order.
    show_length.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::ColumnHeader, "Length").is_none());
    let id = screen.get_by_role(Role::ColumnHeader, "Id");
    assert_eq!(id.sort(), SortDirection::None);
    id.click();
    assert_eq!(
        sort.get_untracked(),
        by_length,
        "a column without a compare function does not sort"
    );

    // The rows follow the caller's data.
    rows.update(|rows| rows.push(track("f", "Abba", 100)));
    sort.set(Some(("title", SortDirection::Ascending)));
    screen.run_until_idle();
    assert_eq!(shown(&screen), ["Abba", "Arcade"]);
    assert_eq!(count.get_untracked(), 6);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_restored_page_number_survives_rows_that_arrive_later() {
    let (screen, (rows, page, keys)) = mount_with(
        RenderOptions {
            window: WindowMode::Real,
            ..Default::default()
        },
        || {
            let rows = signal(Vec::<Track>::new());
            let page = signal(2_usize);
            let model = use_table_model(
                rows,
                |t: &Track| t.id.clone(),
                vec![ColumnDef::new("title", "Title")],
            )
            .page(page, 2_usize);
            let keys = model.keys();
            ((rows, page, keys), node())
        },
    );
    assert_eq!(page.get_untracked(), 2, "no rows yet: the page is kept");
    assert!(keys.get_untracked().is_empty());
    rows.set(vec![
        track("a", "Bloom", 200),
        track("b", "Arcade", 180),
        track("c", "Dune", 240),
        track("d", "Comet", 150),
        track("e", "Bay", 210),
    ]);
    screen.run_until_idle();
    assert_eq!(page.get_untracked(), 2);
    assert_eq!(keys.get_untracked(), ["e"], "the restored page is shown");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_filtered_keys_are_every_page_s_rows_that_pass_the_filter_in_order() {
    let (screen, (query, page, filtered)) = mount_with(
        RenderOptions {
            window: WindowMode::Real,
            ..Default::default()
        },
        || {
            let rows = vec![
                track("a", "Bloom", 200),
                track("b", "Arcade", 180),
                track("c", "Dune", 240),
                track("d", "Comet", 150),
                track("e", "Bay", 210),
            ];
            let sort = signal(Some(("title", SortDirection::Ascending)));
            let query = signal(String::new());
            let page = signal(0_usize);
            let model = use_table_model(
                rows,
                |t: &Track| t.id.clone(),
                vec![
                    ColumnDef::new("title", "Title")
                        .compare(|a: &Track, b: &Track| a.title.cmp(&b.title))
                        .filter(|t: &Track, q: &str| t.title.contains(q)),
                ],
            )
            .sort(sort)
            .filter(query)
            .page(page, 2_usize);
            let filtered = model.filtered_keys();
            let view = text(move || filtered.get().join(","));
            ((query, page, filtered), view)
        },
    );
    // Sorted by title, across both pages: Arcade, Bay, Bloom, Comet, Dune.
    assert_eq!(filtered.get_untracked(), ["b", "e", "a", "d", "c"]);
    page.set(1);
    screen.run_until_idle();
    assert_eq!(
        filtered.get_untracked().len(),
        5,
        "the page does not change it"
    );
    query.set("B".into());
    screen.run_until_idle();
    assert_eq!(filtered.get_untracked(), ["e", "a"]);
    screen.get_by_text("e,a");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
