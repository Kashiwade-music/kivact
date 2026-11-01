//! Role: Tests for the outliner on the core `Tree` through the public input.
//!
//! The outliner on the core `Tree`: keys, rename, activation, drops, the
//! active item, trailing cells and filtering, through the public input only.

use crate::support::clean;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact_ui_creative as ui;

fn scene() -> Vec<ui::TreeItem> {
    vec![
        ui::TreeItem::new(1, None, "Scene", "folder"),
        ui::TreeItem::new(2, Some(1), "Group", "folder"),
        ui::TreeItem::new(3, Some(2), "Cube", "box"),
        ui::TreeItem::new(4, None, "Other", "folder"),
        ui::TreeItem::new(5, Some(1), "Lamp", "bulb"),
    ]
}

fn item<'a>(screen: &'a Screen, name: &str) -> kivact::test::Element<'a> {
    screen.get_by_role(Role::TreeItem, name)
}

#[test]
fn the_rows_follow_the_tree_view_keys_and_are_one_tab_stop() {
    let (screen, (_data, selected)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let data = signal(scene());
            let selected = signal(Vec::new());
            (
                (data, selected),
                column((
                    kivact::ui::button("before").variant(kivact::ui::ButtonVariant::Secondary),
                    app_outliner(
                        data,
                        ui::OutlinerState {
                            selection: selected.into(),
                            ..ui::OutlinerState::new(ids(data))
                        },
                    ),
                )),
            )
        });
    assert!(
        screen
            .get_by_role(Role::Tree, "Outliner")
            .is_multiselectable()
    );
    assert_eq!(item(&screen, "Cube").level(), Some(3));
    assert_eq!(item(&screen, "Lamp").pos_in_set(), Some(2));
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(item(&screen, "Scene").is_focused(), "one tab stop");
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(item(&screen, "Cube").is_focused());
    assert_eq!(selected.get_untracked(), [3]);
    screen.press_key(Key::ArrowLeft);
    assert!(
        item(&screen, "Group").is_focused(),
        "a leaf goes to its parent"
    );
    screen.press_key(Key::ArrowLeft);
    assert_eq!(item(&screen, "Group").is_expanded(), Some(false));
    assert!(screen.query_by_role(Role::TreeItem, "Cube").is_none());
    screen.press_key(Key::ArrowRight);
    assert_eq!(item(&screen, "Group").is_expanded(), Some(true));
    screen.press_key(Key::End);
    assert!(item(&screen, "Other").is_focused());
    screen.press_key(Key::Home);
    screen.press_key(Key::Char('*'));
    assert_eq!(item(&screen, "Scene").is_expanded(), Some(true));
    screen.press_key(Key::Char('o'));
    assert!(item(&screen, "Other").is_focused(), "typeahead");
    clean(&screen);
}

#[test]
fn rename_is_f2_and_a_double_click_unless_the_item_is_activated() {
    let (screen, (data, activated)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let data = signal(scene());
            let activated = signal(Vec::<u64>::new());
            (
                (data, activated),
                app_outliner(data, ui::OutlinerState::new(ids(data))).on_item_activate(move |id| {
                    activated.update(|all| all.push(id));
                }),
            )
        });
    // With an activation callback a double click and Enter activate.
    item(&screen, "Other").double_click();
    assert_eq!(activated.get_untracked(), [4], "once per double click");
    assert!(
        screen
            .query_by_role(Role::TextBox, "Rename tree item")
            .is_none()
    );
    screen.press_key(Key::Enter);
    assert_eq!(activated.get_untracked(), [4, 4]);
    // F2 renames.
    item(&screen, "Lamp").click();
    screen.press_key(Key::F(2));
    screen
        .get_by_role(Role::TextBox, "Rename tree item")
        .type_text("Light");
    screen.press_key(Key::Enter);
    assert_eq!(data.get_untracked()[4].label, "Light");
    assert!(item(&screen, "Light").is_focused());
    clean(&screen);

    // Without one, a double click renames, and Escape keeps the name.
    let (screen, (data, rename)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let data = signal(scene());
            let rename = signal(None::<u64>);
            (
                (data, rename),
                app_outliner(
                    data,
                    ui::OutlinerState {
                        renaming: rename.into(),
                        ..ui::OutlinerState::new(ids(data))
                    },
                ),
            )
        });
    item(&screen, "Cube").double_click();
    assert_eq!(rename.get_untracked(), Some(3));
    screen
        .get_by_role(Role::TextBox, "Rename tree item")
        .type_text("Mesh");
    screen.press_key(Key::Escape);
    assert_eq!(rename.get_untracked(), None);
    assert_eq!(data.get_untracked()[2].label, "Cube");
    assert!(item(&screen, "Cube").is_focused());
    item(&screen, "Cube").press_key(Key::F(2));
    screen
        .get_by_role(Role::TextBox, "Rename tree item")
        .type_text("Mesh");
    screen.press_key(Key::Enter);
    assert_eq!(data.get_untracked()[2].label, "Mesh");
    clean(&screen);
}

/// Every item's ID, which an outliner state opens.
fn ids(items: Signal<Vec<ui::TreeItem>>) -> Vec<u64> {
    items.with_untracked(|items| items.iter().map(|item| item.id).collect())
}

fn app_outliner(items: Signal<Vec<ui::TreeItem>>, state: ui::OutlinerState) -> ui::Outliner {
    let initial = signal(Vec::new());
    ui::outliner(items, state)
        .on_item_drop(move |drop: ui::TreeDrop| {
            items.update(|items| {
                let Some(from) = items.iter().position(|item| item.id == drop.id) else {
                    return;
                };
                let mut item = items.remove(from);
                item.parent = drop.target;
                let at = match drop.after {
                    Some(after) => items
                        .iter()
                        .position(|item| item.id == after)
                        .map_or(items.len(), |at| at + 1),
                    None => items
                        .iter()
                        .position(|item| item.parent == drop.target)
                        .unwrap_or(from.min(items.len())),
                };
                items.insert(at, item);
            });
        })
        .on_rename(move |(id, name): (u64, String)| {
            if name.trim().is_empty() {
                return;
            }
            items.update(|items| {
                if let Some(item) = items.iter_mut().find(|item| item.id == id) {
                    item.label = name;
                }
            });
        })
        .on_restriction(move |event: ui::EditorEvent<ui::Restriction>| match event {
            ui::EditorEvent::Begin => initial.set(items.get_untracked()),
            ui::EditorEvent::Request(request) => items.update(|items| {
                for item in items
                    .iter_mut()
                    .filter(|item| request.ids.contains(&item.id))
                {
                    match request.flag {
                        ui::RestrictionFlag::Visible => item.visible = request.value,
                        ui::RestrictionFlag::Selectable => item.selectable = request.value,
                        ui::RestrictionFlag::Rendered => item.rendered = request.value,
                    }
                }
            }),
            ui::EditorEvent::Cancel => items.set(initial.get_untracked()),
            ui::EditorEvent::Commit => {}
        })
}
