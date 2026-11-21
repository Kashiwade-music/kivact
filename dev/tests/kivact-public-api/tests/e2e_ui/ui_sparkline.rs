//! Role: Verify the standard sparkline's visibility, semantics and locale text.
use kivact::i18n::Locale;
use kivact::prelude::*;
use kivact::test::RenderOptions;
use kivact::ui;
use kivact::ui::theme::{SparklineTheme, sparkline_line};
use kivact_public_api_tests::paint::{distance, pixel};
use kivact_public_api_tests::support::{mount, mount_with};

const RED: u32 = 0xff0000;

/// The pixel at `(dx, dy)` from the top left of the element `id`.
fn at(screen: &kivact::test::Screen, id: &str, dx: f64, dy: f64) -> u32 {
    let r = screen.get_by_test_id(id).rect().expect("laid out");
    pixel(&screen.screenshot(), (r.x + dx) as u32, (r.y + dy) as u32)
}

/// Whether the chart with the test id "trend" is shown; a hidden chart is not found.
fn shown(screen: &kivact::test::Screen) -> bool {
    screen
        .query_by_test_id("trend")
        .is_some_and(|chart| chart.is_visible())
}

#[test]
fn a_sparkline_is_hidden_below_two_values() {
    let (screen, values) = mount(|| {
        let values = signal(Vec::<f64>::new());
        (
            values,
            column((ui::sparkline(values).test_id("trend"),)).style(style().width(240)),
        )
    });
    assert!(!shown(&screen), "no values");
    values.set(vec![4.0]);
    screen.run_until_idle();
    assert!(!shown(&screen), "one value");
    values.set(vec![4.0, 6.0]);
    screen.run_until_idle();
    assert!(shown(&screen), "two values");
    screen.assert_layout_clean();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_labelled_sparkline_is_an_image_described_by_its_values() {
    let (screen, values) = mount_with(
        RenderOptions::from_env().locale(Locale::new("en-US")),
        || {
            let values = signal(vec![3.0, 1.0, 9.0, 4.0]);
            (
                values,
                column((ui::sparkline(values).label("Visitors"),)).style(style().width(240)),
            )
        },
    );
    let chart = screen.get_by_role(Role::Image, "Visitors");
    assert_eq!(chart.description(), "4 values from 1 to 9, last 4");
    values.set(vec![2.0, 5.0]);
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_role(Role::Image, "Visitors").description(),
        "2 values from 2 to 5, last 5"
    );
    screen.assert_layout_clean();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_description_formats_numbers_in_the_locale() {
    let (screen, _) = mount_with(
        RenderOptions::from_env().locale(Locale::new("de-DE")),
        || {
            (
                (),
                column((ui::sparkline(vec![1.0, 9.0, 2.5]).label("Visitors"),))
                    .style(style().width(240)),
            )
        },
    );
    assert_eq!(
        screen.get_by_role(Role::Image, "Visitors").description(),
        "3 values from 1 to 9, last 2,5"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn values_that_are_not_finite_break_the_line_and_stay_out_of_the_range() {
    let (screen, _) = mount(|| {
        (
            (),
            column((ui::sparkline(vec![3.0, f64::NAN, 9.0, f64::INFINITY, 4.0])
                .label("Gapped")
                .test_id("gapped"),))
            .style(style().width(240)),
        )
    });
    assert_eq!(
        screen.get_by_role(Role::Image, "Gapped").description(),
        "3 values from 3 to 9, last 4"
    );
    assert!(screen.get_by_test_id("gapped").is_visible());
    screen.assert_layout_clean();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_unlabelled_sparkline_is_decorative() {
    let (screen, _) = mount(|| {
        (
            (),
            column((ui::sparkline(vec![1.0, 2.0, 3.0]).test_id("plain"),))
                .style(style().width(240)),
        )
    });
    let plain = screen.get_by_test_id("plain");
    assert_eq!(plain.role(), None, "no role without a label");
    assert_eq!(plain.name(), "", "no name without a label");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_sparkline_recipe_draws_the_line_from_its_state() {
    // A theme that colors the line red through the recipe, not a token.
    let theme = Theme::light().with(
        SparklineTheme::default().root(|_, look, base| base.paint(sparkline_line(look, rgb(RED)))),
    );
    let (screen, _) = mount_with(RenderOptions::from_env().theme(theme), || {
        (
            (),
            ui::sparkline(vec![1.0, 1.0, 1.0])
                .test_id("spark")
                .style(style().size(60, 21)),
        )
    });
    // A flat series is a horizontal line at the middle of the 21 px box (y = 10.5),
    // so the pixel at (30, 10) is covered by the 1.5 px stroke.
    assert!(distance(at(&screen, "spark", 30.0, 10.0), RED) <= 16);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
