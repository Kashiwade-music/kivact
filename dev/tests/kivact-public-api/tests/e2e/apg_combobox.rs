//! Role: Check editable popup combobox semantics and bounded keyboard navigation.
use crate::support::mount;
use kivact::interaction::{ListItem, use_combobox, use_text_edit};
use kivact::prelude::*;

fn picker() -> impl Element {
    let options = vec![ListItem::new("a", "Apple"), ListItem::new("b", "Banana")];
    let combo = use_combobox(options, signal(None::<&str>)).popup(signal(false));
    column((
        node()
            .name("Fruit")
            .part(use_text_edit(combo.query()).control())
            .part(combo.input()),
        show(combo.is_open(), move || {
            column(for_each(
                combo.visible(),
                |key| *key,
                move |key| {
                    let key = key.get();
                    node().part(combo.item(key)).child(text(key))
                },
            ))
            .part(combo.content())
            .name("Fruits")
        }),
    ))
}

#[test]
fn popup_input_exposes_combobox_role_and_expansion() {
    let (screen, ()) = mount(|| ((), picker()));
    let input = screen.get_by_role(Role::ComboBox, "Fruit");
    assert_eq!(input.is_expanded(), Some(false));
    input.focus();
    input.press_key(Key::ArrowDown);
    assert_eq!(input.is_expanded(), Some(true));
    input.press_key(Key::Escape);
    assert_eq!(input.is_expanded(), Some(false));
    assert!(input.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn popup_arrow_navigation_stops_at_the_boundary() {
    let (screen, ()) = mount(|| ((), picker()));
    let input = screen.get_by_role(Role::ComboBox, "Fruit");
    input.focus();
    input.press_key(Key::ArrowDown);
    screen.run_until_idle();
    input.press_key(Key::ArrowUp);
    assert!(screen.get_by_role(Role::Option, "Apple").is_selected());
    input.press_key(Key::ArrowDown);
    input.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Option, "Banana").is_selected());
    assert!(input.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// APG combobox keyboard interaction and listbox popup notes:
// https://www.w3.org/WAI/ARIA/apg/patterns/combobox/
// The optional Alt arrows open/close without committing; Home/End remain native.
#[test]
fn standard_combobox_alt_arrows_preserve_the_choice_and_editor_focus() {
    let (screen, selected) = mount(|| {
        let selected = signal(Some("a"));
        (
            selected,
            kivact::ui::combobox(
                "Fruit",
                vec![ListItem::new("a", "Apple"), ListItem::new("b", "Banana")],
                selected,
            ),
        )
    });
    let input = screen.get_by_role(Role::ComboBox, "Fruit");
    input.focus();
    let alt = kivact::view::Modifiers {
        alt: true,
        ..Default::default()
    };
    input.press_key_with(Key::ArrowDown, alt);
    let apple = screen.get_by_role(Role::Option, "Apple");
    assert!(input.has_active_descendant(&apple));
    assert!(input.is_focused());
    input.press_key(Key::ArrowDown);
    assert!(input.has_active_descendant(&screen.get_by_role(Role::Option, "Banana")));
    input.press_key_with(Key::ArrowUp, alt);
    assert_eq!(selected.get_untracked(), Some("a"));
    assert_eq!(input.is_expanded(), Some(false));
    assert!(input.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// APG describes the single suggested value. The multiple-recipient extension
// keeps committed selection separate from the active descendant and exposes
// the listbox's multiselectable state.
#[test]
fn multiple_combobox_keeps_active_descendant_separate_from_chosen_options() {
    let (screen, selected) = mount(|| {
        let selected = signal(vec!["a"]);
        (
            selected,
            kivact::ui::combobox(
                "Fruit",
                vec![ListItem::new("a", "Apple"), ListItem::new("b", "Banana")],
                selected,
            )
            .chips(),
        )
    });
    let input = screen.get_by_role(Role::ComboBox, "Fruit");
    input.focus();
    input.press_key(Key::ArrowDown);
    input.press_key(Key::ArrowDown);
    let banana = screen.get_by_role(Role::Option, "Banana");
    assert!(input.has_active_descendant(&banana));
    assert!(!banana.is_selected());
    assert!(screen.get_by_role(Role::Option, "Apple").is_selected());
    input.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), vec!["a", "b"]);
    assert!(input.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
