//! Role: Render options, pixel probes and the error check shared by consumer tests.

use kivact::test::{RenderOptions, Screen};
use kivact_ui_basic as basic;

/// The color of the logical point (`x`, `y`) of the last frame as `0xRRGGBB`.
pub(crate) fn pixel(screen: &Screen, x: f32, y: f32) -> u32 {
    let scale = screen.scale();
    screen
        .screenshot()
        .pixel((x * scale) as u32, (y * scale) as u32)
        .expect("invariant: point inside the frame")
}

/// The largest channel difference between two `0xRRGGBB` colors.
pub(crate) fn distance(a: u32, b: u32) -> u32 {
    [16, 8, 0]
        .iter()
        .map(|shift| ((a >> shift) & 0xff).abs_diff((b >> shift) & 0xff))
        .max()
        .unwrap_or(0)
}

pub(crate) fn assert_clean(screen: &Screen) {
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

pub(crate) fn options() -> RenderOptions {
    RenderOptions {
        theme: basic::theme(kivact::style::ThemeMode::Light),
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}
