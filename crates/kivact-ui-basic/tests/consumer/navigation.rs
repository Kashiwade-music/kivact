//! Role: Menu component tests: dropdown, use_context menu, menu bar and navigation menu.

use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{RenderOptions, Screen};
use kivact::ui::{ButtonVariant, button};
use kivact::view::{Modifiers, MouseButton};
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
pub(super) fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::ColorName::Background)
            .color(basic::ColorName::Foreground)
            .padding(16)
            .gap(16)
            .grow(1)
            .align_items(Align::Start),
    )
}

pub(super) fn options(mode: kivact::style::ThemeMode) -> RenderOptions {
    RenderOptions::from_env()
        .window_size(900.0, 700.0)
        .theme(basic::theme(mode))
        .reduced_motion()
}

pub(super) fn light() -> RenderOptions {
    options(kivact::style::ThemeMode::Light)
}

pub(super) fn open_button(name: &'static str) -> impl Component {
    button(name).variant(ButtonVariant::Outline)
}

pub(super) fn account_items(radio: &str, check: bool) -> Vec<MenuEntry<&'static str>> {
    vec![
        MenuEntry::heading("My Account"),
        MenuEntry::item("profile", "Profile")
            .with_icon("user")
            .with_shortcut("Ctrl+P"),
        MenuEntry::item("billing", "Billing").disabled(true),
        MenuEntry::separator(),
        MenuEntry::checkbox("bars", "Status Bar", check).keep_open(true),
        MenuEntry::radio("top", "Top", radio == "top").keep_open(true),
        MenuEntry::radio("bottom", "Bottom", radio == "bottom").keep_open(true),
        MenuEntry::separator(),
        MenuEntry::submenu(
            "more",
            "More",
            vec![
                MenuEntry::item("email", "Email"),
                MenuEntry::item("message", "Message"),
            ],
        ),
        MenuEntry::item("logout", "Log out").destructive(true),
    ]
}

type Rig = (
    Signal<Vec<&'static str>>,
    Signal<bool>,
    Signal<&'static str>,
);

fn mount_dropdown() -> (Screen, Rig) {
    let policy = kivact::interaction::InteractionPolicy {
        menu: kivact::interaction::MenuPolicy {
            disabled_focus: kivact::interaction::DisabledMenuFocus::Skip,
            ..Default::default()
        },
        ..Default::default()
    };
    let options = light().theme(basic::theme(kivact::style::ThemeMode::Light).with(policy));
    kivact::test::render_with_handle(options, || {
        let chosen = signal(Vec::new());
        let check = signal(true);
        let radio = signal("top");
        let items = move || account_items(radio.get(), check.get());
        (
            (chosen, check, radio),
            page((
                menu("Account", items)
                    .on_activate(move |key: &'static str| {
                        chosen.update(|chosen| chosen.push(key));
                        match key {
                            "bars" => check.update(|on| *on = !*on),
                            "top" | "bottom" => radio.set(key),
                            _ => {}
                        }
                    })
                    .at_least_anchor_width()
                    .attach(open_button("Account")),
                button("After"),
            )),
        )
    })
}

fn menu_item<'a>(screen: &'a Screen, name: &str) -> kivact::test::Element<'a> {
    screen.get_by_role(Role::MenuItem, name)
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

#[test]
fn dropdown_menu_navigates_by_keyboard_and_skips_labels() {
    let (screen, (chosen, _, _)) = mount_dropdown();
    let trigger = screen.get_by_role(Role::Button, "Account");
    assert_eq!(trigger.is_expanded(), Some(false));
    trigger.press_key(Key::ArrowDown);
    assert_eq!(trigger.is_expanded(), Some(true));
    screen.get_by_role(Role::Menu, "Account");
    // The heading is text, not an item; the first enabled item has the focus.
    assert!(screen.query_by_role(Role::MenuItem, "My Account").is_none());
    screen.get_by_text("My Account");
    assert!(menu_item(&screen, "Profile").is_focused());
    assert_eq!(menu_item(&screen, "Profile").description(), "Ctrl+P");
    screen.press_key(Key::ArrowDown);
    assert!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Status Bar")
            .is_focused(),
        "the disabled item and the separator are skipped"
    );
    assert_eq!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Status Bar")
            .checked(),
        Some(CheckedState::Checked)
    );
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), ["bars"]);
    assert_eq!(
        trigger.is_expanded(),
        Some(true),
        "a check row keeps the menu open"
    );
    assert!(
        screen
            .get_by_role(Role::MenuItemCheckBox, "Status Bar")
            .is_focused(),
        "and the focus stays on it"
    );
    // A plain item closes the menu and returns the focus to the trigger.
    screen.press_key(Key::ArrowUp);
    assert!(menu_item(&screen, "Profile").is_focused());
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), ["bars", "profile"]);
    assert!(trigger.is_focused(), "choosing returns the focus");
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
    let _ = shift();
}

