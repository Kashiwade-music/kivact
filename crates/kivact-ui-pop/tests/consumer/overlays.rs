//! Role: Modal, notification, command search and menu through the public API.

use kivact::interaction::{Command, MenuEntry, toaster};
use kivact::prelude::*;
use kivact::view::LiveRegion;
use std::time::Duration;

#[test]
fn modal_dismisses_and_restores_focus_to_its_opener() {
    let (screen, open) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let open = signal(false);
            (
                open,
                column((
                    kivact::ui::button("Open").on_press(move || open.set(true)),
                    kivact::ui::dialog("Confirm", open)
                        .content(|| {
                            kivact::ui::text_input("Memo", kivact::state::signal(String::new()))
                        })
                        .footer(move || kivact::ui::button("OK").on_press(move || open.set(false))),
                )),
            )
        },
    );
    let opener = screen.get_by_role(Role::Button, "Open");
    opener.click();
    screen.get_by_role(Role::Dialog, "Confirm");
    screen.get_by_role(Role::TextBox, "Memo").type_text("Ready");
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Memo").value(),
        Some("Ready".to_string())
    );
    screen.get_by_role(Role::Button, "OK").click();
    assert!(!open.get_untracked());
    assert!(opener.is_focused());
    opener.click();
    screen.get_by_role(Role::TextBox, "Memo").focus();
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(screen.query_by_role(Role::Dialog, "Confirm").is_none());
    assert!(opener.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn notification_has_live_semantics_and_expires_on_the_host_clock() {
    let (screen, toaster) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let toaster = toaster();
            toaster.max_visible(4);
            (toaster, kivact::ui::toast_region(toaster))
        },
    );
    toaster.push(
        kivact::interaction::Toast::new("Saved")
            .tone(kivact::interaction::Tone::Success)
            .body("Request updated"),
    );
    toaster.push(
        kivact::interaction::Toast::new("Failed")
            .tone(kivact::interaction::Tone::Danger)
            .body("Request rejected"),
    );
    toaster.push(
        kivact::interaction::Toast::new("Heads up")
            .tone(kivact::interaction::Tone::Warning)
            .body("Quota almost used"),
    );
    screen.run_until_idle();
    assert!(screen.accessibility_tree().contains("Saved"));
    let saved = screen.get_by_role(Role::Status, "Saved");
    let groups = screen.all_by_role(Role::Group, "");
    assert_eq!(groups[0].live_region(), LiveRegion::Polite);
    assert_eq!(groups[1].live_region(), LiveRegion::Assertive);
    assert!(saved.exists());
    screen.get_by_role(Role::Alert, "Failed");
    screen.get_by_role(Role::Alert, "Heads up");
    assert_eq!(screen.all_by_role(Role::Button, "Close").len(), 3);
    screen.advance(Duration::from_millis(4999));
    assert!(screen.query_by_text("Saved").is_some());
    screen.advance(Duration::from_millis(1));
    assert!(screen.query_by_text("Saved").is_none());
    assert!(screen.query_by_text("Failed").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn command_search_rebuilds_rows_and_executes_the_filtered_action() {
    let (screen, (open, chosen)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let open = signal(true);
            let chosen = signal(0usize);
            (
                (open, chosen),
                kivact::ui::command_palette(
                    vec![
                        Command::new(1, "Alpha").with_icon("circle-dot"),
                        Command::new(2, "Beta").with_icon("circle-dot"),
                    ],
                    open,
                )
                .on_activate(move |key: usize| chosen.set(key))
                .footer(text("↑↓ Move   Enter Run   Esc Close")),
            )
        },
    );
    screen.get_by_text("↑↓ Move   Enter Run   Esc Close");
    let bounds = screen
        .get_by_role(Role::Dialog, "Command palette")
        .rect()
        .expect("palette bounds");
    assert_eq!(bounds.width, 560.0, "Pop owns the palette panel width");
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    search.type_text("Beta");
    assert!(screen.query_by_role(Role::Option, "Alpha").is_none());
    screen.get_by_role(Role::Option, "Beta");
    search.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), 2);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn menu_skips_disabled_items_and_restores_the_custom_trigger() {
    let policy = kivact::interaction::InteractionPolicy {
        menu: kivact::interaction::MenuPolicy {
            disabled_focus: kivact::interaction::DisabledMenuFocus::Skip,
            ..Default::default()
        },
        ..Default::default()
    };
    let options = kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
        kivact::style::ThemeMode::Light,
    ))
    .reduced_motion()
    .theme(kivact_ui_pop::theme(kivact::style::ThemeMode::Light).with(policy));
    let (screen, chosen) = kivact::test::render_with_handle(options, || {
        let chosen = signal(0usize);
        (
            chosen,
            kivact::ui::menu(
                "Actions",
                vec![
                    MenuEntry::item(100, "Unavailable").disabled(true),
                    MenuEntry::item(1, "First"),
                    MenuEntry::item(101, "Blocked").disabled(true),
                    MenuEntry::item(2, "Last"),
                ],
            )
            .on_activate(move |key: usize| chosen.set(key))
            .attach(
                kivact::ui::button("Actions")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(kivact::view::icon("dots")),
            ),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Actions");
    trigger.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::MenuItem, "First").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::MenuItem, "Last").is_focused());
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), 2);
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
