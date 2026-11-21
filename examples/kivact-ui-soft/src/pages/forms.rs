//! Demonstrate standard form recipes through controlled public components.
use super::labelled;
use kivact::interaction::ListItem;
use kivact::interaction::ScalarRange;
use kivact::state::signal;
use kivact::style::style;
use kivact::ui::{ControlSize, field, number_input, text_area, text_input};
use kivact::ui::{input_addon, input_group};
use kivact::view::{Element, Node, column};
use kivact_ui_soft::{
    FieldStatusVariant, StatusKind, checkbox_list, field_status, power_search, rich_text_editor,
};

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
        .pick_on_press(true)
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
            .multiple(true)
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

// example: text-input
pub(super) fn example_text_input() -> Node {
    column((
        labelled(
            "Empty, filled and disabled",
            column((
                text_input(signal(String::new()))
                    .label("Search")
                    .placeholder("Search"),
                text_input(signal("Avery".to_owned())).label("Name"),
                text_input(signal("Archived".to_owned()))
                    .label("Unavailable")
                    .disabled(true),
            ))
            .style(style().gap(8).width(260)),
        ),
        labelled(
            "Small and large read-only inputs",
            column((
                text_input(signal("Avery".to_owned()))
                    .label("Small")
                    .size(ControlSize::Small),
                text_input(signal("avery@example.com".to_owned()))
                    .label("Read only")
                    .size(ControlSize::Large)
                    .read_only(true),
            ))
            .style(style().gap(8).width(260)),
        ),
    ))
    .style(style().gap(12))
}
// end example

// example: text-area
pub(super) fn example_text_area() -> Node {
    column((
        labelled(
            "Automatic height",
            text_area(signal(String::new()))
                .label("Notes")
                .wrap(true)
                .rows(2..=5)
                .placeholder("Add notes")
                .style(style().width(280)),
        ),
        labelled(
            "Disabled notes",
            text_area(signal("First line\nSecond line".to_owned()))
                .label("Archived notes")
                .wrap(true)
                .rows(2..=5)
                .disabled(true)
                .style(style().width(280)),
        ),
    ))
    .style(style().gap(12))
}
// end example

// example: field
pub(super) fn example_field() -> Node {
    labelled(
        "Label, description and validation",
        column((
            field("Name", text_input(signal(String::new())).label("Name"))
                .hint("Shown on your profile"),
            field(
                "Email",
                text_input(signal("invalid".to_owned())).label("Email"),
            )
            .error(Some("Enter an email address".to_owned())),
        ))
        .style(style().gap(12).width(280)),
    )
}
// end example

// example: field-status
pub(super) fn example_field_status() -> Node {
    column((
        labelled(
            "Attached success",
            column((
                text_input(signal("Avery".to_owned()))
                    .label("Name")
                    .style(style().z_index(1)),
                field_status(StatusKind::Success, "Name is available")
                    .variant(FieldStatusVariant::Attached),
            ))
            .style(style().width(280)),
        ),
        labelled(
            "Detached warning",
            column((
                text_input(signal("Avery".to_owned())).label("Display name"),
                field_status(StatusKind::Warning, "Your changes are pending")
                    .variant(FieldStatusVariant::Detached),
            ))
            .style(style().width(280)),
        ),
    ))
    .style(style().gap(12))
}
// end example

// example: selector
pub(super) fn example_selector() -> Node {
    column((
        labelled(
            "Placeholder and disabled option",
            kivact::ui::select(teams(), signal(None))
                .label("Team")
                .open_on_arrow(true)
                .placeholder("Choose a team")
                .style(style().width(220)),
        ),
        labelled(
            "Selected and disabled",
            column((
                kivact::ui::select(teams(), signal(Some("design")))
                    .label("Assigned team")
                    .open_on_arrow(true),
                kivact::ui::select(teams(), signal(Some("engineering")))
                    .label("Locked team")
                    .open_on_arrow(true)
                    .disabled(true),
            ))
            .style(style().gap(8).width(220)),
        ),
    ))
    .style(style().gap(12))
}
// end example

// example: complex-selector
pub(super) fn example_complex_selector() -> Node {
    labelled(
        "Custom selector content",
        kivact::ui::select(teams(), signal(Some("design")))
            .label("Workspace")
            .trigger(kivact::view::text("Workspace details")),
    )
}
// end example

// example: multi-selector
pub(super) fn example_multi_selector() -> Node {
    labelled(
        "Multiple selections",
        kivact::ui::combobox(teams(), signal(vec!["design"]))
            .label("Teams")
            .chips()
            .style(style().width(280)),
    )
}
// end example

// example: typeahead
pub(super) fn example_typeahead() -> Node {
    labelled(
        "Search suggestions",
        kivact::ui::combobox(teams(), signal(None))
            .label("Team")
            .style(style().width(280)),
    )
}
// end example

// example: power-search
pub(super) fn example_power_search() -> Node {
    labelled(
        "Editable filters",
        power_search("Team filters", teams(), signal(Vec::new())),
    )
}
// end example

// example: checkbox-list
pub(super) fn example_checkbox_list() -> Node {
    labelled(
        "Choose several teams",
        checkbox_list("Teams", teams(), signal(vec!["design"])),
    )
}
// end example

// example: input-group
pub(super) fn example_input_group() -> Node {
    labelled(
        "Address with prefix",
        input_group(
            "Website",
            (
                input_addon("https://"),
                text_input(signal("example.com".to_owned())).label("Domain"),
            ),
        )
        .style(style().width(280)),
    )
}
// end example

// example: file-input
pub(super) fn example_file_input() -> Node {
    column((
        labelled("Choose a file", file_target("Attachment", false)),
        labelled("Choose or drop images", file_target("Images", true)),
    ))
    .style(style().gap(12))
}
// end example

// example: rich-text-editor
pub(super) fn example_rich_text_editor() -> Node {
    labelled(
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
    )
}
// end example

// example: number-input
pub(super) fn example_number_input() -> Node {
    labelled(
        "Stepped input",
        number_input(
            signal(3.0),
            ScalarRange::new(0.0, 10.0).expect("invariant: finite range"),
        )
        .label("Quantity")
        .style(style().width(160)),
    )
}
// end example

// example: tokenizer
pub(super) fn example_tokenizer() -> Node {
    labelled(
        "Add and remove tags",
        kivact::ui::combobox(teams(), signal(vec!["design"]))
            .label("Teams")
            .chips()
            .exclude_selected()
            .style(style().width(320)),
    )
}
// end example
