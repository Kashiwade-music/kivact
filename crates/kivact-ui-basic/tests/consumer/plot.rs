//! Role: Plot tests: the description, the range edits and the locale.

use kivact::geometry::Point;
use kivact::i18n::Locale;
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, Screen};
use kivact::view::{EditPhase, Key, MouseButton, Role};
use kivact_ui_basic as basic;
use kivact_ui_basic::{
    PlotAxis, PlotData, PlotModel, PlotRange, PlotSeries, PlotSeriesKind, PlotView, plot,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

type Edits = Rc<RefCell<Vec<EditPhase>>>;

/// Linear axes, a line from x 0 to 10 and a point series in an explicit color.
fn model() -> PlotModel {
    PlotModel::new(PlotAxis::new("Hz"), PlotAxis::new("dB"))
        .series(PlotSeries::new(
            "Left",
            PlotSeriesKind::Line {
                x: PlotData::new(vec![0.0, 5.0, 10.0]),
                y: PlotData::new(vec![0.0, -3.0, -12.0]),
            },
        ))
        .series(
            PlotSeries::new(
                "Right",
                PlotSeriesKind::Points {
                    x: PlotData::new(vec![2.0, 8.0]),
                    y: PlotData::new(vec![-1.0, -6.0]),
                    radius: 3.0,
                },
            )
            .color(rgb(0xe11d48)),
        )
}

fn view(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> PlotView {
    PlotView {
        x_min,
        x_max,
        y_min,
        y_max,
    }
}

/// Mounts the plot named "Frequency" with its range starting at `initial`;
/// returns the screen and the range binding.
fn mount(initial: PlotRange, edits: Edits, locale: Option<Locale>) -> (Screen, Signal<PlotRange>) {
    let mut options = RenderOptions::with_theme(basic::theme(ThemeMode::Light))
        .window_size(640.0, 400.0)
        .reduced_motion();
    if let Some(locale) = locale {
        options = options.locale(locale);
    }
    kivact::test::render_with_handle(options, move || {
        let range = signal(initial);
        let observed = edits.clone();
        (
            range,
            plot("Frequency", model())
                .range(range)
                .on_edit(move |phase| observed.borrow_mut().push(phase)),
        )
    })
}

fn manual(range: Signal<PlotRange>) -> PlotView {
    match range.get_untracked() {
        PlotRange::Manual(view) => view,
        PlotRange::Auto => panic!("the range is still automatic"),
    }
}

fn span(view: PlotView) -> (f64, f64) {
    (view.x_max - view.x_min, view.y_max - view.y_min)
}

#[test]
fn a_plot_is_a_focusable_image_described_by_its_series_and_range() {
    let (screen, _) = mount(PlotRange::Auto, Edits::default(), None);
    let text = screen.find_by_role(Role::Image, "Frequency").description();
    assert!(text.contains("Left, Right"), "{text}");
    assert!(text.contains("Hz") && text.contains("dB"), "{text}");
    screen.assert_layout_clean();
}

#[test]
fn the_description_is_written_in_the_locale() {
    let window = view(0.0, 0.5, -12.0, 0.0);
    let (screen, _) = mount(
        PlotRange::Manual(window),
        Edits::default(),
        Some(Locale::new("de-DE")),
    );
    let text = screen.find_by_role(Role::Image, "Frequency").description();
    assert!(text.contains("0,5"), "{text}");
}

#[test]
fn a_drag_pans_the_range_live_and_commits_once_on_release() {
    let edits = Edits::default();
    let start = view(0.0, 10.0, -12.0, 0.0);
    let (screen, range) = mount(PlotRange::Manual(start), edits.clone(), None);
    let rect = screen
        .find_by_role(Role::Image, "Frequency")
        .rect()
        .expect("laid out");
    let center = Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(center.x, center.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(center.x + 60.0, center.y);
    let moved = manual(range);
    assert!(
        moved.x_min < 0.0 && moved.x_max < 10.0,
        "content follows the pointer"
    );
    assert!((span(moved).0 - 10.0).abs() < 1e-9, "the span is unchanged");
    assert_eq!((moved.y_min, moved.y_max), (-12.0, 0.0));
    assert_eq!(*edits.borrow(), vec![EditPhase::Begin]);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*edits.borrow(), vec![EditPhase::Begin, EditPhase::Commit]);
}

#[test]
fn a_press_without_a_move_reports_no_edit() {
    let edits = Edits::default();
    let (screen, range) = mount(PlotRange::Auto, edits.clone(), None);
    let rect = screen
        .find_by_role(Role::Image, "Frequency")
        .rect()
        .expect("laid out");
    let center = Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(center.x, center.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(range.get_untracked(), PlotRange::Auto);
    assert!(edits.borrow().is_empty());
}

#[test]
fn escape_during_a_drag_restores_the_range_it_started_from() {
    let edits = Edits::default();
    let start = view(0.0, 10.0, -12.0, 0.0);
    let (screen, range) = mount(PlotRange::Manual(start), edits.clone(), None);
    let rect = screen
        .find_by_role(Role::Image, "Frequency")
        .rect()
        .expect("laid out");
    let center = Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(center.x, center.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(center.x + 60.0, center.y);
    screen.press_key(Key::Escape);
    assert_eq!(range.get_untracked(), PlotRange::Manual(start));
    assert_eq!(*edits.borrow(), vec![EditPhase::Begin, EditPhase::Cancel]);
}

#[test]
fn a_wheel_step_zooms_x_in_one_edit_at_once() {
    let edits = Edits::default();
    let (screen, range) = mount(PlotRange::Auto, edits.clone(), None);
    let auto = model().auto_view();
    screen
        .find_by_role(Role::Image, "Frequency")
        .scroll(0.0, 120.0);
    let zoomed = manual(range);
    assert!(span(zoomed).0 > span(auto).0, "the x span widens");
    assert_eq!((zoomed.y_min, zoomed.y_max), (auto.y_min, auto.y_max));
    assert_eq!(*edits.borrow(), vec![EditPhase::Begin, EditPhase::Commit]);
    screen.advance(Duration::from_millis(400));
    assert_eq!(edits.borrow().len(), 2, "no later phase arrives");
}

#[test]
fn a_horizontal_wheel_pans_x_towards_larger_values() {
    let start = view(0.0, 10.0, -12.0, 0.0);
    let (screen, range) = mount(PlotRange::Manual(start), Edits::default(), None);
    screen
        .find_by_role(Role::Image, "Frequency")
        .scroll(60.0, 0.0);
    let panned = manual(range);
    assert!(panned.x_min > 0.0, "x moves to larger values");
    assert!(
        (span(panned).0 - 10.0).abs() < 1e-9,
        "the span is unchanged"
    );
    assert_eq!((panned.y_min, panned.y_max), (-12.0, 0.0));
}

#[test]
fn keys_zoom_pan_and_fit_with_one_edit_each() {
    let edits = Edits::default();
    let start = view(0.0, 10.0, -12.0, 0.0);
    let (screen, range) = mount(PlotRange::Manual(start), edits.clone(), None);
    screen.find_by_role(Role::Image, "Frequency").focus();

    screen.press_key(Key::Char('+'));
    let narrower = manual(range);
    assert!(span(narrower).0 < 10.0, "+ narrows x");
    assert_eq!(narrower.y_min, start.y_min);

    screen.press_key(Key::ArrowRight);
    let shifted = manual(range);
    let step = span(narrower).0 * 0.1;
    assert!(
        (shifted.x_min - (narrower.x_min + step)).abs() < 1e-9,
        "ArrowRight moves x by 10%"
    );

    screen.press_key(Key::PageUp);
    assert!(span(manual(range)).1 < 12.0, "PageUp narrows y");

    screen.press_key(Key::Char('0'));
    assert_eq!(range.get_untracked(), PlotRange::Auto);
    let pair = [EditPhase::Begin, EditPhase::Commit];
    let expected: Vec<EditPhase> = pair.iter().cycle().take(8).copied().collect();
    assert_eq!(*edits.borrow(), expected);
}

#[test]
fn a_double_click_returns_the_range_to_auto_in_one_edit() {
    let edits = Edits::default();
    let start = view(0.0, 4.0, -2.0, 0.0);
    let (screen, range) = mount(PlotRange::Manual(start), edits.clone(), None);
    screen.find_by_role(Role::Image, "Frequency").double_click();
    assert_eq!(range.get_untracked(), PlotRange::Auto);
    assert_eq!(*edits.borrow(), vec![EditPhase::Begin, EditPhase::Commit]);
}
