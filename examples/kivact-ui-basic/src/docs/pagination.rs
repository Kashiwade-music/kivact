//! Role: Catalog page of Pagination: pages helper, manual links, labels and long lists.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

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
                "Manual",
                "pagination_link、pagination_previous、pagination_next、pagination_ellipsis を自分で並べることもできます。",
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
        basic::pagination(page, 5usize),
        text(move || format!("Page {} of 5", page.get() + 1)).style(style().font_size(12)),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: manual
fn manual() -> impl IntoView {
    basic::pagination_content((
        basic::pagination_previous(),
        basic::pagination_link("1"),
        basic::pagination_link("2").active(true),
        basic::pagination_link("3"),
        basic::pagination_ellipsis(),
        basic::pagination_next(),
    ))
}
// end example

// example: labels
fn labels() -> impl IntoView {
    let page = signal(1usize);
    basic::pagination(page, 5usize).label_format(|number| format!("p.{number}"))
}
// end example

// example: long
fn long() -> impl IntoView {
    let page = signal(41usize);
    basic::pagination(page, 100usize)
}
// end example
