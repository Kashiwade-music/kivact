//! Demonstrate Soft layout, collection, table and chat components.
use super::labelled;
use kivact::interaction::FilterValue;
use kivact::interaction::TreeNode;
use kivact::state::signal;
use kivact::style::style;
use kivact::ui::{DataColumn, DataTableDensity, FilterField, button, card, data_table};
use kivact::view::{Element, Node, column, row, text};
use kivact_ui_soft::{
    DictationState, FormDirection, GridColumns, Space, ToolCall, ToolStatus, center,
    chat_dictation_button, chat_tool_calls, divider, form_layout, form_row, hstack, layout,
    layout_content, layout_grid, layout_header, nav_icon, overflow_list, scrollable_area, section,
    stack_item,
};

// example: collapsible
pub(super) fn example_collapsible() -> Node {
    labelled(
        "Open by default",
        kivact::ui::disclosure("Delivery details", signal(true))
            .animated(true)
            .content(|| text("Ships in two business days.")),
    )
}
// end example

// example: list
// example: item
pub(super) fn example_list() -> Node {
    labelled(
        "Workspace items",
        kivact::ui::item_list((
            kivact::ui::item_row("Design").description("Active project"),
            kivact::ui::item_row("Documentation").description("In review"),
            kivact::ui::item_row("Archive"),
        ))
        .style(style().width(240)),
    )
}
// end example

// example: card
pub(super) fn example_card() -> Node {
    labelled("Project surface", card(text("Project notes")))
}
// end example

// example: clickable-card
pub(super) fn example_clickable_card() -> Node {
    labelled(
        "Open project",
        card(text("Project notes"))
            .name("Open project")
            .on_press(|| {}),
    )
}
// end example

// example: selectable-card
pub(super) fn example_selectable_card() -> Node {
    labelled(
        "Select project",
        card(text("Project notes"))
            .name("Select project")
            .selected(signal(false)),
    )
}
// end example

// example: carousel
pub(super) fn example_carousel() -> Node {
    labelled(
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
    )
}
// end example

// example: section
pub(super) fn example_section() -> Node {
    labelled("Section content", section(text("Workspace settings")))
}
// end example

// example: center
pub(super) fn example_center() -> Node {
    labelled(
        "Center content",
        center(text("Centered content")).style(style().size(240, 96)),
    )
}
// end example

// example: form-layout
pub(super) fn example_form_layout() -> Node {
    labelled(
        "Horizontal labels stack at window widths of 480 px or less",
        form_layout((
            form_row(
                text("Project"),
                kivact::ui::text_input(signal("Design notes".to_owned())).label("Project"),
            ),
            form_row(
                text("Review owner"),
                kivact::ui::text_input(signal("Avery Morgan".to_owned())).label("Review owner"),
            ),
        ))
        .direction(FormDirection::HorizontalLabels)
        .style(style().max_width(400)),
    )
}
// end example

// example: layout
pub(super) fn example_layout() -> Node {
    labelled(
        "Header and content",
        layout()
            .header(layout_header(text("Project notes")).divider(true))
            .content(layout_content(text("Workspace content")))
            .style(style().width(320).height(144)),
    )
}
// end example

// example: scrollable-area
pub(super) fn example_scrollable_area() -> Node {
    labelled(
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
    )
}
// end example

// example: divider
pub(super) fn example_divider() -> Node {
    labelled(
        "Separate related content",
        kivact::view::column((text("Details"), divider(), text("History"))),
    )
}
// end example

// example: stack
pub(super) fn example_stack() -> Node {
    labelled(
        "Horizontal stack",
        hstack((
            stack_item(text("Design")),
            stack_item(text("Documentation")),
        )),
    )
}
// end example

// example: grid
pub(super) fn example_grid() -> Node {
    labelled(
        "Two columns",
        layout_grid((card(text("Design")), card(text("Documentation"))))
            .columns(GridColumns::Fixed(2))
            .gap(Space::S2),
    )
}
// end example

// example: aspect-ratio
pub(super) fn example_aspect_ratio() -> Node {
    labelled(
        "Wide preview",
        card(text("Preview")).style(style().width(240).aspect_ratio(16.0 / 9.0)),
    )
}
// end example

// example: breadcrumbs
pub(super) fn example_breadcrumbs() -> Node {
    labelled(
        "Project location",
        kivact::ui::breadcrumb(vec![
            kivact::interaction::ListItem::new(0, "Workspace"),
            kivact::interaction::ListItem::new(1, "Design notes"),
        ])
        .on_activate(|_| {}),
    )
}
// end example

// example: pagination
pub(super) fn example_pagination() -> Node {
    labelled(
        "Navigate result pages",
        kivact::ui::pagination(12usize, signal(0usize))
            .window(kivact::ui::PaginationWindow::Buffered)
            .first_last(true)
            .summary(kivact_ui_soft::pagination_status),
    )
}
// end example

// example: metadata-list
pub(super) fn example_metadata_list() -> Node {
    labelled(
        "Document properties",
        kivact::ui::description_list((
            kivact::ui::description("Owner", text("Avery Morgan")),
            kivact::ui::description("Status", text("In review")),
        )),
    )
}
// end example

// example: tree-list
pub(super) fn example_tree_list() -> Node {
    labelled(
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
        .reveal(kivact::motion::Transition::tween(
            734.77,
            kivact::motion::Easing::CubicBezier(0.327, 0.694, 0.116, 1.0),
        ))
        .reveal_gap(2.0)
        .style(style().width(240)),
    )
}
// end example

// example: nav-icon
pub(super) fn example_nav_icon() -> Node {
    labelled(
        "Navigation icon",
        row((
            nav_icon(kivact_ui_soft::icon_view("columns-2")),
            text("Documents"),
        ))
        .style(style().gap(8)),
    )
}
// end example

// example: overflow-list
pub(super) fn example_overflow_list() -> Node {
    labelled(
        "Items in a narrow space",
        overflow_list(
            vec!["Design", "Engineering", "Documentation", "Support"],
            |item| *item,
            |item| text(move || item.get().to_owned()),
        )
        .style(style().width(230)),
    )
}
// end example

// example: chat
pub(super) fn example_chat() -> Node {
    labelled(
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
    )
}
// end example

// example: chat-dictation-button
pub(super) fn example_chat_dictation_button() -> Node {
    labelled(
        "Dictation available",
        chat_dictation_button(signal(DictationState::Idle)),
    )
}
// end example

// example: chat-tool-calls
pub(super) fn example_chat_tool_calls() -> Node {
    labelled(
        "Tool activity",
        chat_tool_calls(vec![
            ToolCall::new("read", "Read notes").target("notes.md"),
            ToolCall::new("review", "Review notes").status(ToolStatus::Running),
        ])
        .expanded(signal(true)),
    )
}
// end example

// example: table
pub(super) fn example_table() -> Node {
    let controls = table_controls();
    let pinned = table_pinned();
    let folders = table_folders();
    let static_table = table_static();
    column((static_table, controls, pinned, folders)).style(style().gap(12))
}
// end example

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
    labelled(
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
    )
}

fn table_pinned() -> Node {
    labelled(
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
    )
}

fn table_folders() -> Node {
    labelled(
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
        .style(style().width(350)),
    )
}

fn table_static() -> Node {
    labelled(
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
            .variant(kivact_ui_soft::TableRowDividers(true))
            .variant(kivact_ui_soft::TableRowHover(true)),)),
        )),
    )
}

// example: resizable
pub(super) fn example_resizable() -> Node {
    {
        let width = signal(144.0);
        labelled(
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
        )
    }
}
// end example
