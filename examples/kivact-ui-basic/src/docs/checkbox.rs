//! Role: Catalog page of Checkbox: basic, description, disabled and select all.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "checkbox",
        name: "Checkbox",
        description: "オンとオフを切り替えるチェックボックスです。標準の kivact::ui::checkbox に Lyra の見た目を当てたもので、状態は呼び出し側の signal が持ちます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("checkbox.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "ラベルと signal を渡すだけで使えます。クリックと Space で切り替わります。",
                basic,
            ),
            Example::new(
                "description",
                "With description",
                "ラベルの下に補足の文章を添える例です。",
                description,
            ),
            Example::new(
                "disabled",
                "Disabled",
                "disabled のチェックボックスは薄く表示され、操作できません。",
                disabled,
            ),
            Example::new(
                "invalid",
                "Invalid",
                "invalid を true にすると、枠が destructive の色になり、薄い ring が付きます。支援技術にも伝わります。",
                invalid,
            ),
            Example::new(
                "select-all",
                "Select all",
                "indeterminate で一部だけ選ばれた状態を表します。親の状態は子の signal から導きます。",
                select_all,
            ),
            Example::new(
                "field",
                "In a field",
                "basic::field_set に複数のチェックボックスを入れ、見出しと説明を付けます。",
                in_field,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let accepted = signal(false);
    column((
        checkbox("Accept terms and conditions", accepted),
        text(move || {
            if accepted.get() {
                "Accepted".to_string()
            } else {
                "Not accepted".to_string()
            }
        })
        .style(style().font_size(12)),
    ))
    .style(style().gap(12))
}
// end example

// example: description
fn description() -> impl IntoView {
    let marketing = signal(true);
    column((
        checkbox("Marketing emails", marketing),
        basic::muted("Receive product news and special offers.").style(style().padding_left(24)),
    ))
    .style(style().gap(4))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    column((
        checkbox("Enable notifications", signal(false)).disabled(true),
        checkbox("Sync across devices", signal(true)).disabled(true),
    ))
    .style(style().gap(12))
}
// end example

// example: invalid
fn invalid() -> impl IntoView {
    let accepted = signal(false);
    checkbox("Accept terms and conditions", accepted).invalid(move || !accepted.get())
}
// end example

// example: select-all
fn select_all() -> impl IntoView {
    let items = [signal(true), signal(false), signal(false)];
    let count = move || items.iter().filter(|item| item.get()).count();
    let all = binding(
        move || count() == items.len(),
        move |on: bool| items.iter().for_each(|item| item.set(on)),
    );
    column((
        checkbox("Select all", all).indeterminate(move || count() > 0 && count() < items.len()),
        column((
            checkbox("Desktop", items[0]),
            checkbox("Documents", items[1]),
            checkbox("Downloads", items[2]),
        ))
        .style(style().gap(12).padding_left(24)),
    ))
    .style(style().gap(12))
}
// end example

// example: field
fn in_field() -> impl IntoView {
    let (a, b) = (signal(true), signal(false));
    basic::field_set(
        column((checkbox("Hard disks", a), checkbox("External disks", b))).style(style().gap(12)),
    )
    .legend("Show on desktop")
    .description("Select the items to show on the desktop.")
}
// end example
