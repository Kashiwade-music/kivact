//! Role: Theme variables restyle mounted views on a theme switch, follow their aliases, and report cycles and unsupported sums.

use kivact::ErrorKind;
use kivact::prelude::*;
use kivact::style::{Length, ThemeVar};
use kivact_public_api_tests::paint::{at, canvas_of, expect};
use kivact_public_api_tests::support::mount;

static CYCLE_A_VAR: ThemeVar<f64> = ThemeVar::new("test.cycle_a", 1.0);
static CYCLE_B_VAR: ThemeVar<f64> = ThemeVar::new("test.cycle_b", 2.0);

/// A theme whose two variables are aliases of each other.
fn cyclic() -> Theme {
    Theme::light()
        .set(&CYCLE_A_VAR, &CYCLE_B_VAR)
        .set(&CYCLE_B_VAR, &CYCLE_A_VAR)
}

#[test]
fn a_theme_variable_restyles_on_a_theme_switch() {
    let (screen, theme) = mount(move || {
        let theme = signal(Theme::light());
        let swatch = node().style(at(0, 0, 20, 20).background(color::ACCENT));
        (theme, provide_theme(theme, move || canvas_of(swatch)))
    });
    expect(
        &screen.screenshot(),
        10,
        10,
        0x2563eb,
        2,
        "the light accent",
    );
    theme.set(Theme::light().set(&color::ACCENT_VAR, rgb(0xff0000)));
    screen.run_until_idle();
    expect(
        &screen.screenshot(),
        10,
        10,
        0xff0000,
        2,
        "the switched accent",
    );
}

#[test]
fn a_variable_cycle_is_reported_once() {
    let (screen, theme) = mount(move || {
        let theme = signal(cyclic());
        let view = node().style(at(0, 0, 10, 10));
        (theme, provide_theme(theme, move || canvas_of(view)))
    });
    theme.set(cyclic());
    screen.run_until_idle();
    let reports = screen
        .errors()
        .iter()
        .filter(|e| *e.kind() == ErrorKind::Unsupported && e.message().contains("theme variable"))
        .count();
    assert_eq!(reports, 2, "{:?}", screen.errors());
}

#[test]
fn a_sum_with_a_variable_follows_the_theme() {
    let (screen, theme) = mount(move || {
        let theme = signal(Theme::light());
        let boxed = node()
            .test_id("box")
            .style(at(0, 0, 10, 10).width(space::MD + 2));
        (theme, provide_theme(theme, move || canvas_of(boxed)))
    });
    let width = |screen: &kivact::test::Screen| {
        screen.get_by_test_id("box").rect().expect("laid out").width
    };
    assert_eq!(width(&screen), 10.0);
    theme.set(Theme::light().set(&space::MD_VAR, 20));
    screen.run_until_idle();
    assert_eq!(width(&screen), 22.0);
}

#[test]
fn an_invalid_sum_is_reported() {
    let screen = kivact::test::render(|| {
        node()
            .test_id("bad")
            .style(at(0, 0, 10, 10).width(Length::Auto + 2))
    });
    let reports = screen
        .errors()
        .iter()
        .filter(|e| *e.kind() == ErrorKind::Unsupported && e.message().contains("length"))
        .count();
    assert_eq!(reports, 1, "{:?}", screen.errors());
}
