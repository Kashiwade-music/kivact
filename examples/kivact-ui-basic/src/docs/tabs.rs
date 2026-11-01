//! Role: Catalog page of Tabs: default, disabled tab, vertical and the line variant.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "tabs",
        name: "Tabs",
        description: "同じ領域で複数の内容を切り替えます。標準の kivact::ui::tabs に Lyra の見た目（default の variant）を当てたもので、左右の矢印キーでタブを移動します。line の variant は .variant(basic::TabsVariant::Line) で、塗りつぶしの代わりに選択中のタブの下へ線を引きます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("tabs.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "ListItem の一覧と選択中のキーを渡し、show に選ばれたキーごとのパネルを返す関数を渡します。",
                default,
            ),
            Example::new(
                "disabled",
                "Disabled tab",
                "ListItem の disabled で、選べないタブを作れます。",
                disabled,
            ),
            Example::new(
                "vertical",
                "Vertical",
                "vertical で、タブを縦に並べます。上下の矢印キーで移動します。",
                vertical,
            ),
            Example::new(
                "line",
                "Line",
                ".variant(basic::TabsVariant::Line) で、その tabs が line の variant になります。選択中のタブの下に 2 px の線が引かれ、背景は塗られません。",
                line,
            ),
            Example::new(
                "line-vertical",
                "Line, vertical",
                "vertical にすると線はタブの右に付き、上下の矢印キーで移動します。パネルはタブの右に並びます。",
                line_vertical,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let current = signal("account");
    let options = vec![
        ListItem::new("account", "Account"),
        ListItem::new("password", "Password"),
    ];
    tabs("Account settings", options, current).content(|key| {
        let (heading, body) = match key {
            "account" => ("Account", "Make changes to your account here."),
            _ => ("Password", "Change your password here."),
        };
        column((basic::h4(heading), basic::muted(body))).style(style().gap(8).padding_top(16))
    })
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    let current = signal("home");
    let options = vec![
        ListItem::new("home", "Home"),
        ListItem::new("settings", "Settings").disabled(true),
        ListItem::new("about", "About"),
    ];
    tabs("Site", options, current).content(|key| {
        let (heading, body) = match key {
            "home" => ("Home", "The Settings tab is disabled."),
            _ => ("About", "Kivact is a Rust UI toolkit."),
        };
        column((basic::h4(heading), basic::muted(body))).style(style().gap(8).padding_top(16))
    })
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    let current = signal("overview");
    let options = vec![
        ListItem::new("overview", "Overview"),
        ListItem::new("analytics", "Analytics"),
        ListItem::new("reports", "Reports"),
    ];
    tabs("Project", options, current)
        .orientation(Orientation::Vertical)
        .content(|key| {
            let (heading, body) = match key {
                "overview" => ("Overview", "A summary of the project."),
                "analytics" => ("Analytics", "Traffic and conversions."),
                _ => ("Reports", "Monthly reports."),
            };
            column((basic::h4(heading), basic::muted(body))).style(style().gap(8).padding_top(16))
        })
}
// end example

// example: line
fn line() -> impl IntoView {
    let current = signal("overview");
    let options = vec![
        ListItem::new("overview", "Overview"),
        ListItem::new("analytics", "Analytics"),
        ListItem::new("reports", "Reports"),
    ];
    tabs("Project", options, current)
        .variant(basic::TabsVariant::Line)
        .content(|key| {
            let (heading, body) = match key {
                "overview" => ("Overview", "A summary of the project."),
                "analytics" => ("Analytics", "Traffic and conversions."),
                _ => ("Reports", "Monthly reports."),
            };
            column((basic::h4(heading), basic::muted(body))).style(style().gap(8).padding_top(16))
        })
}
// end example

// example: line-vertical
fn line_vertical() -> impl IntoView {
    let current = signal("general");
    let options = vec![
        ListItem::new("general", "General"),
        ListItem::new("security", "Security"),
        ListItem::new("billing", "Billing"),
    ];
    tabs("Settings", options, current)
        .variant(basic::TabsVariant::Line)
        .orientation(Orientation::Vertical)
        .content(|key| {
            let body = match key {
                "general" => "Name and language.",
                "security" => "Password and sessions.",
                _ => "Plans and invoices.",
            };
            column((basic::h4(key), basic::muted(body))).style(style().gap(8).padding_left(16))
        })
}
// end example
