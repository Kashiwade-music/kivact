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
        "text" => specimen(
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
        ),
        "heading" => specimen(
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
        ),
        "icon" => specimen(
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
        ),
        "kbd" => specimen(
            "Keyboard shortcuts",
            row((kbd("mod+shift+p"), kbd("escape"), kbd("up+down"))).style(style().gap(12)),
            "kbd(\"mod+shift+p\")",
        ),
        "empty-state" => specimen(
            "Empty project",
            kivact::ui::empty_state("No documents").announce_changes()
                .description("Create a document to get started.")
                .icon( icon_view("search"))
                .actions( kivact::ui::button("New document")),
            "empty_state(\"No documents\").description(\"Create a document to get started.\")",
        ),
        "avatar" | "avatar-group" | "avatar-group-overflow" | "avatar-status-dot" => specimen(
            "Initials and grouped members",
            row((
                avatar("Avery Morgan"),
                avatar_group((
                    avatar("Sam Rivera"),
                    avatar("Taylor Chen"),
                )).total(5usize),
            ))
            .style(style().gap(16)),
            "avatar(\"Avery Morgan\")",
        ),
        "visually-hidden" => specimen(
            "Screen reader text",
            column((
                styled_text("The hint is available to assistive technology."),
                visually_hidden("Additional hint"),
            )),
            "visually_hidden(\"Additional hint\")",
        ),
        "link" => specimen(
            "External link",
            link("Reference").external(true).on_press(|| {}),
            "link(\"Reference\").external(true).on_press(open_reference)",
        ),
        "thumbnail" => specimen(
            "Attachment states",
            row((
                thumbnail().label("Cover image"),
                thumbnail().label("Uploading image").loading(true),
            ))
            .style(style().gap(12)),
            "thumbnail().label(\"Uploading image\").busy(true)",
        ),
        "blockquote" => specimen(
            "Quotation and attribution",
            kivact_ui_soft::blockquote_text("A simple interface lets the content lead.")
                .cite("Design notes"),
            "blockquote_text(\"A simple interface lets the content lead.\").cite(\"Design notes\")",
        ),
        "code" => specimen(
            "Inline code",
            kivact_ui_soft::code("cargo run"),
            "code(\"cargo run\")",
        ),
        "code-block" => specimen(
            "Rust example",
            kivact::ui::code_block("fn main() {\n    println!(\"Hello\");\n}")
                .language("rust")
                .highlight(
                    || kivact::style::use_tokens::<kivact_ui_soft::Tokens>().colors.syntax,
                    |palette, line, _: &()| (kivact_ui_soft::code_highlight(&kivact_ui_soft::CodeLanguageInput::from("rust"), line, palette), ()),
                )
                .title("Example source")
                .language_label(false)
                .collapsible(true)
                .collapsible_threshold(3),
            "code_block(source).language(\"rust\").title(\"Example source\").collapsible(true).collapsible_threshold(3)",
        ),
        "markdown" => specimen(
            "Formatted content",
            kivact_ui_soft::markdown(
                "## Project notes\n\nUse **Soft** to style your interface.\n\nThe project reference lives in [the guide](https://example.com/guide) and wraps with the paragraph.\n\n- Review the catalog\n- Try the controls",
            )
            .content_width(Some(240.0)),
            "markdown(source).content_width(Some(240.0))",
        ),
        "citation" => specimen(
            "Source reference",
            kivact_ui_soft::citation(
                1,
                kivact_ui_soft::CitationSource {
                    title: Some("Design notes".to_owned()),
                    ..Default::default()
                },
            ),
            "citation(1, source)",
        ),
        "timer" => specimen("Elapsed time", kivact_ui_soft::timer(), "timer()"),
        "timestamp" => specimen(
            "Timestamp",
            kivact_ui_soft::timestamp(kivact::state::signal(kivact_ui_soft::TimestampValue::unix_seconds(
                1_793_664_000,
            ))),
            "timestamp(instant)",
        ),
        _ => return None,
    })
}
