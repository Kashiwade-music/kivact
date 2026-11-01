//! Role: Catalog page of Hover Card: a preview that opens on hover.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::view::Placement;
use kivact_ui_basic as basic;
use std::time::Duration;

pub(crate) fn page() -> Page {
    Page {
        slug: "hover-card",
        name: "Hover Card",
        description: "リンクなどにポインターを重ねる（またはフォーカスする）と、遅れて詳細を表示するカードです。カードの上にポインターを移しても開いたままで、離れてから少し遅れて閉じます。Escape と外側の押下ですぐ閉じます。",
        usage: "use kivact::ui::Side;\nuse kivact::prelude::*;\nuse kivact_ui_basic as basic;\nuse std::time::Duration;",
        source: include_str!("hover_card.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "カードの中身を作る関数を渡し、attach でトリガーに取り付けます。既定では 600 ms 後に開き、離れて 300 ms 後に閉じます。",
                default,
            ),
            Example::new(
                "delay",
                "Delay and placement",
                "open_delay と close_delay で待ち時間を、placement（kivact::view::Placement）で開く側を選びます。",
                delay,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    basic::hover_card(|| {
        column((
            text("@nextjs").style(style().font_size(14).weight(600)),
            text("The React Framework – created and maintained by @vercel."),
            row((
                icon("calendar"),
                basic::muted("Joined December 2021").style(style().font_size(12)),
            ))
            .style(style().gap(8).align_items(Align::Center)),
        ))
        .style(style().gap(8))
    })
    .attach(button("@nextjs").variant(ButtonVariant::Link))
}
// end example

// example: delay
fn delay() -> impl IntoView {
    basic::hover_card(|| text("Opens after 100 ms and closes right away."))
        .open_delay(Duration::from_millis(100))
        .close_delay(Duration::ZERO)
        .placement(Placement::End)
        .attach(button("Hover me").variant(ButtonVariant::Outline))
}
// end example
