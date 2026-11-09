//! Role: Catalog page of Progress: static, label with value and interactive.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::{ControlSize, progress};
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "progress",
        name: "Progress",
        description: "処理の進み具合を示すバーです。kivact::ui::progress に 0 から 1 の値を Some で渡し、範囲外は丸められます。None は不定の進み方で、動く区間を描きます。Role::ProgressBar として公開されます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::{ControlSize, progress};\nuse kivact_ui_basic as basic;",
        source: include_str!("progress.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "値を Some で渡すだけのバーです。名前は name で付けます。",
                default,
            ),
            Example::new(
                "label",
                "Label",
                "見出しと割合は呼び出し側の text で添えます。",
                label,
            ),
            Example::new(
                "interactive",
                "Interactive",
                "値はシグナルに追従します。ボタンで増減させます。",
                interactive,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    progress(Some(0.33))
        .name("Progress")
        .style(style().width(420))
}
// end example

// example: label
fn label() -> impl IntoView {
    column((
        row((text("Upload progress"), spacer(), basic::muted("66%"))),
        progress(Some(0.66)).name("Upload progress"),
    ))
    .style(style().gap(12).width(420))
}
// end example

// example: interactive
fn interactive() -> impl IntoView {
    let value = signal(0.4);
    column((
        row((
            text("Export"),
            spacer(),
            basic::muted(move || format!("{:.0}%", value.get() * 100.0)),
        )),
        progress(move || Some(value.get())).name("Export"),
        row((
            button("-10")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .on_press(move || value.update(|v| *v = (*v - 0.1).max(0.0))),
            button("+10")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .on_press(move || value.update(|v| *v = (*v + 0.1).min(1.0))),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(16).width(420))
}
// end example
