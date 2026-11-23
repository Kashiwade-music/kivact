//! Catalog page of Sidebar: groups, icon mode, sub-menus and frames.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Orientation;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("sidebar", "Sidebar", "Components", include_str!("sidebar.rs"))
        .description("A collapsible side area that holds navigation. Pass a binding for the open state, the sidebar and the next page to sidebar_provider. It opens and closes with sidebar_trigger, the rail at the edge, and Ctrl+B (Cmd+B on Mac); the demo keeps everything inside its frame. To use Ctrl+B anywhere in the window, make the provider the root view of the app.")
        .usage("use kivact::prelude::*;\nuse kivact::style::use_theme;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass views to the header, content and footer of the sidebar. sidebar_group makes a group and sidebar_menu_button makes a row, which can take active, badge and icon. The next page holds a heading row and a dashboard frame.",
            basic,
        ))
        .example(Example::new(
            "icon",
            "Collapsible icon",
            "Passing Icon to collapsible makes it a 48 px column of icons when closed. Pointing at a closed row shows its label as a tooltip.",
            icon,
        ))
        .example(Example::new(
            "submenu",
            "Sub menu and action",
            "Passing sidebar_menu_sub_button as sub lists the child rows beside a rule. action is an operation placed at the end of a row.",
            submenu,
        ))
        .example(Example::new(
            "floating",
            "Floating on the right",
            "Pass Floating as variant and Right as side, and a bordered panel 8 px off the edge attaches on the right. rail is a thin strip at the edge that opens and closes it when pressed.",
            floating,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    /// A bounded frame for a sidebar demo.
    fn frame(provider: impl kivact::view::Element) -> impl kivact::view::Element {
        column(provider).style(
            style()
                .width(580)
                .height(340)
                .border(1, basic::BORDER)
                .padding(1),
        )
    }
    /// The page beside the sidebar: a header row and a dashboard sketch.
    fn page_area(title: &'static str) -> impl IntoView {
        let tile = || kivact::ui::skeleton().style(style().flex_grow(1).height(56));
        column((
            row((
                kivact::ui::sidebar_trigger(),
                basic::separator()
                    .orientation(Orientation::Vertical)
                    .style(style().height(16)),
                kivact::ui::breadcrumb(vec![
                    kivact::interaction::ListItem::new("root", "Dashboard"),
                    kivact::interaction::ListItem::new("current", title),
                ])
                .separator(|| kivact::view::icon("chevron-right").style(style().size(14, 14))),
            ))
            .style(
                style()
                    .height(48)
                    .padding_x(12)
                    .gap(8)
                    .align_items(Align::Center),
            ),
            column((
                row((tile(), tile(), tile())).style(style().gap(12)),
                kivact::ui::skeleton().style(style().flex_grow(1).min_height(80)),
            ))
            .style(style().flex_grow(1).padding(12).padding_top(0).gap(12)),
        ))
        .style(style().flex_grow(1))
    }
    let open = signal(true);
    frame(
        kivact::ui::sidebar_provider(
            open,
            kivact::ui::sidebar()
                .header(basic::large("Acme Inc"))
                .content({
                    (
                        kivact::ui::sidebar_group("Application").content({
                            kivact::ui::sidebar_menu((
                                kivact::ui::sidebar_menu_button("Home")
                                    .icon(kivact::view::icon("house"))
                                    .active(true),
                                kivact::ui::sidebar_menu_button("Inbox")
                                    .icon(kivact::view::icon("inbox"))
                                    .badge("12"),
                                kivact::ui::sidebar_menu_button("Calendar")
                                    .icon(kivact::view::icon("calendar")),
                            ))
                        }),
                        kivact::ui::sidebar_separator(),
                        kivact::ui::sidebar_group("Settings").content({
                            kivact::ui::sidebar_menu((kivact::ui::sidebar_menu_button(
                                "Preferences",
                            )
                            .icon(kivact::view::icon("settings")),))
                        }),
                    )
                }),
            page_area("Overview"),
        )
        .collapsible(kivact::ui::SidebarCollapsible::None),
    )
}
// end example

// example: icon
fn icon() -> impl IntoView {
    /// A bounded frame for a sidebar demo.
    fn frame(provider: impl kivact::view::Element) -> impl kivact::view::Element {
        column(provider).style(
            style()
                .width(580)
                .height(340)
                .border(1, basic::BORDER)
                .padding(1),
        )
    }
    /// The page beside the sidebar: a header row and a dashboard sketch.
    fn page_area(title: &'static str) -> impl IntoView {
        let tile = || kivact::ui::skeleton().style(style().flex_grow(1).height(56));
        column((
            row((
                kivact::ui::sidebar_trigger(),
                basic::separator()
                    .orientation(Orientation::Vertical)
                    .style(style().height(16)),
                kivact::ui::breadcrumb(vec![
                    kivact::interaction::ListItem::new("root", "Dashboard"),
                    kivact::interaction::ListItem::new("current", title),
                ])
                .separator(|| kivact::view::icon("chevron-right").style(style().size(14, 14))),
            ))
            .style(
                style()
                    .height(48)
                    .padding_x(12)
                    .gap(8)
                    .align_items(Align::Center),
            ),
            column((
                row((tile(), tile(), tile())).style(style().gap(12)),
                kivact::ui::skeleton().style(style().flex_grow(1).min_height(80)),
            ))
            .style(style().flex_grow(1).padding(12).padding_top(0).gap(12)),
        ))
        .style(style().flex_grow(1))
    }
    let open = signal(true);
    frame(
        kivact::ui::sidebar_provider(
            open,
            kivact::ui::sidebar().rail(true).content({
                kivact::ui::sidebar_group("Platform").content({
                    kivact::ui::sidebar_menu((
                        kivact::ui::sidebar_menu_button("Playground")
                            .icon(kivact::view::icon("square-terminal")),
                        kivact::ui::sidebar_menu_button("Models")
                            .icon(kivact::view::icon("bot"))
                            .active(true),
                        kivact::ui::sidebar_menu_button("Documentation")
                            .icon(kivact::view::icon("book-open")),
                    ))
                })
            }),
            page_area("Playground"),
        )
        .collapsible(kivact::ui::SidebarCollapsible::Icon),
    )
}
// end example

// example: submenu
fn submenu() -> impl IntoView {
    /// A bounded frame for a sidebar demo.
    fn frame(provider: impl kivact::view::Element) -> impl kivact::view::Element {
        column(provider).style(
            style()
                .width(580)
                .height(340)
                .border(1, basic::BORDER)
                .padding(1),
        )
    }
    /// The page beside the sidebar: a header row and a dashboard sketch.
    fn page_area(title: &'static str) -> impl IntoView {
        let tile = || kivact::ui::skeleton().style(style().flex_grow(1).height(56));
        column((
            row((
                kivact::ui::sidebar_trigger(),
                basic::separator()
                    .orientation(Orientation::Vertical)
                    .style(style().height(16)),
                kivact::ui::breadcrumb(vec![
                    kivact::interaction::ListItem::new("root", "Dashboard"),
                    kivact::interaction::ListItem::new("current", title),
                ])
                .separator(|| kivact::view::icon("chevron-right").style(style().size(14, 14))),
            ))
            .style(
                style()
                    .height(48)
                    .padding_x(12)
                    .gap(8)
                    .align_items(Align::Center),
            ),
            column((
                row((tile(), tile(), tile())).style(style().gap(12)),
                kivact::ui::skeleton().style(style().flex_grow(1).min_height(80)),
            ))
            .style(style().flex_grow(1).padding(12).padding_top(0).gap(12)),
        ))
        .style(style().flex_grow(1))
    }
    let open = signal(true);
    let add = button("Add project")
        .hide_label(true)
        .icon(kivact::view::icon("plus"))
        .variant(ButtonVariant::Ghost);
    frame(
        kivact::ui::sidebar_provider(
            open,
            kivact::ui::sidebar().content({
                kivact::ui::sidebar_group("Projects").action(add).content({
                    kivact::ui::sidebar_menu((
                        kivact::ui::sidebar_menu_button("Design Engineering")
                            .icon(kivact::view::icon("frame"))
                            .sub({
                                (
                                    kivact::ui::sidebar_menu_sub_button("Overview").active(true),
                                    kivact::ui::sidebar_menu_sub_button("Roadmap"),
                                )
                            }),
                        kivact::ui::sidebar_menu_button("Sales & Marketing")
                            .icon(kivact::view::icon("chart-pie"))
                            .action(kivact::view::icon("ellipsis")),
                        kivact::ui::sidebar_menu_skeleton(),
                    ))
                })
            }),
            page_area("Projects"),
        )
        .collapsible(kivact::ui::SidebarCollapsible::None),
    )
}
// end example

// example: floating
fn floating() -> impl IntoView {
    /// A bounded frame for a sidebar demo.
    fn frame(provider: impl kivact::view::Element) -> impl kivact::view::Element {
        column(provider).style(
            style()
                .width(580)
                .height(340)
                .border(1, basic::BORDER)
                .padding(1),
        )
    }
    /// The page beside the sidebar: a header row and a dashboard sketch.
    fn page_area(title: &'static str) -> impl IntoView {
        let tile = || kivact::ui::skeleton().style(style().flex_grow(1).height(56));
        column((
            row((
                kivact::ui::sidebar_trigger(),
                basic::separator()
                    .orientation(Orientation::Vertical)
                    .style(style().height(16)),
                kivact::ui::breadcrumb(vec![
                    kivact::interaction::ListItem::new("root", "Dashboard"),
                    kivact::interaction::ListItem::new("current", title),
                ])
                .separator(|| kivact::view::icon("chevron-right").style(style().size(14, 14))),
            ))
            .style(
                style()
                    .height(48)
                    .padding_x(12)
                    .gap(8)
                    .align_items(Align::Center),
            ),
            column((
                row((tile(), tile(), tile())).style(style().gap(12)),
                kivact::ui::skeleton().style(style().flex_grow(1).min_height(80)),
            ))
            .style(style().flex_grow(1).padding(12).padding_top(0).gap(12)),
        ))
        .style(style().flex_grow(1))
    }
    let open = signal(true);
    frame(
        kivact::ui::sidebar_provider(
            open,
            kivact::ui::sidebar().rail(true).content({
                kivact::ui::sidebar_group("Help").content({
                    kivact::ui::sidebar_menu((
                        kivact::ui::sidebar_menu_button("Support")
                            .icon(kivact::view::icon("life-buoy")),
                        kivact::ui::sidebar_menu_button("Feedback")
                            .icon(kivact::view::icon("send")),
                    ))
                })
            }),
            page_area("Help"),
        )
        .side(kivact::ui::SidebarSide::Right)
        .frame(kivact::ui::SidebarFrame::Floating)
        .collapsible(kivact::ui::SidebarCollapsible::Offcanvas),
    )
}
// end example
