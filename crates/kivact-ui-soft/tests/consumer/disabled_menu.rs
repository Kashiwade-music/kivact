//! Role: Verify Soft's opt-in keyboard focus for disabled menu items.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use kivact::interaction::MenuEntry;
use kivact::style::ThemeMode;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::ui::menu;
use kivact::view::A11yAction;
use kivact::view::Key;
use kivact::view::Role;

#[test]
fn soft_disabled_menu_item_can_explain_why_but_cannot_activate() {
    let chosen = Rc::new(Cell::new(0));
    let report = chosen.clone();
    let screen = render_with(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)),
        move || {
            menu(
                "Actions",
                vec![
                    MenuEntry::item("open", "Open"),
                    MenuEntry::item("remove", "Remove")
                        .disabled(true)
                        .with_disabled_reason("Only the owner can remove this"),
                    MenuEntry::item("copy", "Copy"),
                ],
            )
            .on_activate(move |_| report.set(report.get() + 1))
        },
    );

    screen
        .get_by_role(Role::Button, "Actions")
        .press_key(Key::ArrowDown);
    assert_eq!(screen.focused().unwrap().name(), "Open");
    screen.press_key(Key::ArrowDown);
    let remove = screen.get_by_role(Role::MenuItem, "Remove");
    assert!(remove.is_focused());
    assert!(remove.is_disabled());
    assert_eq!(remove.description(), "Only the owner can remove this");
    let before_tooltip = screen.screenshot().hash();
    screen.wait_for(Duration::from_secs(2), |screen| {
        screen
            .query_by_text("Only the owner can remove this")
            .is_some()
    });
    assert_ne!(screen.screenshot().hash(), before_tooltip);

    screen.press_key(Key::Enter);
    screen.press_key(Key::Space);
    remove.a11y(A11yAction::Activate);
    let bounds = remove.rect().unwrap();
    screen.click_at(
        bounds.x + bounds.width * 0.5,
        bounds.y + bounds.height * 0.5,
    );
    assert_eq!(chosen.get(), 0);
    screen.mouse_leave();
    screen.press_key(Key::ArrowDown);
    assert_eq!(screen.focused().unwrap().name(), "Copy");
    assert!(
        screen
            .query_by_text("Only the owner can remove this")
            .is_none()
    );
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn explicit_skip_policy_skips_disabled_menu_entries() {
    let policy = kivact::interaction::InteractionPolicy {
        menu: kivact::interaction::MenuPolicy {
            disabled_focus: kivact::interaction::DisabledMenuFocus::Skip,
            ..Default::default()
        },
        ..Default::default()
    };
    let screen = render_with(
        RenderOptions::with_theme(Theme::light().with(policy)),
        || {
            menu(
                "Actions",
                vec![
                    MenuEntry::item("remove", "Remove")
                        .disabled(true)
                        .with_disabled_reason("Owner only"),
                    MenuEntry::item("copy", "Copy"),
                ],
            )
            .on_activate(|_| {})
        },
    );
    screen
        .get_by_role(Role::Button, "Actions")
        .press_key(Key::ArrowDown);
    assert_eq!(screen.focused().unwrap().name(), "Copy");
    assert!(screen.query_by_text("Owner only").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
