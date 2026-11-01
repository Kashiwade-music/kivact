//! Role: Tests for the interactions of Creative controls through the public API.

use super::*;
use kivact::view::Handled;

#[test]
fn tooltip_accepts_focus_without_a_frame_loop_and_keeps_custom_input() {
    let (screen, custom) = kivact::test::render_with_handle(crate::options(), || {
        let custom = signal(0usize);
        (
            custom,
            column((
                kivact::ui::tooltip("Keyboard help")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("Help")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .on_pointer(move |_| {
                                custom.update(|count| *count += 1);
                                Handled::No
                            }),
                    ),
                kivact::ui::button("Elsewhere").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    });
    let loops = screen.stats().frame_loops;
    assert_eq!(
        screen.get_by_role(Role::Button, "Help").description(),
        "Keyboard help"
    );
    screen.get_by_role(Role::Button, "Help").focus();
    screen.advance(Duration::from_millis(499));
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Keyboard help")
            .is_none()
    );
    screen.advance(Duration::from_millis(1));
    assert!(
        screen.get_by_role(Role::Tooltip, "Keyboard help").exists(),
        "the overlay is named by the title"
    );
    assert_eq!(screen.stats().frame_loops, loops);
    screen.get_by_role(Role::Button, "Elsewhere").focus();
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Keyboard help")
            .is_none()
    );
    screen.get_by_role(Role::Button, "Help").hover();
    assert!(custom.get_untracked() > 0);
    screen.advance(Duration::from_millis(500));
    assert!(screen.get_by_role(Role::Tooltip, "Keyboard help").exists());
    clean(&screen);
}

#[test]
fn creative_segments_and_numeric_semantics_survive_a_theme_change() {
    let (screen, (theme, selected)) = kivact::test::render_with_handle(crate::options(), || {
        let theme = signal(ui::theme(kivact::style::ThemeMode::Dark));
        let selected = signal(Some("first"));
        (
            (theme, selected),
            kivact::view::provide_theme(theme, move || {
                column((
                    kivact::ui::radio_group(
                        "Order",
                        choice_options(&[("first", "First"), ("second", "Second")]),
                        selected,
                    )
                    .orientation(kivact::view::Orientation::Horizontal)
                    .variant(ui::Segmented),
                    ui::value_slider("Opacity", signal(0.5)).range(0.0, 1.0),
                ))
            }),
        )
    });
    screen
        .get_by_role(Role::RadioButton, "First")
        .press_key(Key::ArrowRight);
    let before = screen.snapshot();
    theme.set(
        ui::theme(kivact::style::ThemeMode::Dark)
            .accent(ui::Accent::Aqua)
            .density(ui::Density::Compact),
    );
    screen.run_until_idle();
    assert_eq!(screen.snapshot(), before);
    screen
        .get_by_role(Role::RadioButton, "Second")
        .press_key(Key::ArrowRight);
    assert_eq!(
        selected.get_untracked(),
        Some("first"),
        "segments are a standard radio group, which wraps"
    );
    clean(&screen);
}

#[test]
fn reordered_document_tabs_keep_identity_and_navigate_in_visible_order() {
    let (screen, (selected, reorders)) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal("first");
        let reorders = signal(Vec::<(&'static str, &'static str)>::new());
        let tabs = signal(choice_options(&[
            ("first", "First"),
            ("second", "Second"),
            ("third", "Third"),
        ]));
        (
            (selected, reorders),
            ui::document_tabs("Documents", tabs, selected)
                .on_reorder(move |(moved, neighbor): (&'static str, &'static str)| {
                    reorders.update(|items| items.push((moved, neighbor)));
                    tabs.update(|items| {
                        let a = items.iter().position(|item| *item.key() == moved);
                        let b = items.iter().position(|item| *item.key() == neighbor);
                        if let (Some(a), Some(b)) = (a, b) {
                            items.swap(a, b);
                        }
                    });
                })
                .on_close(|_| {}),
        )
    });
    let first = screen.get_by_role(Role::Tab, "First");
    first.focus();
    let rect = first.rect().expect("first tab");
    first.drag(
        rect.center(),
        Point::new(rect.center().x + 65.0, rect.center().y),
    );
    assert_eq!(reorders.get_untracked(), vec![("first", "second")]);
    assert_eq!(selected.get_untracked(), "first");
    screen
        .get_by_role(Role::Tab, "First")
        .press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), "third");
    assert_eq!(
        screen.focused().map(|item| item.name()),
        Some("Third".into())
    );
    screen.press_key(Key::Home);
    assert_eq!(selected.get_untracked(), "second");
    assert_eq!(
        screen.focused().map(|item| item.name()),
        Some("Second".into())
    );
    clean(&screen);
}

#[test]
fn dropdown_select_and_invoke_offer_one_write_then_restore_the_trigger() {
    let (screen, (selected, writes)) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(Some("first"));
        let writes = signal(0usize);
        let binding = binding(
            (move || selected.get()).into_value(),
            move |value: Option<&'static str>| {
                writes.update(|count| *count += 1);
                selected.set(value);
            },
        );
        (
            (selected, writes),
            kivact::ui::select(
                "Selection",
                choice_options(&[("first", "First"), ("second", "Second")]),
                binding,
            )
            .placeholder("")
            .test_id("dropdown"),
        )
    });
    let trigger = screen.get_by_test_id("dropdown");
    for (action, label, expected, count) in [
        (A11yAction::Select, "Second", Some("second"), 1),
        (A11yAction::Activate, "First", Some("first"), 2),
    ] {
        trigger.a11y(A11yAction::Expand);
        screen.get_by_role(Role::Option, label).a11y(action);
        assert_eq!(selected.get_untracked(), expected);
        assert_eq!(writes.get_untracked(), count);
        assert!(trigger.is_focused());
        assert!(screen.query_by_role(Role::Option, label).is_none());
    }
    clean(&screen);
}

