//! Role: Verify the Soft selector's opt-in arrow opening through public APIs.
use kivact::interaction::ListItem;
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::test::RenderOptions;
use kivact::test::render_with_handle;
use kivact::ui::select;
use kivact::view::Key;
use kivact::view::Role;
use kivact_ui_soft::{selector, theme};

fn options() -> RenderOptions {
    RenderOptions::from_env().theme(theme(ThemeMode::Light))
}

fn choices() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("first", "First"),
        ListItem::new("middle", "Middle"),
        ListItem::new("last", "Last"),
    ]
}

#[test]
fn soft_selector_arrows_open_at_the_edges_without_committing() {
    let (screen, value) = render_with_handle(options(), || {
        let value = signal(Some("middle"));
        (value, selector("Choice", choices(), value))
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Choice");
    screen.press_key(Key::Tab);
    assert!(trigger.is_focused());

    screen.press_key(Key::ArrowDown);
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(trigger.is_focused());
    assert!(trigger.has_active_descendant(&screen.get_by_role(Role::Option, "First")));
    assert_eq!(value.get_untracked(), Some("middle"));

    screen.press_key(Key::Escape);
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.is_focused());
    assert_eq!(value.get_untracked(), Some("middle"));

    screen.press_key(Key::ArrowUp);
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(trigger.is_focused());
    assert!(trigger.has_active_descendant(&screen.get_by_role(Role::Option, "Last")));
    assert_eq!(value.get_untracked(), Some("middle"));

    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), Some("last"));
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn standard_select_keeps_its_closed_arrow_selection() {
    let (screen, value) = render_with_handle(options(), || {
        let value = signal(Some("middle"));
        (value, select("Standard choice", choices(), value))
    });
    screen.press_key(Key::Tab);
    screen.press_key(Key::ArrowDown);
    assert_eq!(value.get_untracked(), Some("last"));
    assert_eq!(
        screen
            .get_by_role(Role::ComboBox, "Standard choice")
            .is_expanded(),
        Some(false),
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
