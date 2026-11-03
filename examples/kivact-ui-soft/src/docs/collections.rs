//! Role: Demonstrate the tabs, disclosure, list, table and split recipes.
use super::specimen::specimen;
use kivact::control::{Node, NodeBuilder, text};
use kivact::interaction::ListActivation;
use kivact::signal;
use kivact::style::style;
use kivact::ui::{DataColumn, ListItem, data_table, disclosure, split, tabs};
use kivact_ui_soft::{
    ChatSender, GridColumns, Space, aspect_ratio, breadcrumb_item, breadcrumbs, card, chat_bubble,
    chat_composer, chat_layout, chat_message, chat_system_message, divider, grid, hstack, item,
    list, list_item, metadata_item, metadata_list, overflow_list, pagination, section, stack_item,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "tab-list" | "tab" => specimen(
            "Manual activation",
            tabs(
                "Project views",
                vec![
                    ListItem::new("summary", "Summary"),
                    ListItem::new("activity", "Activity"),
                    ListItem::new("settings", "Settings"),
                ],
                signal("summary"),
            )
            .activation(ListActivation::Manual)
            .content(|key| text(format!("Selected view: {key}"))),
            "tabs(\"Project views\", options, selected).activation(ListActivation::Manual)",
        ),
        "collapsible" | "collapsible-group" => specimen(
            "Open by default",
            disclosure("Delivery details", signal(true), || {
                text("Ships in two business days.")
            }),
            "disclosure(\"Delivery details\", signal(true), details)",
        ),
        "list" | "list-item" | "item" => specimen(
            "Workspace items",
            list((
                list_item("Design").description("Active project"),
                list_item("Documentation").description("In review"),
                item("Archive"),
            ))
            .style(style().width(240)),
            "list((list_item(\"Design\"), list_item(\"Documentation\")))",
        ),
        "card" => specimen(
            "Project surface",
            card(text("Project notes")),
            "card(text(\"Project notes\"))",
        ),
        "section" => specimen(
            "Section content",
            section(text("Workspace settings")),
            "section(text(\"Workspace settings\"))",
        ),
        "divider" => specimen(
            "Separate related content",
            kivact::control::column((text("Details"), divider(), text("History"))),
            "divider()",
        ),
        "stack" | "stack-item" => specimen(
            "Horizontal stack",
            hstack((
                stack_item(text("Design")),
                stack_item(text("Documentation")),
            )),
            "hstack((stack_item(first), stack_item(second)))",
        ),
        "grid" => specimen(
            "Two columns",
            grid((card(text("Design")), card(text("Documentation"))))
                .columns(GridColumns::Fixed(2))
                .gap(Space::S2),
            "grid(children).columns(GridColumns::Fixed(2)).gap(Space::S2)",
        ),
        "aspect-ratio" => specimen(
            "Wide preview",
            aspect_ratio(16.0 / 9.0, card(text("Preview"))).style(style().width(240)),
            "aspect_ratio(16.0 / 9.0, preview)",
        ),
        "breadcrumbs" | "breadcrumb-item" => specimen(
            "Project location",
            breadcrumbs([
                breadcrumb_item("Workspace").on_press(|| {}),
                breadcrumb_item("Design notes"),
            ]),
            "breadcrumbs([breadcrumb_item(\"Workspace\"), breadcrumb_item(\"Design notes\")])",
        ),
        "pagination" => specimen(
            "Navigate result pages",
            pagination(signal(1)).total_pages(12).first_last(true),
            "pagination(page).total_pages(12).first_last(true)",
        ),
        "metadata-list" | "metadata-list-item" => specimen(
            "Document properties",
            metadata_list((
                metadata_item("Owner", text("Avery Morgan")),
                metadata_item("Status", text("In review")),
            )),
            "metadata_list((metadata_item(\"Owner\", text(\"Avery Morgan\")), status))",
        ),
        "overflow-list" => specimen(
            "Items in a narrow space",
            overflow_list(
                vec!["Design", "Engineering", "Documentation", "Support"],
                |item| *item,
                |item| text(move || item.get().to_owned()),
            )
            .style(style().width(230)),
            "overflow_list(teams, |item| *item, render_team)",
        ),
        "chat" => specimen(
            "Messages and composer",
            chat_layout(kivact::control::column((
                chat_system_message("Conversation started"),
                chat_message(
                    ChatSender::User,
                    chat_bubble(text("Can you review the notes?")),
                ),
                chat_message(
                    ChatSender::Assistant,
                    chat_bubble(text("The notes are ready.")),
                ),
            )))
            .composer(|| chat_composer(signal(String::new())))
            .style(style().width(440).height(300)),
            "chat_layout(messages).composer(|| chat_composer(draft))",
        ),
        "table" => specimen(
            "Sortable columns",
            data_table(
                vec![
                    ("Design", "Active"),
                    ("Documentation", "Review"),
                    ("Archive", "Closed"),
                ],
                |row| row.0,
                vec![
                    DataColumn::new("Project", |row: &(&str, &str)| row.0.to_owned()).sortable(),
                    DataColumn::new("Status", |row: &(&str, &str)| row.1.to_owned()),
                ],
            )
            .style(style().width(440)),
            "data_table(rows, |row| row.0, columns)",
        ),
        "resizable" | "resize-handle" => specimen(
            "Split panes",
            split(
                (text("Navigation"), text("Content")),
                signal(vec![140.0, 200.0]),
            )
            .style(style().width(340).height(72)),
            "split((navigation, content), sizes)",
        ),
        _ => return None,
    })
}
