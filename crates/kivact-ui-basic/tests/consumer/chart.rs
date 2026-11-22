//! Role: Chart tests: series colors in both themes, value locale and the tooltip card.

use crate::support::distance;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::Node;
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::BACKGROUND)
            .color(basic::FOREGROUND)
            .padding(16)
            .gap(16)
            .flex_grow(1)
            .align_items(Align::Start),
    )
}

fn config() -> basic::ChartConfig {
    basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop").color(rgb(0xe11d48)),
        basic::ChartSeries::new("mobile", "Mobile").color(rgb(0x2563eb)),
    ])
}

fn rows() -> Vec<basic::ChartRow> {
    vec![
        basic::ChartRow::new("January", [186.0, 80.0]),
        basic::ChartRow::new("February", [305.0, 200.0]),
        basic::ChartRow::new("March", [237.0, 120.0]),
        basic::ChartRow::new("April", [73.0, 190.0]),
    ]
}

/// How many pixels of `screen` inside `rect` are near `color`.
fn count_near(screen: &Screen, rect: kivact::geometry::Rect, color: u32) -> usize {
    let shot = screen.screenshot();
    let scale = screen.scale();
    let (x0, y0) = ((rect.x * scale) as u32, (rect.y * scale) as u32);
    let x1 = ((rect.x + rect.width) * scale) as u32;
    let y1 = ((rect.y + rect.height) * scale) as u32;
    let mut found = 0;
    for y in y0..y1.min(shot.height) {
        for x in x0..x1.min(shot.width) {
            if shot.pixel(x, y).is_some_and(|p| distance(p, color) <= 6) {
                found += 1;
            }
        }
    }
    found
}

const RED: u32 = 0xe11d48;
const BLUE: u32 = 0x2563eb;

fn chart_of(kind: basic::ChartKind) -> basic::Chart {
    basic::chart(config(), rows())
        .kind(kind)
        .height(220.0)
        .style(style().width(520))
        .test_id("chart")
}

fn mount_chart(
    mode: kivact::style::ThemeMode,
    chart: impl FnOnce() -> basic::Chart + 'static,
) -> Screen {
    kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(mode))
            .window_size(640.0, 2400.0)
            .reduced_motion(),
        move || ((), page(chart())),
    )
    .0
}

#[test]
fn every_chart_kind_draws_its_series_in_their_colors_in_both_themes() {
    for mode in [
        kivact::style::ThemeMode::Light,
        kivact::style::ThemeMode::Dark,
    ] {
        for kind in [
            basic::ChartKind::Bar,
            basic::ChartKind::StackedBar,
            basic::ChartKind::Line,
            basic::ChartKind::Area,
            basic::ChartKind::Pie,
            basic::ChartKind::Donut,
            basic::ChartKind::Radar,
            basic::ChartKind::Radial,
        ] {
            let screen = mount_chart(mode, move || chart_of(kind));
            let rect = screen.get_by_test_id("chart").rect().expect("laid out");
            for (name, color) in [("red", RED), ("blue", BLUE)] {
                assert!(
                    count_near(&screen, rect, color) >= 10,
                    "{kind:?} draws the {name} series in {mode:?}"
                );
            }
            assert!(screen.errors().is_empty(), "{:?}", screen.errors());
            screen.assert_layout_clean();
            screen.assert_layout();
        }
    }
}

#[test]
fn the_default_value_text_is_written_in_the_app_locale() {
    let screen = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .window_size(640.0, 400.0)
            .reduced_motion()
            .host_locale("de-DE"),
        move || {
            let data = vec![basic::ChartRow::new("January", [1234.5, 80.0])];
            ((), basic::chart(config(), data).height(220.0))
        },
    )
    .0;
    let chart = || screen.find_by_role(kivact::view::Role::Image, "Bar chart: Desktop, Mobile");
    assert!(chart().description().contains("1.234,5"));
    // The tooltip card is written in the locale too, and follows a change.
    let rect = chart().rect().expect("laid out");
    screen.mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.run_until_idle();
    assert!(screen.query_by_text("1.234,5").is_some());
    screen.set_host_locale(Some("en-US"));
    assert!(chart().description().contains("1,234.5"));
    assert!(screen.query_by_text("1.234,5").is_none());
    assert!(screen.query_by_text("1,234.5").is_some());
}

#[test]
fn hovering_a_chart_shows_the_tooltip_card_with_the_values() {
    let screen = mount_chart(kivact::style::ThemeMode::Light, || {
        chart_of(basic::ChartKind::Bar)
    });
    let rect = screen.get_by_test_id("chart").rect().expect("laid out");
    assert!(screen.query_by_text("February").is_none());
    // The second of four categories lies at 3/8 of the plot width.
    screen.mouse_move(rect.x + rect.width * 0.4, rect.y + 100.0);
    screen.run_until_idle();
    screen.get_by_text("February");
    screen.get_by_text("305");
    screen.get_by_text("200");
    assert_eq!(screen.all_by_text("Desktop").len(), 2, "legend and tooltip");
    // The tooltip swatches carry the series colors.
    let card = screen.get_by_text("February").rect().expect("laid out");
    let swatches = kivact::geometry::Rect::new(card.x - 20.0, card.y, 200.0, 80.0);
    assert!(count_near(&screen, swatches, RED) >= 20);
    // Within the same category the card follows the pointer.
    screen.mouse_move(rect.x + rect.width * 0.4 + 6.0, rect.y + 110.0);
    screen.run_until_idle();
    let moved = screen.get_by_text("February").rect().expect("laid out");
    assert_eq!((moved.x - card.x, moved.y - card.y), (6.0, 10.0));
    // The pointer passes through the card: over it, the plot still reports
    // what lies under the pointer.
    screen.mouse_move(card.x + 4.0, card.y + 4.0);
    screen.run_until_idle();
    assert!(screen.query_by_text("February").is_some() || screen.query_by_text("March").is_some());
    screen.assert_layout();
    screen.mouse_leave();
    screen.run_until_idle();
    assert!(screen.query_by_text("February").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}
