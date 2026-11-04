//! Role: Verify the APG menu button popup relationship.
use super::interaction_menu::menu_button;
use super::support::mount_with;
use kivact::interaction::{MenuEntry, use_menu};
use kivact::prelude::*;
use kivact::test::RenderOptions;
use kivact::view::PopupKind;

#[test]
fn menu_button_publishes_menu_popup() {
    let (screen, ()) = mount_with(RenderOptions::from_env(), || {
        let menu = use_menu(vec![MenuEntry::item("open", "Open")]);
        ((), menu_button("Actions", menu))
    });
    let trigger = screen.get_by_role(Role::Button, "Actions");
    assert_eq!(trigger.popup_kind(), Some(PopupKind::Menu));
    assert_eq!(trigger.is_expanded(), Some(false));
    trigger.press_key(Key::Space);
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(screen.get_by_role(Role::MenuItem, "Open").is_focused());
    screen.press_key(Key::Escape);
    assert!(trigger.is_focused());
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
