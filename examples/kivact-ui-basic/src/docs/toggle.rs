//! Role: Catalog page of Toggle: default, outline, sizes, icons and disabled.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::{ControlSize, toggle};
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "toggle",
        name: "Toggle",
        description: "押した状態を保持する二状態のボタンです。kivact::ui::toggle を basic の theme で描きます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::{ControlSize, toggle};\nuse kivact_ui_basic as basic;",
        source: include_str!("toggle.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "名前と bool の binding を渡します。押した状態は binding が持ちます。",
                basic,
            ),
            Example::new(
                "outline",
                "Outline",
                "basic::ToggleVariant::Outline を variant に渡すと枠を付けます。",
                outline,
            ),
            Example::new(
                "sizes",
                "Sizes",
                "size の Small、Medium、Large は高さ 28、32、36 px です。",
                sizes,
            ),
            Example::new(
                "disabled",
                "Disabled",
                "disabled のトグルは薄く表示され、押せません。",
                disabled,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let on = signal(false);
    toggle("Bold", on).icon(icon("bold")).hide_label(true)
}
// end example

// example: outline
fn outline() -> impl IntoView {
    let on = signal(true);
    toggle("Italic", on)
        .variant(basic::ToggleVariant::Outline)
        .icon(icon("italic"))
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    row((
        toggle("Small", signal(false)).size(ControlSize::Small),
        toggle("Default", signal(false)),
        toggle("Large", signal(false)).size(ControlSize::Large),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    toggle("Underline", signal(false)).disabled(true)
}
// end example
