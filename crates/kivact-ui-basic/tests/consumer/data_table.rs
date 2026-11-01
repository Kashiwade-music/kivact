//! Role: Data table tests for the key function, the texts and the root semantics.

use super::assert_clean;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::RenderOptions;
use kivact_ui_basic as basic;

#[derive(Clone, PartialEq)]
struct Person {
    id: u32,
    name: &'static str,
}

fn people() -> Vec<Person> {
    vec![
        Person { id: 1, name: "Ada" },
        Person {
            id: 2,
            name: "Grace",
        },
        Person {
            id: 3,
            name: "Linus",
        },
    ]
}

fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .color(basic::Color::Foreground)
            .padding(16)
            .grow(1)
            .align_items(Align::Start),
    )
}

fn options() -> RenderOptions {
    RenderOptions {
        theme: basic::theme(kivact::style::ThemeMode::Light),
        width: 640.0,
        height: 600.0,
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}

#[test]
fn the_key_function_may_capture_and_names_the_selected_rows() {
    let (screen, selected) = kivact::test::render_with_handle(options(), || {
        let selected = signal(Vec::<u32>::new());
        // A key that closes over its environment: ids are shifted by a base.
        let base = 1000u32;
        (
            selected,
            page(
                kivact::ui::data_table(
                    people(),
                    move |p: &Person| p.id + base,
                    vec![kivact::ui::DataColumn::new("Name", |p: &Person| {
                        p.name.to_string()
                    })],
                )
                .row_label(|p: &Person| p.name.to_string())
                .selection(selected)
                .style(style().width(560)),
            ),
        )
    });
    screen.get_by_role(Role::CheckBox, "Select Grace").click();
    screen.run_until_idle();
    assert_eq!(selected.get_untracked(), vec![1002]);
    screen.get_by_text("1 of 3 row(s) selected.");
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn an_empty_table_says_no_results_found_like_every_other_list() {
    let (screen, ()) = kivact::test::render_with_handle(options(), || {
        (
            (),
            page(
                kivact::ui::data_table(
                    Vec::<Person>::new(),
                    |p: &Person| p.id,
                    vec![kivact::ui::DataColumn::new("Name", |p: &Person| {
                        p.name.to_string()
                    })],
                )
                .style(style().width(560)),
            ),
        )
    });
    screen.get_by_text("No results found.");
    assert_clean(&screen);
}

#[test]
fn the_root_is_a_group_and_the_grid_is_named_by_the_message() {
    let (screen, ()) = kivact::test::render_with_handle(options(), || {
        (
            (),
            page(
                kivact::ui::data_table(
                    people(),
                    |p: &Person| p.id,
                    vec![kivact::ui::DataColumn::new("Name", |p: &Person| {
                        p.name.to_string()
                    })],
                )
                .name("People")
                .style(style().width(560)),
            ),
        )
    });
    screen.get_by_role(Role::Group, "People");
    screen.get_by_role(Role::Grid, "Data table");
    assert_clean(&screen);
}
