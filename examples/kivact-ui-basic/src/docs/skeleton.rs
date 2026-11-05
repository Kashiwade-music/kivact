//! Role: Catalog page of Skeleton: an avatar row and a card placeholder.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "skeleton",
        name: "Skeleton",
        description: "内容を読み込むあいだ、形だけを示す仮の表示です。大きさは呼び出し側が style で決めます。動きを減らす設定では点滅しません。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("skeleton.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "avatar",
                "Avatar",
                "円形のアイコンと 2 行の文字の仮表示です。",
                avatar,
            ),
            Example::new(
                "card",
                "Card",
                "画像と 2 行の文字を持つカードの仮表示です。",
                card,
            ),
        ],
    }
}

// example: avatar
fn avatar() -> impl IntoView {
    row((
        kivact::ui::skeleton().style(style().size(40, 40).radius(20)),
        column((
            kivact::ui::skeleton().style(style().size(250, 16)),
            kivact::ui::skeleton().style(style().size(200, 16)),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(16).align_items(Align::Center))
}
// end example

// example: card
fn card() -> impl IntoView {
    column((
        kivact::ui::skeleton().style(style().size(250, 125)),
        column((
            kivact::ui::skeleton().style(style().size(250, 16)),
            kivact::ui::skeleton().style(style().size(200, 16)),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(12))
}
// end example
