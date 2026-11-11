//! Role: Tests the commands that show and hide an interaction's surface.
//!
//! `show` and `hide` are idempotent: a second call changes nothing. `toggle`
//! inverts, so two calls return to the start. Each handler calls its command
//! twice, so a press outside the surface cannot hide what the first call did.
//! Through the public API only.

use crate::support::mount;
use kivact::interaction::{
    ListItem, MenuEntry, NavOpen, Selection, use_combobox, use_menu, use_navigation_menu,
    use_select, use_text_edit,
};
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::for_each;

fn colors() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("red", "Red"),
        ListItem::new("green", "Green"),
        ListItem::new("pink", "Pink"),
    ]
}

/// Press the button `name` once.
fn press(screen: &Screen, name: &str) {
    screen.get_by_role(Role::Button, name).click();
    screen.run_until_idle();
}

#[test]
fn select_show_twice_stays_open_and_toggle_twice_returns() {
    let (screen, (open, sheet)) = mount(|| {
        let selected = signal(None::<&'static str>);
        let select = use_select(colors(), selected);
        let view = column((
            node()
                .part(select.trigger())
                .name("Color")
                .child(text(select.label())),
            button("Show twice").on_press(move || {
                select.show();
                select.show();
            }),
            button("Hide twice").on_press(move || {
                select.hide();
                select.hide();
            }),
            button("Toggle twice").on_press(move || {
                select.toggle();
                select.toggle();
            }),
            button("Sheet twice").on_press(move || {
                select.show_sheet();
                select.show_sheet();
            }),
        ));
        ((select.is_open(), select.is_sheet_open()), view)
    });
    press(&screen, "Show twice");
    assert!(open.get_untracked(), "a second show does not hide");
    press(&screen, "Hide twice");
    assert!(!open.get_untracked(), "a second hide does not show");
    press(&screen, "Toggle twice");
    assert!(!open.get_untracked(), "two toggles return to hidden");
    press(&screen, "Sheet twice");
    assert!(open.get_untracked(), "a second show_sheet does not hide");
    assert!(sheet.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn combobox_show_keeps_a_shown_list_and_its_filter() {
    let (screen, (open, filtering)) = mount(|| {
        let value = signal(None::<&'static str>);
        let combo =
            use_combobox(colors(), Selection::Single(value.into_binding())).popup(signal(false));
        // Shortcuts on the field: the focus stays in it.
        let shortcuts = move |event: KeyEvent| {
            match event.key() {
                Key::F(5) => {
                    combo.show();
                    combo.show();
                }
                Key::F(6) => {
                    combo.hide();
                    combo.hide();
                }
                Key::F(7) => {
                    combo.toggle();
                    combo.toggle();
                }
                Key::F(8) => combo.toggle(),
                _ => return Handled::No,
            }
            Handled::Yes
        };
        let view = column((
            node()
                .name("Color")
                .on_key(shortcuts)
                .part(use_text_edit(combo.query()).control())
                .part(combo.input()),
            show(combo.is_open(), move || {
                column(for_each(
                    combo.visible(),
                    |key| *key,
                    move |key| {
                        let key = key.with(|key| *key);
                        node().part(combo.item(key)).child(text(key))
                    },
                ))
                .part(combo.content())
                .name("Colors")
            }),
        ));
        ((combo.is_open(), combo.is_filtering()), view)
    });
    let field = screen.get_by_role(Role::ComboBox, "Color");
    field.focus();
    field.type_text("p");
    screen.run_until_idle();
    assert!(
        open.get_untracked() && filtering.get_untracked(),
        "typing filters"
    );
    field.press_key(Key::F(5));
    assert!(open.get_untracked(), "show does not hide");
    assert!(filtering.get_untracked(), "show keeps the query's filter");
    field.press_key(Key::F(7));
    assert!(open.get_untracked(), "two toggles return to shown");
    field.press_key(Key::F(6));
    assert!(!open.get_untracked(), "a second hide does not show");
    field.press_key(Key::F(8));
    assert!(open.get_untracked(), "toggle shows a hidden list");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn menu_show_twice_stays_open() {
    let (screen, open) = mount(|| {
        let menu = use_menu(vec![MenuEntry::item("cut", "Cut")]);
        let view = column((
            node().part(menu.trigger()).name("Edit").child(text("Edit")),
            menu.while_open(move || {
                column(node().part(menu.item("cut")).child(text("Cut")))
                    .part(menu.content())
                    .name("Edit menu")
            }),
            button("Show twice").on_press(move || {
                menu.show();
                menu.show();
            }),
            button("Hide twice").on_press(move || {
                menu.hide();
                menu.hide();
            }),
        ));
        (menu.is_open(), view)
    });
    press(&screen, "Show twice");
    assert!(open.get_untracked(), "a second show does not hide");
    screen.get_by_text("Cut");
    press(&screen, "Hide twice");
    assert!(!open.get_untracked(), "a second hide does not show");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn navigation_menu_show_is_idempotent_and_toggle_twice_returns() {
    let (screen, state) = mount(|| {
        let menu = use_navigation_menu();
        let view = column((
            button("Show twice").on_press(move || {
                menu.show(0);
                menu.show(0);
            }),
            button("Hide twice").on_press(move || {
                menu.hide();
                menu.hide();
            }),
            button("Toggle twice").on_press(move || {
                menu.toggle(0);
                menu.toggle(0);
            }),
        ));
        (menu.state(), view)
    });
    press(&screen, "Show twice");
    assert_eq!(
        state.get_untracked(),
        NavOpen::Pinned(0),
        "a second show does not hide"
    );
    press(&screen, "Toggle twice");
    assert_eq!(
        state.get_untracked(),
        NavOpen::Pinned(0),
        "two toggles return"
    );
    press(&screen, "Hide twice");
    assert_eq!(
        state.get_untracked(),
        NavOpen::Closed,
        "a second hide does not show"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
