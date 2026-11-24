//! Role: Window lifecycle and event tests.
//!
//! The App's window settings on the test driver: the minimum size
//! (`App::min_window_size`) reaches the window with the title in the first
//! revision. `kivact::test::render` has no App, so this needs the
//! crate-internal `Screen::start_client`. The checks of the values
//! themselves open no window (`kivact`'s `lib.rs`).

use kivact::geometry::Size;

use kivact::prelude::{callback, spawn, text};
use kivact::test::RenderOptions;
use kivact_public_api_tests::support::mount;

#[test]
fn resize_requests_are_clamped_to_the_apps_single_minimum() {
    let screen =
        kivact::test::render_with(RenderOptions::default().min_window_size(320, 240), || {
            spawn(kivact::platform::request_window_size(100, 300));
            text("resize")
        });
    screen.run_until_idle();
    assert_eq!(
        screen.requested_sizes(),
        vec![Size {
            width: 320.0,
            height: 300.0
        }]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn resize_dimensions_must_fit_finite_positive_f64() {
    for bad in [
        f64::NAN,
        f64::INFINITY,
        0.0,
        -1.0,
        f64::MAX,
        f64::MIN_POSITIVE,
    ] {
        let screen = kivact::test::render(move || {
            spawn(kivact::platform::request_window_size(bad, 100));
            text("invalid resize")
        });
        assert!(screen.requested_sizes().is_empty());
        assert_eq!(screen.errors().len(), 1);
        assert_eq!(*screen.errors()[0].kind(), kivact::ErrorKind::Message);
    }
}

#[test]
fn set_window_visible_hides_and_shows_the_window_without_ending_the_app() {
    let (screen, set_visible) = mount(|| {
        let set_visible = callback(move |visible: bool| {
            spawn(kivact::platform::set_window_visible(visible));
        });
        (set_visible, text("visible"))
    });
    screen.run_until_idle();
    assert!(screen.is_window_visible(), "a window starts on screen");
    set_visible.call(false);
    screen.run_until_idle();
    assert!(!screen.is_window_visible(), "hidden on request");
    set_visible.call(true);
    screen.run_until_idle();
    assert!(screen.is_window_visible(), "shown again on request");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn set_window_visible_in_root_setup_hides_the_window_from_the_start() {
    let screen = kivact::test::render_with(RenderOptions::default(), || {
        spawn(kivact::platform::set_window_visible(false));
        text("hidden")
    });
    screen.run_until_idle();
    assert!(!screen.is_window_visible());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
