//! Role: Check editable popup combobox semantics and bounded keyboard navigation.
use crate::support::mount;
use kivact::interaction::{ListItem, use_combobox, use_text_edit};
use kivact::prelude::*;

fn picker() -> impl Component {
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
