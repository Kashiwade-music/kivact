//! Role: Verify the Soft Selector keeps the trigger focused while navigating.
use kivact::interaction::ListItem;
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::test::MirrorMode;
use kivact::test::RenderOptions;
use kivact::test::render_with_handle;
use kivact::ui::button;
use kivact::view::Key;
use kivact::view::Modifiers;
use kivact::view::Role;
use kivact::view::column;

#[test]
fn selector_highlights_without_moving_focus_or_committing_until_activation() {
    let (screen, selected) = render_with_handle(
        RenderOptions::default()
            .theme(kivact_ui_soft::theme(ThemeMode::Light))
            .mirror(MirrorMode::Off),
        || {
            let selected = signal(None);
            (
                selected,
                column((
                    button("Before"),
                    kivact::ui::select(
                        "Fruit",
                        vec![
                            ListItem::new(0, "Unavailable").disabled(true),
                            ListItem::new(1, "Apple"),
                            ListItem::new(2, "Banana"),
                        ],
                        selected,
                    )
                    .open_on_arrow(true),
                    button("After"),
                )),
            )
        },
    );
    let trigger = screen.get_by_role(Role::ComboBox, "Fruit");
    trigger.focus();
    screen.press_key(Key::ArrowDown);
    let apple = screen.get_by_role(Role::Option, "Apple");
    let banana = screen.get_by_role(Role::Option, "Banana");
    let unavailable = screen.get_by_role(Role::Option, "Unavailable");
    assert!(trigger.is_focused());
    assert!(trigger.has_active_descendant(&apple));
    assert!(unavailable.is_disabled());
    assert_eq!(selected.get_untracked(), None);

    screen.press_key(Key::ArrowDown);
    assert!(trigger.is_focused());
    assert!(trigger.has_active_descendant(&banana));
    assert_eq!(selected.get_untracked(), None);
    screen.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), Some(2));
    assert!(trigger.is_focused());
    assert_eq!(trigger.is_expanded(), Some(false));

    screen.press_key(Key::Space);
    screen.press_key(Key::Tab);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert_eq!(trigger.is_expanded(), Some(false));

    trigger.focus();
    screen.press_key(Key::Space);
    screen.press_key_with(
        Key::Tab,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    assert!(screen.get_by_role(Role::Button, "Before").is_focused());
    assert_eq!(trigger.is_expanded(), Some(false));

    trigger.click();
    let disabled = screen.get_by_role(Role::Option, "Unavailable");
    let rect = disabled.rect().unwrap();
    screen.click_at(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(selected.get_untracked(), Some(2));
    assert_eq!(trigger.is_expanded(), Some(true));
    let apple = screen.get_by_role(Role::Option, "Apple");
    apple.hover();
    assert!(trigger.has_active_descendant(&apple));
    assert!(trigger.is_focused());
    apple.click();
    assert_eq!(selected.get_untracked(), Some(1));
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn selector_does_not_toggle_closed_on_open_enter_when_no_option_is_enabled() {
    let (screen, selected) = render_with_handle(
        RenderOptions::default()
            .theme(kivact_ui_soft::theme(ThemeMode::Light))
            .mirror(MirrorMode::Off),
        || {
            let selected = signal(None);
            (
                selected,
                kivact::ui::select(
                    "Disabled options",
                    vec![ListItem::new(1, "Unavailable").disabled(true)],
                    selected,
                )
                .open_on_arrow(true),
            )
        },
    );
    let trigger = screen.get_by_role(Role::ComboBox, "Disabled options");
    trigger.focus();
    screen.press_key(Key::Enter);
    assert_eq!(trigger.is_expanded(), Some(true));
    screen.press_key(Key::Enter);
    assert_eq!(trigger.is_expanded(), Some(true));
    assert_eq!(selected.get_untracked(), None);
    assert!(trigger.is_focused());
    screen.press_key(Key::Escape);
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.is_focused());
}
