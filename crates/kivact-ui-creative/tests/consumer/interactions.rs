//! Role: Tests for the interactions of Creative controls through the public API.

use kivact::interaction::use_toaster;
use kivact::prelude::*;
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
                kivact::style::provide_theme(theme, move || {
                    column((
                        kivact::ui::radio_group(
                            "Order",
                            choice_options(&[("first", "First"), ("second", "Second")]),
                            selected,
                        )
                        .with_node(move |node| {
                            node.orientation(kivact::view::Orientation::Horizontal)
                        })
                        .variant(kivact::ui::RadioGroupVariant::Segmented),
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
    theme.set(ui::theme_with(
        kivact::style::ThemeMode::Dark,
        ui::ThemeOptions {
            accent: ui::Accent::Aqua,
            density: ui::Density::Compact,
        },
    ));
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
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
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
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn toasts_are_live_pause_for_the_pointer_and_expire_on_the_host_clock() {
    let (screen, (toaster, (undone, undo))) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let undone = signal(false);
            let undo = kivact::interaction::Toast::new("Muted")
                .tone(kivact::interaction::Tone::Info)
                .body("Track 2")
                .action("Undo", move || undone.set(true))
                .duration(None);
            (
                (use_toaster(), (undone, undo)),
                kivact::ui::toast_region(use_toaster()),
            )
        });
    toaster.push(
        kivact::interaction::Toast::new("Exported")
            .tone(kivact::interaction::Tone::Success)
            .body("Wrote 3 files"),
    );
    toaster.push(
        kivact::interaction::Toast::new("Render failed")
            .tone(kivact::interaction::Tone::Danger)
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
        kivact::interaction::Toast::new("Clipping")
            .tone(kivact::interaction::Tone::Warning)
            .body("Master bus"),
    );
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Close").click();
    assert!(screen.query_by_text("Clipping").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn choice_options(
    entries: &[(&'static str, &'static str)],
) -> Vec<kivact::interaction::ListItem<&'static str>> {
    entries
        .iter()
        .map(|(key, label)| kivact::interaction::ListItem::new(*key, *label))
        .collect()
}
