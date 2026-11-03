//! Role: Demonstrate responsive navigation and application shells.
use super::specimen::specimen;
use kivact::control::{Node, NodeBuilder, column, row, text};
use kivact::signal;
use kivact::style::style;
use kivact::ui::button;
use kivact::view::node_ref;
use kivact_ui_soft::{
    OutlineItem, TabMenuOption, TabPattern, app_shell, mobile_nav, nav_heading_menu,
    nav_heading_menu_item, outline, side_nav, side_nav_item, side_nav_section, step, stepper, tab,
    tab_list, tab_menu, top_nav, top_nav_heading, top_nav_item,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "app-shell" => specimen(
            "Application frame",
            app_shell(|| text("Project notes"))
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
        "side-nav" => specimen(
            "Workspace sections",
            side_nav(side_nav_section(
                "Workspace",
                (side_nav_item("Documents"), side_nav_item("Activity")),
            ))
            .style(style().width(240).height(180)),
            "side_nav(side_nav_section(\"Workspace\", items))",
        ),
        "top-nav" => specimen(
            "Top navigation",
            top_nav()
                .heading(|| top_nav_heading("Workspace"))
                .start(|| (top_nav_item("Documents"), top_nav_item("Activity"))),
            "top_nav().heading(heading).start(items)",
        ),
        "nav-menu" => specimen(
            "Workspace switcher",
            top_nav_heading("Workspace").menu(|| {
                nav_heading_menu([
                    nav_heading_menu_item("Design"),
                    nav_heading_menu_item("Engineering"),
                ])
            }),
            "top_nav_heading(\"Workspace\").menu(|| nav_heading_menu(items))",
        ),
        "outline" => {
            let first = node_ref();
            let second = node_ref();
            specimen(
                "Document outline",
                row((
                    outline(vec![
                        OutlineItem::new("summary", "Summary", 1, first),
                        OutlineItem::new("activity", "Activity", 2, second),
                    ]),
                    column((
                        text("Summary").node_ref(first),
                        text("Activity").node_ref(second),
                    )),
                ))
                .style(style().gap(16)),
                "outline(vec![OutlineItem::new(\"summary\", \"Summary\", 1, summary_ref)])",
            )
        }
        "stepper" | "step" => specimen(
            "Review steps",
            stepper(2, [step(1, "Draft"), step(2, "Review"), step(3, "Publish")]),
            "stepper(active, [step(1, \"Draft\"), step(2, \"Review\"), step(3, \"Publish\")])",
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
