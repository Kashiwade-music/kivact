//! Role: Demonstrate Soft layout, collection, table and chat components.
use super::specimen::specimen;
use kivact::interaction::FilterValue;
use kivact::interaction::TreeNode;
use kivact::state::signal;
use kivact::style::style;
use kivact::ui::{DataColumn, DataTableDensity, FilterField, button, card, data_table};
use kivact::view::{Element, Node, column, row, text};
use kivact_ui_soft::{
    DictationState, FormDirection, GridColumns, Space, ToolCall, ToolStatus, aspect_ratio, center,
    chat_dictation_button, chat_tool_calls, divider, form_layout, form_row, hstack, layout,
    layout_content, layout_grid, layout_header, nav_icon, overflow_list, scrollable_area, section,
    stack_item,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "collapsible" => example_collapsible(),
        "collapsible-group" => example_collapsible_group(),
        "list" | "list-item" | "item" => example_list(),
        "card" => example_card(),
        "clickable-card" => example_clickable_card(),
        "selectable-card" => example_selectable_card(),
        "carousel" => example_carousel(),
        "section" => example_section(),
        "center" => example_center(),
        "form-layout" => example_form_layout(),
        "layout" => example_layout(),
        "scrollable-area" => example_scrollable_area(),
        "divider" => example_divider(),
        "stack" | "stack-item" => example_stack(),
        "grid" => example_grid(),
        "aspect-ratio" => example_aspect_ratio(),
        "breadcrumbs" | "breadcrumb-item" => example_breadcrumbs(),
        "pagination" => example_pagination(),
        "metadata-list" | "metadata-list-item" => example_metadata_list(),
        "tree-list" => example_tree_list(),
        "nav-icon" => example_nav_icon(),
        "overflow-list" => example_overflow_list(),
        "chat" => example_chat(),
        "chat-dictation-button" => example_chat_dictation_button(),
        "chat-tool-calls" => example_chat_tool_calls(),
        "table" => example_table(),
        "resizable" | "resize-handle" => example_resizable(),
        _ => return None,
    })
}

fn example_collapsible() -> Node {
    specimen(
        "Open by default",
        kivact::ui::disclosure("Delivery details", signal(true))
            .animated(true)
            .content(|| text("Ships in two business days.")),
        "disclosure(\"Delivery details\", signal(true)).animated(true).content(details)",
    )
}

fn example_collapsible_group() -> Node {
    specimen(
        "One section open at a time",
        kivact::ui::disclosure_group(signal(Some("delivery")))
            .variant(kivact_ui_soft::DisclosureVariant {
                dividers: true,
                ..Default::default()
            })
            .panel("delivery", "Delivery", || {
                text("Ships in two business days.")
            })
            .panel("returns", "Returns", || {
                text("Returns accepted within 30 days.")
            }),
        "disclosure_group(open).variant(DisclosureVariant { dividers: true, ..Default::default() }).panel(key, title, content)",
    )
}

fn example_list() -> Node {
    specimen(
        "Workspace items",
        kivact::ui::item_list((
            kivact::ui::item_row("Design").description("Active project"),
            kivact::ui::item_row("Documentation").description("In review"),
            kivact::ui::item_row("Archive"),
        ))
        .style(style().width(240)),
        "kivact::ui::item_list((kivact::ui::item_row(\"Design\"), kivact::ui::item_row(\"Documentation\")))",
    )
}

fn example_card() -> Node {
    specimen(
        "Project surface",
        card(text("Project notes")),
        "card(text(\"Project notes\"))",
    )
}

fn example_clickable_card() -> Node {
    specimen(
        "Open project",
        card(text("Project notes"))
            .name("Open project")
            .on_press(|| {}),
        "card(content).name(\"Open project\").on_press(open_project)",
    )
}

fn example_selectable_card() -> Node {
    specimen(
        "Select project",
        card(text("Project notes"))
            .name("Select project")
            .selected(signal(false)),
        "card(content).name(\"Select project\").selected(selected)",
    )
}

