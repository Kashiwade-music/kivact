//! Role: Catalog page of Attachment: default, sizes, states and vertical.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "attachment",
        name: "Attachment",
        description: "添付ファイルの名前、状態、操作を一行にまとめて表示する部品です。on_press で全体を押せるようにでき、操作は actions に置きます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("attachment.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "media、description、actions を持つ基本の形です。",
                default,
            ),
            Example::new(
                "sizes",
                "Sizes",
                "AttachmentSize の Default、Sm、Xs です。",
                sizes,
            ),
            Example::new(
                "states",
                "States",
                "AttachmentState の Idle（破線）、Uploading（スピナー）、Error（警告色）です。",
                states,
            ),
            Example::new(
                "vertical",
                "Vertical",
                "orientation に Orientation::Vertical を渡すと、メディアが上に来ます。",
                vertical,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    basic::attachment("report.pdf")
        .media(icon("file-text"))
        .description("2.4 MB")
        .actions(basic::attachment_action("x", "Remove report"))
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    column((
        basic::attachment("Default size").media(icon("file")),
        basic::attachment("Small size")
            .size(basic::AttachmentSize::Sm)
            .media(icon("file")),
        basic::attachment("Extra small")
            .size(basic::AttachmentSize::Xs)
            .media(icon("file")),
    ))
    .style(style().gap(12).align_items(Align::Start))
}
// end example

// example: states
fn states() -> impl IntoView {
    column((
        basic::attachment("Drop a file")
            .media(icon("paperclip"))
            .state(basic::AttachmentState::Idle),
        basic::attachment("upload.png")
            .media(icon("image"))
            .state(basic::AttachmentState::Uploading),
        basic::attachment("broken.zip")
            .media(icon("archive"))
            .description("Upload failed")
            .state(basic::AttachmentState::Error),
    ))
    .style(style().gap(12).align_items(Align::Start))
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    basic::attachment_group((
        basic::attachment("design.fig")
            .orientation(Orientation::Vertical)
            .media(icon("file"))
            .description("1.1 MB")
            .on_press(|| {}),
        basic::attachment("notes.txt")
            .orientation(Orientation::Vertical)
            .media(icon("file-text"))
            .description("4 KB")
            .on_press(|| {}),
    ))
    .style(style().width(420))
}
// end example
