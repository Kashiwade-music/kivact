//! Role: Catalog page of Button: variants, sizes, icons, loading and disabled.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::task::sleep;
use kivact::ui::ControlSize;
use std::time::Duration;

pub(crate) fn page() -> Page {
    Page {
        slug: "button",
        name: "Button",
        description: "操作を実行するためのボタンです。標準の kivact::ui::button に Lyra の見た目を当てたもので、種類、大きさ、アイコン、読み込み中の状態を持ちます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ControlSize;",
        source: include_str!("button.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "variants",
                "Variants",
                "variant で見た目を選びます。Primary が shadcn の default、Danger が destructive に当たります。",
                variants,
            ),
            Example::new(
                "sizes",
                "Sizes",
                "ControlSize の ExtraSmall、Small、Medium、Large が xs、sm、default、lg の高さ（24、28、32、36 px）になります。",
                sizes,
            ),
            Example::new(
                "with-icon",
                "With icon",
                "icon と trailing_icon で Lucide のアイコンを添えます。アイコンの側の余白は 2 px 狭くなります。icon を付けて hide_label(true) にした button は、名前だけを持つ正方形のボタンになります。",
                with_icon,
            ),
            Example::new(
                "loading",
                "Loading",
                "loading が true のあいだはスピナーを表示し、押せなくなります。押すと 2 秒間だけ読み込み中になります。",
                loading,
            ),
            Example::new(
                "disabled",
                "Disabled",
                "disabled のボタンは薄く表示され、押せません。",
                disabled,
            ),
        ],
    }
}

// example: variants
fn variants() -> impl IntoView {
    row((
        button("Default"),
        button("Secondary").variant(ButtonVariant::Secondary),
        button("Outline").variant(ButtonVariant::Outline),
        button("Ghost").variant(ButtonVariant::Ghost),
        button("Destructive").variant(ButtonVariant::Danger),
        button("Link").variant(ButtonVariant::Link),
    ))
    .style(style().gap(8))
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    column((
        row((
            button("Extra small")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::ExtraSmall),
            button("Small")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small),
            button("Default").variant(ButtonVariant::Outline),
            button("Large")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Large),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        row((
            button("Add")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("plus"))
                .size(ControlSize::ExtraSmall),
            button("Add")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("plus"))
                .size(ControlSize::Small),
            button("Add")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("plus")),
            button("Add")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("plus"))
                .size(ControlSize::Large),
        ))
        .style(style().gap(8).align_items(Align::Center)),
    ))
    .style(style().gap(16).align_items(Align::Center))
}
// end example

// example: with-icon
fn with_icon() -> impl IntoView {
    row((
        button("Send")
            .variant(ButtonVariant::Outline)
            .icon(icon("mail")),
        button("Continue").trailing_icon(icon("arrow-right")),
        button("Delete")
            .hide_label(true)
            .icon(icon("trash-2"))
            .variant(ButtonVariant::Danger),
        button("Settings")
            .hide_label(true)
            .icon(icon("settings"))
            .variant(ButtonVariant::Ghost),
    ))
    .style(style().gap(8))
}
// end example

// example: loading
fn loading() -> impl IntoView {
    let busy = signal(false);
    row((
        button("Please wait")
            .variant(ButtonVariant::Outline)
            .loading(true),
        button("Save changes").loading(busy).on_press(move || {
            busy.set(true);
            spawn::<()>(async move {
                sleep(Duration::from_secs(2)).await;
                busy.set(false);
            });
        }),
    ))
    .style(style().gap(8))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    row((
        button("Default").disabled(true),
        button("Outline")
            .variant(ButtonVariant::Outline)
            .disabled(true),
        button("Secondary")
            .variant(ButtonVariant::Secondary)
            .disabled(true),
        button("Destructive")
            .variant(ButtonVariant::Danger)
            .disabled(true),
    ))
    .style(style().gap(8))
}
// end example
