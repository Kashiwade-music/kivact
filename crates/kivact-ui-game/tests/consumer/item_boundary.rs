//! Role: Read-only game items and application-owned dismissal and sorting.
use super::*;
use kivact::view::SortDirection;

#[test]
fn tags_report_requests_and_leave_the_list_to_the_caller() {
    let (screen, (tags, added, removed)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let tags = signal(vec!["first".to_owned()]);
            let added = signal(String::new());
            let removed = signal(None);
            (
                (tags, added, removed),
                column((
                    kivact::ui::tag_input(
                        "Requests",
                        tags,
                        move |tag| added.set(tag),
                        move |index| removed.set(Some(index)),
                    ),
                    kivact::ui::tag_input("Read only", vec!["fixed".to_owned()], |_| {}, |_| {}),
                )),
            )
        });
    let edit = screen.get_by_role(Role::TextBox, "Requests");
    edit.type_text("  next  ");
    edit.press_key(Key::Enter);
    assert_eq!(added.get_untracked(), "next");
    assert_eq!(edit.value().as_deref(), Some(""));
    assert_eq!(tags.get_untracked(), vec!["first"]);
    edit.press_key(Key::Backspace);
    assert_eq!(removed.get_untracked(), Some(0));
    screen.get_by_role(Role::Button, "Remove first").click();
    assert_eq!(tags.get_untracked(), vec!["first"]);
    let readonly = screen.get_by_role(Role::TextBox, "Read only");
    readonly.type_text("draft");
    readonly.press_key(Key::Enter);
    assert_eq!(readonly.value().as_deref(), Some(""));
    screen.get_by_role(Role::Button, "Remove fixed").click();
    assert!(screen.get_by_text("fixed").exists());
    tags.set(vec!["applied".to_owned()]);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Button, "Remove applied").exists());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn alerts_report_dismissal_and_application_controls_visibility() {
    let (screen, (dismissed, shown)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let dismissed = signal(0);
            let shown = signal(true);
            (
                (dismissed, shown),
                column((
                    g::alert("Persistent")
                        .description("Read only alert")
                        .tone(kivact::ui::Tone::Info),
                    g::alert("Rejected")
                        .description("Still visible")
                        .tone(kivact::ui::Tone::Warning)
                        .on_dismiss(move || dismissed.update(|n| *n += 1)),
                    kivact::view::show(shown, move || {
                        g::alert("Applied")
                            .description("Hide on request")
                            .tone(kivact::ui::Tone::Success)
                            .actions(|| text("Action content"))
                            .on_dismiss(move || shown.set(false))
                    }),
                )),
            )
        });
    assert_eq!(screen.all_by_role(Role::Button, "Close alert").len(), 2);
    screen.all_by_role(Role::Button, "Close alert")[0].click();
    assert_eq!(dismissed.get_untracked(), 1);
    assert!(screen.get_by_text("Still visible").exists());
    screen.all_by_role(Role::Button, "Close alert")[1].click();
    assert!(!shown.get_untracked());
    assert!(screen.query_by_text("Hide on request").is_none());
    assert!(screen.get_by_text("Read only alert").exists());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tree_reacts_to_entries_and_keeps_empty_folders_out_of_leaf_selection() {
    let (screen, (entries, selected)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let entries = signal(vec![
                g::TreeEntry::folder("empty", "Empty", vec![]),
                g::TreeEntry::file("leaf", "Leaf"),
            ]);
            let selected = signal(String::new());
            ((entries, selected), g::tree(entries, selected))
        });
    screen.get_by_role(Role::TreeItem, "Empty").click();
    assert_eq!(selected.get_untracked(), "");
    screen.get_by_role(Role::TreeItem, "Leaf").click();
    assert_eq!(selected.get_untracked(), "leaf");
    entries.update(|items| items[0].folder = false);
    screen.run_until_idle();
    screen.get_by_role(Role::TreeItem, "Empty").click();
    assert_eq!(selected.get_untracked(), "empty");
    selected.set("leaf".to_owned());
    entries.update(|items| items[0].folder = true);
    screen.run_until_idle();
    screen.get_by_role(Role::TreeItem, "Empty").click();
    assert_eq!(selected.get_untracked(), "leaf");
    entries.update(|items| items[0].children.push(g::TreeEntry::file("child", "Child")));
    screen.run_until_idle();
    let folder = screen.get_by_role(Role::TreeItem, "Empty");
    folder.click();
    folder.press_key(Key::ArrowRight);
    assert_eq!(folder.is_expanded(), Some(true));
    assert!(screen.get_by_role(Role::TreeItem, "Child").exists());
    folder.click();
    folder.press_key(Key::ArrowLeft);
    entries.update(|items| items[0].label = "Renamed".to_owned());
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_role(Role::TreeItem, "Renamed").is_expanded(),
        Some(false)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A row of the sorting test.
