//! Role: Catalog page of Popover: the dialog at a trigger, its placement and alignment.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::view::OverlayAlign;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "popover",
        name: "Popover",
        description: "トリガーの近くに重ねて表示する、モーダルではないパネルです。標準の kivact::ui::dialog を DialogMode::Popover にして attach で取り付けると、このテーマでは popover.tsx の見た目で描きます。Escape、外側の押下、フォーカスが離れたときに閉じ、閉じるとフォーカスはトリガーへ戻ります。attach に渡したボタンが開閉のボタンになります。",
        usage: "use kivact::prelude::*;\nuse kivact::interaction::DialogMode;\nuse kivact::view::{OverlayAlign, Placement};\nuse kivact_ui_basic as basic;",
        source: include_str!("popover.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "open の signal と題名を渡し、description と content で中身を作り、attach でトリガーに取り付けます。開くと最初のフォーカスできる要素へフォーカスが移ります。",
                default,
            ),
            Example::new(
                "placement",
                "Placement",
                "placement（kivact::view::Placement）で開く側を選びます。入りきらない場合は反対側へ回ります。題名を見せないときは hide_title(true) にし、題名は支援技術向けの名前だけに使います。",
                placement,
            ),
            Example::new(
                "align",
                "Align",
                "align（kivact::view::OverlayAlign）でトリガーの始まり、中央、終わりのどれに揃えるかを選びます。既定は始まりです。",
                align,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let open = signal(false);
    let width = signal(String::from("100%"));
    let height = signal(String::from("25px"));
    let field = |label: &'static str, value: Signal<String>| {
        row((
            basic::label(label).style(style().width(80)),
            text_input(label, value).style(style().grow(1)),
        ))
        .style(style().gap(8).align_items(Align::Center))
    };
    dialog("Dimensions", open)
        .mode(DialogMode::Popover)
        .description("Set the dimensions for the layer.")
        .content(move || {
            column((field("Width", width), field("Height", height)))
                .style(style().gap(8).width(pct(100)))
        })
        .attach(button("Open popover").variant(ButtonVariant::Outline))
}
// end example

// example: placement
fn placement() -> impl IntoView {
    let sides = [
        ("Top", Placement::Above),
        ("Right", Placement::End),
        ("Bottom", Placement::Below),
        ("Left", Placement::Start),
    ];
    row(Vec::from(sides.map(|(label, placement)| {
        let open = signal(false);
        dialog(format!("{label} popover"), open)
            .mode(DialogMode::Popover)
            .hide_title(true)
            .placement(placement)
            .content(move || text(format!("Opens on the {label} side.")))
            .attach(button(label).variant(ButtonVariant::Outline))
    })))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: align
fn align() -> impl IntoView {
    let aligns = [
        ("Start", OverlayAlign::Start),
        ("Center", OverlayAlign::Center),
        ("End", OverlayAlign::End),
    ];
    row(Vec::from(aligns.map(|(label, align)| {
        let open = signal(false);
        dialog(label, open)
            .mode(DialogMode::Popover)
            .align(align)
            .description("Lined up with the trigger.")
            .attach(button(label).variant(ButtonVariant::Outline))
    })))
    .style(style().gap(8).align_items(Align::Center))
}
// end example
