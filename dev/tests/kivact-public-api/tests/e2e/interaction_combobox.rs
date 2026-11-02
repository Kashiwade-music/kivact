//! Role: Tests the combobox behavior's popup and multiple choice on real windows.
//!
//! A list that is a popup opened by the arrows, closed by Escape, a choice and
//! the loss of the focus, and a combobox that takes several options and
//! removes the last one with Backspace, through the public API only.

use crate::support::mount;
use kivact::interaction::{ListItem, ListSelection, use_combobox, use_text_edit};
use kivact::prelude::*;
use kivact::view::for_each;

fn options() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("red", "Red"),
        ListItem::new("green", "Green"),
        ListItem::new("blue", "Blue").disabled(true),
        ListItem::new("pink", "Pink"),
    ]
}

/// A field over a popup list, and a button to leave it for.
fn picker(
    open: Signal<bool>,
    value: Signal<Option<&'static str>>,
    values: Option<Signal<Vec<&'static str>>>,
) -> impl IntoView {
    let labels = options();
    let selection = match values {
        Some(values) => ListSelection::Multiple(values.into()),
        None => ListSelection::Single(value.into()),
    };
    let combo = use_combobox(labels.clone(), selection).popup(open);
    column((
        node()
            .name("Color")
            .part(use_text_edit(combo.query()).control())
            .part(combo.input()),
        show(combo.shown(), move || {
            let labels = labels.clone();
            column(for_each(
                combo.visible(),
                |key| *key,
                move |key, _| {
                    let label = labels
                        .iter()
                        .find(|option| *option.key() == key)
                        .map(ListItem::label)
                        .unwrap_or_default();
                    node().part(combo.item(key)).child(text(label))
                },
            ))
            .part(combo.content())
            .name("Colors")
        }),
        button("Elsewhere"),
    ))
}

#[test]
fn the_arrows_open_a_closed_list_and_move_the_active_option_once_it_is_open() {
    let (screen, (open, value)) = mount(|| {
        let open = signal(false);
        let value = signal(None::<&'static str>);
        ((open, value), picker(open, value, None))
    });
    let field = screen.get_by_role(Role::TextBox, "Color");
    assert!(screen.query_by_role(Role::ListBox, "Colors").is_none());
    field.focus();
    field.press_key(Key::ArrowDown);
    assert!(open.get_untracked(), "ArrowDown opens it");
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Option, "Red").is_selected());
    field.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Option, "Green").is_selected());
    field.press_key(Key::ArrowDown);
    assert!(
        screen.get_by_role(Role::Option, "Pink").is_selected(),
        "the disabled option is skipped"
    );
    field.press_key(Key::Escape);
    assert!(!open.get_untracked(), "Escape closes it");
    screen.run_until_idle();
    field.press_key(Key::ArrowUp);
    assert!(open.get_untracked(), "ArrowUp opens it too");
    assert_eq!(value.get_untracked(), None);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn enter_chooses_only_in_an_open_list_and_a_choice_closes_it() {
    let (screen, (open, value)) = mount(|| {
        let open = signal(false);
        let value = signal(None::<&'static str>);
        ((open, value), picker(open, value, None))
    });
    let field = screen.get_by_role(Role::TextBox, "Color");
    field.focus();
    field.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), None, "a closed list has no choice");
    assert!(!open.get_untracked());
    open.set(true);
    screen.run_until_idle();
    field.press_key(Key::ArrowDown);
    field.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), Some("green"));
    assert!(!open.get_untracked(), "choosing closes it");
    open.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Option, "Pink").click();
    assert_eq!(value.get_untracked(), Some("pink"), "a click chooses");
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
