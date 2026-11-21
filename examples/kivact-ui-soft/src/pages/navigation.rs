//! Demonstrate responsive navigation and application shells.
use super::labelled;
use kivact::interaction::ListItem;
use kivact::state::signal;
use kivact::style::style;
use kivact::ui::button;
use kivact::ui::{
    SidebarCollapsible, TabOverflow, sidebar, sidebar_group, sidebar_menu, sidebar_menu_button,
    sidebar_provider, sidebar_trigger, tabs,
};
use kivact::view::IntoView;
use kivact::view::node_ref;
use kivact::view::{Element, Node, column, icon, row, text};
use kivact_ui_soft::{
    OutlineItem, app_shell, mobile_nav, nav_heading_menu, nav_heading_menu_item, outline,
    scrollable_area, top_nav, top_nav_featured_card, top_nav_heading, top_nav_item,
    top_nav_mega_menu, top_nav_mega_menu_item, top_nav_menu, top_nav_menu_item,
};

// example: app-shell
pub(super) fn example_app_shell() -> Node {
    labelled(
        "Application frame",
        app_shell(text("Project notes"))
            .top_nav(|| top_nav().heading(kivact::custom::scope(|| top_nav_heading("Workspace"))))
            .sidebar(|| sidebar_menu(vec![sidebar_menu_button("Documents").into_view()]))
            .style(style().width(520).height(216)),
    )
}
// end example

// example: mobile-nav
pub(super) fn example_mobile_nav() -> Node {
    {
        let open = signal(false);
        labelled(
            "Open navigation drawer",
            (
                button("Open navigation").on_press(move || open.set(true)),
                mobile_nav(|| sidebar_menu(vec![sidebar_menu_button("Documents").into_view()]))
                    .open(open),
            ),
        )
    }
}
// end example

// example: side-nav
pub(super) fn example_sidebar() -> Node {
    {
        let selected = signal("Documents");
        let open = signal(true);
        let menu = sidebar().content(
            sidebar_group("Workspace").content(
                sidebar_menu(vec![
                    sidebar_menu_button("Documents")
                        .icon(icon("columns-2"))
                        .active(move || selected.get() == "Documents")
                        .on_press(move || selected.set("Documents"))
                        .into_view(),
                    sidebar_menu_button("Activity")
                        .icon(icon("clock"))
                        .sub(sidebar_menu(vec![
                            sidebar_menu_button("Recent activity")
                                .active(move || selected.get() == "Recent activity")
                                .on_press(move || selected.set("Recent activity"))
                                .into_view(),
                        ]))
                        .into_view(),
                ])
                .into_view(),
            ),
        );
        labelled(
            "Select a page or collapse to the icon rail",
            (sidebar_provider(
                open,
                menu.into_view(),
                column((
                    sidebar_trigger(),
                    text(move || format!("Current page: {}", selected.get())),
                ))
                .into_view(),
            )
            .collapsible(SidebarCollapsible::Icon)
            .style(style().height(180)),),
        )
    }
}
// end example

// example: top-nav
pub(super) fn example_top_nav() -> Node {
    {
        let selected = signal("Documents");
        labelled(
            "Navigate with links and menus",
            (
                top_nav()
                    .heading(kivact::custom::scope(|| top_nav_heading("Workspace")))
                    .start(kivact::custom::scope(move || {
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
                                    top_nav_featured_card("Start here")
                                        .description("Find onboarding material and reference docs.")
                                }),
                        )
                    })),
                text(move || format!("Current page: {}", selected.get())),
            ),
        )
    }
}
// end example

// example: nav-menu
pub(super) fn example_nav_menu() -> Node {
    labelled(
        "Workspace switcher",
        top_nav_heading("Workspace").menu({
            nav_heading_menu([
                nav_heading_menu_item("Design"),
                nav_heading_menu_item("Engineering"),
            ])
        }),
    )
}
// end example

// example: outline
pub(super) fn example_outline() -> Node {
    {
        let viewport = node_ref();
        let content = node_ref();
        let first = node_ref();
        let second = node_ref();
        let third = node_ref();
        let heading = |title, detail, reference| {
            column((
                text(title)
                    .style(style().font_weight(600))
                    .with_node(|node| node.node_ref(reference)),
                text(detail),
            ))
            .style(style().gap(8).height(130).flex_shrink(0))
        };
        let document = column((
            heading("Summary", "A snapshot of the project.", first),
            heading("Activity", "Recent changes and decisions.", second),
            heading("Next steps", "Work planned for the next review.", third),
        ))
        .with_node(|node| node.node_ref(content));
        labelled(
            "Scroll the article or jump to a heading",
            row((
                outline(vec![
                    OutlineItem::new("summary", "Summary", 1, first),
                    OutlineItem::new("activity", "Activity", 3, second),
                    OutlineItem::new("next", "Next steps", 2, third),
                ])
                .scroll_container(viewport)
                .scroll_content(content)
                .style(style().width(144).flex_shrink(0)),
                scrollable_area("Project article", document)
                    .with_node(|node| node.node_ref(viewport))
                    .style(style().height(144).flex_grow(1).min_width(0)),
            ))
            .style(style().gap(16)),
        )
    }
}
// end example

// example: stepper
// example: step
pub(super) fn example_stepper() -> Node {
    labelled(
        "Review steps",
        kivact::ui::stepper(
            vec![
                kivact::ui::StepperItem::new("Draft"),
                kivact::ui::StepperItem::new("Review"),
                kivact::ui::StepperItem::new("Publish"),
            ],
            signal(1usize),
        ),
    )
}
// end example

// example: tab-list
pub(super) fn example_tab_list() -> Node {
    labelled(
        "Manual keyboard activation",
        tabs(
            "Project views",
            vec![
                ListItem::new("summary", "Summary"),
                ListItem::new("activity", "Activity"),
            ],
            signal("summary"),
        )
        .activation(kivact::interaction::ListActivation::Manual)
        .content(|key| text(format!("Selected view: {key}"))),
    )
}
// end example

// example: tab-menu
pub(super) fn example_tab_menu() -> Node {
    labelled(
        "Additional views",
        tabs(
            "Project views",
            vec![
                ListItem::new("summary", "Summary"),
                ListItem::new("activity", "Activity"),
                ListItem::new("history", "History"),
            ],
            signal("summary"),
        )
        .overflow(TabOverflow::Menu)
        .content(|key| text(format!("Selected view: {key}"))),
    )
}
// end example
