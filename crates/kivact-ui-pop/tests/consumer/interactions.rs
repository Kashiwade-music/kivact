//! Role: Tests for component interactions.

use kivact::prelude::*;
use kivact_ui_pop as ui;

#[test]
fn theme_change_preserves_the_shared_tab_and_range_semantic_tree() {
    let (screen, (theme, selected, amount)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
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
                        kivact::ui::slider(amount).label("Amount"),
                    ))
                }),
            )
        },
    );
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
fn theme_switch_changes_existing_nodes_and_pixels_without_remounting_state() {
    let (screen, (dark, checked)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
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
                    move || kivact::ui::card(kivact::ui::checkbox("Sync", checked)),
                ),
            )
        },
    );
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
