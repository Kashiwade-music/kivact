//! Role: Menu component tests: dropdown, context menu, menu bar and navigation menu.

use super::{assert_clean, distance, pixel};
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
            .background(basic::Color::Background)
            .color(basic::Color::Foreground)
            .padding(16)
            .gap(16)
            .grow(1)
            .align_items(Align::Start),
    )
}

pub(super) fn options(mode: kivact::style::ThemeMode) -> RenderOptions {
    RenderOptions {
        theme: basic::theme(mode),
        width: 900.0,
        height: 700.0,
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}

pub(super) fn light() -> RenderOptions {
    options(kivact::style::ThemeMode::Light)
}

pub(super) fn open_button(name: &'static str) -> Node {
    button(name).variant(ButtonVariant::Outline).into_node()
}

pub(super) fn account_items(radio: &str, check: bool) -> Vec<MenuEntry<&'static str>> {
    vec![
        MenuEntry::heading("My Account"),
        MenuEntry::item("profile", "Profile")
            .icon("user")
            .shortcut("Ctrl+P"),
        MenuEntry::item("billing", "Billing").disabled(true),
        MenuEntry::separator(),
        MenuEntry::item("bars", "Status Bar")
            .checked(check)
            .keep_open(true),
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
    kivact::test::render_with_handle(light(), || {
        let chosen = signal(Vec::new());
        let check = signal(true);
        let radio = signal("top");
        let items = move || account_items(radio.get(), check.get());
        (
            (chosen, check, radio),
            page((
                menu("Account", items, move |key: &'static str| {
                    chosen.update(|chosen| chosen.push(key));
                    match key {
                        "bars" => check.update(|on| *on = !*on),
                        "top" | "bottom" => radio.set(key),
                        _ => {}
                    }
                })
                .at_least_trigger_width(true)
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
            .is_checked(),
        Some(true)
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
    assert_clean(&screen);
    let _ = shift();
}

#[test]
fn dropdown_menu_toggles_checks_and_radios_through_the_application() {
    let (screen, (chosen, check, radio)) = mount_dropdown();
    let trigger = screen.get_by_role(Role::Button, "Account");
    trigger.click();
    let bars = || screen.get_by_role(Role::MenuItemCheckBox, "Status Bar");
    assert_eq!(bars().is_checked(), Some(true));
    bars().hover();
    bars().click();
    assert!(!check.get_untracked(), "the application flipped the check");
    assert_eq!(chosen.get_untracked(), ["bars"]);
    assert_eq!(trigger.is_expanded(), Some(true), "the menu stays open");
    assert_eq!(bars().is_checked(), Some(false), "and shows the new state");
    let top = || screen.get_by_role(Role::MenuItemRadio, "Top");
    let bottom = || screen.get_by_role(Role::MenuItemRadio, "Bottom");
    assert_eq!(top().is_checked(), Some(true));
    assert_eq!(bottom().is_checked(), Some(false));
    bottom().hover();
    bottom().click();
    assert_eq!(radio.get_untracked(), "bottom");
    assert_eq!(
        trigger.is_expanded(),
        Some(true),
        "radio rows keep it open too"
    );
    assert_eq!(top().is_checked(), Some(false));
    assert_eq!(bottom().is_checked(), Some(true));
    // A disabled item cannot be chosen.
    let billing = menu_item(&screen, "Billing");
    assert!(billing.is_disabled());
    billing.force_click();
    assert!(!chosen.get_untracked().contains(&"billing"));
    assert_clean(&screen);
}

#[test]
fn dropdown_menu_opens_a_submenu_with_arrow_right_and_marks_destructive_items() {
    let (screen, (chosen, _, _)) = mount_dropdown();
    let trigger = screen.get_by_role(Role::Button, "Account");
    trigger.press_key(Key::ArrowDown);
    let more = menu_item(&screen, "More");
    assert_eq!(more.is_expanded(), Some(false));
    for _ in 0..4 {
        screen.press_key(Key::ArrowDown);
    }
    assert!(more.is_focused());
    screen.press_key(Key::ArrowRight);
    assert_eq!(more.is_expanded(), Some(true));
    assert!(menu_item(&screen, "Email").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(menu_item(&screen, "Message").is_focused());
    screen.press_key(Key::ArrowLeft);
    assert!(more.is_focused());
    assert!(screen.query_by_role(Role::MenuItem, "Email").is_none());
    // The destructive row is drawn in the destructive color, and its
    // highlight keeps that hue.
    let logout = menu_item(&screen, "Log out");
    let at = logout.rect().expect("laid out");
    let tint = |screen: &Screen| pixel(screen, at.x + at.width - 12.0, at.y + at.height / 2.0);
    assert!(
        distance(tint(&screen), 0xffffff) <= 2,
        "at rest the row is plain"
    );
    screen.press_key(Key::ArrowDown);
    assert!(logout.is_focused());
    assert!(
        distance(tint(&screen), 0xfce5e7) <= 4,
        "the focused row is tinted with destructive/10, not accent"
    );
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), ["logout"]);
    assert_clean(&screen);
}

/// The box a context menu is attached to in the tests.
pub(super) fn right_click_area() -> Node {
    column(text("Right click here"))
        .role(Role::Group)
        .name("Right click here")
        .focusable(true)
        .style(
            style()
                .width(150)
                .height(75)
                .align_items(Align::Center)
                .justify_content(kivact::style::Justify::Center)
                .border(1, basic::Color::Border),
        )
}

pub(super) fn edit_items() -> Vec<MenuEntry<&'static str>> {
    vec![
        MenuEntry::heading("Edit"),
        MenuEntry::item("undo", "Undo").shortcut("Ctrl+Z"),
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
                menu("Edit", edit_items(), move |key: &'static str| {
                    chosen.update(|chosen| chosen.push(key))
                })
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
    assert_clean(&screen);
}

pub(super) fn bar_menus() -> Vec<(String, Vec<MenuEntry<&'static str>>)> {
    vec![
        (
            "File".to_string(),
            vec![
                MenuEntry::item("new", "New Tab").shortcut("Ctrl+T"),
                MenuEntry::item("print", "Print"),
            ],
        ),
        (
            "Edit".to_string(),
            vec![
                MenuEntry::item("undo", "Undo"),
                MenuEntry::item("bars", "Bookmarks Bar")
                    .checked(true)
                    .keep_open(true),
            ],
        ),
        ("View".to_string(), vec![MenuEntry::item("zoom", "Zoom")]),
    ]
}

#[test]
fn menubar_moves_between_open_menus_with_the_arrow_keys() {
    let (screen, chosen) = kivact::test::render_with_handle(light(), || {
        let chosen = signal(Vec::new());
        (
            chosen,
            page(basic::menubar(bar_menus(), move |key: &'static str| {
                chosen.update(|chosen| chosen.push(key))
            })),
        )
    });
    screen.get_by_role(Role::MenuBar, "");
    let file = screen.get_by_role(Role::MenuItem, "File");
    let edit = screen.get_by_role(Role::MenuItem, "Edit");
    let view = screen.get_by_role(Role::MenuItem, "View");
    file.press_key(Key::ArrowDown);
    assert_eq!(file.is_expanded(), Some(true));
    assert!(menu_item(&screen, "New Tab").is_focused());
    // Right closes File and opens Edit on its first item.
    screen.press_key(Key::ArrowRight);
    assert_eq!(file.is_expanded(), Some(false));
    assert_eq!(edit.is_expanded(), Some(true));
    assert!(menu_item(&screen, "Undo").is_focused());
    assert!(screen.query_by_role(Role::MenuItem, "New Tab").is_none());
    screen.press_key(Key::ArrowRight);
    assert_eq!(view.is_expanded(), Some(true));
    screen.press_key(Key::ArrowRight);
    assert_eq!(file.is_expanded(), Some(true), "wraps around");
    screen.press_key(Key::ArrowLeft);
    assert_eq!(view.is_expanded(), Some(true), "and back");
    screen.press_key(Key::ArrowLeft);
    assert_eq!(edit.is_expanded(), Some(true));
    // The check row sits at the start of its row.
    let bars = screen.get_by_role(Role::MenuItemCheckBox, "Bookmarks Bar");
    assert_eq!(bars.is_checked(), Some(true));
    screen.press_key(Key::Escape);
    assert!(edit.is_focused(), "Escape returns to the trigger");
    // With the menus closed, the arrows move the focus along the bar.
    screen.press_key(Key::ArrowRight);
    assert!(view.is_focused());
    screen.press_key(Key::ArrowLeft);
    assert!(edit.is_focused());
    assert_clean(&screen);
    assert!(chosen.get_untracked().is_empty());
}

#[test]
fn menubar_opens_the_neighbour_when_the_pointer_rests_on_it() {
    let (screen, chosen) = kivact::test::render_with_handle(light(), || {
        let chosen = signal(Vec::new());
        (
            chosen,
            page(basic::menubar(bar_menus(), move |key: &'static str| {
                chosen.update(|chosen| chosen.push(key))
            })),
        )
    });
    let file = screen.get_by_role(Role::MenuItem, "File");
    let edit = screen.get_by_role(Role::MenuItem, "Edit");
    edit.hover();
    assert_eq!(edit.is_expanded(), Some(false), "nothing is open yet");
    file.click();
    assert_eq!(file.is_expanded(), Some(true));
    edit.hover();
    assert_eq!(file.is_expanded(), Some(false));
    assert_eq!(edit.is_expanded(), Some(true));
    menu_item(&screen, "Undo").hover();
    menu_item(&screen, "Undo").click();
    assert_eq!(chosen.get_untracked(), ["undo"]);
    assert_eq!(edit.is_expanded(), Some(false));
    assert_clean(&screen);
}

pub(super) fn nav_panel(
    links: &'static [&'static str],
    on_press: Signal<usize>,
) -> impl Fn() -> Node + 'static {
    move || {
        column(
            links
                .iter()
                .map(|title| {
                    basic::navigation_menu_link(*title)
                        .description("Re-usable components.")
                        .on_press(move || on_press.update(|n| *n += 1))
                })
                .collect::<Vec<_>>(),
        )
        .style(style().width(280))
    }
}

