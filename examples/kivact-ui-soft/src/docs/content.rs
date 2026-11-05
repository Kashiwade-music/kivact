//! Role: Demonstrate Neutral typography and icon sizes and colors.
use super::specimen::specimen;
use kivact::style::style;
use kivact::ui::{avatar, avatar_group, kbd};
use kivact::view::{Node, column, row};
use kivact_ui_soft::{
    HeadingDisplay, IconColor, IconSize, TextColor, TextKind, icon_view, link, styled_text,
    thumbnail, visually_hidden,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "text" => example_text(),
        "heading" => example_heading(),
        "icon" => example_icon(),
        "kbd" => example_kbd(),
        "empty-state" => example_empty_state(),
        "avatar" | "avatar-group" | "avatar-group-overflow" | "avatar-status-dot" => {
            example_avatar()
        }
        "visually-hidden" => example_visually_hidden(),
        "link" => example_link(),
        "thumbnail" => example_thumbnail(),
        "blockquote" => example_blockquote(),
        "code" => example_code(),
        "code-block" => example_code_block(),
        "markdown" => example_markdown(),
        "citation" => example_citation(),
        "timer" => example_timer(),
        "timestamp" => example_timestamp(),
        _ => return None,
    })
}

fn example_text() -> Node {
    specimen(
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
        "styled_text(\"Supporting text\").kind(TextKind::Supporting).color(TextColor::Secondary)",
    )
}

fn example_heading() -> Node {
    specimen(
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
        "kivact::ui::heading(\"Display\", 1).variant(HeadingDisplay::D1)",
    )
}

fn example_icon() -> Node {
    specimen(
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
        "icon_view(\"circle-check\").color(IconColor::Success)",
    )
}

fn example_kbd() -> Node {
    specimen(
        "Keyboard shortcuts",
        row((kbd("mod+shift+p"), kbd("escape"), kbd("up+down"))).style(style().gap(12)),
        "kbd(\"mod+shift+p\")",
    )
}

fn example_empty_state() -> Node {
    specimen(
        "Empty project",
        kivact::ui::empty_state("No documents")
            .announce_changes()
            .description("Create a document to get started.")
            .icon(icon_view("search"))
            .actions(kivact::ui::button("New document")),
        "empty_state(\"No documents\").description(\"Create a document to get started.\")",
    )
}

fn example_avatar() -> Node {
    specimen(
        "Initials and grouped members",
        row((
            avatar("Avery Morgan"),
            avatar_group((avatar("Sam Rivera"), avatar("Taylor Chen"))).total(5usize),
        ))
        .style(style().gap(16)),
        "avatar(\"Avery Morgan\")",
    )
}

fn example_visually_hidden() -> Node {
    specimen(
        "Screen reader text",
        column((
            styled_text("The hint is available to assistive technology."),
            visually_hidden("Additional hint"),
        )),
        "visually_hidden(\"Additional hint\")",
    )
}

fn example_link() -> Node {
    specimen(
        "External link",
        link("Reference").external(true).on_press(|| {}),
        "link(\"Reference\").external(true).on_press(open_reference)",
    )
}

fn example_thumbnail() -> Node {
    specimen(
        "Attachment states",
        row((
            thumbnail().label("Cover image"),
            thumbnail().label("Uploading image").loading(true),
        ))
        .style(style().gap(12)),
        "thumbnail().label(\"Uploading image\").busy(true)",
    )
}

fn example_blockquote() -> Node {
    specimen(
        "Quotation and attribution",
        kivact_ui_soft::blockquote_text("A simple interface lets the content lead.")
            .cite("Design notes"),
        "blockquote_text(\"A simple interface lets the content lead.\").cite(\"Design notes\")",
    )
}

fn example_code() -> Node {
    specimen(
        "Inline code",
        kivact_ui_soft::code("cargo run"),
        "code(\"cargo run\")",
    )
}

fn example_code_block() -> Node {
    specimen(
        "Rust example",
        kivact::ui::code_block("fn main() {\n    println!(\"Hello\");\n}")
            .language("rust")
            .highlight(
                || {
                    kivact::style::use_tokens::<kivact_ui_soft::Tokens>()
                        .colors
                        .syntax
                },
                |palette, line, _: &()| {
                    (
                        kivact_ui_soft::code_highlight(
                            &kivact_ui_soft::CodeLanguageInput::from("rust"),
                            line,
                            palette,
                        ),
                        (),
                    )
                },
            )
            .title("Example source")
            .language_label(false)
            .collapsible(true)
            .collapsible_threshold(3),
        "code_block(source).language(\"rust\").title(\"Example source\").collapsible(true).collapsible_threshold(3)",
    )
}

fn example_markdown() -> Node {
    specimen(
            "Formatted content",
            kivact_ui_soft::markdown(
                "## Project notes\n\nUse **Soft** to style your interface.\n\nThe project reference lives in [the guide](https://example.com/guide) and wraps with the paragraph.\n\n- Review the catalog\n- Try the controls",
            )
            .content_width(Some(240.0)),
            "markdown(source).content_width(Some(240.0))",
        )
}

fn example_citation() -> Node {
    specimen(
        "Source reference",
        kivact_ui_soft::citation(
            1,
            kivact_ui_soft::CitationSource {
                title: Some("Design notes".to_owned()),
                ..Default::default()
            },
        ),
        "citation(1, source)",
    )
}

fn example_timer() -> Node {
    specimen("Elapsed time", kivact_ui_soft::timer(), "timer()")
}

fn example_timestamp() -> Node {
    specimen(
        "Timestamp",
        kivact_ui_soft::timestamp(kivact::state::signal(
            kivact_ui_soft::TimestampValue::unix_seconds(1_793_664_000),
        )),
        "timestamp(instant)",
    )
}
