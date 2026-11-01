//! Role: The catalog's application frame: sidebar, topbar and a scrolling canvas.
use kivact::control::{Node, column, node, row, scroll};
use kivact::draw::{Point, Rect};
use kivact::prelude::NodeBuilder;
use kivact::primitive::NodeExt;
use kivact::style::ThemeMode;
use kivact::style::{Align, ColorToken, pct, style};
use kivact::view::{IntoView, NodeRef};
use kivact_ui_game::{Color, Tokens};

/// 252 px sidebar, 56 px topbar and centered 1320 px independently scrolling
/// content. The content scroll container is `scroll_node`, so the catalog can
/// scroll it for in-page navigation.
#[track_caller]
pub(super) fn app_shell(
    sidebar: impl IntoView,
    topbar: impl IntoView,
    content: impl IntoView,
    scroll_node: NodeRef,
) -> Node {
    let side = node()
        .style(
            style()
                .width(252)
                .shrink(0)
                .height(pct(100))
                .background(Color::Sidebar)
                .color(Color::SidebarText),
        )
        .child(sidebar);
    let top = row(topbar)
        .style(
            style()
                .height(56)
                .shrink(0)
                .padding_x(24)
                .align_items(Align::Center)
                .gap(12)
                .background(ColorToken::Surface)
                .border_bottom(1, ColorToken::Border),
        )
        .draw_over(|d, _| {
            d.fill_rect(Rect::new(0., 55., 180., 2.), ColorToken::Accent);
            d.fill_rect(
                Rect::new(180., 55., 60., 2.),
                d.theme().get::<Tokens>().cyan,
            );
        });
    let content = node()
        .style(
            style()
                .width(pct(100))
                .max_width(1320)
                .align_self(Align::Center)
                .padding_top(28)
                .padding_x(32)
                .padding_bottom(96),
        )
        .child(content);
    let canvas = scroll(content)
        .node_ref(scroll_node)
        .style(style().grow(1).min_height(0).align_items(Align::Center))
        .style(style().background(Color::Canvas))
        .draw(|d, s| {
            let p = d.theme().get::<Tokens>();
            let color = if p.mode == ThemeMode::Dark {
                kivact::style::rgba(0x3a9dff, 0.05)
            } else {
                kivact::style::rgba(0x1283f0, 0.055)
            };
            let mut x = 0.;
            while x < s.width {
                d.line(Point::new(x, 0.), Point::new(x, s.height), 1., color);
                x += 24.;
            }
            let mut y = 0.;
            while y < s.height {
                d.line(Point::new(0., y), Point::new(s.width, y), 1., color);
                y += 24.;
            }
        });
    row((
        side,
        column((top, canvas)).style(style().grow(1).min_width(0).height(pct(100))),
    ))
    .style(
        style()
            .width(pct(100))
            .height(pct(100))
            .align_items(Align::Stretch),
    )
}
