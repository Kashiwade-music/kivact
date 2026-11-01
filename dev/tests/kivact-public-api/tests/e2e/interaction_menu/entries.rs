//! Role: Radio items, headings and groups of a menu level.

use super::*;

#[test]
fn radio_items_headings_and_groups_are_rows_of_their_level() {
    let entries = vec![
        Entry::item("new", "New").icon("file-plus"),
        Entry::group(
            "Align",
            vec![
                Entry::radio("left", "Left", true).description("Flush left"),
                Entry::radio("right", "Right", false),
            ],
        ),
        Entry::heading("Danger"),
        Entry::group("", vec![Entry::item("delete", "Delete").destructive(true)]),
    ];
    let (screen, (chosen, rows)) = mount_with(real_window(), move || {
        let chosen = signal(Vec::new());
        let edit = use_menu(entries, move |key: &'static str| {
            chosen.update(|chosen| chosen.push(key))
        });
        let rows = edit.entries();
        (
            (chosen, rows),
            column((menu_button("Edit", edit), action("After"))),
        )
    });
    let kinds: Vec<_> = rows
        .get_untracked()
        .into_iter()
        .map(|(row, _)| row)
        .collect();
    assert_eq!(
        kinds,
        [
            MenuRow::Item {
                key: "new",
                submenu: false,
                check: false,
                radio: false
            },
            MenuRow::Label(0),
            MenuRow::Item {
                key: "left",
                submenu: false,
                check: true,
                radio: true
            },
            MenuRow::Item {
                key: "right",
                submenu: false,
                check: true,
                radio: true
            },
            MenuRow::Label(1),
            MenuRow::Item {
                key: "delete",
                submenu: false,
                check: false,
                radio: false
            },
        ],
        "a group opens in place under its heading; an empty heading shows none"
    );
    let (_, heading) = &rows.get_untracked()[1];
    assert!(heading.is_heading() && heading.label() == "Align");
    let (_, new) = &rows.get_untracked()[0];
    assert_eq!(
        new.icon_value().and_then(|icon| icon.name()),
        Some("file-plus")
    );
    screen
        .get_by_role(Role::Button, "Edit")
        .press_key(Key::ArrowDown);
    assert!(item(&screen, "New").is_focused());
    screen.press_key(Key::ArrowDown);
    let left = screen.get_by_role(Role::MenuItemRadio, "Left");
    assert!(left.is_focused(), "the heading is skipped");
    assert_eq!(left.is_checked(), Some(true));
    assert_eq!(
        screen
            .get_by_role(Role::MenuItemRadio, "Right")
            .is_checked(),
        Some(false)
    );
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(
        item(&screen, "Delete").is_focused(),
        "the second heading is skipped"
    );
    screen.press_key(Key::ArrowUp);
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), ["right"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
