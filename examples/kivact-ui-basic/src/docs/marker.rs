//! Role: Catalog page of Marker: default, separator, border and icon.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "marker",
        name: "Marker",
        description: "文章の中に目印や区切りを入れる部品です。会話の日付の区切りなどに使います。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("marker.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "variants",
                "Variants",
                "MarkerVariant の Default、Separator、Border です。",
                variants,
            ),
            Example::new(
                "icon",
                "With icon",
                "icon で内容の前に 14 px のアイコンを添えます。",
                with_icon,
            ),
        ],
    }
}

// example: variants
fn variants() -> impl IntoView {
    column((
        basic::marker("Default"),
        basic::marker("Today").variant(basic::MarkerVariant::Separator),
        basic::marker("Border").variant(basic::MarkerVariant::Border),
    ))
    .style(style().gap(16).width(420))
}
// end example

// example: icon
fn with_icon() -> impl IntoView {
    column((
        basic::marker("Last edited 2 minutes ago").icon(icon("clock")),
        basic::marker("Pinned")
            .icon(icon("bookmark"))
            .variant(basic::MarkerVariant::Separator),
    ))
    .style(style().gap(16).width(420))
}
// end example
