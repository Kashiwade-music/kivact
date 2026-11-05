//! Role: Catalog page of Breadcrumb: basic trail, ellipsis and right-to-left.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page {
        slug: "breadcrumb",
        name: "Breadcrumb",
        description: "現在のページまでの階層を示す部品です。「Breadcrumb」という名前の Navigation として公開され、direction の中では向きが反転します。",
        usage: "use kivact::prelude::*;\nuse kivact::interaction::ListItem;",
        source: include_str!("breadcrumb.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "ListItem のキーと名前を渡し、on_activate でリンクのキーを受け取ります。",
                default,
            ),
            Example::new(
                "ellipsis",
                "Ellipsis",
                "item スロットで省略した階層を表します。",
                ellipsis,
            ),
            Example::new(
                "rtl",
                "Right to left",
                "provide_direction で Rtl を渡すと、並びと区切りの向きが反転します。",
                rtl,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let at = signal(String::from("Components"));
    kivact::ui::breadcrumb(move || {
        vec![
            ListItem::new("home", "Home"),
            ListItem::new("components", "Components"),
            ListItem::new("current", at.get()),
        ]
    })
    .on_activate(move |key| at.set(if key == "home" { "Home" } else { "Components" }.into()))
    .separator(|| icon("chevron-right").style(style().size(14, 14)))
}
// end example

// example: ellipsis
fn ellipsis() -> impl IntoView {
    kivact::ui::breadcrumb(vec![
        ListItem::new("home", "Home"),
        ListItem::new("more", "More").disabled(true),
        ListItem::new("components", "Components"),
        ListItem::new("current", "Breadcrumb"),
    ])
    .on_activate(|_| {})
    .item(|item| {
        if *item.key() == "more" {
            icon("ellipsis").style(style().size(20, 20)).into_view()
        } else {
            text(item.label()).into_view()
        }
    })
    .separator(|| icon("chevron-right").style(style().size(14, 14)))
}
// end example

// example: rtl
fn rtl() -> impl IntoView {
    kivact::view::provide_direction(kivact::view::ReadingDirection::Rtl, || {
        kivact::ui::breadcrumb(vec![
            ListItem::new("home", "Home"),
            ListItem::new("docs", "Docs"),
            ListItem::new("page", "Page"),
        ])
        .on_activate(|_| {})
        .separator(|| icon("chevron-left").style(style().size(14, 14)))
        .style(style().width(280))
    })
}
// end example
