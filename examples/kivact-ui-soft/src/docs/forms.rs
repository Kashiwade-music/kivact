//! Role: Demonstrate standard form recipes through controlled public components.
use super::examples::specimen;
use kivact::control::{Node, NodeBuilder, column};
use kivact::interaction::ScalarRange;
use kivact::signal;
use kivact::style::style;
use kivact::ui::{ListItem, field, number_input, select, tag_input, text_area, text_input};

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
            select(
                "Team",
                vec![
                    ListItem::new("design", "Design"),
                    ListItem::new("engineering", "Engineering"),
                    ListItem::new("archive", "Archive").disabled(true),
                ],
                signal(None::<&str>),
            )
            .placeholder("Choose a team")
            .style(style().width(220)),
            "select(\"Team\", options, selected).placeholder(\"Choose a team\")",
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
        "tokenizer" => {
            let values = signal(vec!["Design".to_owned(), "Engineering".to_owned()]);
            specimen(
                "Add and remove tags",
                tag_input(
                    "Teams",
                    values,
                    move |tag| values.update(|values| values.push(tag)),
                    move |index| {
                        values.update(|values| {
                            if index < values.len() {
                                values.remove(index);
                            }
                        });
                    },
                )
                .style(style().width(320)),
                "tag_input(\"Teams\", values, add_tag, remove_tag)",
            )
        }
        _ => return None,
    })
}
