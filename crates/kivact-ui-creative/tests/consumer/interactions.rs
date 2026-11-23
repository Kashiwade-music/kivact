//! Role: Tests for the interactions of Creative controls through the public API.

use kivact::prelude::*;
use kivact_ui_creative as ui;

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

fn choice_options(
    entries: &[(&'static str, &'static str)],
) -> Vec<kivact::interaction::ListItem<&'static str>> {
    entries
        .iter()
        .map(|(key, label)| kivact::interaction::ListItem::new(*key, *label))
        .collect()
}
