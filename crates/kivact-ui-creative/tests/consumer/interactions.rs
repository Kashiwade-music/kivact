//! Role: Tests for the interactions of Creative controls through the public API.

use crate::support::clean;
use kivact::geometry::Point;
use kivact::interaction::{MenuEntry, toaster};
use kivact::prelude::*;
use kivact::view::A11yAction;
use kivact_ui_creative as ui;

use std::time::Duration;

#[test]
fn creative_segments_and_numeric_semantics_survive_a_theme_change() {
    let (screen, (theme, selected)) =
        kivact::test::render_with_handle(crate::support::options(), || {
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
                        .with_node(move |node| {
                            node.orientation(kivact::view::Orientation::Horizontal)
                        })
                        .variant(ui::Segmented),
                        ui::value_slider("Opacity", signal(0.5)).range(
                            kivact::interaction::ScalarRange::new(0.0, 1.0)
                                .expect("invariant: finite ordered control range"),
                        ),
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
    let (screen, (selected, reorders)) =
        kivact::test::render_with_handle(crate::support::options(), || {
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
fn toasts_are_live_pause_for_the_pointer_and_expire_on_the_host_clock() {
    let (screen, (toaster, (undone, undo))) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let undone = signal(false);
            let undo = kivact::ui::Toast::new("Muted")
                .tone(kivact::ui::Tone::Info)
                .body("Track 2")
                .action("Undo", move || undone.set(true))
                .duration(None);
            (
                (toaster(), (undone, undo)),
                kivact::ui::toast_region(toaster()),
            )
        });
    toaster.push(
        kivact::ui::Toast::new("Exported")
            .tone(kivact::ui::Tone::Success)
            .body("Wrote 3 files"),
    );
    toaster.push(
        kivact::ui::Toast::new("Render failed")
            .tone(kivact::ui::Tone::Danger)
            .body("Out of memory"),
    );
    toaster.push(undo);
    screen.run_until_idle();
    screen.get_by_role(Role::Status, "Exported");
    screen.get_by_role(Role::Alert, "Render failed");
    let groups = screen.all_by_role(Role::Group, "");
    assert_eq!(groups[0].live_region(), kivact::view::LiveRegion::Polite);
    assert_eq!(groups[1].live_region(), kivact::view::LiveRegion::Assertive);
    screen.get_by_text("Wrote 3 files").hover();
    screen.advance(Duration::from_secs(30));
    assert!(
        screen.query_by_text("Exported").is_some(),
        "held by the pointer"
    );
    assert!(screen.query_by_text("Render failed").is_none());
    assert!(
        screen.query_by_text("Muted").is_some(),
        "no duration: stays"
    );
    screen.mouse_leave();
    screen.advance(Duration::from_secs(5));
    assert!(screen.query_by_text("Exported").is_none());
    screen.get_by_role(Role::Button, "Undo").click();
    assert!(undone.get_untracked());
    assert!(screen.query_by_text("Muted").is_none());
    toaster.push(
        kivact::ui::Toast::new("Clipping")
            .tone(kivact::ui::Tone::Warning)
            .body("Master bus"),
    );
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Close").click();
    assert!(screen.query_by_text("Clipping").is_none());
    clean(&screen);
}

#[test]
fn check_switch_toggle_and_disclosure_follow_the_shared_toggle_rules() {
    let (screen, (check, switch, bold, open)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let check = signal(false);
            let switch = signal(false);
            let bold = signal(false);
            let open = signal(false);
            (
                (check, switch, bold, open),
                column((
                    kivact::ui::checkbox("Visible", check),
                    kivact::ui::switch("Enabled", switch),
                    kivact::ui::toggle_button("Bold", bold),
                    kivact::ui::disclosure("Transform", open, || text("Position"))
                        .keep_mounted(true),
                    kivact::ui::checkbox("Locked", signal(false))
                        .with_node(move |node| node.disabled(true)),
                )),
            )
        });
    let check_box = screen.get_by_role(Role::CheckBox, "Visible");
    let switch_node = screen.get_by_role(Role::Switch, "Enabled");
    let button = screen.get_by_role(Role::Button, "Bold");
    check_box.press_key(Key::Enter);
    switch_node.press_key(Key::Enter);
    assert!(!check.get_untracked() && !switch.get_untracked());
    button.press_key(Key::Enter);
    assert!(bold.get_untracked(), "Enter flips a toggle button");
    assert_eq!(button.checked(), Some(CheckedState::Checked));
    for node in [&check_box, &switch_node, &button] {
        node.press_key(Key::Space);
        assert_eq!(node.value(), None, "role and checked, no On/Off text");
    }
    assert!(check.get_untracked() && switch.get_untracked() && !bold.get_untracked());
    check_box.a11y(A11yAction::Activate);
    switch_node.a11y(A11yAction::Toggle);
    assert!(!check.get_untracked() && !switch.get_untracked());
    let locked = screen.get_by_role(Role::CheckBox, "Locked");
    locked.force_click();
    locked.a11y(A11yAction::Toggle);
    assert_eq!(locked.checked(), Some(CheckedState::Unchecked));

    let transform = screen.get_by_role(Role::Button, "Transform");
    assert_eq!(transform.is_expanded(), Some(false));
    transform.press_key(Key::Enter);
    assert!(open.get_untracked());
    transform.press_key(Key::Space);
    assert!(!open.get_untracked());
    transform.a11y(A11yAction::Toggle);
    assert_eq!(transform.is_expanded(), Some(true));
    transform.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked());
    clean(&screen);
}

#[test]
fn modal_contains_input_and_restores_opener_focus() {
    let (screen, _) = kivact::test::render_with_handle(crate::support::options(), || {
        (
            (),
            column((
                {
                    let open = signal(false);
                    (
                        kivact::ui::button("Export...")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .with_node(move |node| node.expanded(open))
                            .on_press(move || open.set(true)),
                        kivact::ui::dialog("Export...", open)
                            .closable(false)
                            .content(|| {
                                kivact::ui::text_input("File name", signal("shot.exr".into()))
                                    .into_view()
                            })
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
    let opener = screen.get_by_role(Role::Button, "Export...");
    opener.focus();
    opener.click();
    screen.get_by_role(Role::TextBox, "File name").focus();
    screen.press_key(Key::Escape);
    assert_eq!(screen.focused().map(|e| e.name()), Some("Export...".into()));
    clean(&screen);
}

#[test]
fn nested_menu_keys_choose_a_leaf_key_and_delayed_tooltip_dismisses() {
    let (screen, selection) = kivact::test::render_with_handle(crate::support::options(), || {
        let selection = signal(None::<&'static str>);
        (
            selection,
            row((
                kivact::ui::menu(
                    "Add",
                    vec![
                        MenuEntry::submenu(
                            "mesh",
                            "Mesh",
                            vec![
                                MenuEntry::item("plane", "Plane"),
                                MenuEntry::item("cube", "Cube"),
                            ],
                        ),
                        MenuEntry::item("camera", "Camera"),
                    ],
                )
                .on_activate(move |key: &'static str| selection.set(Some(key)))
                .offset(0.0)
                .attach(
                    kivact::ui::button("Add")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .trailing_icon(kivact::view::icon("chevron-down")),
                ),
                kivact::ui::tooltip("Camera View")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .content(move || {
                        column((
                            text("Camera View").style(style().weight(600)),
                            text("Preview active camera").style(style().font_size(11)),
                        ))
                    })
                    .attach(
                        kivact::ui::button("Help")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .with_node(move |node| node.test_id("tooltip-target")),
                    ),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Add").click();
    screen
        .get_by_role(Role::MenuItem, "Mesh")
        .press_key(Key::ArrowRight);
    screen
        .get_by_role(Role::MenuItem, "Plane")
        .press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(selection.get_untracked(), Some("cube"));
    assert!(screen.query_by_role(Role::Menu, "Add").is_none());
    screen.get_by_test_id("tooltip-target").hover();
    screen.advance(Duration::from_millis(499));
    assert!(screen.query_by_role(Role::Tooltip, "Camera View").is_none());
    screen.advance(Duration::from_millis(1));
    assert!(screen.get_by_role(Role::Tooltip, "Camera View").exists());
    screen.get_by_role(Role::Button, "Help").click();
    assert!(screen.query_by_role(Role::Tooltip, "Camera View").is_none());
    clean(&screen);
}

fn choice_options(
    entries: &[(&'static str, &'static str)],
) -> Vec<kivact::interaction::ListItem<&'static str>> {
    entries
        .iter()
        .map(|(key, label)| kivact::interaction::ListItem::new(*key, *label))
        .collect()
}
