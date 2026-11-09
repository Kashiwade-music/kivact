//! Role: Catalog page of Sidebar: groups, icon mode, sub-menus and frames.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "sidebar",
        name: "Sidebar",
        description: "ナビゲーションを置く、開閉できる側面の領域です。sidebar_provider に開閉の束縛、sidebar、隣のページを渡します。sidebar_trigger、端のレール、Ctrl+B（Mac は Cmd+B）で開閉し、デモは枠の中に収めています。Ctrl+B をウィンドウのどこでも使うには、provider をアプリのルートの view にします。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact::style::use_theme;\nuse kivact_ui_basic as basic;",
        source: include_str!("sidebar.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "sidebar の header、content、footer に view を渡します。グループは sidebar_group、行は sidebar_menu_button で、active、badge、icon を付けられます。隣のページには、見出しの行とダッシュボードの枠を置いています。",
                basic,
            ),
            Example::new(
                "icon",
                "Collapsible icon",
                "collapsible に Icon を渡すと、閉じたときに 48 px のアイコンの列になります。閉じた行にポインタを置くと、ラベルがツールチップで出ます。",
                icon,
            ),
            Example::new(
                "submenu",
                "Sub menu and action",
                "sub に sidebar_menu_sub_button を渡すと、罫線の奥に子の行が並びます。action は行の端に置く操作です。",
                submenu,
            ),
            Example::new(
                "floating",
                "Floating on the right",
                "variant に Floating、side に Right を渡すと、8 px 浮いた枠付きのパネルが右に付きます。rail は端の細い帯で、押すと開閉します。",
                floating,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    /// A bounded frame for a sidebar demo.
    fn frame(provider: impl kivact::view::Element) -> impl kivact::view::Element {
        column(provider).style(
            style()
                .width(580)
                .height(340)
                .border(1, basic::ColorName::Border)
                .padding(1),
        )
    }
    /// The page beside the sidebar: a header row and a dashboard sketch.
    fn page_area(title: &'static str) -> impl IntoView {
        let tile = || kivact::ui::skeleton().style(style().grow(1).height(56));
        column((
            row((
                basic::sidebar_trigger(),
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
                kivact::ui::skeleton().style(style().grow(1).min_height(80)),
            ))
            .style(style().grow(1).padding(12).padding_top(0).gap(12)),
        ))
        .style(style().grow(1))
    }
    let open = signal(true);
    frame(basic::sidebar_provider(
        open,
        basic::sidebar()
            .collapsible(basic::SidebarCollapsible::None)
            .header(basic::large("Acme Inc"))
            .content({
                (
                    basic::sidebar_group("Application").content({
                        basic::sidebar_menu((
                            basic::sidebar_menu_button("Home")
                                .icon(kivact::view::icon("house"))
                                .active(true),
                            basic::sidebar_menu_button("Inbox")
                                .icon(kivact::view::icon("inbox"))
                                .badge("12"),
                            basic::sidebar_menu_button("Calendar")
                                .icon(kivact::view::icon("calendar")),
                        ))
                    }),
                    basic::sidebar_separator(),
                    basic::sidebar_group("Settings").content({
                        basic::sidebar_menu(
                            basic::sidebar_menu_button("Preferences")
                                .icon(kivact::view::icon("settings")),
                        )
                    }),
                )
            }),
        page_area("Overview"),
    ))
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
                .border(1, basic::ColorName::Border)
                .padding(1),
        )
    }
    /// The page beside the sidebar: a header row and a dashboard sketch.
    fn page_area(title: &'static str) -> impl IntoView {
        let tile = || kivact::ui::skeleton().style(style().grow(1).height(56));
        column((
            row((
                basic::sidebar_trigger(),
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
                kivact::ui::skeleton().style(style().grow(1).min_height(80)),
            ))
            .style(style().grow(1).padding(12).padding_top(0).gap(12)),
        ))
        .style(style().grow(1))
    }
    let open = signal(true);
    frame(basic::sidebar_provider(
        open,
        basic::sidebar()
            .collapsible(basic::SidebarCollapsible::Icon)
            .rail(true)
            .content({
                basic::sidebar_group("Platform").content({
                    basic::sidebar_menu((
                        basic::sidebar_menu_button("Playground")
                            .icon(kivact::view::icon("square-terminal")),
                        basic::sidebar_menu_button("Models")
                            .icon(kivact::view::icon("bot"))
                            .active(true),
                        basic::sidebar_menu_button("Documentation")
                            .icon(kivact::view::icon("book-open")),
                    ))
                })
            }),
        page_area("Playground"),
    ))
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
                .border(1, basic::ColorName::Border)
                .padding(1),
        )
    }
    /// The page beside the sidebar: a header row and a dashboard sketch.
    fn page_area(title: &'static str) -> impl IntoView {
        let tile = || kivact::ui::skeleton().style(style().grow(1).height(56));
        column((
            row((
                basic::sidebar_trigger(),
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
                kivact::ui::skeleton().style(style().grow(1).min_height(80)),
            ))
            .style(style().grow(1).padding(12).padding_top(0).gap(12)),
        ))
        .style(style().grow(1))
    }
    let open = signal(true);
    let add = button("Add project")
        .hide_label(true)
        .icon(kivact::view::icon("plus"))
        .variant(ButtonVariant::Ghost);
    frame(basic::sidebar_provider(
        open,
        basic::sidebar()
            .collapsible(basic::SidebarCollapsible::None)
            .content({
                basic::sidebar_group("Projects").action(add).content({
                    basic::sidebar_menu((
                        basic::sidebar_menu_button("Design Engineering")
                            .icon(kivact::view::icon("frame"))
                            .sub({
                                (
                                    basic::sidebar_menu_sub_button("Overview").active(true),
                                    basic::sidebar_menu_sub_button("Roadmap"),
                                )
                            }),
                        basic::sidebar_menu_button("Sales & Marketing")
                            .icon(kivact::view::icon("chart-pie"))
                            .action(kivact::view::icon("ellipsis")),
                        basic::sidebar_menu_skeleton(),
                    ))
                })
            }),
        page_area("Projects"),
    ))
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
                .border(1, basic::ColorName::Border)
                .padding(1),
        )
    }
    /// The page beside the sidebar: a header row and a dashboard sketch.
    fn page_area(title: &'static str) -> impl IntoView {
        let tile = || kivact::ui::skeleton().style(style().grow(1).height(56));
        column((
            row((
                basic::sidebar_trigger(),
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
                kivact::ui::skeleton().style(style().grow(1).min_height(80)),
            ))
            .style(style().grow(1).padding(12).padding_top(0).gap(12)),
        ))
        .style(style().grow(1))
    }
    let open = signal(true);
    frame(basic::sidebar_provider(
        open,
        basic::sidebar()
            .side(basic::SidebarSide::Right)
            .variant(basic::SidebarVariant::Floating)
            .collapsible(basic::SidebarCollapsible::Offcanvas)
            .rail(true)
            .content({
                basic::sidebar_group("Help").content({
                    basic::sidebar_menu((
                        basic::sidebar_menu_button("Support").icon(kivact::view::icon("life-buoy")),
                        basic::sidebar_menu_button("Feedback").icon(kivact::view::icon("send")),
                    ))
                })
            }),
        page_area("Help"),
    ))
}
// end example