fn example_carousel() -> Node {
    specimen(
        "Browse slides",
        kivact::ui::carousel(
            signal(0usize),
            ["Design", "Documentation", "Research", "Release"]
                .into_iter()
                .map(|title| card(text(title)).style(style().size(148, 72)))
                .collect::<Vec<_>>(),
        )
        .name("Project highlights")
        .mode(kivact::ui::CarouselMode::Scroll)
        .snap(kivact::interaction::Snap::Mandatory)
        .style(style().width(320)),
        "let slides = [\"Design\", \"Documentation\", \"Research\", \"Release\"]\n    .into_iter().map(|title| card(text(title)).style(style().size(148, 72))).collect::<Vec<_>>();\nkivact::ui::carousel(signal(0usize), slides).name(\"Project highlights\").mode(CarouselMode::Scroll).snap(Snap::Mandatory).style(style().width(320))",
    )
}

fn example_section() -> Node {
    specimen(
        "Section content",
        section(text("Workspace settings")),
        "section(text(\"Workspace settings\"))",
    )
}

fn example_center() -> Node {
    specimen(
        "Center content",
        center(text("Centered content")).style(style().size(240, 96)),
        "center(text(\"Centered content\"))",
    )
}

fn example_form_layout() -> Node {
    specimen(
        "Horizontal labels stack at window widths of 480 px or less",
        form_layout((
            form_row(
                text("Project"),
                kivact::ui::text_input("Project", signal("Design notes".to_owned())),
            ),
            form_row(
                text("Review owner"),
                kivact::ui::text_input("Review owner", signal("Avery Morgan".to_owned())),
            ),
        ))
        .direction(FormDirection::HorizontalLabels)
        .style(style().max_width(400)),
        r#"form_layout((
    form_row(text("Project"), text_input("Project", project)),
    form_row(text("Review owner"), text_input("Review owner", owner)),
))
.direction(FormDirection::HorizontalLabels)
.style(style().max_width(400))"#,
    )
}

fn example_layout() -> Node {
    specimen(
        "Header and content",
        layout()
            .header(layout_header(text("Project notes")).divider(true))
            .content(layout_content(text("Workspace content")))
            .style(style().width(320).height(144)),
        "layout().header(header).content(content)",
    )
}

fn example_scrollable_area() -> Node {
    specimen(
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
    )
}

fn example_divider() -> Node {
    specimen(
        "Separate related content",
        kivact::view::column((text("Details"), divider(), text("History"))),
        "divider()",
    )
}

fn example_stack() -> Node {
    specimen(
        "Horizontal stack",
        hstack((
            stack_item(text("Design")),
            stack_item(text("Documentation")),
        )),
        "hstack((stack_item(first), stack_item(second)))",
    )
}

fn example_grid() -> Node {
    specimen(
        "Two columns",
        layout_grid((card(text("Design")), card(text("Documentation"))))
            .columns(GridColumns::Fixed(2))
            .gap(Space::S2),
        "layout_grid(children).columns(GridColumns::Fixed(2)).gap(Space::S2)",
    )
}

fn example_aspect_ratio() -> Node {
    specimen(
        "Wide preview",
        aspect_ratio(16.0 / 9.0, card(text("Preview"))).style(style().width(240)),
        "aspect_ratio(16.0 / 9.0, preview)",
    )
}

fn example_breadcrumbs() -> Node {
    specimen(
        "Project location",
        kivact::ui::breadcrumb(vec![
            kivact::interaction::ListItem::new(0, "Workspace"),
            kivact::interaction::ListItem::new(1, "Design notes"),
        ])
        .on_activate(|_| {}),
        "breadcrumb(vec![ListItem::new(0, \"Workspace\"), ListItem::new(1, \"Design notes\")])",
    )
}

fn example_pagination() -> Node {
    specimen(
        "Navigate result pages",
        kivact::ui::pagination(12usize, signal(0usize))
            .window(kivact::ui::PaginationWindow::Buffered)
            .first_last(true)
            .summary(kivact_ui_soft::pagination_status),
        "pagination(12usize, page).window(PaginationWindow::Buffered).first_last(true).summary(soft::pagination_status)",
    )
}