#[test]
fn navigation_menu_shows_one_panel_at_a_time() {
    let screen = kivact::test::render_with_handle(light(), || {
        (
            (),
            page(
                kivact::ui::navigation_menu("Navigation")
                    .link("alpha", "Alpha", || text("Alpha panel"))
                    .link("beta", "Beta", || text("Beta panel"))
                    .close_delay(std::time::Duration::ZERO),
            ),
        )
    })
    .0;
    let alpha = screen.get_by_role(Role::Button, "Alpha");
    let beta = screen.get_by_role(Role::Button, "Beta");
    let shown = |text: &str| screen.query_by_text(text).is_some();
    // Hovering a trigger shows its panel, and the trigger reports it.
    alpha.hover();
    assert!(shown("Alpha panel"));
    assert_eq!(alpha.is_expanded(), Some(true));
    // Escape on the focused trigger closes a hover-opened panel.
    alpha.focus();
    screen.press_key(Key::Escape);
    assert!(!shown("Alpha panel"));
    assert_eq!(alpha.is_expanded(), Some(false));
    // Hovering the next trigger replaces the panel: one is open at a time.
    screen.mouse_move(850.0, 650.0);
    beta.hover();
    assert!(shown("Beta panel"));
    alpha.hover();
    assert!(shown("Alpha panel"));
    assert!(!shown("Beta panel"));
    assert_eq!(alpha.is_expanded(), Some(true));
    assert_eq!(beta.is_expanded(), Some(false));
    // ArrowDown on a trigger pins its panel.
    screen.mouse_move(850.0, 650.0);
    assert!(!shown("Alpha panel"));
    alpha.focus();
    screen.press_key(Key::ArrowRight);
    assert!(beta.is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(shown("Beta panel"));
    assert!(!shown("Alpha panel"));
    assert_eq!(alpha.is_expanded(), Some(false));
    assert_eq!(beta.is_expanded(), Some(true));
    screen.press_key(Key::Escape);
    assert!(!shown("Beta panel"));
    assert_eq!(beta.is_expanded(), Some(false));
    // A click on a hover-opened panel pins it: it stays after the pointer leaves.
    alpha.hover();
    alpha.click();
    screen.mouse_move(850.0, 650.0);
    assert!(shown("Alpha panel"));
    assert_eq!(alpha.is_expanded(), Some(true));
    screen.press_key(Key::Escape);
    assert!(!shown("Alpha panel") && !shown("Beta panel"));
    assert_eq!(alpha.is_expanded(), Some(false));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

/// The box a use_context menu is attached to in the tests.
pub(super) fn right_click_area() -> Node {
    column(text("Right click here"))
        .with_node(|root| root.role(Role::Group))
        .name("Right click here")
        .focusable(true)
        .style(
            style()
                .width(150)
                .height(75)
                .align_items(Align::Center)
                .justify_content(kivact::style::Justify::Center)
                .border(1, basic::ColorName::Border),
        )
}

pub(super) fn edit_items() -> Vec<MenuEntry<&'static str>> {
    vec![
        MenuEntry::heading("Edit"),
        MenuEntry::item("undo", "Undo").with_shortcut("Ctrl+Z"),
        MenuEntry::item("redo", "Redo"),
        MenuEntry::separator(),
        MenuEntry::submenu(
            "share",
            "Share",
            vec![MenuEntry::item("email", "Email link")],
        ),
    ]
}

#[test]
fn context_menu_opens_at_the_pointer_and_by_keyboard() {
    let (screen, chosen) = kivact::test::render_with_handle(light(), || {
        let chosen = signal(Vec::new());
        (
            chosen,
            page(
                menu("Edit", edit_items())
                    .on_activate(move |key: &'static str| chosen.update(|chosen| chosen.push(key)))
                    .attach_context(right_click_area().test_id("area")),
            ),
        )
    });
    let area = screen.get_by_test_id("area");
    assert!(
        screen
            .query_by_role(Role::Menu, "Right click here")
            .is_none()
    );
    // A right click opens the menu below and after the pointer.
    area.click_with(MouseButton::Secondary);
    let menu = screen.get_by_role(Role::Menu, "Right click here");
    assert!(menu.exists());
    let rect = area.rect().expect("laid out");
    let menu_rect = menu.rect().expect("laid out");
    assert!(
        menu_rect.x > rect.x + 1.0,
        "placed at the pointer, not at the area start"
    );
    assert!(menu_rect.width >= 144.0, "min-w-36");
    assert!(menu_item(&screen, "Undo").is_focused());
    screen.press_key(Key::Escape);
    assert!(
        screen
            .query_by_role(Role::Menu, "Right click here")
            .is_none()
    );
    assert!(area.is_focused(), "the focus returns to the area");

    // Shift+F10 and the menu key open it at the area.
    area.press_key_with(Key::F(10), shift());
    assert!(menu_item(&screen, "Undo").is_focused());
    screen.press_key(Key::Escape);
    area.press_key(Key::ContextMenu);
    assert!(menu_item(&screen, "Undo").is_focused());
    // The heading is not an item; the submenu row opens with ArrowRight.
    screen.press_key(Key::ArrowDown);
    assert!(menu_item(&screen, "Redo").is_focused());
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowRight);
    assert!(menu_item(&screen, "Email link").is_focused());
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), ["email"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}
