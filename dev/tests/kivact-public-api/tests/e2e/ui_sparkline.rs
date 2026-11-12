//! Role: Verify the standard sparkline's visibility, semantics and locale text.
use crate::support::{mount, mount_with};
use kivact::i18n::Locale;
use kivact::prelude::*;
use kivact::test::RenderOptions;
use kivact::ui;

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
