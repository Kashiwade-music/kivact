//! Demonstrate Neutral typography and icon sizes and colors.
use super::labelled;
use kivact::style::style;
use kivact::ui::{avatar, avatar_group, kbd};
use kivact::view::IntoView;
use kivact::view::{Node, column, row, text};
use kivact_ui_soft::{
    HeadingDisplay, IconColor, IconSize, TextColor, TextKind, icon_view, link, styled_text,
    thumbnail, visually_hidden,
};

// example: text
pub(super) fn example_text() -> Node {
    labelled(
        "Type roles",
        column((
            styled_text("Large text").kind(TextKind::Large),
            styled_text("Body text"),
            styled_text("Supporting text")
                .kind(TextKind::Supporting)
                .color(TextColor::Secondary),
            styled_text("let ready = true;").kind(TextKind::Code),
        ))
        .style(style().gap(8)),
    )
}
// end example

// example: heading
pub(super) fn example_heading() -> Node {
    labelled(
        "Heading scale",
        column((
            kivact::ui::heading("Display", 1).variant(HeadingDisplay::D1),
            kivact::ui::heading("Heading one", 1),
            kivact::ui::heading("Heading two", 2),
            kivact::ui::heading("Heading three", 3),
            kivact::ui::heading("Heading four", 4),
            kivact::ui::heading("Heading five", 5),
            kivact::ui::heading("Heading six", 6),
        ))
        .style(style().gap(8)),
    )
}
// end example

// example: icon
pub(super) fn example_icon() -> Node {
    labelled(
        "Sizes and semantic colors",
        row((
            icon_view("search").size(IconSize::Xs),
            icon_view("search").size(IconSize::Sm),
            icon_view("circle-check").color(IconColor::Success),
            icon_view("triangle-alert")
                .size(IconSize::Lg)
                .color(IconColor::Warning),
        ))
        .style(style().gap(16)),
    )
}
// end example

// example: kbd
pub(super) fn example_kbd() -> Node {
    labelled(
        "Keyboard shortcuts",
        row((kbd("mod+shift+p"), kbd("escape"), kbd("up+down"))).style(style().gap(12)),
    )
}
// end example

// example: empty-state
pub(super) fn example_empty_state() -> Node {
    labelled(
        "Empty project",
        column((
            kivact::ui::empty_state("No documents")
                .announce_changes(true)
                .icon(icon_view("search"))
                .actions(kivact::ui::button("New document")),
            text("Create a document to get started."),
        )),
    )
}
// end example

// example: avatar
// example: avatar-group
pub(super) fn example_avatar() -> Node {
    labelled(
        "Initials and grouped members",
        row((
            avatar("Avery Morgan"),
            avatar_group((avatar("Sam Rivera"), avatar("Taylor Chen"))).total(5usize),
        ))
        .style(style().gap(16)),
    )
}
// end example

// example: visually-hidden
pub(super) fn example_visually_hidden() -> Node {
    labelled(
        "Screen reader text",
        column((
            styled_text("The hint is available to assistive technology."),
            visually_hidden("Additional hint"),
        )),
    )
}
// end example

// example: link
pub(super) fn example_link() -> Node {
    labelled("External link", link("Reference").external(true))
}
// end example

// example: thumbnail
pub(super) fn example_thumbnail() -> Node {
    labelled(
        "Attachment states",
        row((
            thumbnail().label("Cover image"),
            thumbnail().label("Uploading image").loading(true),
        ))
        .style(style().gap(12)),
    )
}
// end example

// example: blockquote
pub(super) fn example_blockquote() -> Node {
    labelled(
        "Quotation and attribution",
        kivact_ui_soft::blockquote(text("A simple interface lets the content lead.").into_view())
            .cite("Design notes".to_owned()),
    )
}
// end example

// example: code
pub(super) fn example_code() -> Node {
    labelled("Inline code", kivact_ui_soft::code("cargo run"))
}
// end example

// example: code-block
pub(super) fn example_code_block() -> Node {
    labelled(
        "Rust example",
        kivact::ui::code_block("fn main() {\n    println!(\"Hello\");\n}")
            .language("rust")
            .highlight(kivact::ui::highlighter(
                || (),
                |_, line, _: &()| {
                    (
                        kivact_ui_soft::code_highlight(
                            &kivact_ui_soft::CodeLanguageInput::from("rust"),
                            line,
                        ),
                        (),
                    )
                },
            ))
            .title("Example source")
            .language_label(false)
            .collapsible(true)
            .collapsible_threshold(3),
    )
}
// end example

// example: markdown
pub(super) fn example_markdown() -> Node {
    labelled("Formatted content", kivact_ui_soft::markdown(
                "## Project notes\n\nUse **Soft** to style your interface.\n\nThe project reference lives in [the guide](https://example.com/guide) and wraps with the paragraph.\n\n- Review the catalog\n- Try the controls",
            )
            .content_width(Some(240.0)))
}
// end example

// example: citation
pub(super) fn example_citation() -> Node {
    labelled(
        "Source reference",
        kivact_ui_soft::citation(
            1u32,
            kivact_ui_soft::CitationSource {
                title: Some("Design notes".to_owned()),
                ..Default::default()
            },
        ),
    )
}
// end example

// example: timer
pub(super) fn example_timer() -> Node {
    labelled("Elapsed time", kivact_ui_soft::timer())
}
// end example

// example: timestamp
pub(super) fn example_timestamp() -> Node {
    labelled(
        "Timestamp",
        kivact_ui_soft::timestamp(kivact::state::signal(
            kivact_ui_soft::TimestampValue::unix_seconds(1_793_664_000),
        )),
    )
}
// end example
