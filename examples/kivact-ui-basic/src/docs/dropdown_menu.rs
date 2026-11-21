//! Catalog page of Dropdown Menu: items, checkboxes, radio group and submenu.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::OverlayAlign;
use kivact::view::Placement;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("dropdown-menu", "Dropdown Menu", "Components", include_str!("dropdown_menu.rs"))
        .description("A menu of actions that opens from a button. It is the standard kivact::ui::menu in the Lyra look. The arrow keys, Home, End and typing the first letter move between items; Enter or Space chooses. The keys do not stop on headings or separators.")
        .usage("use kivact::prelude::*;\nuse kivact::view::{OverlayAlign, Placement};")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass a list of MenuEntry and a function that receives the key of the chosen item. icon, shortcut, destructive(bool) and disabled(bool) are methods of an item. The menu opens from the button passed to attach, and at_least_trigger_width(true) makes it at least as wide as the button.",
            basic,
        ))
        .example(Example::new(
            "checkboxes",
            "Checkboxes",
            "A checked(bool) item only calls the function that receives the choice each time. keep_open(true) keeps the menu open. The app holds the checked state and feeds it back in the checked of the next list.",
            checkboxes,
        ))
        .example(Example::new(
            "radio-group",
            "Radio Group",
            "MenuEntry::radio is a Role::MenuItemRadio item, and the chosen item shows a check. As with check items, keep_open(true) keeps the menu open after a choice.",
            radio_group,
        ))
        .example(Example::new(
            "submenu",
            "Submenu",
            "MenuEntry::submenu takes a key and a list of children. The right arrow opens it, and the left arrow or Escape closes it. placement (kivact::view::Placement) and align (kivact::style::Align) choose where it opens.",
            submenu,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let picked = signal("Nothing chosen yet.".to_string());
    let entries = vec![
        MenuEntry::heading("My Account"),
        MenuEntry::item("profile", "Profile")
            .with_icon("user")
            .with_shortcut("Ctrl+P"),
        MenuEntry::item("billing", "Billing").with_icon("credit-card"),
        MenuEntry::item("settings", "Settings")
            .with_icon("settings")
            .disabled(true),
        MenuEntry::separator(),
        MenuEntry::item("logout", "Log out")
            .with_icon("log-out")
            .destructive(true),
    ];
    column((
        menu("Open", entries)
            .on_activate(move |key: &'static str| picked.set(format!("Chosen: {key}")))
            .at_least_anchor_width()
            .attach(button("Open").variant(ButtonVariant::Outline)),
        basic::muted(picked),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: checkboxes
// xtask-allow(state): legacy: status, activity; the Status Bar checkbox, flipped only by its own menu entry; the Activity Bar entry never touches it, so both, either or neither
fn checkboxes() -> impl IntoView {
    let status = signal(true);
    let activity = signal(false);
    let entries = move || {
        vec![
            MenuEntry::heading("Appearance"),
            MenuEntry::separator(),
            MenuEntry::checkbox("status", "Status Bar", status.get()).keep_open(true),
            MenuEntry::checkbox("activity", "Activity Bar", activity.get()).keep_open(true),
        ]
    };
    menu("Appearance", entries)
        .on_activate(move |key: &'static str| match key {
            "status" => status.update(|on| *on = !*on),
            _ => activity.update(|on| *on = !*on),
        })
        .at_least_anchor_width()
        .attach(button("Appearance").variant(ButtonVariant::Outline))
}
// end example

// example: radio-group
fn radio_group() -> impl IntoView {
    let position = signal("bottom");
    let entries = move || {
        let radio =
            |key, label| MenuEntry::radio(key, label, position.get() == key).keep_open(true);
        vec![
            MenuEntry::heading("Panel Position"),
            MenuEntry::separator(),
            radio("top", "Top"),
            radio("bottom", "Bottom"),
            radio("right", "Right"),
        ]
    };
    menu("Position", entries)
        .on_activate(move |key: &'static str| position.set(key))
        .at_least_anchor_width()
        .attach(button("Position").variant(ButtonVariant::Outline))
}
// end example

// example: submenu
fn submenu() -> impl IntoView {
    let entries = vec![
        MenuEntry::item("new", "New File")
            .with_icon("file")
            .with_shortcut("Ctrl+N"),
        MenuEntry::submenu(
            "share",
            "Share",
            vec![
                MenuEntry::item("email", "Email link"),
                MenuEntry::item("message", "Message"),
                MenuEntry::separator(),
                MenuEntry::item("copy", "Copy link"),
            ],
        )
        .with_icon("share"),
        MenuEntry::separator(),
        MenuEntry::item("print", "Print"),
    ];
    menu("File", entries)
        .on_activate(|_key: &'static str| {})
        .placement(Placement::Below)
        .align(OverlayAlign::Start)
        .at_least_anchor_width()
        .attach(button("File").variant(ButtonVariant::Outline))
}
// end example
