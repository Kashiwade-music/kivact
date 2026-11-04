//! Role: Catalog page of EmptyState: icon media, outline and with actions.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "empty",
        name: "EmptyState",
        description: "表示する内容がないときの状態を伝える部品です。メディア、題名、説明、操作を中央に並べます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact_ui_basic as basic;",
        source: include_str!("empty.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "media_variant に Icon を渡すと、アイコンが 32 px の四角に収まります。",
                default,
            ),
            Example::new(
                "outline",
                "Outline",
                "枠線は style の border で足します。",
                outline,
            ),
            Example::new("actions", "Actions", "content に操作を並べます。", actions),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    basic::empty_state("No Projects Yet")
        .media(icon("folder"))
        .media_variant(basic::EmptyMediaVariant::Icon)
        .description("Get started by creating your first project.")
        .style(style().width(420))
}
// end example

// example: outline
fn outline() -> impl IntoView {
    basic::empty_state("No results")
        .media(icon("search"))
        .media_variant(basic::EmptyMediaVariant::Icon)
        .description("Try adjusting your search terms.")
        .style(style().border(1, basic::Color::Border))
        .style(style().width(420))
}
// end example

// example: actions
fn actions() -> impl IntoView {
    basic::empty_state("Inbox zero")
        .media(icon("inbox"))
        .media_variant(basic::EmptyMediaVariant::Icon)
        .description("You are all caught up.")
        .content({
            row((
                button("Compose"),
                button("Refresh").variant(ButtonVariant::Outline),
            ))
            .style(style().gap(8))
        })
        .style(style().width(420))
}
// end example