#[test]
fn navigation_menu_opens_panels_on_hover_and_pins_them_on_click() {
    let (screen, (links, docs)) = kivact::test::render_with_handle(light(), || {
        let links = signal(0usize);
        let docs = signal(0usize);
        (
            (links, docs),
            page(basic::navigation_menu(vec![
                basic::NavItem::menu(
                    "Getting started",
                    nav_panel(&["Introduction", "Installation"], links),
                ),
                basic::NavItem::menu("Components", nav_panel(&["Alert Dialog"], links)),
                basic::NavItem::link("Docs", move || docs.update(|n| *n += 1)),
            ])),
        )
    });
    screen.get_by_role(Role::Group, "Navigation");
    let started = screen.get_by_role(Role::Button, "Getting started");
    let components = screen.get_by_role(Role::Button, "Components");
    assert!(screen.query_by_role(Role::Link, "Introduction").is_none());
    started.hover();
    screen.get_by_role(Role::Link, "Introduction");
    assert_eq!(started.is_expanded(), Some(true));
    // One panel at a time.
    components.hover();
    assert!(screen.query_by_role(Role::Link, "Introduction").is_none());
    screen.get_by_role(Role::Link, "Alert Dialog");
    // Leaving closes a panel that only hover opened.
    screen.mouse_move(850.0, 650.0);
    assert!(screen.query_by_role(Role::Link, "Alert Dialog").is_none());
    assert_eq!(components.is_expanded(), Some(false));

    // A click pins it open: the pointer may leave; Escape closes it.
    started.click();
    screen.mouse_move(850.0, 650.0);
    screen.get_by_role(Role::Link, "Installation");
    started.focus();
    started.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Link, "Installation").is_none());

    // Keyboard: ArrowDown opens, a link press calls back and closes the panel.
    started.press_key(Key::ArrowDown);
    assert_eq!(started.is_expanded(), Some(true));
    screen.get_by_role(Role::Link, "Installation").click();
    assert_eq!(links.get_untracked(), 1);
    assert!(screen.query_by_role(Role::Link, "Installation").is_none());

    let docs_link = screen.get_by_role(Role::Link, "Docs");
    docs_link.click();
    docs_link.focus();
    docs_link.press_key(Key::Enter);
    assert_eq!(docs.get_untracked(), 2);
    // Arrows move along the row, from the last entry around to the first.
    docs_link.press_key(Key::ArrowRight);
    assert!(started.is_focused());
    started.press_key(Key::ArrowRight);
    assert!(components.is_focused());
    components.press_key(Key::ArrowLeft);
    assert!(started.is_focused());
    assert_clean(&screen);
}

