//! Role: Catalog page of Carousel: default, several per view, vertical and loop.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "carousel",
        name: "Carousel",
        description: "スライドを 1 枚ずつ（または複数枚ずつ）切り替える部品です。表示中の位置は束縛で渡し、前へ・次へのボタンと左右の矢印キー（縦は上下）で動きます。ボタンは枠の外 48 px にあります。",
        usage: "use kivact::prelude::*;\nuse kivact::style::use_theme;\nuse kivact_ui_basic as basic;",
        source: include_str!("carousel.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "スライドの view の並びと、位置の束縛を渡します。端ではボタンが無効になります。枠に Tab でフォーカスして矢印キーでも動かせます。",
                default,
            ),
            Example::new(
                "per-view",
                "Items per view",
                "items_per_view で同時に見せる枚数を指定します。最後の位置は、まだ全部の枚数が見える所です。",
                per_view,
            ),
            Example::new(
                "vertical",
                "Vertical",
                "orientation に Orientation::Vertical を渡すと縦に並び、ボタンは上下に付きます。高さを指定してください。",
                vertical,
            ),
            Example::new(
                "loop",
                "Loop",
                "looping(true) で、最後から最初へ、最初から最後へ戻ります。",
                looped,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    /// A bordered slide with a number.
    fn slide(number: usize) -> Node {
        node()
            .style(
                style()
                    .height(120)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border(1, basic::Color::Border)
                    .font_size(30)
                    .weight(600),
            )
            .child(text(number.to_string()))
    }
    let index = signal(0usize);
    basic::carousel((1..=5).map(slide).collect::<Vec<_>>(), index)
        .style(style().width(240).margin_x(48))
}
// end example

// example: per-view
fn per_view() -> impl IntoView {
    /// A bordered slide with a number.
    fn slide(number: usize) -> Node {
        node()
            .style(
                style()
                    .height(120)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border(1, basic::Color::Border)
                    .font_size(30)
                    .weight(600),
            )
            .child(text(number.to_string()))
    }
    let index = signal(0usize);
    basic::carousel((1..=6).map(slide).collect::<Vec<_>>(), index)
        .items_per_view(2.0)
        .style(style().width(360).margin_x(48))
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    /// A bordered slide with a number.
    fn slide(number: usize) -> Node {
        node()
            .style(
                style()
                    .height(120)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border(1, basic::Color::Border)
                    .font_size(30)
                    .weight(600),
            )
            .child(text(number.to_string()))
    }
    let index = signal(0usize);
    basic::carousel((1..=5).map(slide).collect::<Vec<_>>(), index)
        .orientation(Orientation::Vertical)
        .style(style().width(240).height(120).margin_y(48))
}
// end example

// example: loop
fn looped() -> impl IntoView {
    /// A bordered slide with a number.
    fn slide(number: usize) -> Node {
        node()
            .style(
                style()
                    .height(120)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border(1, basic::Color::Border)
                    .font_size(30)
                    .weight(600),
            )
            .child(text(number.to_string()))
    }
    let index = signal(0usize);
    basic::carousel((1..=3).map(slide).collect::<Vec<_>>(), index)
        .looping()
        .style(style().width(240).margin_x(48))
}
// end example
