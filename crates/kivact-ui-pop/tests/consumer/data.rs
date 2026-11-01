//! Role: The table's search, sort, selection and paging through the public API.

use kivact::prelude::*;

#[test]
fn table_search_sort_selection_paging_are_external_consumer_behavior() {
    #[derive(Clone, PartialEq)]
    struct Member {
        id: u64,
        name: &'static str,
        level: u32,
    }
    let (screen, selected) = kivact::test::render_with_handle(crate::support::options(), || {
        let selected = signal(Vec::<u64>::new());
        let sort = signal(None);
        let page = signal(0_usize);
        (
            selected,
            kivact::ui::data_table(
                vec![
                    Member {
                        id: 1,
                        name: "Beta",
                        level: 2,
                    },
                    Member {
                        id: 2,
                        name: "Alpha",
                        level: 10,
                    },
                    Member {
                        id: 3,
                        name: "Gamma",
                        level: 4,
                    },
                ],
                |m: &Member| m.id,
                vec![
                    kivact::ui::DataColumn::new("Name", |m: &Member| m.name.to_owned())
                        .width(150.0)
                        .compare(|a, b| a.name.cmp(b.name))
                        .filter(|m, query| m.name.contains(query))
                        .cell(|m| text(m.name)),
                    kivact::ui::DataColumn::new("Level", |m: &Member| m.level.to_string())
                        .width(80.0)
                        .compare(|a, b| a.level.cmp(&b.level))
                        .numeric(true),
                ],
            )
            .selection(selected)
            .sort(sort)
            .filter(signal(String::new()))
            .page(page, 2),
        )
    });
    screen.get_by_role(Role::ColumnHeader, "Name").click();
    screen.get_by_text("Alpha");
    screen.get_by_role(Role::CheckBox, "Select Alpha").click();
    assert_eq!(selected.get_untracked(), vec![2]);
    assert!(
        screen
            .get_by_role(Role::CheckBox, "Select all")
            .is_indeterminate()
    );
    screen.get_by_role(Role::Button, "Next").click();
    screen.get_by_text("Gamma");
    screen
        .get_by_role(Role::TextBox, "Filter…")
        .type_text("Beta");
    assert!(screen.query_by_text("Gamma").is_none());
    screen.get_by_text("Beta");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
