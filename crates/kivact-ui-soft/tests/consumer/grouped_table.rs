//! Role: Exercise grouped standard table rows through the public facade.
use kivact::interaction::ListActivation;
use kivact::interaction::ListOptions;
use kivact::interaction::SortOrder;
use kivact::interaction::TableFocus;
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::test::MirrorMode;
use kivact::test::RenderOptions;
use kivact::test::render_with_handle;
use kivact::ui::DataColumn;
use kivact::ui::data_table;
use kivact::view::Key;
use kivact::view::Role;

#[derive(Clone, PartialEq)]
struct Person {
    id: u32,
    name: &'static str,
    team: &'static str,
}

#[test]
fn group_disclosure_preserves_selection_and_pages_and_skips_collapsed_rows() {
    let (screen, (collapsed, selected, page, filter)) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)).mirror(MirrorMode::Off),
        || {
            let people = vec![
                Person {
                    id: 4,
                    name: "Zeta",
                    team: "Engineering",
                },
                Person {
                    id: 3,
                    name: "Gamma",
                    team: "Design",
                },
                Person {
                    id: 2,
                    name: "Beta",
                    team: "Engineering",
                },
                Person {
                    id: 1,
                    name: "Alpha",
                    team: "Design",
                },
            ];
            let collapsed = signal(Vec::<String>::new());
            let selected = signal(vec![2]);
            let page = signal(0);
            let filter = signal(String::new());
            let sort = signal(Some((0, SortOrder::Ascending)));
            let table = data_table(
                people,
                |person: &Person| person.id,
                vec![
                    DataColumn::new("Name", |person: &Person| person.name.to_owned())
                        .compare(|a, b| a.name.cmp(b.name))
                        .filter(|person, query| person.name.contains(query)),
                ],
            )
            .sort(sort)
            .filter(filter)
            .page(page, 3)
            .selection(selected)
            .group_by(|person| person.team, collapsed)
            .focus(TableFocus::Rows(ListOptions {
                activation: ListActivation::Manual,
                ..ListOptions::default()
            }));
            ((collapsed, selected, page, filter), table)
        },
    );
    let design = screen.get_by_role(Role::Button, "Collapse group Design");
    let engineering = screen.get_by_role(Role::Button, "Collapse group Engineering");
    let design_row = screen.get_by_role(Role::Row, "Design");
    assert_eq!(design_row.get_by_role(Role::Text, "(2)").text(), "(2)");
    let alpha = screen.get_by_role(Role::Row, "Alpha");
    let gamma = screen.get_by_role(Role::Row, "Gamma");
    assert!(design.rect().unwrap().y < alpha.rect().unwrap().y);
    assert!(alpha.rect().unwrap().y < gamma.rect().unwrap().y);
    assert!(gamma.rect().unwrap().y < engineering.rect().unwrap().y);
    assert!(screen.query_by_role(Role::Row, "Zeta").is_none());
    assert_eq!(engineering.is_expanded(), Some(true));

    engineering.focus();
    screen.press_key(Key::Space);
    assert_eq!(collapsed.get_untracked(), ["Engineering"]);
    assert_eq!(selected.get_untracked(), [2]);
    assert!(screen.query_by_role(Role::Row, "Beta").is_none());
    assert!(screen.query_by_role(Role::Row, "Zeta").is_none());
    let engineering = screen.get_by_role(Role::Button, "Expand group Engineering");
    assert_eq!(engineering.is_expanded(), Some(false));
    assert!(engineering.is_focused());

    gamma.focus();
    screen.press_key(Key::ArrowUp);
    assert!(alpha.is_focused());
    assert_eq!(selected.get_untracked(), [2]);

    page.set(1);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Row, "Zeta").is_none());
    screen
        .get_by_role(Role::Button, "Expand group Engineering")
        .click();
    assert!(screen.query_by_role(Role::Row, "Zeta").is_some());
    assert_eq!(page.get_untracked(), 1);

    filter.set("Beta".to_owned());
    page.set(0);
    screen.run_until_idle();
    assert!(
        screen
            .query_by_role(Role::Button, "Collapse group Design")
            .is_none()
    );
    assert!(screen.query_by_role(Role::Row, "Beta").is_some());
    assert_eq!(selected.get_untracked(), [2]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn remote_pages_group_the_supplied_rows_without_slicing_them_again() {
    let (screen, page) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)).mirror(MirrorMode::Off),
        || {
            let page = signal(7);
            let collapsed = signal(Vec::<String>::new());
            (
                page,
                data_table(
                    vec![(41, "Ada"), (42, "Grace")],
                    |row| row.0,
                    vec![DataColumn::new("Name", |row: &(i32, &str)| {
                        row.1.to_owned()
                    })],
                )
                .remote_page(page, 2, false)
                .group_by(|_| "Engineering", collapsed),
            )
        },
    );
    let header = screen.get_by_role(Role::Row, "Engineering");
    assert_eq!(header.get_by_role(Role::Text, "(2)").text(), "(2)");
    assert!(screen.query_by_role(Role::Row, "Ada").is_some());
    assert!(screen.query_by_role(Role::Row, "Grace").is_some());
    assert_eq!(page.get_untracked(), 7);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
