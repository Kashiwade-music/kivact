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

#[test]
fn listbox_list_tab_enters_first_selected_option_in_item_order() {
    let (screen, selected) = mount(|| {
        let selected = signal(vec!["c", "b"]);
        let items = vec![
            ListItem::new("a", "Apple"),
            ListItem::new("b", "Banana"),
            ListItem::new("c", "Cherry"),
        ];
        let list = kivact::interaction::use_list(
            items.clone(),
            selected,
            kivact::interaction::ListOptions::default(),
        );
        (
            selected,
            column((
                node()
                    .role(Role::Button)
                    .name("Before")
                    .focusable(true)
                    .child(text("Before")),
                node().part(list.list()).name("Fruits").child(for_each(
                    items,
                    |item| *item.key(),
                    move |item| {
                        let key = *item.get().key();
                        node()
                            .part(list.item(key))
                            .child(text(move || item.get().label()))
                            .style(style().height(30))
                    },
                )),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Before").focus();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Option, "Banana").is_focused());
    assert_eq!(selected.get_untracked(), vec!["c", "b"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn listbox_list_tab_reentry_follows_external_selection() {
    let (screen, selected) = mount(|| {
        let selected = signal(Some("a"));
        (
            selected,
            column((
                node()
                    .role(Role::Button)
                    .name("Before")
                    .focusable(true)
                    .child(text("Before")),
                list(
                    "Fruits",
                    vec![ListItem::new("a", "Apple"), ListItem::new("b", "Banana")],
                )
                .selection(selected),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Before").focus();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Option, "Apple").is_focused());
    screen.press_tab(true);
    selected.set(Some("b"));
    screen.run_until_idle();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Option, "Banana").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn listbox_list_virtual_options_publish_logical_positions() {
    let (screen, ()) = mount(|| {
        let items = vec![
            ListItem::new("a", "Apple"),
            ListItem::new("b", "Banana"),
            ListItem::new("c", "Cherry"),
            ListItem::new("d", "Date"),
        ];
        let list = kivact::interaction::use_list(
            items,
            signal(Some("b")),
            kivact::interaction::ListOptions::default(),
        );
        (
            (),
            node().part(list.list()).name("Fruits").child(column((
                node().part(list.item("b")).child(text("Banana")),
                node().part(list.item("c")).child(text("Cherry")),
            ))),
        )
    });
    let banana = screen.get_by_role(Role::Option, "Banana");
    let cherry = screen.get_by_role(Role::Option, "Cherry");
    assert_eq!(banana.set_size(), Some(4));
    assert_eq!(banana.pos_in_set(), Some(2));
    assert_eq!(cherry.set_size(), Some(4));
    assert_eq!(cherry.pos_in_set(), Some(3));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
