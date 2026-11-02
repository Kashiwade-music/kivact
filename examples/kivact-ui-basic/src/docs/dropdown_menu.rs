//! Role: Catalog page of Dropdown Menu: items, checkboxes, radio group and submenu.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::view::{OverlayAlign, Placement};
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "dropdown-menu",
        name: "Dropdown Menu",
        description: "ボタンから開く、操作の一覧メニューです。標準の kivact::ui::menu を Lyra の見た目で描きます。矢印キー、Home、End、先頭の文字の入力で項目を移動し、Enter か Space で選びます。見出しと区切りには移動しません。",
        usage: "use kivact::prelude::*;\nuse kivact::view::{OverlayAlign, Placement};",
        source: include_str!("dropdown_menu.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "MenuEntry の一覧と、選ばれた項目の key を受け取る関数を渡します。icon、shortcut、destructive(bool)、disabled(bool) は項目のメソッドで付けます。メニューは attach に渡したボタンから開き、at_least_trigger_width(true) でボタン以上の幅になります。",
                basic,
            ),
            Example::new(
                "checkboxes",
                "Checkboxes",
                "checked(bool) の項目は選ぶたびに受け取る関数が呼ばれるだけです。keep_open(true) でメニューを開いたままにします。チェックの状態はアプリが持ち、次の一覧の checked に反映します。",
                checkboxes,
            ),
            Example::new(
                "radio-group",
                "Radio Group",
                "MenuEntry::radio は Role::MenuItemRadio の項目で、選ばれている項目にチェックが付きます。チェックと同じく、keep_open(true) で選んでもメニューを閉じません。",
                radio_group,
            ),
            Example::new(
                "submenu",
                "Submenu",
                "MenuEntry::submenu に key と子の一覧を渡します。右矢印で開き、左矢印か Escape で閉じます。placement（kivact::view::Placement）と align（kivact::style::Align）で開く位置を選べます。",
                submenu,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let picked = signal("Nothing chosen yet.".to_string());
    let entries = vec![
        MenuEntry::heading("My Account"),
        MenuEntry::item("profile", "Profile")
            .icon("user")
            .shortcut("Ctrl+P"),
        MenuEntry::item("billing", "Billing").icon("credit-card"),
        MenuEntry::item("settings", "Settings")
            .icon("settings")
            .disabled(true),
        MenuEntry::separator(),
        MenuEntry::item("logout", "Log out")
            .icon("log-out")
            .destructive(true),
    ];
    column((
        menu("Open", entries, move |key: &'static str| {
            picked.set(format!("Chosen: {key}"))
        })
        .at_least_trigger_width(true)
        .attach(button("Open").variant(ButtonVariant::Outline)),
        basic::muted(picked),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: checkboxes
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
    menu("Appearance", entries, move |key: &'static str| match key {
        "status" => status.update(|on| *on = !*on),
        _ => activity.update(|on| *on = !*on),
    })
    .at_least_trigger_width(true)
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
    menu("Position", entries, move |key: &'static str| {
        position.set(key)
    })
    .at_least_trigger_width(true)
    .attach(button("Position").variant(ButtonVariant::Outline))
}
// end example

// example: submenu
fn submenu() -> impl IntoView {
    let entries = vec![
        MenuEntry::item("new", "New File")
            .icon("file")
            .shortcut("Ctrl+N"),
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
        .icon("share"),
        MenuEntry::separator(),
        MenuEntry::item("print", "Print"),
    ];
    menu("File", entries, |_key: &'static str| {})
        .placement(Placement::Below)
        .align(OverlayAlign::Start)
        .at_least_trigger_width(true)
        .attach(button("File").variant(ButtonVariant::Outline))
}
// end example