#[derive(Clone, PartialEq)]
struct Task {
    id: &'static str,
    number: i32,
    priority: u8,
    label: &'static str,
}

#[test]
fn table_sorts_by_the_column_compare_and_filters_by_the_column_filters() {
    let (screen, (sort, query, rows)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let sort = signal(Some((0, SortDirection::Ascending)));
            let query = signal(String::new());
            let rows = signal(vec![
                Task {
                    id: "secret-id",
                    number: 10,
                    priority: 9,
                    label: "Urgent",
                },
                Task {
                    id: "other",
                    number: 2,
                    priority: 1,
                    label: "Later",
                },
                Task {
                    id: "text",
                    number: 3,
                    priority: 5,
                    label: "Soon",
                },
            ]);
            let selected = signal(Vec::<&'static str>::new());
            (
                (sort, query, rows),
                kivact::ui::data_table(
                    rows,
                    |task: &Task| task.id,
                    vec![
                        // As text, "10" sorts before "2".
                        kivact::ui::DataColumn::new("Text", |task: &Task| task.number.to_string())
                            .width(160.)
                            .compare(|a: &Task, b: &Task| {
                                a.number.to_string().cmp(&b.number.to_string())
                            }),
                        kivact::ui::DataColumn::new("Priority", |task: &Task| {
                            task.label.to_owned()
                        })
                        .width(160.)
                        .cell(|task: &Task| g::TableCell::Badge {
                            label: task.label.into(),
                            tone: kivact::ui::Tone::Info,
                        })
                        .compare(|a: &Task, b: &Task| a.priority.cmp(&b.priority))
                        .filter(|task: &Task, query: &str| {
                            task.label.to_lowercase().contains(&query.to_lowercase())
                        }),
                        kivact::ui::DataColumn::new("Custom", |task: &Task| {
                            format!("#{}", task.number)
                        })
                        .width(160.)
                        .compare(|a: &Task, b: &Task| a.number.cmp(&b.number)),
                    ],
                )
                .selection(selected)
                .sort(sort)
                .filter(query),
            )
        });
    let y = |name: &str| screen.get_by_role(Role::GridCell, name).rect().unwrap().y;
    assert!(y("10") < y("2"));
    sort.set(Some((2, SortDirection::Ascending)));
    screen.run_until_idle();
    assert!(y("2") < y("3") && y("3") < y("10"));
    sort.set(Some((1, SortDirection::Descending)));
    screen.run_until_idle();
    assert!(y("Urgent") < y("Soon") && y("Soon") < y("Later"));
    screen
        .get_by_role(Role::ColumnHeader, "Priority")
        .press_key(Key::Enter);
    assert_ne!(sort.get_untracked(), Some((1, SortDirection::Descending)));
    query.set("secret-id".into());
    screen.run_until_idle();
    assert!(
        screen.query_by_role(Role::GridCell, "10").is_none(),
        "the key is not a filtered column"
    );
    query.set("urgent".into());
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::GridCell, "10").exists());
    assert!(screen.query_by_role(Role::GridCell, "2").is_none());
    assert_eq!(rows.get_untracked().len(), 3);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
