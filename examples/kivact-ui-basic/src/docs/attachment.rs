//! Catalog page of Attachment: default, sizes, states and vertical.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Orientation;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("attachment", "Attachment", "Components", include_str!("attachment.rs"))
        .description("Shows the name, the state and the actions of an attached file on one line. on_press makes the whole row pressable, and the actions go in actions.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "The basic form, with media, a description and actions.",
            default,
        ))
        .example(Example::new(
            "sizes",
            "Sizes",
            "The AttachmentSize values Default, Sm and Xs.",
            sizes,
        ))
        .example(Example::new(
            "states",
            "States",
            "The AttachmentState values Idle (dashed), Uploading (a spinner) and Error (the warning color).",
            states,
        ))
        .example(Example::new(
            "vertical",
            "Vertical",
            "Pass Orientation::Vertical to orientation to put the media on top.",
            vertical,
        ))
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
