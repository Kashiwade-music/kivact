//! Role: Window lifecycle and event tests.
//!
//! The App's window settings on the test driver: the minimum size
//! (`App::min_window_size`) reaches the window with the title in the first
//! revision. `kivact::test::render` has no App, so this needs the
//! crate-internal `Screen::start_client`. The checks of the values
//! themselves open no window (`kivact`'s `lib.rs`).

use std::panic::Location;

use kivact_types::Size;

use super::model::Screen;
use crate::client::{AppClient, AppConfig};
use crate::prelude::{IntoView, Theme, spawn, text};
use crate::test::RenderOptions;
use crate::view::View;

#[test]
fn the_minimum_window_size_reaches_the_window() {
    let at = Location::caller();
    let min = Size {
        width: 320.0,
        height: 240.0,
    };
    let root: Box<dyn FnOnce() -> View> = Box::new(|| text("min").into_view());
    let mut config = AppConfig::new("min".into(), Theme::default(), root);
    config.min_window_size = Some(min);
    let client = AppClient::new(config, Some(2), at);
    let screen = Screen::start_client(RenderOptions::default(), false, client, at);
    assert_eq!(screen.with_host(|h| h.min_window_size()), Some(min));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());

    let screen = crate::test::render(|| text("no min"));
    assert_eq!(
        screen.with_host(|h| h.min_window_size()),
        None,
        "no minimum unless the App sets one"
    );
}

#[test]
fn resize_requests_are_clamped_to_the_apps_single_minimum() {
    let at = Location::caller();
    let min = Size {
        width: 320.0,
        height: 240.0,
    };
    let root: Box<dyn FnOnce() -> View> = Box::new(|| {
        spawn(crate::platform::request_window_size(100, 300));
        text("resize").into_view()
    });
    let mut config = AppConfig::new("resize".into(), Theme::default(), root);
    config.min_window_size = Some(min);
    let client = AppClient::new(config, Some(2), at);
    let screen = Screen::start_client(RenderOptions::default(), false, client, at);
    screen.run_until_idle();
    assert_eq!(
        screen.with_host(|h| h.requested_sizes().to_vec()),
        vec![Size {
            width: 320.0,
            height: 300.0
        }]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn resize_dimensions_must_fit_finite_positive_f32() {
    for bad in [
        f64::NAN,
        f64::INFINITY,
        0.0,
        -1.0,
        f64::MAX,
        f64::MIN_POSITIVE,
    ] {
        let screen = crate::test::render(move || {
            spawn(crate::platform::request_window_size(bad, 100));
            text("invalid resize")
        });
        assert!(screen.with_host(|h| h.requested_sizes().is_empty()));
        assert_eq!(screen.errors().len(), 1);
        assert_eq!(
            *screen.errors()[0].kind(),
            kivact_reactive::ErrorKind::Message
        );
    }
}
