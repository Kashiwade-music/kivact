//! Role: Verify column filtering through the standard data table public API.
use kivact::state::signal;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::ui::DataColumn;
use kivact::ui::button;
use kivact::ui::data_table;
use kivact::view::Element;
use kivact::view::Role;
use kivact::view::column;
use kivact::view::text;

#[derive(Clone, PartialEq)]
struct Entry {
    id: u8,
    name: &'static str,
    group: &'static str,
}

#[test]
fn column_filters_combine_with_global_or_and_ignore_ineligible_columns() {
    let screen = render_with(RenderOptions::from_env(), || {
        let rows = vec![
            Entry {
                id: 1,
                name: "amy",
                group: "red",
            },
            Entry {
                id: 2,
                name: "bob",
                group: "amber",
            },
            Entry {
                id: 3,
                name: "abe",
                group: "blue",
            },
            Entry {
                id: 4,
                name: "zoe",
                group: "red",
            },
        ];
        let query = signal("a".to_owned());
        let visible = signal(false);
        let filters = signal(Vec::new());
        let table = data_table(
            rows,
            |entry: &Entry| entry.id,
            vec![
                DataColumn::new("Name", |entry: &Entry| entry.name.to_owned())
                    .filter(|entry, query| entry.name.contains(query)),
                DataColumn::new("Group", |entry: &Entry| entry.group.to_owned())
                    .filter(|entry, query| entry.group.contains(query)),
                DataColumn::new("ID", |entry: &Entry| entry.id.to_string()),
                DataColumn::new("Hidden", |entry: &Entry| entry.name.to_owned())
                    .filter(|entry, query| entry.name.contains(query))
                    .visible(visible),
            ],
        )
        .filter(query)
        .column_filters(filters)
        .footer(move |context| {
            let keys = context.model.filtered_keys();
            text(move || format!("Keys: {:?}", keys.get()))
        });
        column((
            button("Apply queries").on_press(move || {
                filters.set(vec![
                    (0, " a ".to_owned()),
                    (1, "red".to_owned()),
                    (1, "  ".to_owned()),
                    (2, "ignored".to_owned()),
                    (3, "no such name".to_owned()),
                    (99, "stale".to_owned()),
                ])
            }),
            button("Show hidden column").on_press(move || visible.set(true)),
            table,
        ))
    });

    screen.get_by_role(Role::Text, "Keys: [1, 2, 3]");
    screen.get_by_role(Role::Button, "Apply queries").click();
    screen.get_by_role(Role::Text, "Keys: [1]");
    screen
        .get_by_role(Role::Button, "Show hidden column")
        .click();
    screen.get_by_role(Role::Text, "Keys: []");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
