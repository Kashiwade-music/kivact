//! Role: The overview page: a gradient hero with the headline, its actions and the key statistics.

use super::common::*;
use catalog_shell::{Example, Page};
use kivact::draw::{LineCap, LineJoin};
use kivact::prelude::*;
use kivact::state::IntoText;
use kivact::style::linear_gradient;
use kivact::style::rgba;
use kivact::view::Node;
use kivact_ui_pop as ui;
/// The page of the overview.
pub fn page() -> Page {
    Page::new(
        "overview",
        "Dashboard",
        "Overview",
        include_str!("overview.rs"),
    )
    .description("A dashboard header with the headline, the main action and the key figures.")
    .example(Example::new(
        "overview",
        "Dashboard hero",
        "A gradient hero with a headline, two actions and four statistics.",
        overview_demo,
    ))
}

fn overview_demo() -> impl IntoView {
    // example: overview
    let action = notify();
    let start = move || {
        action.call((
            ui::Hue::Yellow,
            "Mission started".into(),
            "Patrol of the outer district is under way.".into(),
        ))
    };
    let stats = [
        ("Enrolled students", "124", "+6 this week"),
        ("Open requests", "18", "3 due soon"),
        ("Credits", "8.2M", "+12.4%"),
    ]
    .into_iter()
    .map(|(label, value, delta)| {
        column((
            text(label).style(style().font_size(12).color(rgba(0xffffff, 0.85))),
            text(value).style(
                style()
                    .font_family("Barlow")
                    .font_style(kivact::style::FontStyle::Italic)
                    .font_weight(800)
                    .font_size(28)
                    .color(Color::WHITE)
                    .line_height(1.1),
            ),
            text(delta).style(style().font_size(12).color(rgb(0xffdc3c))),
        ))
        .style(
            style()
                .padding_x(14)
                .padding_y(12)
                .gap(2)
                .border_radius(10.0)
                .flex_grow(1.0)
                .background(rgba(0xffffff, 0.14))
                .border(1, rgba(0xffffff, 0.24)),
        )
    })
    .collect::<Vec<_>>();
    let heading = kivact::view::rich_text(vec![
        kivact::view::span("Welcome back, "),
        kivact::view::span("Sensei.").background(
            linear_gradient(180)
                .stop(Color::TRANSPARENT, pct(62))
                .stop(rgba(0xffdc3c, 0.6), pct(62)),
        ),
    ])
    .role(Role::Heading)
    .style(
        style()
            .color(Color::WHITE)
            .font_size(28)
            .font_weight(700)
            .line_height(1.3),
    );
    let actions = (
        kivact::ui::button("Start mission")
            .size(kivact::ui::ControlSize::Large)
            .variant(ui::ButtonTone::Accent)
            .variant(kivact::ui::Slant)
            .icon(kivact::view::icon("player-play"))
            .on_press(start),
        kivact::ui::button("Reports")
            .size(kivact::ui::ControlSize::Large)
            .variant(ui::ButtonTone::Glass)
            .icon(kivact::view::icon("command")),
    );
    column((
        hero(
            heading,
            "The students are waiting for you today. A soft, bright and clear set of native SaaS UI components.",
            actions,
            stats,
        ),
        columns(
            (
                kivact::ui::stat("Active students", "1,284")
                    .delta("↗ 8.2%")
                    .icon(kivact::view::icon("users"))
                    .chart(kivact::ui::sparkline(vec![2., 4., 3., 5., 4., 7., 6., 9., 8., 11.]))
                    .variant(ui::Hue::Mint),
                kivact::ui::stat("Request completion", "96.4%")
                    .delta("↗ 3.1%")
                    .icon(kivact::view::icon("circle-check"))
                    .chart(kivact::ui::sparkline(vec![3., 4., 3., 6., 5., 7., 6., 9., 8., 11.]))
                    .variant(ui::Hue::Mint),
                kivact::ui::stat("Weekly spend", "¥2.4M")
                    .delta("↘ 1.4%")
                    .icon(kivact::view::icon("coin"))
                    .chart(kivact::ui::sparkline(vec![10., 8., 9., 5., 7., 4., 5., 3., 4., 2.]))
                    .variant(ui::Hue::Red),
                kivact::ui::stat("Average bond rank", "18.6")
                    .delta("↗ 12%")
                    .icon(kivact::view::icon("heart"))
                    .chart(kivact::ui::sparkline(vec![2., 3., 5., 4., 7., 9., 8., 12., 14., 17.]))
                    .variant(ui::Hue::Mint),
            ),
            4,
        ),
    ))
    .style(style().gap(16))
    // end example
}

