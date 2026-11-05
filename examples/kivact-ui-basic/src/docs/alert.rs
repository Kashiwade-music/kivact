//! Role: Catalog page of Alert: default, destructive and with an action.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::ControlSize;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "alert",
        name: "Alert",
        description: "ユーザーの注意を引くための短いメッセージを表示する部品です。Role::Alert として公開され、アイコン、説明、操作を添えられます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;",
        source: include_str!("alert.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "alert にタイトルを渡し、icon と description を添えます。",
                default,
            ),
            Example::new(
                "destructive",
                "Destructive",
                "variant に Destructive を渡すと、文字とアイコンが警告色になります。",
                destructive,
            ),
            Example::new(
                "action",
                "Action",
                "action で右端に操作を置けます。操作は alert の高さの中央にそろいます（shadcn は右上）。",
                with_action,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    kivact::ui::alert("Heads up!")
        .icon(icon("info"))
        .description("You can add components to your app using the CLI.")
        .style(style().width(420))
}
// end example

// example: destructive
fn destructive() -> impl IntoView {
    kivact::ui::alert("Unable to process your payment.")
        .variant(basic::AlertVariant::Destructive)
        .icon(icon("circle-alert"))
        .description("Please verify your billing information and try again.")
        .style(style().width(420))
}
// end example

// example: action
fn with_action() -> impl IntoView {
    kivact::ui::alert("Update available")
        .icon(icon("download"))
        .description("A new version is ready to install.")
        .actions({
            button("Undo")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
        })
        .style(style().width(420))
}
// end example
