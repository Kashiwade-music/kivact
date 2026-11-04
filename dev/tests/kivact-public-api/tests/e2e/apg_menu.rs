//! Role: Verify APG menu focus and submenu semantics.
use super::interaction_menu::menu_button;
use super::support::mount_with;
use kivact::interaction::{MenuEntry, use_menu};
use kivact::prelude::*;
use kivact::test::RenderOptions;
use kivact::view::{A11yAction, PopupKind};

#[test]
fn menu_disabled_items_receive_focus_without_activation() {
    let (screen, chosen) = mount_with(RenderOptions::from_env(), || {
        let chosen = signal(0);
        let menu = use_menu(vec![
            MenuEntry::item("disabled", "Disabled").disabled(true),
            MenuEntry::item("enabled", "Enabled"),
        ])
        .on_activate(move |_| chosen.update(|count| *count += 1));
        (chosen, menu_button("Actions", menu))
    });
    screen
        .get_by_role(Role::Button, "Actions")
        .press_key(Key::Enter);
    let disabled = screen.get_by_role(Role::MenuItem, "Disabled");
    assert!(disabled.is_focused());
    screen.press_key(Key::Enter);
    screen.press_key(Key::Space);
    disabled.a11y(A11yAction::Activate);
    assert_eq!(chosen.get_untracked(), 0);
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::MenuItem, "Enabled").is_focused());
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn menu_submenu_parent_publishes_menu_popup() {
    let (screen, ()) = mount_with(RenderOptions::from_env(), || {
        let menu = use_menu(vec![MenuEntry::submenu(
            "more",
            "More",
            vec![MenuEntry::item("child", "Child")],
        )]);
        ((), menu_button("Actions", menu))
    });
    screen
        .get_by_role(Role::Button, "Actions")
        .press_key(Key::Enter);
    let more = screen.get_by_role(Role::MenuItem, "More");
    assert_eq!(more.popup_kind(), Some(PopupKind::Menu));
    assert_eq!(more.is_expanded(), Some(false));
    screen.press_key(Key::ArrowRight);
    assert_eq!(more.is_expanded(), Some(true));
    assert!(screen.get_by_role(Role::MenuItem, "Child").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
