//! Role: Tests for component interactions.

use super::*;

#[test]
fn theme_change_preserves_the_shared_tab_and_range_semantic_tree() {
    let (screen, (theme, selected, amount)) =
        kivact::test::render_with_handle(crate::options(), || {
            let theme = signal(ui::theme(kivact::style::ThemeMode::Light));
            let selected = signal("first");
            let amount = signal(25.0);
            (
                (theme, selected, amount),
                kivact::view::provide_theme(theme, move || {
                    column((
                        kivact::ui::tabs(
                            "Tabs",
                            vec![
                                ListItem::new("first", "First"),
                                ListItem::new("second", "Second"),
                            ],
                            selected,
                        ),
                        kivact::ui::slider("Amount", amount),
                    ))
                }),
            )
        });
    screen
        .get_by_role(Role::Tab, "First")
        .press_key(Key::ArrowRight);
    let before = screen.snapshot();
    assert!(
        ui::theme(kivact::style::ThemeMode::Light) == theme.get_untracked(),
        "the kit's themes share one recipe table and icon set, so an equal theme compares equal"
    );
    theme.set(ui::theme(kivact::style::ThemeMode::Dark));
    screen.run_until_idle();
    assert_eq!(screen.snapshot(), before);
    assert_eq!(selected.get_untracked(), "second");
    assert_eq!(amount.get_untracked(), 25.0);
    screen
        .get_by_role(Role::Tab, "Second")
        .press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), "first", "Pop explicitly wraps");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn menu_wrapper_preserves_activation_and_reports_a_fallible_action_once() {
    let (screen, (presses, attempts)) = kivact::test::render_with_handle(crate::options(), || {
        let presses = signal(0usize);
        let attempts = signal(0usize);
        (
            (presses, attempts),
            kivact::ui::menu(
                "",
                vec![MenuEntry::item("save", "Save")],
                move |_: &'static str| {
                    attempts.update(|count| *count += 1);
                    if attempts.get_untracked() == 1 {
                        Err(kivact::UiError::msg("save failed"))
                    } else {
                        Ok(())
                    }
                },
            )
            .attach(
                kivact::ui::button("Actions")
                    .on_press(move || presses.update(|count| *count += 1))
                    .into_node(),
            ),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Actions");
    trigger.click();
    assert_eq!(presses.get_untracked(), 1);
    screen
        .get_by_role(Role::MenuItem, "Save")
        .a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), 1);
    assert_eq!(screen.errors().len(), 1);
    assert!(screen.get_by_role(Role::MenuItem, "Save").exists());
    screen
        .get_by_role(Role::MenuItem, "Save")
        .a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), 2);
    assert!(screen.query_by_role(Role::MenuItem, "Save").is_none());
    assert!(trigger.is_focused());
    trigger.press_key(Key::Space);
    assert_eq!(presses.get_untracked(), 2);
    screen.press_key(Key::Escape);
    trigger.a11y(A11yAction::Activate);
    assert_eq!(presses.get_untracked(), 3);
    assert_eq!(screen.errors().len(), 1);
}

#[test]
fn palette_navigation_keeps_input_focus_and_reports_failed_activation_once() {
    let (screen, (open, chosen, attempts)) =
        kivact::test::render_with_handle(crate::options(), || {
            let open = signal(false);
            let chosen = signal(false);
            let attempts = signal(Vec::<&'static str>::new());
            (
                (open, chosen, attempts),
                column((
                    kivact::ui::button("Open palette").on_press(move || open.set(true)),
                    kivact::ui::command_palette(
                        open,
                        vec![Command::new("alpha", "Alpha"), Command::new("fail", "Fail")],
                        move |key: &'static str| match key {
                            "alpha" => {
                                attempts.update(|items| items.push("alpha"));
                                chosen.set(true);
                                Ok(())
                            }
                            _ => {
                                attempts.update(|items| items.push("fail"));
                                Err(kivact::UiError::msg("command failed"))
                            }
                        },
                    ),
                )),
            )
        });
    let opener = screen.get_by_role(Role::Button, "Open palette");
    opener.click();
    let input = screen.get_by_role(Role::TextBox, "Search commands");
    input.press_key(Key::ArrowDown);
    assert!(input.is_focused());
    assert!(screen.get_by_role(Role::Option, "Fail").is_selected());
    screen
        .get_by_role(Role::Option, "Fail")
        .a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), vec!["fail"]);
    assert_eq!(screen.errors().len(), 1);
    assert!(open.get_untracked());
    assert!(input.is_focused());
    input.type_text("Alpha");
    screen
        .get_by_role(Role::Option, "Alpha")
        .a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), vec!["fail", "alpha"]);
    assert!(chosen.get_untracked());
    assert!(!open.get_untracked());
    assert!(opener.is_focused());
    assert_eq!(screen.errors().len(), 1);
}

#[test]
fn tooltip_shows_on_focus_below_the_trigger_and_wraps_a_long_label() {
    let long = "Exports every page of the document with its comments and revision history \
                as a single archive that other people can open without this application.";
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), move || {
        (
            (),
            column((
                kivact::ui::tooltip(long)
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("Export")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::control::icon("download")),
                    ),
                kivact::ui::tooltip("Add")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("Add")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::control::icon("plus")),
                    ),
                kivact::ui::button("Elsewhere"),
            ))
            .style(style().padding(30).gap(24)),
        )
    });
    let export = screen.get_by_role(Role::Button, "Export");
    assert_eq!(export.description(), long);
    export.focus();
    screen.advance(Duration::from_millis(499));
    assert!(screen.query_by_role(Role::Tooltip, long).is_none());
    screen.advance(Duration::from_millis(1));
    let tip = screen.get_by_role(Role::Tooltip, long).rect().unwrap();
    let trigger = export.rect().unwrap();
    assert!(tip.y >= trigger.y + trigger.height, "below the trigger");
    assert!(tip.width <= 320.5, "{tip:?}");
    assert!(tip.height > 30.0, "wrapped: {tip:?}");
    screen.get_by_role(Role::Button, "Elsewhere").focus();
    assert!(screen.query_by_role(Role::Tooltip, long).is_none());
    screen.get_by_role(Role::Button, "Add").focus();
    assert!(
        screen.query_by_role(Role::Tooltip, "Add").is_some(),
        "the next tooltip within the skip delay shows without a delay"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
