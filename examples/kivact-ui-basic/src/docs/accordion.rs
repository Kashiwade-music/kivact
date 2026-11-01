//! Role: Catalog page of Accordion: single, multiple and controlled open sections.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::{ControlSize, disclosure};
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "accordion",
        name: "Accordion",
        description: "縦に積んだ見出しを開閉して、内容を折りたたむ並びです。kivact::ui::disclosure を縦に並べ、開いている項目のキーを持つ signal を各項目の open の binding に結んで作ります。basic の theme は項目の下に罫線を引きます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::{ControlSize, disclosure};\nuse kivact_ui_basic as basic;",
        source: include_str!("accordion.rs"),
        align: PreviewAlign::Start,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "開いている項目は signal の Option で持ちます。1 つ開くと、開いていた項目は閉じます。",
                default,
            ),
            Example::new(
                "multiple",
                "Multiple",
                "開いている項目を Vec で持てば、複数の項目を同時に開いたままにできます。",
                multiple,
            ),
            Example::new(
                "controlled",
                "Controlled",
                "開閉の状態は呼び出し側の signal です。ボタンから、すべて開く・閉じるを操作できます。",
                controlled,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let open = signal(Some("product"));
    let section = move |key: &'static str, title: &'static str, body: &'static str| {
        let is_open = binding(
            move || open.get() == Some(key),
            move |on| open.set(on.then_some(key)),
        );
        disclosure(title, is_open, move || text(body)).animated(true)
    };
    column((
        section(
            "product",
            "Product Information",
            "Our flagship product combines cutting-edge technology with sleek design.",
        ),
        section(
            "shipping",
            "Shipping Details",
            "We offer worldwide shipping through trusted courier partners.",
        ),
        section(
            "returns",
            "Return Policy",
            "We stand behind our products with a 30-day return policy.",
        ),
    ))
    .style(style().width(448))
}
// end example

// example: multiple
fn multiple() -> impl IntoView {
    let open = signal(vec!["one", "two"]);
    let section = move |key: &'static str, title: &'static str, body: &'static str| {
        let is_open = binding(
            move || open.with(|keys| keys.contains(&key)),
            move |on| {
                open.update(|keys| {
                    keys.retain(|k| *k != key);
                    if on {
                        keys.push(key);
                    }
                })
            },
        );
        disclosure(title, is_open, move || text(body)).animated(true)
    };
    column((
        section(
            "one",
            "Is it accessible?",
            "Yes. It adheres to the WAI-ARIA design pattern.",
        ),
        section(
            "two",
            "Is it styled?",
            "Yes. It comes with the default Lyra styles.",
        ),
        section(
            "three",
            "Is it animated?",
            "Yes. It follows the reduced motion setting.",
        ),
    ))
    .style(style().width(448))
}
// end example

// example: controlled
fn controlled() -> impl IntoView {
    let open = signal(Vec::<&'static str>::new());
    let all = ["a", "b", "c"];
    let section = move |key: &'static str| {
        let is_open = binding(
            move || open.with(|keys| keys.contains(&key)),
            move |on| {
                open.update(|keys| {
                    keys.retain(|k| *k != key);
                    if on {
                        keys.push(key);
                    }
                })
            },
        );
        disclosure(format!("Section {key}"), is_open, move || {
            text(format!("The content of section {key}."))
        })
        .animated(true)
    };
    column((
        row((
            button("Open all")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .on_press(move || open.set(all.to_vec())),
            button("Close all")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .on_press(move || open.set(Vec::new())),
            basic::muted(move || format!("{} open", open.get().len())),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        column(all.map(section).into_iter().collect::<Vec<_>>()).style(style().width(448)),
    ))
    .style(style().gap(12))
}
// end example
