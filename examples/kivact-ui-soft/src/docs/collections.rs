//! Role: Demonstrate Soft layout, collection, table and chat components.
use super::specimen::specimen;
use kivact::control::{Node, NodeBuilder, column, row, text};
use kivact::interaction::TreeNode;
use kivact::signal;
use kivact::style::style;
use kivact::ui::{DataColumn, data_table, split};
use kivact_ui_soft::{
    ChatSender, DictationState, GridColumns, Space, ToolCall, ToolStatus, aspect_ratio,
    breadcrumb_item, breadcrumbs, card, carousel, center, chat_bubble, chat_composer,
    chat_dictation_button, chat_layout, chat_message, chat_system_message, chat_tool_calls,
    clickable_card, collapsible_default, collapsible_group, divider, form_layout, form_row, grid,
    hstack, item, layout, layout_content, layout_header, list, list_item, metadata_item,
    metadata_list, nav_icon, overflow_list, pagination, scrollable_area, section, selectable_card,
    stack_item, tree_list,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "collapsible" => specimen(
            "Open by default",
            collapsible_default("Delivery details", true)
                .animate_layout(true)
                .content(|| text("Ships in two business days.")),
            "collapsible_default(\"Delivery details\", true).animate_layout(true).content(details)",
        ),
        "collapsible-group" => specimen(
            "One section open at a time",
            collapsible_group(
                signal(vec!["delivery".to_owned()]),
                vec![
                    collapsible_default("Delivery", false)
                        .value("delivery")
                        .content(|| text("Ships in two business days.")),
                    collapsible_default("Returns", false)
                        .value("returns")
                        .content(|| text("Returns accepted within 30 days.")),
                ],
            )
            .dividers(true),
            "collapsible_group(open, sections).dividers(true)",
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
        "clickable-card" => specimen(
            "Open project",
            clickable_card("Open project", text("Project notes")).on_press(|| {}),
            "clickable_card(\"Open project\", content, open_project)",
        ),
        "selectable-card" => specimen(
            "Select project",
            selectable_card("Select project", signal(false), text("Project notes")),
            "selectable_card(\"Select project\", selected, content)",
        ),
        "carousel" => specimen(
            "Browse slides",
            carousel(
                "Project highlights",
                [card(text("Design")), card(text("Documentation"))],
            )
            .style(style().width(320)),
            "carousel(\"Project highlights\", slides)",
        ),
        "section" => specimen(
            "Section content",
            section(text("Workspace settings")),
            "section(text(\"Workspace settings\"))",
        ),
        "center" => specimen(
            "Center content",
            center(text("Centered content")).style(style().size(240, 96)),
            "center(text(\"Centered content\"))",
        ),
        "form-layout" => specimen(
            "Aligned fields",
            form_layout(form_row(
                text("Project"),
                kivact::ui::text_input("Project", signal("Design notes".to_owned())),
            ))
            .style(style().width(320)),
            "form_layout(form_row(label, input))",
        ),
        "layout" => specimen(
            "Header and content",
            layout()
                .header(|| layout_header(text("Project notes")).divider(true))
                .content(|| layout_content(text("Workspace content")))
                .style(style().width(320).height(144)),
            "layout().header(header).content(content)",
        ),
        "scrollable-area" => specimen(
            "Scroll with the keyboard",
            scrollable_area(
                "Activity",
                column(
                    (0..12)
                        .map(|index| text(format!("Activity {}", index + 1)))
                        .collect::<Vec<_>>(),
                ),
            )
            .style(style().width(240).height(96)),
            "scrollable_area(\"Activity\", events)",
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
        "tree-list" => specimen(
            "Expandable files",
            tree_list(
                "Files",
                vec![
                    TreeNode::new("src", None, "src"),
                    TreeNode::new("main.rs", Some("src"), "main.rs"),
                    TreeNode::new("README.md", None, "README.md"),
                ],
                signal(vec!["src"]),
            )
            .style(style().width(240)),
            "tree_list(\"Files\", files, expanded)",
        ),
        "nav-icon" => specimen(
            "Navigation icon",
            row((
                nav_icon(kivact_ui_soft::icon("file-text")),
                text("Documents"),
            ))
            .style(style().gap(8)),
            "nav_icon(\"file-text\")",
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
        "chat-dictation-button" => specimen(
            "Dictation available",
            chat_dictation_button(signal(DictationState {
                supported: true,
                ..DictationState::default()
            })),
            "chat_dictation_button(dictation_state)",
        ),
        "chat-tool-calls" => specimen(
            "Tool activity",
            chat_tool_calls(vec![
                ToolCall::new("read", "Read notes").target("notes.md"),
                ToolCall::new("review", "Review notes").status(ToolStatus::Running),
            ])
            .expanded(signal(true)),
            "chat_tool_calls(calls).expanded(expanded)",
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
