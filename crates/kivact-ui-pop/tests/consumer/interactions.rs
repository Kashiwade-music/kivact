//! Role: Tests for component interactions.

use kivact::interaction::ListItem;
use kivact::prelude::*;
use kivact_ui_pop as ui;
use std::time::Duration;

#[test]
fn theme_change_preserves_the_shared_tab_and_range_semantic_tree() {
    let (screen, (theme, selected, amount)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let theme = signal(ui::theme(kivact::style::ThemeMode::Light));
            let selected = signal("first");
            let amount = signal(25.0);
            (
                (theme, selected, amount),
                kivact::style::provide_theme(theme, move || {
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
fn tooltip_shows_on_focus_below_the_trigger_and_wraps_a_long_label() {
    let long = "Exports every page of the document with its comments and revision history \
                as a single archive that other people can open without this application.";
    let (screen, ()) = kivact::test::render_with_handle(crate::support::options(), move || {
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
                            .icon(kivact::view::icon("download")),
                    ),
                kivact::ui::tooltip("Add")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("Add")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("plus")),
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

#[test]
fn theme_switch_changes_existing_nodes_and_pixels_without_remounting_state() {
    let (screen, (dark, checked)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let dark = signal(false);
            let checked = signal(true);
            (
                (dark, checked),
                kivact::style::provide_theme(
                    move || {
                        ui::theme(if dark.get() {
                            kivact::style::ThemeMode::Dark
                        } else {
                            kivact::style::ThemeMode::Light
                        })
                    },
                    move || ui::panel(kivact::ui::checkbox("Sync", checked)),
                ),
            )
        });
    let before = screen.screenshot().hash();
    dark.set(true);
    screen.run_until_idle();
    assert!(checked.get_untracked());
    assert_ne!(screen.screenshot().hash(), before);
    assert_eq!(
        screen.get_by_role(Role::CheckBox, "Sync").checked(),
        Some(CheckedState::Checked)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
