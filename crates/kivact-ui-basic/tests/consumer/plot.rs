//! Role: Plot tests: the named region, double-click reset, wheel and drag.

use kivact::geometry::Point;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::Role;
use kivact_ui_basic as basic;
use kivact_ui_basic::plot::{
    AxisScale, LegendPosition, LineDash, PlotAxis, PlotColor, PlotData, PlotModel, PlotRange,
    PlotSeries, PlotSeriesKind, PlotView, TickFormat, plot,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

type Commits = Rc<RefCell<Vec<PlotRange>>>;

fn axis(label: &str) -> PlotAxis {
    PlotAxis {
        label: label.into(),
        scale: AxisScale::Linear,
        format: TickFormat::Plain,
    }
}

fn model() -> PlotModel {
    PlotModel {
        title: "Response".into(),
        x: axis("Hz"),
        y: axis("dB"),
        series: vec![
            PlotSeries {
                name: "Left".into(),
                color: PlotColor::Palette(0),
                width: 2.0,
                dash: LineDash::Solid,
                kind: PlotSeriesKind::Line {
                    x: PlotData::new(vec![0.0, 5.0, 10.0]),
                    y: PlotData::new(vec![0.0, -3.0, -12.0]),
                },
            },
            PlotSeries {
                name: "Right".into(),
                color: PlotColor::Rgb(0xe11d48),
                width: 1.0,
                dash: LineDash::Dashed,
                kind: PlotSeriesKind::Points {
                    x: PlotData::new(vec![2.0, 8.0]),
                    y: PlotData::new(vec![-1.0, -6.0]),
                    radius: 3.0,
                },
            },
        ],
        legend: LegendPosition::TopRight,
    }
}

/// Mounts a plot whose range starts at `initial`; returns the screen and the range binding.
fn mount(initial: PlotRange, commits: Commits) -> (Screen, Signal<PlotRange>) {
    kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .window_size(640.0, 400.0)
            .reduced_motion(),
        move || {
            let range = signal(initial);
            (
                range,
                plot("Frequency", model(), range)
                    .on_range_commit(move |value| commits.borrow_mut().push(value)),
            )
        },
    )
}

#[test]
fn a_plot_is_a_named_canvas_with_a_region_for_its_series() {
    let (screen, _) = mount(PlotRange::Auto, Commits::default());
    screen.find_by_role(Role::Canvas, "Frequency");
    screen.find_by_role(Role::Image, "Frequency: 2 series");
}

#[test]
fn a_double_click_returns_the_range_to_auto() {
    let view = PlotView {
        x_min: 0.0,
        x_max: 4.0,
        y_min: -2.0,
        y_max: 0.0,
    };
    let commits = Commits::default();
    let (screen, range) = mount(PlotRange::Manual(view), commits.clone());
    screen
        .find_by_role(Role::Canvas, "Frequency")
        .double_click();
    assert_eq!(range.get_untracked(), PlotRange::Auto);
    assert_eq!(commits.borrow().last(), Some(&PlotRange::Auto));
}

#[test]
fn a_wheel_step_commits_once_the_wheel_has_settled() {
    let commits = Commits::default();
    let (screen, range) = mount(PlotRange::Auto, commits.clone());
    screen
        .find_by_role(Role::Canvas, "Frequency")
        .scroll(0.0, 40.0);
    assert!(
        commits.borrow().is_empty(),
        "the step waits for the settle time"
    );
    screen.advance(Duration::from_millis(400));
    assert!(matches!(
        commits.borrow().last(),
        Some(PlotRange::Manual(_))
    ));
    assert!(matches!(range.get_untracked(), PlotRange::Manual(_)));
}

#[test]
fn a_drag_pans_and_commits_on_release() {
    let commits = Commits::default();
    let (screen, _) = mount(PlotRange::Auto, commits.clone());
    let element = screen.find_by_role(Role::Canvas, "Frequency");
    let rect = element.rect().expect("laid out");
    let from = Point::new(rect.width / 2.0, rect.height / 2.0);
    element.drag(from, Point::new(from.x + 60.0, from.y));
    assert_eq!(commits.borrow().len(), 1, "one commit at the release");
    assert!(matches!(
        commits.borrow().last(),
        Some(PlotRange::Manual(_))
    ));
}
