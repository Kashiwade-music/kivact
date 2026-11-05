//! Role: Demonstrate responsive navigation and application shells.
use super::specimen::specimen;
use kivact::state::signal;
use kivact::style::style;
use kivact::ui::button;
use kivact::view::node_ref;
use kivact::view::{Node, NodeBuilder, column, icon, row, text};
use kivact_ui_soft::{
    OutlineItem, SideNavCollapse, TabMenuOption, TabPattern, app_shell, mobile_nav,
    nav_heading_menu, nav_heading_menu_item, outline, scrollable_area, side_nav, side_nav_item,
    side_nav_section, tab, tab_list, tab_menu, top_nav, top_nav_featured_card, top_nav_heading,
    top_nav_item, top_nav_mega_menu, top_nav_mega_menu_item, top_nav_menu, top_nav_menu_item,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "app-shell" => specimen(
            "Application frame",
            app_shell(text("Project notes"))
                .top_nav(|| top_nav().heading(|| top_nav_heading("Workspace")))
                .side_nav(|| side_nav(side_nav_item("Documents")))
                .style(style().width(520).height(216)),
            "app_shell(content).top_nav(top_navigation).side_nav(side_navigation)",
        ),
        "mobile-nav" => {
            let open = signal(false);
            specimen(
                "Open navigation drawer",
                (
                    button("Open navigation").on_press(move || open.set(true)),
                    mobile_nav(|| side_nav(side_nav_item("Documents"))).open(open),
                ),
                "mobile_nav(|| side_nav(items)).open(open)",
            )
        }
        "side-nav" => {
            let selected = signal("Documents");
            specimen(
                "Select a page or collapse to the icon rail",
                (
                    side_nav(side_nav_section(
                        "Workspace",
                        (
                            side_nav_item("Documents")
                                .icon(icon("columns-2"))
                                .with_node(|node| {
                                    node.selected(move || selected.get() == "Documents")
                                })
                                .on_press(move || selected.set("Documents")),
                            side_nav_item("Activity")
                                .icon(icon("clock"))
                                .children(move || {
                                    side_nav_item("Recent activity")
                                        .with_node(|node| {
                                            node.selected(move || {
                                                selected.get() == "Recent activity"
                                            })
                                        })
                                        .on_press(move || selected.set("Recent activity"))
                                }),
                        ),
                    ))
                    .collapsible(SideNavCollapse::Uncontrolled { default: false })
                    .style(style().height(180)),
                    text(move || format!("Current page: {}", selected.get())),
                ),
                r#"let selected = signal("Documents");
let documents = side_nav_item("Documents")
    .icon(icon("columns-2"))
    .with_node(|node| node.selected(move || selected.get() == "Documents"))
    .on_press(move || selected.set("Documents"));
let activity = side_nav_item("Activity").icon(icon("clock")).children(move || {
    side_nav_item("Recent activity")
        .with_node(|node| node.selected(move || selected.get() == "Recent activity"))
        .on_press(move || selected.set("Recent activity"))
});
side_nav(side_nav_section("Workspace", (documents, activity)))
    .collapsible(SideNavCollapse::Uncontrolled { default: false })
    .style(style().height(180))"#,
            )
        }
        "top-nav" => {
            let selected = signal("Documents");
            specimen(
                "Navigate with links and menus",
                (
                    top_nav()
                        .heading(|| top_nav_heading("Workspace"))
                        .start(move || {
                            (
                                top_nav_item("Documents")
                                    .with_node(|node| {
                                        node.selected(move || selected.get() == "Documents")
                                    })
                                    .on_press(move || selected.set("Documents")),
                                top_nav_menu(
                                    "Activity",
                                    [top_nav_menu_item("Recent updates")
                                        .on_press(move || selected.set("Recent updates"))],
                                ),
                                top_nav_mega_menu("Resources")
                                    .items(move || {
                                        top_nav_mega_menu_item("Guides")
                                            .description("Learn how the workspace works")
                                            .on_press(move || selected.set("Guides"))
                                    })
                                    .featured(|| {
                                        top_nav_featured_card("Start here").description(
                                            "Find onboarding material and reference docs.",
                                        )
                                    }),
                            )
                        }),
                    text(move || format!("Current page: {}", selected.get())),
                ),
                r#"top_nav().heading(|| top_nav_heading("Workspace")).start(move || (
    top_nav_item("Documents")
        .with_node(|node| node.selected(move || selected.get() == "Documents"))
        .on_press(move || selected.set("Documents")),
    top_nav_menu("Activity", [
        top_nav_menu_item("Recent updates").on_press(move || selected.set("Recent updates")),
    ]),
    top_nav_mega_menu("Resources")
        .items(move || top_nav_mega_menu_item("Guides")
            .description("Learn how the workspace works")
            .on_press(move || selected.set("Guides")))
        .featured(|| top_nav_featured_card("Start here")
            .description("Find onboarding material and reference docs.")),
))"#,
            )
        }
        "nav-menu" => specimen(
            "Workspace switcher",
            top_nav_heading("Workspace").menu({
                nav_heading_menu([
                    nav_heading_menu_item("Design"),
                    nav_heading_menu_item("Engineering"),
                ])
            }),
            "top_nav_heading(\"Workspace\").menu(|| nav_heading_menu(items))",
        ),
        "outline" => {
            let viewport = node_ref();
            let content = node_ref();
            let first = node_ref();
            let second = node_ref();
            let third = node_ref();
            let heading = |title, detail, reference| {
                column((
                    text(title)
                        .style(style().weight(600))
                        .with_node(|node| node.node_ref(reference)),
                    text(detail),
                ))
                .style(style().gap(8).height(130).shrink(0))
            };
            let document = column((
                heading("Summary", "A snapshot of the project.", first),
                heading("Activity", "Recent changes and decisions.", second),
                heading("Next steps", "Work planned for the next review.", third),
            ))
            .with_node(|node| node.node_ref(content));
            specimen(
                "Scroll the article or jump to a heading",
                row((
                    outline(vec![
                        OutlineItem::new("summary", "Summary", 1, first),
                        OutlineItem::new("activity", "Activity", 3, second),
                        OutlineItem::new("next", "Next steps", 2, third),
                    ])
                    .scroll_container(viewport)
                    .scroll_content(content)
                    .style(style().width(144).shrink(0)),
                    scrollable_area("Project article", document)
                        .with_node(|node| node.node_ref(viewport))
                        .style(style().height(144).grow(1).min_width(0)),
                ))
                .style(style().gap(16)),
                r#"let viewport = node_ref();
let content = node_ref();
let document = column((summary, activity, next_steps)).with_node(|node| node.node_ref(content));
row((
    outline(items).scroll_container(viewport).scroll_content(content),
    scrollable_area("Project article", document)
        .with_node(|node| node.node_ref(viewport)).style(style().height(144)),
))"#,
            )
        }
        "stepper" | "step" => specimen(
            "Review steps",
            kivact::ui::stepper(
                vec![
                    kivact::ui::StepperItem::new("Draft"),
                    kivact::ui::StepperItem::new("Review"),
                    kivact::ui::StepperItem::new("Publish"),
                ],
                1usize,
            ),
            "stepper(vec![StepperItem::new(\"Draft\"), StepperItem::new(\"Review\"), StepperItem::new(\"Publish\")], 1usize)",
        ),
        "tab-list" | "tab" => specimen(
            "Manual keyboard activation",
            tab_list("Project views", signal("summary"))
                .pattern(TabPattern::Tabs)
                .tabs([tab("summary", "Summary"), tab("activity", "Activity")])
                .content(|key| text(format!("Selected view: {key}"))),
            "tab_list(\"Project views\", selected).pattern(TabPattern::Tabs).tabs(views)",
        ),
        "tab-menu" => specimen(
            "Additional views",
            tab_list("Project views", signal("summary"))
                .tab(tab("summary", "Summary"))
                .menu(tab_menu(
                    "More",
                    vec![
                        TabMenuOption::new("activity", "Activity"),
                        TabMenuOption::new("history", "History"),
                    ],
                )),
            "tab_list(\"Project views\", selected).menu(tab_menu(\"More\", views))",
        ),
        _ => return None,
    })
}
