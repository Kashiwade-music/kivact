//! Role: Catalog page of Command: the standard command palette in a dialog.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::{Command, command_palette};
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "command",
        name: "Command",
        description: "検索しながらコマンドや項目を選ぶメニューです。標準の command_palette をモーダルのダイアログに表示します。入力で項目を絞り込み、矢印キーで移動し、Enter かクリックで選びます。グループ、アイコン、ショートカットを持てます。",
        usage: "use kivact::prelude::*;
use kivact::ui::{Command, command_palette};
use kivact_ui_basic as basic;",
        source: include_str!("command.rs"),
        align: PreviewAlign::Start,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "Command の一覧と、選ばれたキーを受け取るコールバックを渡します。開くショートカットの割り当てはアプリの役目です。選ぶと閉じます。",
                default,
            ),
            Example::new(
                "simple",
                "Simple",
                "グループを使わない一覧です。placeholder と empty_text で文言を差し替えます。",
                simple,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let open = signal(false);
    let last = signal(String::from("Nothing chosen yet."));
    column((
        button("Open command menu")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        basic::muted(move || last.get()),
        command_palette(
            open,
            vec![
                Command::new("calendar", "Calendar")
                    .group("Suggestions")
                    .icon("calendar"),
                Command::new("emoji", "Search Emoji")
                    .group("Suggestions")
                    .icon("smile")
                    .keywords(["face"]),
                Command::new("profile", "Profile")
                    .group("Settings")
                    .icon("user")
                    .hint("Ctrl P"),
                Command::new("billing", "Billing")
                    .group("Settings")
                    .icon("credit-card")
                    .hint("Ctrl B"),
                Command::new("settings", "Settings")
                    .group("Settings")
                    .icon("settings")
                    .hint("Ctrl S"),
            ],
            move |key: &str| last.set(format!("Ran {key}.")),
        ),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: simple
fn simple() -> impl IntoView {
    let open = signal(false);
    column((
        button("Choose a theme")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        command_palette(
            open,
            vec![
                Command::new("light", "Light"),
                Command::new("dark", "Dark"),
                Command::new("system", "System"),
            ],
            |_: &str| {},
        )
        .placeholder("Choose a theme...")
        .empty_text("No theme found."),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example
