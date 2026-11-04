//! Role: Window lifecycle and event tests.
//!
//! The App's window settings on the test driver: the minimum size
//! (`App::min_window_size`) reaches the window with the title in the first
//! revision. `kivact::test::render` has no App, so this needs the
//! crate-internal `Screen::start_client`. The checks of the values
//! themselves open no window (`kivact`'s `lib.rs`).

use kivact_types::Size;

use crate::RenderOptions;
use kivact_core::prelude::text;

#[test]
fn the_minimum_window_size_reaches_the_window() {
    let min = Size {
        width: 320.,
        height: 240.,
    };
    let screen = crate::render_with(RenderOptions::default().min_window_size(320, 240), || {
        text("min")
    });
    assert_eq!(screen.with_host(|h| h.min_window_size()), Some(min));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());

    let screen = crate::render(|| text("no min"));
    assert_eq!(
        screen.with_host(|h| h.min_window_size()),
        None,
        "no minimum unless the App sets one"
    );
}
