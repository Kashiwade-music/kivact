//! Role: Verify APG menubar navigation across popup surfaces.
use super::support::mount_with;
use kivact::interaction::{MenuEntry, use_menu, use_menubar};
use kivact::prelude::*;
use kivact::test::RenderOptions;
use kivact::view::{Placement, overlay};

#[test]
fn menubar_leaf_arrows_move_between_open_menus() {
    let (screen, ()) = mount_with(RenderOptions::from_env(), || {
        let file = use_menu(vec![MenuEntry::item("new", "New")]);
        let edit = use_menu(vec![MenuEntry::item("copy", "Copy")]);
        let bar = use_menubar(
            vec!["File".to_string(), "Edit".to_string()],
            vec![file, edit],
        );
        let file_anchor = node_ref();
        let edit_anchor = node_ref();
        (
            (),
            column((
                row((
                    node()
                        .part(file.trigger())
                        .part(bar.item(0))
                        .node_ref(file_anchor)
                        .name("File")
                        .child(text("File")),
                    node()
                        .part(edit.trigger())
                        .part(bar.item(1))
                        .node_ref(edit_anchor)
                        .name("Edit")
                        .child(text("Edit")),
                ))
                .part(bar.bar()),
                file.show(move || {
                    overlay(
                        file_anchor,
                        Placement::Below,
                        node().part(file.item("new")).child(text("New")),
                    )
                    .with_node(move |node| node.part(file.content()))
                    .name("File")
                }),
                edit.show(move || {
                    overlay(
                        edit_anchor,
                        Placement::Below,
                        node().part(edit.item("copy")).child(text("Copy")),
                    )
                    .with_node(move |node| node.part(edit.content()))
                    .name("Edit")
                }),
            )),
        )
    });
    screen
        .get_by_role(Role::MenuItem, "File")
        .press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::MenuItem, "New").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(screen.query_by_role(Role::Menu, "File").is_none());
    assert!(screen.get_by_role(Role::MenuItem, "Copy").is_focused());
    screen.press_key(Key::ArrowLeft);
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert!(screen.get_by_role(Role::MenuItem, "New").is_focused());
    screen.press_key(Key::Escape);
    assert!(screen.get_by_role(Role::MenuItem, "File").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::MenuItem, "Edit").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn menubar_trigger_part_order_keeps_menuitem_role() {
    let (screen, ()) = mount_with(RenderOptions::from_env(), || {
        let file = use_menu(vec![MenuEntry::item("new", "New")]);
        let edit = use_menu(vec![MenuEntry::item("copy", "Copy")]);
        let bar = use_menubar(
            vec!["File".to_string(), "Edit".to_string()],
            vec![file, edit],
        );
        (
            (),
            row((
                node()
                    .part(file.trigger())
                    .part(bar.item(0))
                    .name("File")
                    .child(text("File")),
                node()
                    .part(bar.item(1))
                    .part(edit.trigger())
                    .name("Edit")
                    .child(text("Edit")),
            ))
            .part(bar.bar()),
        )
    });
    assert!(screen.get_by_role(Role::MenuItem, "File").exists());
    assert!(screen.get_by_role(Role::MenuItem, "Edit").exists());
    assert!(screen.query_by_role(Role::Button, "Edit").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