fn example_metadata_list() -> Node {
    specimen(
        "Document properties",
        kivact::ui::description_list((
            kivact::ui::description("Owner", text("Avery Morgan")),
            kivact::ui::description("Status", text("In review")),
        )),
        "kivact::ui::description_list((kivact::ui::description(\"Owner\", text(\"Avery Morgan\")), status))",
    )
}

fn example_tree_list() -> Node {
    specimen(
        "Expandable files",
        kivact::ui::tree(
            "Files",
            vec![
                TreeNode::new("src", None, "src"),
                TreeNode::new("main.rs", Some("src"), "main.rs"),
                TreeNode::new("README.md", None, "README.md"),
            ],
            signal(None),
        )
        .expanded(signal(vec!["src"]))
        .reveal(
            2.0,
            kivact::motion::Transition::tween(
                734.77,
                kivact::motion::Easing::CubicBezier(0.327, 0.694, 0.116, 1.0),
            ),
        )
        .style(style().width(240)),
        "kivact::ui::tree(\"Files\", files, selected).expanded(expanded).reveal(2.0, transition)",
    )
}

fn example_nav_icon() -> Node {
    specimen(
        "Navigation icon",
        row((
            nav_icon(kivact_ui_soft::icon_view("columns-2")),
            text("Documents"),
        ))
        .style(style().gap(8)),
        "nav_icon(icon(\"columns-2\"))",
    )
}

fn example_overflow_list() -> Node {
    specimen(
        "Items in a narrow space",
        overflow_list(
            vec!["Design", "Engineering", "Documentation", "Support"],
            |item| *item,
            |item| text(move || item.get().to_owned()),
        )
        .style(style().width(230)),
        "overflow_list(teams, |item| *item, render_team)",
    )
}

fn example_chat() -> Node {
    specimen(
        "Messages and composer",
        kivact::ui::chat_thread(kivact::view::column((
            kivact::ui::chat_message("")
                .sender(kivact::ui::ChatSender::System)
                .content(text("Conversation started")),
            kivact::ui::chat_message("You")
                .sender(kivact::ui::ChatSender::User)
                .content(text("Can you review the notes?")),
            kivact::ui::chat_message("Assistant").content(text("The notes are ready.")),
        )))
        .composer(kivact::ui::chat_composer(signal(String::new())))
        .style(style().width(440).height(300)),
        "chat_thread(messages).composer(chat_composer(draft))",
    )
}

fn example_chat_dictation_button() -> Node {
    specimen(
        "Dictation available",
        chat_dictation_button(signal(DictationState::Idle)),
        "chat_dictation_button(dictation_state)",
    )
}

fn example_chat_tool_calls() -> Node {
    specimen(
        "Tool activity",
        chat_tool_calls(vec![
            ToolCall::new("read", "Read notes").target("notes.md"),
            ToolCall::new("review", "Review notes").status(ToolStatus::Running),
        ])
        .expanded(signal(true)),
        "chat_tool_calls(calls).expanded(expanded)",
    )
}

fn example_table() -> Node {
    let controls = table_controls();
    let pinned = table_pinned();
    let folders = table_folders();
    let static_table = table_static();
    column((static_table, controls, pinned, folders)).style(style().gap(12))
}

fn table_controls() -> Node {
    let filters = signal(Vec::<(usize, String)>::new());
    let expanded = signal(Vec::new());
    let sort = signal(Vec::new());
    let typed = signal(std::collections::HashMap::new());
    let table = data_table(
        vec![
            ("Design", "Active"),
            ("Documentation", "Review"),
            ("Archive", "Closed"),
        ],
        |row| row.0,
        vec![
            DataColumn::new("Project", |row: &(&str, &str)| row.0.to_owned())
                .sortable()
                .flex(2.0),
            DataColumn::new("Status", |row: &(&str, &str)| row.1.to_owned())
                .sortable()
                .filter(|row, query| row.1 == query)
                .filter_field(
                    FilterField::Choice(
                        ["Active", "Review", "Closed"]
                            .map(|value| (value.to_owned(), value.to_owned()))
                            .into(),
                    ),
                    |row, value| matches!(value, FilterValue::Choice(choice) if row.1 == choice),
                )
                .flex(1.0),
        ],
    )
    .multi_sort(sort)
    .column_filters(filters)
    .typed_column_filters(typed)
    .row_details(|item| text(move || item.with(|row| format!("{} is {}", row.0, row.1))))
    .expanded_rows(expanded)
    .style(style().width(440));
    specimen(
        "Sort, filter, and expand rows",
        column((
            row((
                button("Show active projects")
                    .on_press(move || filters.set(vec![(1, "Active".into())])),
                button("Clear filter").on_press(move || filters.set(Vec::new())),
            ))
            .style(style().gap(8)),
            table,
        ))
        .style(style().gap(8)),
        "data_table(rows, |row| row.0, columns)\n    .multi_sort(sort)\n    .typed_column_filters(typed)\n    .row_details(render_details)\n    .expanded_rows(expanded)",
    )
}

