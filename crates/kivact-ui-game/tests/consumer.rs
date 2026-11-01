//! Role: Consumer E2E: imports only the public Kivact facade and this library.
use std::time::Duration;

use kivact::control::{rich_text, span};
use kivact::prelude::*;
use kivact::style::pct;
use kivact::test::RenderOptions;
use kivact::ui::{Command, MenuEntry, Toast, Tone, toaster};
use kivact::view::A11yAction;
use kivact_ui_game as g;
use kivact_ui_game::ThemeOptions as _;
#[path = "consumer/interactions.rs"]
mod interactions;

fn options(reduced_motion: bool) -> RenderOptions {
    RenderOptions {
        theme: g::theme(kivact::style::ThemeMode::Light),
        reduced_motion,
        ..RenderOptions::from_env()
    }
}

#[test]
fn consumer_buttons_checks_mixed_states_and_modifiers() {
    let (screen, (count, checked, mixed)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let count = signal(0);
            let checked = signal(false);
            let mixed = signal(true);
            (
                (count, checked, mixed),
                column((
                    kivact::ui::button("Approve")
                        .variant(kivact_ui_game::Slant)
                        .test_id("approve")
                        .style(style().height(46))
                        .on_press(move || count.update(|v| *v += 1)),
                    kivact::ui::button("Disabled")
                        .variant(kivact_ui_game::Slant)
                        .disabled(true),
                    kivact::ui::checkbox("Permissions", checked).indeterminate(mixed),
                    kivact::ui::switch("Network", checked),
                ))
                .style(style().gap(20).padding(20)),
            )
        });
    let button = screen.get_by_test_id("approve");
    assert_eq!(button.accessibility_bounds().map(|r| r.height), Some(46.));
    button.click();
    button.press_key(Key::Space);
    assert_eq!(count.get_untracked(), 2);
    assert!(screen.get_by_role(Role::Button, "Disabled").is_disabled());
    let check = screen.get_by_role(Role::CheckBox, "Permissions");
    assert!(check.is_indeterminate());
    check.a11y(A11yAction::Toggle);
    assert!(checked.get_untracked());
    mixed.set(false);
    screen.run_until_idle();
    assert!(!check.is_indeterminate());
    screen
        .get_by_role(Role::Switch, "Network")
        .press_key(Key::Space);
    assert!(!checked.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_checkbox_switch_and_disclosure_follow_the_shared_toggle_rules() {
    let (screen, (check, switch, open)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let check = signal(false);
            let switch = signal(false);
            let open = signal(false);
            (
                (check, switch, open),
                column((
                    kivact::ui::checkbox("Notify", check),
                    kivact::ui::switch("Network", switch),
                    kivact::ui::disclosure("Settings", open, || text("Inspector content")),
                    kivact::ui::checkbox("Locked", kivact::signal(false)).disabled(true),
                ))
                .style(style().padding(20).gap(16)),
            )
        });
    let check_box = screen.get_by_role(Role::CheckBox, "Notify");
    let switch_node = screen.get_by_role(Role::Switch, "Network");
    check_box.press_key(Key::Enter);
    switch_node.press_key(Key::Enter);
    assert!(!check.get_untracked() && !switch.get_untracked(), "Enter");
    for node in [&check_box, &switch_node] {
        node.press_key(Key::Space);
        assert_eq!(node.value(), None, "role and checked, no On/Off text");
    }
    assert!(check.get_untracked() && switch.get_untracked());
    check_box.a11y(A11yAction::Activate);
    switch_node.a11y(A11yAction::Toggle);
    assert!(!check.get_untracked() && !switch.get_untracked());
    let locked = screen.get_by_role(Role::CheckBox, "Locked");
    locked.force_click();
    locked.a11y(A11yAction::Toggle);
    assert_eq!(locked.is_checked(), Some(false));

    let settings = screen.get_by_role(Role::Button, "Settings");
    assert_eq!(settings.is_expanded(), Some(false));
    settings.press_key(Key::Enter);
    assert!(open.get_untracked());
    settings.press_key(Key::Space);
    assert!(!open.get_untracked());
    settings.a11y(A11yAction::Toggle);
    assert_eq!(settings.is_expanded(), Some(true));
    settings.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_table_preserves_selection_and_updates_cells() {
    let (screen, (rows, selected, query)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let rows = signal(vec![("a", "Zulu".to_owned()), ("b", "Alpha".to_owned())]);
            let selected = signal(Vec::new());
            let query = signal(String::new());
            (
                (rows, selected, query),
                column((
                    kivact::ui::text_input("Filter", query),
                    kivact::ui::data_table(
                        rows,
                        |row: &(&'static str, String)| row.0,
                        vec![
                            kivact::ui::DataColumn::new("NAME", |row: &(&str, String)| {
                                row.1.clone()
                            })
                            .width(300.)
                            .compare(|a: &(&str, String), b: &(&str, String)| a.1.cmp(&b.1))
                            .filter(|row: &(&str, String), query: &str| row.1.contains(query)),
                        ],
                    )
                    .selection(selected)
                    .row_label(|row: &(&'static str, String)| row.0.to_owned())
                    .footer(|_| text("Footer renderer"))
                    .row_actions(100.0, |_| text("Row renderer"))
                    .toolbar(|_| text("Bulk renderer"))
                    .filter(query),
                ))
                .style(style().padding(20).gap(12)),
            )
        });
    screen.get_by_role(Role::CheckBox, "Select a").click();
    assert_eq!(selected.get_untracked(), vec!["a"]);
    screen.get_by_text("Footer renderer");
    screen.get_by_text("Bulk renderer");
    assert!(
        screen
            .get_by_role(Role::CheckBox, "Select all")
            .is_indeterminate()
    );
    screen
        .get_by_role(Role::ColumnHeader, "NAME")
        .a11y(A11yAction::Activate);
    query.set("Alpha".into());
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Row, "a").is_none());
    assert_eq!(selected.get_untracked(), vec!["a"]);
    screen.get_by_role(Role::CheckBox, "Select all").click();
    assert_eq!(selected.get_untracked().len(), 2);
    rows.update(|v| v[1].1 = "Changed".into());
    query.set(String::new());
    screen.run_until_idle();
    screen.get_by_text("Changed");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_modal_menu_command_search_and_focus_restore() {
    let (screen, (open, command_open, hits)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let open = signal(false);
            let command_open = signal(false);
            let hits = signal(0);
            (
                (open, command_open, hits),
                column((
                    kivact::ui::button("Create")
                        .variant(kivact_ui_game::Slant)
                        .on_press(move || open.set(true)),
                    kivact::ui::menu(
                        "Actions",
                        vec![MenuEntry::item("run", "Run")],
                        move |_: &'static str| hits.update(|v| *v += 1),
                    )
                    .offset(0.)
                    .attach(
                        kivact::ui::button("Actions")
                            .variant(kivact_ui_game::Slant)
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .into_node(),
                    ),
                    kivact::ui::button("Commands")
                        .variant(kivact_ui_game::Slant)
                        .on_press(move || command_open.set(true)),
                    kivact::ui::dialog("New operation", open)
                        .content(|| kivact::ui::text_input("Title", kivact::signal(String::new()))),
                    kivact::ui::command_palette(
                        command_open,
                        vec![
                            Command::new("deploy", "Deploy").hint("RUN"),
                            Command::new("help", "Help").hint("DOCS"),
                        ],
                        move |key: &'static str| {
                            if key == "deploy" {
                                hits.update(|v| *v += 1);
                            }
                        },
                    ),
                ))
                .style(style().padding(20).gap(20)),
            )
        });
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
    let (screen, toaster) = kivact::test::render_with_handle(crate::options(true), || {
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
                    g::banner("NOTICE", kivact::ui::Tone::Warning, ())
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
                        kivact::ui::Tone::Success,
                    )]),
                )),
                kivact::ui::toast_region(toaster),
            ),
        )
    });
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
fn consumer_theme_density_and_halo_pixels_follow_public_theme() {
    let (screen, (mode, density)) = kivact::test::render_with_handle(crate::options(true), || {
        let mode = signal(kivact::style::ThemeMode::Light);
        let density = signal(g::Density::Comfortable);
        (
            (mode, density),
            kivact::view::provide_theme(
                move || g::theme(mode.get()).density(density.get()),
                || {
                    column((
                        kivact::ui::button("Action")
                            .variant(kivact_ui_game::Slant)
                            .test_id("themed-action"),
                        g::halo().style(style().size(34, 34)).test_id("small-halo"),
                    ))
                    .style(
                        style()
                            .size(pct(100), pct(100))
                            .padding(24)
                            .gap(24)
                            .background(ColorToken::Surface),
                    )
                },
            ),
        )
    });
    let light = screen.screenshot();
    assert_eq!(light.pixel(790, 590), Some(0xffffff));
    assert_eq!(
        screen
            .get_by_test_id("themed-action")
            .rect()
            .map(|r| r.height),
        Some(34.)
    );
    assert_eq!(
        screen
            .get_by_test_id("small-halo")
            .rect()
            .map(|r| (r.width, r.height)),
        Some((34., 34.))
    );
    mode.set(kivact::style::ThemeMode::Dark);
    density.set(g::Density::Compact);
    screen.run_until_idle();
    let dark = screen.screenshot();
    assert_eq!(dark.pixel(790, 590), Some(0x0f1c2c));
    assert_eq!(
        screen
            .get_by_test_id("themed-action")
            .rect()
            .map(|r| r.height),
        Some(28.)
    );
    assert_ne!(light.hash(), dark.hash());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_tooltip_focus_delay_and_blur_use_setup_bound_observation() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(true), || {
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
    });
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
    let (screen, opens) = kivact::test::render_with_handle(crate::options(true), || {
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
                    .content(|| kivact::ui::text_input("Name", kivact::signal(String::new()))),
                kivact::ui::dialog("Details", sheet)
                    .kind(kivact::ui::DialogKind::Drawer(kivact::ui::Side::Right))
                    .content(|| text("Body")),
                kivact::ui::dialog("Help topic", popover)
                    .kind(kivact::ui::DialogKind::Popover)
                    .content(|| text("Help content"))
                    .attach(kivact::ui::button("Open help").variant(kivact_ui_game::Slant)),
            )),
        )
    });
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
        assert!(!open.get_untracked(), "{title}: Dismiss closes it");
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
