//! Role: Window lifecycle and event tests.
//!
//! The App's window settings on the test driver: the minimum size
//! (`App::min_window_size`) reaches the window with the title in the first
//! revision. `kivact::test::render` has no App, so this needs the
//! crate-internal `Screen::start_client`. The checks of the values
//! themselves open no window (`kivact`'s `lib.rs`).

use kivact::draw::Size;

use kivact::prelude::{spawn, text};
use kivact::test::RenderOptions;

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
        assert_eq!(
            *screen.errors()[0].kind(),
            kivact::diagnostics::ErrorKind::Message
        );
    }
}