#[test]
fn modal_window_restores_the_focus_captured_before_accessibility_open() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column((
                kivact::ui::button("Previous focus").variant(kivact::ui::ButtonVariant::Secondary),
                ui::window_button(
                    "Open modal",
                    ui::WindowOptions::default()
                        .title("Confirmation")
                        .modal(true),
                    |close| {
                        kivact::ui::button("Accept")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .on_press(move || close.close())
                            .into_view()
                    },
                ),
            )),
        )
    });
    let previous = screen.get_by_role(Role::Button, "Previous focus");
    let opener = screen.get_by_role(Role::Button, "Open modal");
    previous.focus();
    opener.a11y(A11yAction::Activate);
    screen.get_by_role(Role::Button, "Accept").click();
    assert!(previous.is_focused());
    opener.a11y(A11yAction::Activate);
    screen.press_key(Key::Escape);
    assert!(previous.is_focused());
    clean(&screen);
}

#[test]
fn floating_window_keeps_the_primary_drag_when_a_secondary_button_releases() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            ui::window_button("Tool", ui::WindowOptions::default(), |_| {
                text("Contents").into_view()
            }),
        )
    });
    screen.get_by_role(Role::Button, "Tool").click();
    let title = screen.get_by_role(Role::Button, "Tool title bar");
    let original = title.rect().expect("title bar");
    let start = original.center();
    screen.mouse_move(start.x, start.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(start.x + 20.0, start.y);
    screen.mouse_down(MouseButton::Secondary);
    screen.mouse_up(MouseButton::Secondary);
    screen.mouse_move(start.x + 40.0, start.y);
    screen.mouse_up(MouseButton::Primary);
    let moved = title.rect().expect("moved title bar");
    assert!((moved.x - original.x - 40.0).abs() < 0.5);
    clean(&screen);
}

#[test]
fn menu_accessibility_select_runs_one_business_action_and_preserves_retry() {
    let (screen, attempts) = kivact::test::render_with_handle(crate::options(), || {
        let attempts = signal(Vec::<&'static str>::new());
        (
            attempts,
            kivact::ui::menu(
                "Actions",
                vec![MenuEntry::item("save", "Save")],
                move |key: &'static str| {
                    attempts.update(|items| items.push(key));
                    if attempts.with_untracked(Vec::len) == 1 {
                        Err(kivact::UiError::msg("save failed"))
                    } else {
                        Ok(())
                    }
                },
            )
            .offset(0.0)
            .attach(
                kivact::ui::button("Actions")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .trailing_icon(kivact::control::icon("chevron-down"))
                    .into_node(),
            ),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Actions");
    trigger.click();
    screen
        .get_by_role(Role::MenuItem, "Save")
        .a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), vec!["save"]);
    assert_eq!(screen.errors().len(), 1);
    assert!(screen.get_by_role(Role::MenuItem, "Save").exists());
    screen
        .get_by_role(Role::MenuItem, "Save")
        .a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), vec!["save", "save"]);
    assert_eq!(screen.errors().len(), 1);
    assert!(screen.query_by_role(Role::MenuItem, "Save").is_none());
    assert!(trigger.is_focused());
}

#[test]
fn popup_and_dialog_buttons_open_named_dialogs_that_assistive_technology_can_dismiss() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column((
                ui::popup_button("Viewport Options", || text("Overlays")),
                {
                    let open = signal(false);
                    (
                        kivact::ui::button("Export...")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .expanded(open)
                            .on_press(move || open.set(true)),
                        kivact::ui::dialog("Export...", open)
                            .closable(false)
                            .content(|| text("Format"))
                            .footer(move || {
                                kivact::ui::button(kivact::i18n::message(
                                    "creative.dialog.close",
                                    "Close",
                                ))
                                .variant(kivact::ui::ButtonVariant::Secondary)
                                .on_press(move || open.set(false))
                            }),
                    )
                },
                kivact::ui::button("Behind").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    });
    for (label, modal) in [("Viewport Options", false), ("Export...", true)] {
        let opener = screen.get_by_role(Role::Button, label);
        opener.focus();
        opener.click();
        assert_eq!(opener.is_expanded(), Some(true), "{label}");
        let dialog = screen.get_by_role(Role::Dialog, label);
        screen.get_by_role(Role::Heading, label);
        dialog.a11y(A11yAction::Dismiss);
        assert!(
            screen.query_by_role(Role::Dialog, label).is_none(),
            "{label}: Dismiss closes it"
        );
        assert_eq!(opener.is_expanded(), Some(false), "{label}");
        assert!(opener.is_focused(), "{label}: focus returns to the opener");

        opener.click();
        screen.get_by_role(Role::Button, "Close").click();
        assert!(screen.query_by_role(Role::Dialog, label).is_none());
        assert!(opener.is_focused(), "{label}: Close returns the focus");

        // A popup toggles on its button; a modal keeps the page behind it out.
        opener.click();
        if modal {
            screen.get_by_role(Role::Button, "Close").click();
        } else {
            opener.click();
            assert!(screen.query_by_role(Role::Dialog, label).is_none());
        }
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
