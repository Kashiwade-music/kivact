//! Role: Standard compound components apply recipes to shared interaction parts.

use crate::support::mount;
use kivact::interaction::{ListItem, MenuEntry, TreeNode};
use kivact::prelude::*;
use kivact::ui;
use kivact::view::{Orientation, Placement};
use std::time::Duration;

#[test]
fn roving_group_has_one_tab_stop_and_leaves_editing_arrows_to_the_caret() {
    let (screen, ()) = mount(|| {
        let text = signal("ab".to_owned());
        (
            (),
            column((
                button("Before"),
                row((button("First"), text_input("Edit", text), button("Last")))
                    .roving_focus(Orientation::Horizontal),
                button("After"),
            )),
        )
    });
    screen.press_tab(false);
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "First").is_focused());
    screen.press_key(Key::ArrowRight);
    let editor = screen.get_by_role(Role::TextBox, "Edit");
    assert!(editor.is_focused());
    screen.press_key(Key::Home);
    for _ in 0..2 {
        screen.press_key(Key::ArrowRight);
        assert!(editor.is_focused());
    }
    screen.press_key(Key::ArrowRight);
    let last = screen.get_by_role(Role::Button, "Last");
    assert!(last.is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    screen.press_tab(true);
    assert!(last.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn standard_tree_expands_and_moves_focus_into_the_branch() {
    let (screen, expanded) = mount(|| {
        let expanded = signal(Vec::<u32>::new());
        let selected = signal(None);
        let nodes = vec![
            TreeNode::new(1, None, "Folder"),
            TreeNode::new(2, Some(1), "Document"),
        ];
        (
            expanded,
            ui::tree("Files", nodes, selected.into_selection()).expanded(expanded),
        )
    });
    screen.get_by_role(Role::Tree, "Files");
    let folder = screen.get_by_role(Role::TreeItem, "Folder");
    assert_eq!(folder.is_expanded(), Some(false));
    assert!(screen.query_by_role(Role::TreeItem, "Document").is_none());
    folder.press_key(Key::ArrowRight);
    assert_eq!(expanded.get_untracked(), [1]);
    folder.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::TreeItem, "Document").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn select_keeps_reactive_options_placeholder_and_keyboard_selection() {
    let (screen, (selected, label)) = mount(|| {
        let selected = signal(None::<u32>);
        let label = signal("Alpha".to_owned());
        let options = vec![ListItem::new(1, label), ListItem::new(2, "Beta")];
        (
            (selected, label),
            ui::select("Original", options, selected)
                .name("Choice")
                .placeholder("Pick one"),
        )
    });
    screen.get_by_text("Pick one");
    screen.get_by_role(Role::ComboBox, "Choice").click();
    screen.get_by_role(Role::ListBox, "Choice");
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), Some(2));
    selected.set(Some(1));
    label.set("Renamed".to_owned());
    screen.run_until_idle();
    screen.get_by_text("Renamed");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn menu_renders_nested_levels_and_custom_trigger_keeps_its_style() {
    let (screen, chosen) = mount(|| {
        let chosen = signal(None);
        (
            chosen,
            ui::menu(
                "Actions",
                vec![
                    MenuEntry::submenu(1, "Export", vec![MenuEntry::item(2, "PNG")]),
                    MenuEntry::separator(),
                    MenuEntry::checkbox(3, "Grid", true),
                ],
            )
            .on_activate(move |key| chosen.set(Some(key)))
            .attach(
                ui::button("Custom")
                    .style(style().size(140, 30))
                    .test_id("trigger"),
            ),
        )
    });
    let trigger = screen.get_by_test_id("trigger");
    assert_eq!(trigger.rect().expect("layout").width, 140.0);
    trigger.click();
    screen.get_by_role(Role::Menu, "Custom");
    let parent = screen
        .get_by_role(Role::Menu, "Custom")
        .rect()
        .expect("root menu");
    let anchor = trigger.rect().expect("trigger");
    assert!((parent.y - anchor.y - anchor.height - 4.0).abs() < 0.5);
    let export = screen.get_by_role(Role::MenuItem, "Export");
    let item = export.rect().expect("submenu item");
    export.press_key(Key::ArrowRight);
    let submenu = screen
        .get_by_role(Role::Menu, "Export")
        .rect()
        .expect("submenu");
    assert!(
        (submenu.x - item.x - item.width).abs() < 0.5,
        "submenu has no added gap: {submenu:?} {item:?}"
    );
    screen
        .get_by_role(Role::MenuItem, "PNG")
        .press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), Some(2));
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dialog_preserves_fresh_content_dismissal_and_focus_restoration() {
    let (screen, (open, mounts)) = mount(|| {
        let open = signal(false);
        let mounts = signal(0);
        (
            (open, mounts),
            column((
                ui::button("Open").on_press(move || open.set(true)),
                ui::dialog("Settings", open)
                    .description("Preferences")
                    .content(move || {
                        mounts.update(|n| *n += 1);
                        ui::text_input("Name", signal(String::new()))
                    }),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Open").click();
    screen.get_by_role(Role::Dialog, "Settings");
    screen.get_by_role(Role::Heading, "Settings");
    assert_eq!(mounts.get_untracked(), 1);
    screen.get_by_role(Role::Button, "Close").click();
    assert!(!open.get_untracked());
    assert!(screen.get_by_role(Role::Button, "Open").is_focused());
    screen.get_by_role(Role::Button, "Open").click();
    assert_eq!(mounts.get_untracked(), 2);
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tabs_render_shared_key_navigation_and_selected_panel_lifetime() {
    let (screen, selected) = mount(|| {
        let selected = signal(1);
        (
            selected,
            ui::tabs(
                "Tabs",
                vec![ListItem::new(1, "One"), ListItem::new(2, "Two")],
                selected,
            )
            .content(|key| text(format!("Panel {key}"))),
        )
    });
    screen.get_by_role(Role::Tab, "One").focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), 2);
    screen.get_by_text("Panel 2");
    assert!(screen.query_by_text("Panel 1").is_none());
    screen.get_by_role(Role::TabPanel, "Two");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tooltip_uses_shared_delay_and_native_text_area_remains_controlled() {
    let (screen, value) = mount(|| {
        let value = signal("Initial".to_owned());
        (
            value,
            column((
                ui::tooltip("A hint")
                    .content(|| text("Rich detail").style(style().weight(700)))
                    .placement(Placement::Below)
                    .attach(ui::button("Help")),
                ui::text_area("Notes", value),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Help").focus();
    screen.advance(Duration::from_millis(500));
    screen.get_by_role(Role::Tooltip, "A hint");
    screen.get_by_text("Rich detail");
    let field = screen.get_by_role(Role::TextBox, "Notes");
    field.press_key(Key::End);
    field.type_text(" more");
    assert_eq!(value.get_untracked(), "Initial more");
    value.set("Changed".to_owned());
    screen.run_until_idle();
    assert_eq!(field.value().as_deref(), Some("Changed"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_soft_wrapping_text_area_grows_to_its_visual_row_limit() {
    let (screen, value) = mount(|| {
        let value = signal("alpha".to_owned());
        (
            value,
            column((ui::text_area("Notes", value)
                .wrap(true)
                .rows(1..=3)
                .style(style().width(104).font_size(16)),)),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Notes");
    let height = || field.rect().expect("text area layout").height;
    let one = height();
    value.set("alpha beta gamma".to_owned());
    screen.run_until_idle();
    assert!(height() > one, "soft rows increase the height");
    value.set("alpha beta gamma ".repeat(8));
    screen.run_until_idle();
    let capped = height();
    value.set("alpha beta gamma ".repeat(16));
    screen.run_until_idle();
    assert_eq!(height(), capped, "additional visual rows scroll");
    assert!(!value.get_untracked().contains('\n'));
    value.set("alpha".to_owned());
    screen.run_until_idle();
    assert_eq!(height(), one, "a shorter value shrinks the field");
}

#[test]
fn context_menu_reuses_renderer_for_keyboard_and_pointer_origins() {
    use kivact::view::MouseButton;
    let (screen, chosen) = mount(|| {
        let chosen = signal(0);
        (
            chosen,
            column((
                ui::button("Outside"),
                ui::menu("Context", vec![MenuEntry::item(1, "Inspect")])
                    .on_activate(move |key| chosen.set(key))
                    .attach_context(
                        node()
                            .name("Target")
                            .role(Role::Button)
                            .focusable(true)
                            .style(style().size(180, 70))
                            .child(text("Target")),
                    ),
            )),
        )
    });
    let target = screen.get_by_role(Role::Button, "Target");
    target.focus();
    screen.press_key(Key::ContextMenu);
    screen.get_by_role(Role::Menu, "Target");
    screen
        .get_by_role(Role::MenuItem, "Inspect")
        .press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), 1);
    assert!(target.is_focused());
    let outside = screen.get_by_role(Role::Button, "Outside");
    outside.focus();
    target.click_with(MouseButton::Secondary);
    screen.get_by_role(Role::MenuItem, "Inspect");
    screen.press_key(Key::Escape);
    assert!(
        target.is_focused(),
        "the native context press focused its target before opening"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