/// The gradient hero: the heading, the subtitle, the actions and the statistics.
fn hero(
    heading: impl IntoView,
    subtitle: impl IntoText,
    actions: impl IntoView,
    statistics: impl IntoView,
) -> Node {
    row((
        column((
            node().child(heading).style(style().margin_bottom(8)),
            text(subtitle).style(style().max_width(520).color(rgba(0xffffff, 0.88))),
            row(actions).style(style().gap(10).flex_wrap(true).margin_top(20)),
        ))
        .style(style().flex_grow(1.0).flex_basis(540).min_width(0)),
        row(statistics).style(
            style()
                .gap(10)
                .flex_grow(0.0)
                .flex_basis(440)
                .align_items(Align::End),
        ),
    ))
    .style(move || {
        style()
            .padding(32)
            .min_height(208)
            .gap(28)
            .align_items(Align::End)
            .border_radius(14.0)
            .color(Color::WHITE)
            .background(
                linear_gradient(120)
                    .stop(rgb(0x1a66a8), pct(0))
                    .stop(rgb(0x2b8bd8), pct(45))
                    .stop(rgb(0x5cbde8), pct(100)),
            )
            .overflow(kivact::style::Overflow::Hidden)
    })
    .draw(hero_ornament)
}

/// Blueprint stripes, a yellow slash and a halo ring behind the hero's text.
fn hero_ornament(d: &mut kivact::draw::Draw, s: kivact::geometry::Size) {
    let stripe = rgba(0xffffff, 0.06);
    for i in 0..((s.width / 22.0) as usize + 20) {
        let x = i as f64 * 22.0 - 250.0;
        d.line(
            kivact::geometry::Point::new(x, s.height),
            kivact::geometry::Point::new(x + s.height * 0.47, 0.0),
            2.0,
            stripe,
        );
    }
    let x = s.width * 0.7;
    d.fill_path(
        &kivact::draw::Path::new()
            .move_to(kivact::geometry::Point::new(x, -40.0))
            .line_to(kivact::geometry::Point::new(x + 20.0, -40.0))
            .line_to(kivact::geometry::Point::new(x - 80.0, s.height))
            .line_to(kivact::geometry::Point::new(x - 100.0, s.height))
            .close(),
        rgba(0xffdc3c, 0.4),
    );
    let center = kivact::geometry::Point::new(s.width - 130.0, 110.0);
    let mut circle = kivact::draw::Path::new();
    for i in 0..=80 {
        let a = std::f64::consts::TAU * i as f64 / 80.0;
        let pt =
            kivact::geometry::Point::new(center.x + 180.0 * a.cos(), center.y + 180.0 * a.sin());
        circle = if i == 0 {
            circle.move_to(pt)
        } else {
            circle.line_to(pt)
        };
    }
    d.stroke_path(
        &circle,
        &Stroke::new(14.0).cap(LineCap::Round).join(LineJoin::Round),
        rgba(0xffffff, 0.16),
    );
    let inner = kivact::draw::Path::new()
        .arc(center, 122, 0, 360, false)
        .close();
    d.stroke_path(
        &inner,
        &kivact::draw::Stroke::new(2).dash([4.0, 4.0]),
        rgba(0xffffff, 0.3),
    );
}
