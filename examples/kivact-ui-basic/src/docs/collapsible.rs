//! Role: Catalog page of Collapsible: a trigger that shows and hides a region.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::ControlSize;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "collapsible",
        name: "Collapsible",
        description: "トリガーで領域を開閉する、見た目を持たない部品です。collapsible(open).content(..).attach(trigger) と書き、attach に渡したボタンが開閉のボタン（expanded 状態を持つ）になり、内容は開いている間だけ作られます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;",
        source: include_str!("collapsible.rs"),
        align: PreviewAlign::Start,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "開閉の状態を渡し、content に内容を作る関数を、attach にトリガーを渡します。アイコン付きのボタンをトリガーにしています。",
                default,
            ),
            Example::new(
                "open",
                "Initially open",
                "signal の初期値を true にすると、開いた状態から始まります。状態は呼び出し側から読めます。",
                initially_open,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let open = signal(false);
    let repository = |name: &'static str| {
        text(name).style(
            style()
                .padding_x(16)
                .padding_y(8)
                .font_size(12)
                .border(1, basic::Color::Border),
        )
    };
    basic::collapsible(open)
        .content(move || {
            column((
                repository("@radix-ui/primitives"),
                repository("@radix-ui/colors"),
                repository("@stitches/react"),
            ))
            .style(style().gap(8).padding_top(8))
        })
        .attach(
            button("@peduarte starred 3 repositories")
                .variant(ButtonVariant::Outline)
                .icon(icon("chevrons-up-down"))
                .style(style().width(pct(100))),
        )
        .style(style().width(350))
}
// end example

// example: open
fn initially_open() -> impl IntoView {
    let open = signal(true);
    column((
        basic::muted(move || {
            if open.get() {
                "Details shown"
            } else {
                "Details hidden"
            }
            .to_string()
        }),
        basic::collapsible(open)
            .content(|| basic::paragraph("Order #4189 shipped on Friday and arrives on Tuesday."))
            .attach(
                button("Toggle details")
                    .variant(ButtonVariant::Outline)
                    .hide_label(true)
                    .icon(icon("chevrons-up-down"))
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small),
            ),
    ))
    .style(style().gap(8).align_items(Align::Start))
}
// end example