/// The least of the three channels over the pixels of `rect`: how dark the
/// darkest stroke of the text inside it is.
fn darkest(screen: &Screen, rect: kivact::draw::Rect) -> u32 {
    let mut least = 0xff;
    for y in rect.y as u32..(rect.y + rect.height) as u32 {
        for x in rect.x as u32..(rect.x + rect.width) as u32 {
            let color = pixel(screen, x as f32, y as f32);
            least = least
                .min((color >> 16) & 0xff)
                .min((color >> 8) & 0xff)
                .min(color & 0xff);
        }
    }
    least
}

#[test]
fn a_hovered_row_is_highlighted_and_its_shortcut_follows_the_highlight() {
    let (screen, _) = mount_dropdown();
    screen.get_by_role(Role::Button, "Account").click();
    // The focus goes to the next row, so only a hover can light Profile.
    screen.press_key(Key::ArrowDown);
    screen.mouse_move(850.0, 650.0);
    let profile = menu_item(&screen, "Profile");
    assert!(!profile.is_focused());
    let row = profile.rect().expect("laid out");
    let shortcut = screen.get_by_text("Ctrl+P").rect().expect("laid out");
    let background = |screen: &Screen| pixel(screen, row.x + 4.0, row.y + row.height / 2.0);
    assert!(distance(background(&screen), 0xffffff) <= 2, "at rest");
    let calm = darkest(&screen, shortcut);
    assert!(calm > 0x60, "the shortcut is muted at rest: {calm:02x}");
    profile.hover();
    screen.run_until_idle();
    assert!(
        distance(background(&screen), 0xf5f5f5) <= 2,
        "a hover alone highlights the row: {:06x}",
        background(&screen)
    );
    let lit = darkest(&screen, shortcut);
    assert!(
        lit + 0x20 < calm,
        "the shortcut takes the accent foreground with the highlight: {calm:02x} -> {lit:02x}"
    );
    assert_clean(&screen);
}

