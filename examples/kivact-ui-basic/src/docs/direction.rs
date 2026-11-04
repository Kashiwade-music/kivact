//! Role: Catalog page of Direction: provide a reading direction and read it back.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page {
        slug: "direction",
        name: "Direction",
        description: "部分木の読み方向（左から右、右から左）を伝える境界です。方向に対応した部品が、構築のときに読んで行の並びと矢印の向きを反転します。標準の kivact::ui の部品は反転しません。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("direction.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "provider",
                "Provider",
                "basic::direction に方向と子を作る関数を渡します。内側では use_direction が渡した方向を返します。",
                provider,
            ),
            Example::new(
                "toolbar",
                "Toolbar",
                "use_direction を読んで、行の並びを自分で反転する例です。",
                toolbar,
            ),
        ],
    }
}

// example: provider
fn provider() -> impl IntoView {
    let describe = || format!("{:?}", kivact::view::use_direction().get());
    column((
        kivact::view::provide_direction(kivact::view::ReadingDirection::Ltr, move || {
            text(format!("Left to right: {}", describe()))
        }),
        kivact::view::provide_direction(kivact::view::ReadingDirection::Rtl, move || {
            text(format!("Right to left: {}", describe()))
        }),
    ))
    .style(style().gap(8))
}
// end example

// example: toolbar
fn toolbar() -> impl IntoView {
    let actions = |label: &'static str| {
        let reversed = kivact::view::use_direction().get() == kivact::view::ReadingDirection::Rtl;
        let mut items = vec![icon("arrow-left"), text(label), icon("arrow-right")];
        if reversed {
            items.reverse();
        }
        row(items).style(style().gap(8).align_items(Align::Center))
    };
    column((
        kivact::view::provide_direction(kivact::view::ReadingDirection::Ltr, move || {
            actions("Previous, next")
        }),
        kivact::view::provide_direction(kivact::view::ReadingDirection::Rtl, move || {
            actions("Previous, next")
        }),
    ))
    .style(style().gap(16))
}
// end example
