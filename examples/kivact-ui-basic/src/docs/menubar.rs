//! Role: Catalog page of Menubar: basic menus, checks and radios, and a submenu.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "menubar",
        name: "Menubar",
        description: "アプリのメニューバーです。左右の矢印キーで隣のメニューへ移り、メニューを開いたまま切り替わります。メニューを 1 つ開くと、ほかの見出しにポインタを置くだけで開きます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("menubar.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "見出しと項目の一覧の組を並べて渡します。項目の種類は Dropdown Menu と同じです。",
                basic,
            ),
            Example::new(
                "checkboxes",
                "Checkboxes and radios",
                "check と radio の状態はアプリが持ち、一覧を返す関数に反映します。",
                checkboxes,
            ),
            Example::new(
                "submenu",
                "Submenu",
                "MenuEntry::submenu で入れ子にします。最後の項目から右矢印を押すと隣のメニューへ移ります。",
                submenu,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let last = signal("Nothing chosen yet.".to_string());
    column((
        basic::menubar(
            vec![
                (
                    "File".to_string(),
                    vec![
                        MenuEntry::item("tab", "New Tab").shortcut("Ctrl+T"),
                        MenuEntry::item("window", "New Window").shortcut("Ctrl+N"),
                        MenuEntry::separator(),
                        MenuEntry::item("print", "Print...").shortcut("Ctrl+P"),
                    ],
                ),
                (
                    "Edit".to_string(),
                    vec![
                        MenuEntry::item("undo", "Undo").shortcut("Ctrl+Z"),
                        MenuEntry::item("redo", "Redo").shortcut("Ctrl+Y"),
                    ],
                ),
            ],
            move |key: &'static str| last.set(format!("Chosen: {key}")),
        )
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
                MenuEntry::item("bookmarks", "Always Show Bookmarks Bar")
                    .checked(bookmarks.get())
                    .keep_open(true),
                MenuEntry::separator(),
                MenuEntry::heading("Profiles"),
                MenuEntry::radio("andy", "Andy", profile.get() == "andy").keep_open(true),
                MenuEntry::radio("benoit", "Benoit", profile.get() == "benoit").keep_open(true),
            ],
        )]
    };
    basic::menubar(menus, move |key: &'static str| match key {
        "bookmarks" => bookmarks.update(|on| *on = !*on),
        other => profile.set(other),
    })
    .style(style().width(240))
}
// end example

// example: submenu
fn submenu() -> impl IntoView {
    basic::menubar(
        vec![
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
        ],
        |_key: &'static str| {},
    )
    .style(style().width(240))
}
// end example
