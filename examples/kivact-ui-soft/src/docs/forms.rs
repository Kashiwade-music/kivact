//! Role: Demonstrate standard form recipes through controlled public components.
use super::specimen::specimen;
use kivact::interaction::ListItem;
use kivact::interaction::ScalarRange;
use kivact::state::signal;
use kivact::style::style;
use kivact::ui::{ControlSize, field, number_input, text_area, text_input};
use kivact::view::{Node, NodeBuilder, column};
use kivact_ui_soft::{
    FieldStatusVariant, StatusKind, checkbox_list, field_status, input_group_text,
    input_group_with, power_search, rich_text_editor,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "text-input" => column((
            specimen(
                "Empty, filled and disabled",
                column((
                    text_input("Search", signal(String::new())).placeholder("Search"),
                    text_input("Name", signal("Avery".to_owned())),
                    text_input("Unavailable", signal("Archived".to_owned())).disabled(true),
                ))
                .style(style().gap(8).width(260)),
                r#"text_input("Search", signal(String::new())).placeholder("Search")
text_input("Name", signal("Avery".to_owned()))
text_input("Unavailable", signal("Archived".to_owned())).disabled(true)"#,
            ),
            specimen(
                "Small and large read-only inputs",
                column((
                    text_input("Small", signal("Avery".to_owned())).size(ControlSize::Small),
                    text_input("Read only", signal("avery@example.com".to_owned()))
                        .size(ControlSize::Large)
                        .read_only(true),
                ))
                .style(style().gap(8).width(260)),
                r#"text_input("Small", signal("Avery".to_owned())).size(ControlSize::Small)
text_input("Read only", signal("avery@example.com".to_owned()))
    .size(ControlSize::Large)
    .read_only(true)"#,
            ),
        ))
        .style(style().gap(12)),
        "text-area" => column((
            specimen(
                "Automatic height",
                text_area("Notes", signal(String::new()))
                    .wrap(true)
                    .rows(2..=5)
                    .placeholder("Add notes")
                    .style(style().width(280)),
                r#"text_area("Notes", signal(String::new()))
    .wrap(true)
    .rows(2..=5)
    .placeholder("Add notes")"#,
            ),
            specimen(
                "Disabled notes",
                text_area(
                    "Archived notes",
                    signal("First line\nSecond line".to_owned()),
                )
                .wrap(true)
                .rows(2..=5)
                .disabled(true)
                .style(style().width(280)),
                r#"text_area("Archived notes", signal("First line\nSecond line".to_owned()))
    .wrap(true)
    .rows(2..=5)
    .disabled(true)"#,
            ),
        ))
        .style(style().gap(12)),
        "field" | "field-label" => specimen(
            "Label, description and validation",
            column((
                field("Name", text_input("Name", signal(String::new())))
                    .hint("Shown on your profile"),
                field("Email", text_input("Email", signal("invalid".to_owned())))
                    .error(Some("Enter an email address".to_owned())),
            ))
            .style(style().gap(12).width(280)),
            r#"field("Name", text_input("Name", signal(String::new())))
    .with_hint("Shown on your profile")
field("Email", text_input("Email", signal("invalid".to_owned())))
    .error(Some("Enter an email address".to_owned()))"#,
        ),
        "field-status" => column((
            specimen(
                "Attached success",
                column((
                    text_input("Name", signal("Avery".to_owned())).style(style().z_index(1)),
                    field_status(StatusKind::Success, "Name is available")
                        .variant(FieldStatusVariant::Attached),
                ))
                .style(style().width(280)),
                r#"column((
    text_input("Name", signal("Avery".to_owned())).style(style().z_index(1)),
    field_status(StatusKind::Success, "Name is available")
        .variant(FieldStatusVariant::Attached),
))"#,
            ),
            specimen(
                "Detached warning",
                column((
                    text_input("Display name", signal("Avery".to_owned())),
                    field_status(StatusKind::Warning, "Your changes are pending")
                        .variant(FieldStatusVariant::Detached),
                ))
                .style(style().width(280)),
                r#"column((
    text_input("Display name", signal("Avery".to_owned())),
    field_status(StatusKind::Warning, "Your changes are pending")
        .variant(FieldStatusVariant::Detached),
))"#,
            ),
        ))
        .style(style().gap(12)),
        "selector" => column((
            specimen(
                "Placeholder and disabled option",
                kivact::ui::select("Team", teams(), signal(None)).open_on_arrow(true)
                    .placeholder("Choose a team")
                    .style(style().width(220)),
                r#"kivact::ui::select("Team", teams(), signal(None)).open_on_arrow(true).placeholder("Choose a team")"#,
            ),
            specimen(
                "Selected and disabled",
                column((
                    kivact::ui::select("Assigned team", teams(), signal(Some("design"))).open_on_arrow(true),
                    kivact::ui::select("Locked team", teams(), signal(Some("engineering"))).open_on_arrow(true).disabled(true),
                ))
                .style(style().gap(8).width(220)),
                r#"kivact::ui::select("Assigned team", teams(), signal(Some("design"))).open_on_arrow(true)
kivact::ui::select("Locked team", teams(), signal(Some("engineering"))).open_on_arrow(true).disabled(true)"#,
            ),
        ))
        .style(style().gap(12)),
        "complex-selector" => specimen(
            "Custom selector content",
            kivact::ui::selection_field("Workspace", "Design").content(|| {
                kivact::view::text("Workspace details")
            }),
            "selection_field(\"Workspace\", \"Design\").content(content)",
        ),
        "multi-selector" => specimen(
            "Multiple selections",
            kivact::ui::combobox("Teams", teams(), signal(vec!["design"])).chips().style(style().width(280)),
            "kivact::ui::combobox(\"Teams\", options, selected).chips()",
        ),
        "typeahead" | "base-typeahead" => specimen(
            "Search suggestions",
            kivact::ui::combobox("Team", teams(), signal(None)).style(style().width(280)),
            "kivact::ui::combobox(\"Team\", options, selected)",
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
        "file-input" => column((
            specimen("Choose a file", file_target("Attachment", false), "dropzone(label).pick_on_press().on_files(store_files)"),
            specimen("Choose or drop images", file_target("Images", true), "dropzone(label).multiple().pick_on_press().filters(filters).validate(validate_image).on_files(store_files)"),
        )).style(style().gap(12)),
        "rich-text-editor" => specimen(
            "Edit blocks, switch to Markdown source, or preview",
            rich_text_editor(
                "Notes",
                signal(
                    "# Project notes\n\n**Ready for review.**\n\n- [ ] Confirm the delivery date"
                        .to_owned(),
                ),
            )
            .block_editing(true)
            .style(style().width(440)),
            "rich_text_editor(\"Notes\", source).block_editing(true)",
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
            kivact::ui::combobox("Teams", teams(), signal(vec!["design"])).chips().exclude_selected().style(style().width(320)),
            "kivact::ui::combobox(\"Teams\", options, selected).chips().exclude_selected()",
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

fn file_target(label: &'static str, accepts_drop: bool) -> Node {
    use kivact::interaction::FileInfo;
    use kivact::ui::{ButtonVariant, button, dropzone};
    use kivact::view::{node_ref, show};
    let files = signal(Vec::<FileInfo>::new());
    let target = node_ref();
    let field = dropzone(label)
        .icon(kivact::view::icon("upload"))
        .pick_on_press()
        .node_ref(target)
        .hint(move || {
            files.with(|items| {
                items
                    .iter()
                    .map(FileInfo::name)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
        })
        .on_files(move |accepted| files.set(accepted))
        .style(style().width(280));
    let field = if accepts_drop {
        field
            .multiple()
            .filters(vec![(
                "Images".to_owned(),
                vec!["png".to_owned(), "jpg".to_owned()],
            )])
            .validate(|file| {
                if !file
                    .path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|ext| {
                        ext.eq_ignore_ascii_case("png") || ext.eq_ignore_ascii_case("jpg")
                    })
                {
                    Some("One or more files are not allowed.".to_owned())
                } else if std::fs::metadata(&file.path).map_or(true, |metadata| {
                    !metadata.is_file() || metadata.len() > 5 * 1024 * 1024
                }) {
                    Some("One or more files exceed the size limit or could not be read.".to_owned())
                } else {
                    None
                }
            })
    } else {
        field
            .variant(kivact_ui_soft::DropzoneVariant::Field)
            .accept_drop(false)
    };
    column((
        field,
        show(
            move || files.with(|files| !files.is_empty()),
            move || {
                button("Clear files")
                    .variant(ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(kivact::view::icon("x"))
                    .on_press(move || {
                        files.set(Vec::new());
                        target.focus();
                    })
            },
        ),
    ))
    .style(style().gap(4))
}
