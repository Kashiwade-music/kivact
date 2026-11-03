//! Role: Demonstrate standard form recipes through controlled public components.
use super::specimen::specimen;
use kivact::control::{Node, NodeBuilder, column};
use kivact::interaction::ScalarRange;
use kivact::signal;
use kivact::style::style;
use kivact::ui::{ListItem, field, number_input, text_area, text_input};
use kivact_ui_soft::{
    checkbox_list, complex_selector, file_input, input_group_text, input_group_with,
    multi_selector, power_search, rich_text_editor, selector, tokenizer, typeahead,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "text-input" => specimen(
            "Empty, filled and disabled",
            column((
                text_input("Search", signal(String::new())).placeholder("Search"),
                text_input("Name", signal("Avery".to_owned())),
                text_input("Unavailable", signal("Archived".to_owned())).disabled(true),
            ))
            .style(style().gap(8).width(260)),
            "text_input(\"Search\", signal(String::new())).placeholder(\"Search\")",
        ),
        "text-area" => specimen(
            "Automatic height",
            text_area("Notes", signal(String::new()))
                .wrap(true)
                .rows(2..=5)
                .placeholder("Add notes")
                .style(style().width(280)),
            "text_area(\"Notes\", notes).wrap(true).rows(2..=5)",
        ),
        "field" | "field-label" | "field-status" => specimen(
            "Label, description and validation",
            column((
                field("Name", text_input("Name", signal(String::new())))
                    .hint("Shown on your profile"),
                field("Email", text_input("Email", signal("invalid".to_owned())))
                    .error(Some("Enter an email address".to_owned())),
            ))
            .style(style().gap(12).width(280)),
            "field(\"Name\", text_input(\"Name\", name)).hint(\"Shown on your profile\")",
        ),
        "selector" => specimen(
            "Placeholder and disabled option",
            selector("Team", teams(), signal(None))
                .placeholder("Choose a team")
                .style(style().width(220)),
            "selector(\"Team\", options, selected).placeholder(\"Choose a team\")",
        ),
        "complex-selector" => specimen(
            "Custom selector content",
            complex_selector("Workspace", "Design", || {
                kivact::control::text("Workspace details")
            }),
            "complex_selector(\"Workspace\", \"Design\", content)",
        ),
        "multi-selector" => specimen(
            "Multiple selections",
            multi_selector("Teams", teams(), signal(vec!["design"])).style(style().width(280)),
            "multi_selector(\"Teams\", options, selected)",
        ),
        "typeahead" | "base-typeahead" => specimen(
            "Search suggestions",
            typeahead("Team", teams(), signal(None)).style(style().width(280)),
            "typeahead(\"Team\", options, selected)",
        ),
        "power-search" => specimen(
            "Editable filters",
            power_search("Team filters", teams(), signal(Vec::new())),
            "power_search(\"Team filters\", fields, filters)",
        ),
        "checkbox-list" | "checkbox-list-item" => specimen(
            "Choose several teams",
            checkbox_list("Teams", teams(), signal(vec!["design"])),
            "checkbox_list(\"Teams\", options, selected)",
        ),
        "input-group" | "input-group-text" => specimen(
            "Address with prefix",
            input_group_with("Website", || {
                (
                    input_group_text("https://"),
                    text_input("Domain", signal("example.com".to_owned())),
                )
            })
            .style(style().width(280)),
            "input_group_with(\"Website\", || (input_group_text(\"https://\"), domain()))",
        ),
        "file-input" => specimen(
            "Choose or drop a file",
            file_input("Attachment", signal(Vec::new())).style(style().width(280)),
            "file_input(\"Attachment\", files)",
        ),
        "rich-text-editor" => specimen(
            "Markdown source and preview",
            rich_text_editor(
                "Notes",
                signal("# Project notes\n\n**Ready for review.**".to_owned()),
            )
            .style(style().width(440)),
            "rich_text_editor(\"Notes\", source)",
        ),
        "number-input" => specimen(
            "Stepped input",
            number_input(
                "Quantity",
                signal(3.0),
                ScalarRange::new(0.0, 10.0).expect("invariant: finite range"),
            )
            .style(style().width(160)),
            "number_input(\"Quantity\", quantity, range)",
        ),
        "tokenizer" => specimen(
            "Add and remove tags",
            tokenizer("Teams", teams(), signal(vec!["design"])).style(style().width(320)),
            "tokenizer(\"Teams\", options, selected)",
        ),
        _ => return None,
    })
}

fn teams() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("design", "Design"),
        ListItem::new("engineering", "Engineering"),
        ListItem::new("archive", "Archive").disabled(true),
    ]
}
