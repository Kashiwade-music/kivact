//! Role: Tests for the interactions of Creative controls through the public API.

use super::{choice_options, clean};
use kivact::draw::Point;
use kivact::prelude::*;
use kivact_ui_creative as ui;
use kivact_ui_creative::ThemeOptions;

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
