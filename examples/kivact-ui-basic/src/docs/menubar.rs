//! Catalog page of Menubar: basic menus, checks and radios, and a submenu.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("menubar", "Menubar", "Components", include_str!("menubar.rs"))
        .description("The menu bar of an application. The left and right arrow keys move to the next menu and it stays open as you move. Once one menu is open, merely pointing at another heading opens that one.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass the pairs of heading and list of items side by side. The kinds of items are the same as in Dropdown Menu.",
            basic,
        ))
        .example(Example::new(
            "checkboxes",
            "Checkboxes and radios",
            "The app holds the state of check and radio items and reflects it in the function that returns the list.",
            checkboxes,
        ))
        .example(Example::new(
            "submenu",
            "Submenu",
            "MenuEntry::submenu nests a menu. From the last item, the right arrow moves to the next menu.",
            submenu,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let last = signal("Nothing chosen yet.".to_string());
    column((
        basic::menubar(vec![
            (
                "File".to_string(),
                vec![
                    MenuEntry::item("tab", "New Tab").with_shortcut("Ctrl+T"),
                    MenuEntry::item("window", "New Window").with_shortcut("Ctrl+N"),
                    MenuEntry::separator(),
                    MenuEntry::item("print", "Print...").with_shortcut("Ctrl+P"),
                ],
            ),
            (
                "Edit".to_string(),
                vec![
                    MenuEntry::item("undo", "Undo").with_shortcut("Ctrl+Z"),
                    MenuEntry::item("redo", "Redo").with_shortcut("Ctrl+Y"),
                ],
            ),
        ])
        .on_activate(move |key: &'static str| last.set(format!("Chosen: {key}")))
        .style(style().width(240)),
        basic::muted(last),
    ))
    .style(style().gap(12).align_items(Align::Start))
}
// end example

// example: checkboxes
fn checkboxes() -> impl IntoView {
    let bookmarks = signal(true);
    let profile = signal("benoit");
    let menus = move || {
        vec![(
            "View".to_string(),
            vec![
                MenuEntry::checkbox("bookmarks", "Always Show Bookmarks Bar", bookmarks.get())
                    .keep_open(true),
                MenuEntry::separator(),
                MenuEntry::heading("Profiles"),
                MenuEntry::radio("andy", "Andy", profile.get() == "andy").keep_open(true),
                MenuEntry::radio("benoit", "Benoit", profile.get() == "benoit").keep_open(true),
            ],
        )]
    };
    basic::menubar(menus)
        .on_activate(move |key: &'static str| match key {
            "bookmarks" => bookmarks.update(|on| *on = !*on),
            other => profile.set(other),
        })
        .style(style().width(240))
}
// end example

// example: submenu
fn submenu() -> impl IntoView {
    basic::menubar(vec![
        (
            "File".to_string(),
            vec![
                MenuEntry::submenu(
                    "share",
                    "Share",
                    vec![
                        MenuEntry::item("email", "Email link"),
                        MenuEntry::item("messages", "Messages"),
                    ],
                ),
                MenuEntry::separator(),
                MenuEntry::item("print", "Print..."),
            ],
        ),
        ("Help".to_string(), vec![MenuEntry::item("about", "About")]),
    ])
    .on_activate(|_key: &'static str| {})
    .style(style().width(240))
}
// end example
