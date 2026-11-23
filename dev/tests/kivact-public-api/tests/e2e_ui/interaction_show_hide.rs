//! Role: Tests the commands that show and hide an interaction's surface.
//!
//! `show` and `hide` are idempotent: a second call changes nothing. `toggle`
//! inverts, so two calls return to the start. Each handler calls its command
//! twice, so a press outside the surface cannot hide what the first call did.
//! Through the public API only.

use kivact::interaction::Selection;
use kivact::interaction::{
    ComboboxOptions, NavOpen, use_combobox, use_menu, use_navigation_menu, use_select,
    use_text_edit,
};
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::Placement;
use kivact_public_api_tests::support::mount;

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
        let select = use_select(colors(), selected, signal(false));
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
        let combo = use_combobox(
            colors(),
            Selection::Single(value),
            signal(String::new()),
            signal(false),
            ComboboxOptions::default(),
        );
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
        let menu = use_menu(vec![MenuEntry::item("cut", "Cut")], signal(false));
        let view = column((
            node().part(menu.trigger()).name("Edit").child(text("Edit")),
            menu.surface(Placement::Below, move || {
                column(node().part(menu.item("cut")).child(text("Cut"))).name("Edit menu")
            }),
            button("Show twice").on_press(move || {
                menu.show();
                menu.show();
            }),
            button("Toggle twice").on_press(move || {
                menu.toggle();
                menu.toggle();
            }),
            button("Hide twice").on_press(move || {
                menu.hide();
                menu.hide();
            }),
        ));
        (menu.is_open(), view)
    });
    press(&screen, "Toggle twice");
    assert!(!open.get_untracked(), "two toggles return to hidden");
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
        let menu = use_navigation_menu(signal(false));
        let first = menu.panel();
        let view = column((
            button("Show twice").on_press(move || {
                first.show();
                first.show();
            }),
            button("Hide twice").on_press(move || {
                menu.hide();
                menu.hide();
            }),
            button("Toggle twice").on_press(move || {
                first.toggle();
                first.toggle();
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

#[test]
fn leaving_a_panel_keeps_another_panels_pending_open() {
    let (screen, state) = mount(|| {
        let menu = use_navigation_menu(signal(false));
        let first = menu.panel();
        let second = menu.panel();
        let view = column((
            button("Open first").on_press(move || first.hover()),
            button("Wait for second")
                .on_press(move || second.hover_after(std::time::Duration::from_millis(300))),
            button("Leave first").on_press(move || first.leave()),
        ));
        (menu.state(), view)
    });
    press(&screen, "Open first");
    assert_eq!(state.get_untracked(), NavOpen::Hover(0));
    press(&screen, "Wait for second");
    press(&screen, "Leave first");
    assert_eq!(state.get_untracked(), NavOpen::Closed);
    screen.advance(std::time::Duration::from_millis(400));
    assert_eq!(
        state.get_untracked(),
        NavOpen::Hover(1),
        "the close that the caller's open echoes must not cancel the second panel's pending open"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn disclosure_popup_and_dialog_commands_are_idempotent() {
    use kivact::interaction::{DialogMode, use_dialog, use_disclosure, use_popup};
    let (screen, (disclosure, popup, dialog)) = mount(|| {
        let disclosure = use_disclosure(signal(false));
        let popup = use_popup(signal(false));
        let dialog = use_dialog(signal(false), DialogMode::Modal);
        let view = column((
            button("Show twice").on_press(move || {
                disclosure.show();
                disclosure.show();
                popup.show();
                popup.show();
                dialog.show();
                dialog.show();
            }),
            button("Hide twice").on_press(move || {
                disclosure.hide();
                disclosure.hide();
                popup.hide();
                popup.hide();
                dialog.hide();
                dialog.hide();
            }),
            button("Toggle twice").on_press(move || {
                disclosure.toggle();
                disclosure.toggle();
                popup.toggle();
                popup.toggle();
                dialog.toggle();
                dialog.toggle();
            }),
        ));
        (
            (disclosure.is_open(), popup.is_open(), dialog.is_open()),
            view,
        )
    });
    let all_open = || {
        (
            disclosure.get_untracked(),
            popup.get_untracked(),
            dialog.get_untracked(),
        )
    };
    press(&screen, "Show twice");
    assert_eq!(
        all_open(),
        (true, true, true),
        "a second show does not hide"
    );
    press(&screen, "Hide twice");
    assert_eq!(
        all_open(),
        (false, false, false),
        "a second hide does not show"
    );
    press(&screen, "Toggle twice");
    assert_eq!(all_open(), (false, false, false), "two toggles return");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn popup_surface_mounts_the_content_at_its_trigger() {
    use kivact::interaction::use_popup;
    use kivact::view::Placement;
    let (screen, _open) = mount(|| {
        let popup = use_popup(signal(false));
        (
            popup.is_open(),
            column((
                node()
                    .part(popup.trigger())
                    .name("Options")
                    .child(text("Options")),
                popup.surface(Placement::Below, || column((button("Inside"),))),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Options").click();
    screen.get_by_text("Inside");
    screen.press_key(Key::Escape);
    assert!(screen.query_by_text("Inside").is_none());
    assert!(screen.get_by_role(Role::Button, "Options").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_surface_opens_with_the_given_align_and_offset() {
    use kivact::interaction::use_popup;
    use kivact::view::{OverlayAlign, Placement};
    let (screen, _open) = mount(|| {
        let popup = use_popup(signal(false));
        (
            popup.is_open(),
            column((
                node()
                    .part(popup.trigger())
                    .name("Options")
                    .child(text("Options"))
                    .test_id("trigger"),
                popup.surface(
                    Placement::Below.align(OverlayAlign::Start).offset(8.0),
                    || column((button("Inside"),)).test_id("surface"),
                ),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Options").click();
    let anchor = screen
        .get_by_test_id("trigger")
        .rect()
        .expect("trigger layout");
    let surface = screen
        .get_by_test_id("surface")
        .rect()
        .expect("surface layout");
    assert!(
        (surface.x - anchor.x).abs() < 0.5,
        "start edges line up: {surface:?} {anchor:?}"
    );
    assert!(
        (surface.y - (anchor.y + anchor.height + 8.0)).abs() < 0.5,
        "top edge is 8px below the anchor: {surface:?} {anchor:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
