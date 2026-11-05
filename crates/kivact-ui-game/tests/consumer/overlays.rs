//! Role: Modal menus, command search, toasts, tooltips and dialogs via the public API.

use kivact::interaction::{Command, MenuEntry, Toast, Tone, toaster};
use kivact::prelude::*;
use kivact::view::A11yAction;
use kivact::view::{rich_text, span};
use kivact_ui_game as g;
use std::time::Duration;

#[test]
fn consumer_modal_menu_command_search_and_focus_restore() {
    let (screen, (open, command_open, hits)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_game::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let open = signal(false);
            let command_open = signal(false);
            let hits = signal(0);
            (
                (open, command_open, hits),
                column((
                    kivact::ui::button("Create")
                        .variant(kivact_ui_game::Slant)
                        .on_press(move || open.set(true)),
                    kivact::ui::menu("Actions", vec![MenuEntry::item("run", "Run")])
                        .on_activate(move |_: &'static str| hits.update(|v| *v += 1))
                        .offset(0.)
                        .attach(
                            kivact::ui::button("Actions")
                                .variant(kivact_ui_game::Slant)
                                .variant(kivact::ui::ButtonVariant::Secondary),
                        ),
                    kivact::ui::button("Commands")
                        .variant(kivact_ui_game::Slant)
                        .on_press(move || command_open.set(true)),
                    kivact::ui::dialog("New operation", open).content(|| {
                        kivact::ui::text_input("Title", kivact::state::signal(String::new()))
                    }),
                    kivact::ui::command_palette(
                        vec![
                            Command::new("deploy", "Deploy").with_hint("RUN"),
                            Command::new("help", "Help").with_hint("DOCS"),
                        ],
                        command_open,
                    )
                    .on_activate(move |key: &'static str| {
                        if key == "deploy" {
                            hits.update(|v| *v += 1);
                        }
                    }),
                ))
                .style(style().padding(20).gap(20)),
            )
        },
    );
    let create = screen.get_by_role(Role::Button, "Create");
    create.focus();
    create.click();
    assert!(open.get_untracked());
    screen.get_by_role(Role::Dialog, "New operation");
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(create.is_focused());
    screen.get_by_role(Role::Button, "Actions").click();
    screen
        .get_by_role(Role::MenuItem, "Run")
        .press_key(Key::Enter);
    assert_eq!(hits.get_untracked(), 1);
    screen.get_by_role(Role::Button, "Commands").click();
    let query = screen.get_by_role(Role::TextBox, "Search commands");
    query.type_text("Deploy");
    query.press_key(Key::Enter);
    assert_eq!(hits.get_untracked(), 2);
    assert!(!command_open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_toast_expiry_and_hover_pause() {
    let (screen, toaster) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_game::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let toaster = toaster().max_visible(4);
            (
                toaster,
                (
                    column((
                        kivact::ui::button("Notify")
                            .variant(kivact_ui_game::Slant)
                            .on_press(move || {
                                toaster.push(
                                    Toast::new("Saved")
                                        .body("Ready")
                                        .tone(Tone::Success)
                                        .duration(Some(Duration::from_millis(400))),
                                );
                            }),
                        kivact::ui::alert("NOTICE")
                            .variant(kivact::ui::AlertVariant::Banner)
                            .tone(kivact::interaction::Tone::Warning)
                            .icon(kivact::view::icon("alert-triangle"))
                            .actions(())
                            .icon(
                                g::mono("NOTICE").style(
                                    style()
                                        .font_size(10)
                                        .weight(800)
                                        .padding_x(6)
                                        .padding_y(4)
                                        .background(g::ColorName::Yellow),
                                ),
                            )
                            .with_node(|node| {
                                node.role(Role::Status)
                                    .live_region(kivact::view::LiveRegion::Polite)
                            })
                            .content(move || {
                                rich_text(vec![
                                    span("Maintenance:").weight(700).color(rgb(0xffc928)),
                                    span(" read only"),
                                ])
                            })
                            .name("Service notice"),
                        g::activity(vec![g::ActivityEntry::new(
                            "09:40",
                            rich_text(vec![span("Review").weight(700), span(" passed")]),
                            kivact::interaction::Tone::Success,
                        )]),
                    )),
                    kivact::ui::toast_region(toaster),
                ),
            )
        },
    );
    assert!(screen.get_by_text("Maintenance: read only").exists());
    assert!(screen.get_by_text("Review passed").exists());
    screen.get_by_role(Role::Button, "Notify").click();
    assert!(screen.get_by_role(Role::Status, "Saved").exists());
    screen.get_by_text("Ready").hover();
    screen.advance(Duration::from_secs(1));
    assert_eq!(toaster.toasts().get_untracked().len(), 1);
    screen.mouse_leave();
    screen.advance(Duration::from_millis(399));
    assert_eq!(toaster.toasts().get_untracked().len(), 1);
    screen.advance(Duration::from_millis(1));
    assert!(toaster.toasts().get_untracked().is_empty());
    for _ in 0..5 {
        screen.get_by_role(Role::Button, "Notify").click();
    }
    assert_eq!(toaster.toasts().get_untracked().len(), 4, "four are shown");
    screen.advance(Duration::from_millis(400));
    assert_eq!(
        toaster.toasts().get_untracked().len(),
        1,
        "the fifth takes its place"
    );
    screen.advance(Duration::from_millis(400));
    assert!(toaster.toasts().get_untracked().is_empty());
    toaster.push(Toast::new("Failed").body("Rejected").tone(Tone::Danger));
    screen.run_until_idle();
    screen.get_by_role(Role::Alert, "Failed");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_tooltip_focus_delay_and_blur_use_setup_bound_observation() {
    let (screen, ()) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_game::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            (
                (),
                row((
                    kivact::ui::tooltip("Save with Ctrl+S")
                        .placement(kivact::view::Placement::Below)
                        .align(kivact::view::OverlayAlign::Center)
                        .offset(6.)
                        .attach(kivact::ui::button("Save").variant(kivact_ui_game::Slant)),
                    kivact::ui::button("Elsewhere").variant(kivact_ui_game::Slant),
                ))
                .style(style().padding(30).gap(24)),
            )
        },
    );
    assert_eq!(
        screen.get_by_role(Role::Button, "Save").description(),
        "Save with Ctrl+S"
    );
    screen.get_by_role(Role::Button, "Save").focus();
    screen.advance(Duration::from_millis(499));
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Save with Ctrl+S")
            .is_none()
    );
    screen.advance(Duration::from_millis(1));
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Save with Ctrl+S")
            .is_some()
    );
    screen.get_by_role(Role::Button, "Elsewhere").focus();
    assert!(
        screen
            .query_by_role(Role::Tooltip, "Save with Ctrl+S")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_dialog_drawer_and_popover_are_named_dialogs_that_assistive_technology_can_dismiss() {
    let (screen, opens) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_game::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let (modal, sheet, popover) = (signal(false), signal(false), signal(false));
            (
                (modal, sheet, popover),
                column((
                    kivact::ui::button("Open modal")
                        .variant(kivact_ui_game::Slant)
                        .on_press(move || modal.set(true)),
                    kivact::ui::button("Open sheet")
                        .variant(kivact_ui_game::Slant)
                        .on_press(move || sheet.set(true)),
                    kivact::ui::dialog("Rename", modal)
                        .description("The new name shows everywhere.")
                        .content(|| {
                            kivact::ui::text_input("Name", kivact::state::signal(String::new()))
                        }),
                    kivact::ui::dialog("Details", sheet)
                        .mode(kivact::interaction::DialogMode::Drawer(
                            kivact::geometry::Edge::Right,
                        ))
                        .content(|| text("Body")),
                    kivact::ui::dialog("Help topic", popover)
                        .mode(kivact::interaction::DialogMode::Popover)
                        .content(|| text("Help content"))
                        .attach(kivact::ui::button("Open help").variant(kivact_ui_game::Slant)),
                )),
            )
        },
    );
    let (modal, sheet, popover) = opens;
    for (opener, title, open) in [
        ("Open modal", "Rename", modal),
        ("Open sheet", "Details", sheet),
        ("Open help", "Help topic", popover),
    ] {
        let opener = screen.get_by_role(Role::Button, opener);
        opener.focus();
        opener.click();
        let dialog = screen.get_by_role(Role::Dialog, title);
        screen.get_by_role(Role::Heading, title);
        dialog.a11y(A11yAction::Dismiss);
        assert!(!open.get_untracked(), "{title}: DismissOn closes it");
        assert!(opener.is_focused(), "{title}: focus returns to the opener");
    }
    screen.get_by_role(Role::Button, "Open modal").click();
    let described: Vec<String> = screen
        .get_by_role(Role::Dialog, "Rename")
        .described_by()
        .iter()
        .map(|e| e.text())
        .collect();
    assert_eq!(described, ["The new name shows everywhere."]);
    screen.get_by_role(Role::Button, "Close").click();
    assert!(!modal.get_untracked(), "the close button closes the dialog");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