fn table_pinned() -> Node {
    specimen(
        "Compact table with pinned edges",
        data_table(
            vec![
                ["Design", "Kyoto", "Avery", "Active"],
                ["Research", "Tokyo", "Morgan", "Review"],
            ],
            |row| row[0],
            ["Workspace", "Office", "Owner", "State"]
                .into_iter()
                .enumerate()
                .map(|(index, label)| {
                    DataColumn::new(label, move |row: &[&str; 4]| row[index].to_owned())
                        .width(116.0)
                })
                .collect(),
        )
        .sticky_columns(vec![0], vec![3])
        .density(DataTableDensity::Compact)
        .style(style().width(350)),
        "data_table(rows, |row| row[0], columns)\n    .sticky_columns(vec![0], vec![3])\n    .density(DataTableDensity::Compact)",
    )
}

fn table_folders() -> Node {
    specimen(
        "Folder hierarchy",
        data_table(
            vec![
                (1, None, "Planning"),
                (2, Some(1), "Brief"),
                (3, None, "Delivery"),
            ],
            |row| row.0,
            vec![DataColumn::new(
                "Folder",
                |row: &(u32, Option<u32>, &str)| row.2.to_owned(),
            )],
        )
        .tree(|row| row.1, signal(vec![1]))
        .expand_all_control()
        .style(style().width(350)),
        "data_table(rows, |row| row.0, columns)\n    .tree(|row| row.1, expanded)\n    .expand_all_control()",
    )
}

fn table_static() -> Node {
    specimen(
        "Static rows with Soft appearance",
        kivact::ui::table((
            kivact::ui::table_header((kivact::ui::table_row((
                kivact::ui::table_head(text("Project")),
                kivact::ui::table_head(text("Status")),
            )),)),
            kivact::ui::table_body((kivact::ui::table_row((
                kivact::ui::table_cell(text("Design")),
                kivact::ui::table_cell(kivact_ui_soft::table_row_status(
                    "Passed",
                    kivact_ui_soft::RowStatusTone::Success,
                )),
            ))
            .variant(kivact_ui_soft::TableRowAppearance::new(true, true, false)),)),
        )),
        "table((table_header(rows), table_body(rows)))\n    // table_row(...).variant(TableRowAppearance::new(true, true, false))",
    )
}

fn example_resizable() -> Node {
    {
        let width = signal(144.0);
        specimen(
            "Drag or use the arrow keys. Enter toggles collapse.",
            (
                kivact::ui::splitter(
                    "Navigation width",
                    width,
                    kivact::interaction::ScalarRange::new(88.0, 216.0)
                        .expect("invariant: ordered panel bounds"),
                )
                .collapsible(0.2)
                .snaps(vec![0.0, 0.4375, 1.0])
                .start(text("Navigation"))
                .end(text("Content"))
                .style(style().width(360).height(92)),
                text(move || format!("Navigation width: {:.0} px", width.get())),
            ),
            r#"let width = signal(144.0);
kivact::ui::splitter("Navigation width", width,
    ScalarRange::new(88.0, 216.0).expect("ordered bounds"))
    .collapsible(0.2).snaps(vec![0.0, 0.4375, 1.0])
    .start(text("Navigation")).end(text("Content"))"#,
        )
    }
}
