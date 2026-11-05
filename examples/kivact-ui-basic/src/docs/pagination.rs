//! Role: Catalog page of Pagination: pages helper, manual links, labels and long lists.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page {
        slug: "pagination",
        name: "Pagination",
        description: "ページ番号で長い一覧を移動する部品です。pagination は 0 から数える現在ページの束縛と総ページ数から、前後の移動と省略記号を含む全体を作ります。ページ番号の書き方は label_format で変えられます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("pagination.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "pages",
                "Pages",
                "先頭、末尾、現在とその前後のページを表示し、端では前へ、次へが無効になります。",
                pages,
            ),
            Example::new(
                "manual",
                "Custom cell contents",
                "item スロットでセルの見た目を変更しても、移動と選択は同じページ束縛を使います。",
                manual,
            ),
            Example::new(
                "labels",
                "Page labels",
                "label_format は 1 から数えたページ番号を受け取り、リンクの文字と名前を返します。",
                labels,
            ),
            Example::new(
                "long",
                "Many pages",
                "100 ページでも、表示するのは現在の周辺だけです。",
                long,
            ),
        ],
    }
}

// example: pages
fn pages() -> impl IntoView {
    let page = signal(0usize);
    column((
        kivact::ui::pagination(5usize, page).item(|item| match item {
            PageItem::Previous => row((icon("chevron-left"), text("Previous"))).into_view(),
            PageItem::Next => row((text("Next"), icon("chevron-right"))).into_view(),
            PageItem::Page(n) => text((n + 1).to_string()).into_view(),
            _ => icon("ellipsis").into_view(),
        }),
        text(move || format!("Page {} of 5", page.get() + 1)).style(style().font_size(12)),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: manual
fn manual() -> impl IntoView {
    let page = signal(1usize);
    kivact::ui::pagination(10usize, page)
        .siblings(0usize)
        .item(|item| match item {
            PageItem::Previous => row((icon("chevron-left"), text("Previous"))).into_view(),
            PageItem::Next => row((text("Next"), icon("chevron-right"))).into_view(),
            PageItem::Page(n) => text((n + 1).to_string()).into_view(),
            _ => icon("ellipsis").into_view(),
        })
}
// end example

// example: labels
fn labels() -> impl IntoView {
    let page = signal(1usize);
    kivact::ui::pagination(5usize, page).label_format(|number| format!("p.{number}"))
}
// end example

// example: long
fn long() -> impl IntoView {
    let page = signal(41usize);
    kivact::ui::pagination(100usize, page).item(|item| match item {
        PageItem::Previous => row((icon("chevron-left"), text("Previous"))).into_view(),
        PageItem::Next => row((text("Next"), icon("chevron-right"))).into_view(),
        PageItem::Page(n) => text((n + 1).to_string()).into_view(),
        _ => icon("ellipsis").into_view(),
    })
}
// end example
