//! Catalog page of Context Menu: basic, submenu, checkboxes and a custom target.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::style::BorderStyle;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("context-menu", "Context Menu", "Components", include_str!("context_menu.rs"))
        .description("A menu that opens on a right click. Attach the standard kivact::ui::menu to a target with attach_context; this theme draws it at the width of context-menu.tsx. A right click opens it at the pointer, and Shift+F10 or the menu key opens it at the target. Closing returns focus to where it was. The kinds of items and the actions are the same as in Dropdown Menu.")
        .usage("use kivact::prelude::*;\nuse kivact::style::{BorderStyle, use_theme};\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Make a standard menu and pass the target Node to attach_context. Make the target focusable with a role so that keys reach it. Focusing the frame and pressing Shift+F10 opens it too.",
            basic,
        ))
        .example(Example::new(
            "submenu",
            "Submenu",
            "MenuEntry::submenu nests a menu. The right arrow enters the child menu.",
            submenu,
        ))
        .example(Example::new(
            "checkboxes",
            "Checkboxes and radios",
            "The app holds the state of check and radio items. Update it in the function that receives the chosen key.",
            checkboxes,
        ))
        .example(Example::new(
            "target",
            "Custom target",
            "Any Node can be the target. The name of the menu is the name given to the target.",
            target,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let area = node()
        .role(Role::Group)
        .name("Right click here")
        .focusable(true)
        .style(
            style()
                .width(150)
                .height(75)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .border(1, basic::BORDER)
                .border_style(BorderStyle::Dashed)
                .font_size(12),
        )
        .child(text("Right click here"));
    menu(
        "Context menu",
        vec![
            MenuEntry::item("back", "Back").with_shortcut("Alt+Left"),
            MenuEntry::item("forward", "Forward").disabled(true),
            MenuEntry::item("reload", "Reload").with_shortcut("Ctrl+R"),
        ],
    )
    .on_activate(|_key: &'static str| {})
    .attach_context(area)
}
// end example

// example: submenu
fn submenu() -> impl IntoView {
    let area = node()
        .role(Role::Group)
        .name("Right click here")
        .focusable(true)
        .style(
            style()
                .width(150)
                .height(75)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .border(1, basic::BORDER)
                .border_style(BorderStyle::Dashed)
                .font_size(12),
        )
        .child(text("Right click here"));
    menu(
        "Context menu",
        vec![
            MenuEntry::item("copy", "Copy").with_icon("copy"),
            MenuEntry::submenu(
                "more",
                "More Tools",
                vec![
                    MenuEntry::item("save", "Save Page As..."),
                    MenuEntry::item("window", "Create Shortcut..."),
                    MenuEntry::separator(),
                    MenuEntry::item("devtools", "Developer Tools"),
                ],
            )
            .with_icon("wrench"),
            MenuEntry::separator(),
            MenuEntry::item("delete", "Delete")
                .with_icon("trash-2")
                .destructive(true),
        ],
    )
    .on_activate(|_key: &'static str| {})
    .attach_context(area)
}
// end example

// example: checkboxes
fn checkboxes() -> impl IntoView {
    let area = node()
        .role(Role::Group)
        .name("Right click here")
        .focusable(true)
        .style(
            style()
                .width(150)
                .height(75)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .border(1, basic::BORDER)
                .border_style(BorderStyle::Dashed)
                .font_size(12),
        )
        .child(text("Right click here"));
    let bookmarks = signal(true);
    let person = signal("pedro");
    let items = move || {
        vec![
            MenuEntry::checkbox("bookmarks", "Show Bookmarks", bookmarks.get()).keep_open(true),
            MenuEntry::separator(),
            MenuEntry::heading("People"),
            MenuEntry::radio("pedro", "Pedro Duarte", person.get() == "pedro").keep_open(true),
            MenuEntry::radio("colm", "Colm Tuite", person.get() == "colm").keep_open(true),
        ]
    };
    menu("Context menu", items)
        .on_activate(move |key: &'static str| match key {
            "bookmarks" => bookmarks.update(|on| *on = !*on),
            other => person.set(other),
        })
        .attach_context(area)
}
// end example

// example: target
fn target() -> impl IntoView {
    let area = node()
        .role(Role::Group)
        .name("Notes")
        .focusable(true)
        .style(
            style()
                .width(260)
                .height(90)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .background(basic::MUTED)
                .font_size(12),
        )
        .child(text("Right click the notes"));
    menu(
        "Context menu",
        vec![
            MenuEntry::item("note", "New note").with_icon("pencil"),
            MenuEntry::item("archive", "Archive").with_icon("archive"),
        ],
    )
    .on_activate(|_key: &'static str| {})
    .attach_context(area)
}
// end example
