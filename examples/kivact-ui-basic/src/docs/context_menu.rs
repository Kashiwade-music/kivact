//! Role: Catalog page of Context Menu: basic, submenu, checkboxes and a custom target.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::style::BorderStyle;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "context-menu",
        name: "Context Menu",
        description: "右クリックで開くメニューです。標準の kivact::ui::menu を attach_context で対象に取り付けると、このテーマでは context-menu.tsx の幅で描きます。右クリックはポインタの位置に、Shift+F10 とメニューキーは対象の位置に開き、閉じると元のフォーカスに戻ります。項目の種類と操作は Dropdown Menu と同じです。",
        usage: "use kivact::prelude::*;\nuse kivact::style::{BorderStyle, current_theme};\nuse kivact_ui_basic as basic;",
        source: include_str!("context_menu.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "標準の menu を作り、対象の Node を attach_context に渡します。対象はキー操作が届くように focusable にして role を付けます。枠にフォーカスして Shift+F10 を押しても開きます。",
                basic,
            ),
            Example::new(
                "submenu",
                "Submenu",
                "MenuEntry::submenu で入れ子にします。右矢印で子のメニューへ入ります。",
                submenu,
            ),
            Example::new(
                "checkboxes",
                "Checkboxes and radios",
                "check と radio の状態はアプリが持ちます。選んだ key を受け取る関数で更新します。",
                checkboxes,
            ),
            Example::new(
                "target",
                "Custom target",
                "対象にはどんな Node でも渡せます。メニューの名前は、対象に付けた name になります。",
                target,
            ),
        ],
    }
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
                .border(1, basic::Color::Border)
                .border_style(BorderStyle::Dashed)
                .font_size(12),
        )
        .child(text("Right click here"));
    menu(
        "Context menu",
        vec![
            MenuEntry::item("back", "Back").shortcut("Alt+Left"),
            MenuEntry::item("forward", "Forward").disabled(true),
            MenuEntry::item("reload", "Reload").shortcut("Ctrl+R"),
        ],
        |_key: &'static str| {},
    )
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
                .border(1, basic::Color::Border)
                .border_style(BorderStyle::Dashed)
                .font_size(12),
        )
        .child(text("Right click here"));
    menu(
        "Context menu",
        vec![
            MenuEntry::item("copy", "Copy").icon("copy"),
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
            .icon("wrench"),
            MenuEntry::separator(),
            MenuEntry::item("delete", "Delete")
                .icon("trash-2")
                .destructive(true),
        ],
        |_key: &'static str| {},
    )
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
                .border(1, basic::Color::Border)
                .border_style(BorderStyle::Dashed)
                .font_size(12),
        )
        .child(text("Right click here"));
    let bookmarks = signal(true);
    let person = signal("pedro");
    let items = move || {
        vec![
            MenuEntry::item("bookmarks", "Show Bookmarks")
                .checked(bookmarks.get())
                .keep_open(true),
            MenuEntry::separator(),
            MenuEntry::heading("People"),
            MenuEntry::radio("pedro", "Pedro Duarte", person.get() == "pedro").keep_open(true),
            MenuEntry::radio("colm", "Colm Tuite", person.get() == "colm").keep_open(true),
        ]
    };
    menu("Context menu", items, move |key: &'static str| match key {
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
                .background(basic::Color::Muted)
                .font_size(12),
        )
        .child(text("Right click the notes"));
    menu(
        "Context menu",
        vec![
            MenuEntry::item("note", "New note").icon("pencil"),
            MenuEntry::item("archive", "Archive").icon("archive"),
        ],
        |_key: &'static str| {},
    )
    .attach_context(area)
}
// end example