#[test]
fn check_radio_and_submenu_rows_can_be_disabled_and_rows_can_be_inset() {
    let (screen, chosen) = kivact::test::render_with_handle(light(), || {
        let chosen = signal(Vec::new());
        (
            chosen,
            page(
                menu(
                    "Open",
                    vec![
                        MenuEntry::item("bars", "Status Bar")
                            .checked(true)
                            .keep_open(true)
                            .disabled(true),
                        MenuEntry::radio("top", "Top", false)
                            .keep_open(true)
                            .disabled(true),
                        MenuEntry::submenu("more", "More", vec![MenuEntry::item("a", "A")])
                            .disabled(true),
                        MenuEntry::item("plain", "Plain"),
                        MenuEntry::item("inset", "Inset").inset(true),
                        MenuEntry::item("calm", "Calm").destructive(false),
                        MenuEntry::heading("Heading"),
                        MenuEntry::heading("Inset heading").inset(true),
                    ],
                    move |key: &'static str| chosen.update(|chosen| chosen.push(key)),
                )
                .attach(open_button("Open")),
            ),
        )
    });
    screen.get_by_role(Role::Button, "Open").click();
    for (role, name) in [
        (Role::MenuItemCheckBox, "Status Bar"),
        (Role::MenuItemRadio, "Top"),
        (Role::MenuItem, "More"),
    ] {
        let row = screen.get_by_role(role, name);
        assert!(row.is_disabled(), "{name}");
        row.force_click();
    }
    assert!(chosen.get_untracked().is_empty());
    assert!(
        menu_item(&screen, "Plain").is_focused() || !menu_item(&screen, "More").is_focused(),
        "the keys skip the disabled rows"
    );
    let x = |name: &str| screen.get_by_text(name).rect().expect("laid out").x;
    assert_eq!(x("Inset") - x("Plain"), 20.0, "pl-7 against pl-2");
    assert_eq!(x("Calm"), x("Plain"));
    assert_eq!(
        x("Inset heading") - x("Heading"),
        20.0,
        "a heading is inset like a row (pl-7 against pl-2)"
    );
    assert_eq!(x("Heading"), x("Plain"));
    assert_clean(&screen);
}

#[test]
fn a_context_menu_without_a_named_target_is_called_by_its_label() {
    let (screen, ()) = kivact::test::render_with_handle(light(), || {
        (
            (),
            page(
                menu("Edit", edit_items(), |_key: &'static str| {})
                    .attach_context(column(text("Area")).focusable(true).test_id("area")),
            ),
        )
    });
    screen
        .get_by_test_id("area")
        .click_with(MouseButton::Secondary);
    screen.get_by_role(Role::Menu, "Edit");
    assert_clean(&screen);
}
