//! Role: Tests a menu's placement, width, inset rows, submenu surface and code control.
//!
//! On real windows through the public API: where the root surface opens
//! (`Menu::placement`, `Menu::align`), that it can be at least as wide as
//! its trigger, the inset box of inset items and headings, the submenu's
//! own surface recipe, a context menu at the pointer with its own surface
//! recipe, and opening and closing the menu from code through
//! `Menu::behavior`.

use crate::paint::{distance, pixel};
use crate::support::{mount, mount_with};
use kivact::interaction::PopupState;
use kivact::prelude::*;
use kivact::style::{Align, rgb};
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::ui::{MenuEntry, UiRecipes};
use kivact::view::{OverlayAlign, Placement};

const RED: u32 = 0xff0000;

fn rect(screen: &Screen, role: Role, name: &str) -> kivact::draw::Rect {
    screen.get_by_role(role, name).rect().expect("laid out")
}

#[test]
fn a_menu_opens_on_its_side_and_alignment_at_least_as_wide_as_its_trigger() {
    let (screen, ()) = mount(|| {
        let entries = vec![MenuEntry::item("a", "A"), MenuEntry::item("b", "B")];
        let trigger = button("A trigger wider than the menu")
            .test_id("trigger")
            .style(style().width(320));
        let view = column(
            menu("Actions", entries, |_: &str| ())
                .placement(Placement::Above)
                .align(OverlayAlign::End)
                .at_least_trigger_width(true)
                .attach(trigger),
        )
        .style(style().padding_top(240).padding_left(40));
        ((), view)
    });
    screen.get_by_test_id("trigger").click();
    let trigger = screen.get_by_test_id("trigger").rect().expect("laid out");
    let surface = rect(&screen, Role::Menu, "A trigger wider than the menu");
    assert!(
        surface.y + surface.height <= trigger.y + 0.5,
        "above the trigger: {surface:?} {trigger:?}"
    );
    assert!(
        surface.width >= trigger.width - 0.5,
        "at least as wide: {surface:?} {trigger:?}"
    );
    assert!(
        (surface.x + surface.width - (trigger.x + trigger.width)).abs() <= 0.5,
        "end-aligned: {surface:?} {trigger:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn inset_items_and_headings_keep_the_inset_box_before_their_text() {
    let (screen, ()) = mount(|| {
        let entries = vec![
            MenuEntry::item("plain", "Plain"),
            MenuEntry::item("indented", "Indented").inset(true),
            MenuEntry::heading("Section").inset(true),
            MenuEntry::heading("Flush"),
        ];
        ((), menu("Rows", entries, |_: &str| ()))
    });
    screen.get_by_role(Role::Button, "Rows").click();
    let x = |text: &str| screen.get_by_text(text).rect().expect("laid out").x;
    // The default inset box is the check box's 16 px, then the row's gap.
    assert!(
        (x("Indented") - x("Plain") - 24.0).abs() <= 0.5,
        "{} {}",
        x("Indented"),
        x("Plain")
    );
    assert!(
        x("Section") - x("Flush") >= 16.0 - 0.5,
        "{} {}",
        x("Section"),
        x("Flush")
    );
    assert_eq!(
        screen.get_by_role(Role::MenuItem, "Indented").name(),
        "Indented",
        "the inset box adds no name"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_submenu_draws_its_surface_with_the_submenu_surface_recipe() {
    let theme = Theme::light();
    let recipes = UiRecipes::clone(&theme.get::<UiRecipes>()).menu(|menu| {
        menu.sub_surface(|prev| {
            move |theme: &Theme, state: &kivact::ui::Look<PopupState>| {
                prev(theme, state).background(rgb(RED))
            }
        })
    });
    let options = RenderOptions {
        theme: theme.with(recipes),
        window: WindowMode::Real,
        reduced_motion: true,
        ..RenderOptions::from_env()
    };
    let (screen, ()) = mount_with(options, || {
        let entries = vec![MenuEntry::submenu(
            "share",
            "Share",
            vec![MenuEntry::item("mail", "Mail")],
        )];
        ((), menu("Send", entries, |_: &str| ()))
    });
    screen.get_by_role(Role::Button, "Send").click();
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowRight);
    let root = rect(&screen, Role::Menu, "Send");
    let sub = rect(&screen, Role::Menu, "Share");
    let shot = screen.screenshot();
    // In the surface's padding, clear of its rounded corners.
    let inside =
        |r: kivact::draw::Rect| pixel(&shot, (r.x + r.width / 2.0) as u32, (r.y + 2.0) as u32);
    assert!(distance(inside(sub), RED) <= 16, "{:06x}", inside(sub));
    assert!(distance(inside(root), RED) > 64, "{:06x}", inside(root));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn code_opens_and_closes_a_themed_menu_through_its_behavior() {
    let (screen, screen_behavior) = mount(|| {
        let entries = vec![MenuEntry::item("undo", "Undo")];
        let edit = menu("Edit", entries, |_: &str| ());
        let behavior = edit.behavior();
        let view = column((
            edit,
            button("Open from code").on_press(move || behavior.open()),
        ));
        (behavior, view)
    });
    let behavior = screen_behavior;
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    screen.get_by_role(Role::Button, "Open from code").click();
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Menu, "Edit").exists());
    assert!(screen.get_by_role(Role::MenuItem, "Undo").is_focused());
    behavior.close();
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Menu, "Edit").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_trigger_keeps_a_role_of_its_own_and_a_plain_node_becomes_a_button() {
    let (screen, ()) = mount(|| {
        let entries = || vec![MenuEntry::item("new", "New")];
        let view = row((
            menu("File", entries(), |_: &str| ())
                .attach(node().role(Role::MenuItem).child(text("File"))),
            menu("Edit", entries(), |_: &str| ()).attach(node().child(text("Edit"))),
        ))
        .role(Role::MenuBar);
        ((), view)
    });
    let file = screen.get_by_role(Role::MenuItem, "File");
    assert_eq!(file.is_expanded(), Some(false));
    file.click();
    assert_eq!(file.is_expanded(), Some(true));
    screen.press_key(Key::Escape);
    screen.get_by_role(Role::Button, "Edit");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_context_menu_opens_right_at_the_pointer_with_its_own_surface_recipe() {
    use kivact::view::MouseButton;
    let theme = Theme::light();
    let recipes = UiRecipes::clone(&theme.get::<UiRecipes>()).menu(|menu| {
        menu.context_surface(|prev| {
            move |theme: &Theme, state: &kivact::ui::Look<PopupState>| {
                prev(theme, state).background(rgb(RED))
            }
        })
    });
    let options = RenderOptions {
        theme: theme.with(recipes),
        window: WindowMode::Real,
        reduced_motion: true,
        ..RenderOptions::from_env()
    };
    let (screen, ()) = mount_with(options, || {
        let entries = vec![MenuEntry::item("copy", "Copy")];
        let area = node()
            .name("Canvas")
            .role(Role::Group)
            .focusable(true)
            .style(style().size(200, 120));
        let view = column((
            menu("Edit", vec![MenuEntry::item("undo", "Undo")], |_: &str| ()),
            menu("Context", entries, |_: &str| ()).attach_context(area),
        ))
        .style(style().padding(40).gap(8).align_items(Align::Start));
        ((), view)
    });
    let area = rect(&screen, Role::Group, "Canvas");
    screen
        .get_by_role(Role::Group, "Canvas")
        .click_with(MouseButton::Secondary);
    let surface = rect(&screen, Role::Menu, "Canvas");
    let pointer = (area.x + area.width / 2.0, area.y + area.height / 2.0);
    assert!(
        (surface.x - pointer.0).abs() <= 0.5 && (surface.y - pointer.1).abs() <= 0.5,
        "no gap at the pointer: {surface:?} {pointer:?}"
    );
    let shot = screen.screenshot();
    let inside =
        |r: kivact::draw::Rect| pixel(&shot, (r.x + r.width / 2.0) as u32, (r.y + 2.0) as u32);
    assert!(
        distance(inside(surface), RED) <= 16,
        "{:06x}",
        inside(surface)
    );
    screen.press_key(Key::Escape);
    screen.get_by_role(Role::Button, "Edit").click();
    let shot = screen.screenshot();
    let dropdown = rect(&screen, Role::Menu, "Edit");
    let trigger = rect(&screen, Role::Button, "Edit");
    let inside =
        |r: kivact::draw::Rect| pixel(&shot, (r.x + r.width / 2.0) as u32, (r.y + 2.0) as u32);
    assert!(
        distance(inside(dropdown), RED) > 64,
        "the dropdown keeps the surface recipe"
    );
    assert!(
        dropdown.y >= trigger.y + trigger.height + 3.5,
        "a dropdown keeps its gap: {dropdown:?} {trigger:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
