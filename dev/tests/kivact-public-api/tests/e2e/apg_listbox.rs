//! Role: Check select popup listbox active-option semantics with retained focus.
use crate::support::mount;
use kivact::interaction::{ListItem, use_select};
use kivact::prelude::*;
use kivact::view::{Modifiers, Placement, overlay};

#[test]
fn listbox_select_active_option_is_selected_without_committing_the_value() {
    let (screen, selected) = mount(|| {
        let selected = signal(Some("a"));
        let options = vec![ListItem::new("a", "Apple"), ListItem::new("b", "Banana")];
        let select = use_select(options.clone(), selected);
        let anchor = node_ref();
        (
            selected,
            column((
                node()
                    .node_ref(anchor)
                    .name("Fruit")
                    .part(select.trigger())
                    .child(text(select.label())),
                show(select.is_open(), move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        column(for_each(
                            options.clone(),
                            |row| *row.key(),
                            move |row| {
                                let key = *row.get().key();
                                node().part(select.item(key)).child(text(if key == "a" {
                                    "Apple"
                                } else {
                                    "Banana"
                                }))
                            },
                        )),
                    )
                    .with_node(|node| node.part(select.content()).name("Fruits"))
                }),
            )),
        )
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Fruit");
    trigger.focus();
    trigger.press_key_with(
        Key::ArrowDown,
        Modifiers {
            alt: true,
            ..Modifiers::default()
        },
    );
    screen.run_until_idle();
    trigger.press_key(Key::ArrowDown);
    assert!(trigger.is_focused());
    assert!(trigger.has_active_descendant(&screen.get_by_role(Role::Option, "Banana")));
    assert!(screen.get_by_role(Role::Option, "Banana").is_selected());
    assert!(!screen.get_by_role(Role::Option, "Apple").is_selected());
    assert_eq!(selected.get_untracked(), Some("a"));
    trigger.press_key(Key::Escape);
    assert_eq!(selected.get_untracked(), Some("a"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
