//! Role: External-consumer tests: no internal Kivact or renderer imports are available.

use kivact::test::{RenderOptions, Screen};
use kivact_ui_basic as basic;

#[path = "consumer/code_block.rs"]
mod code_block;
#[path = "consumer/data.rs"]
mod data;
#[path = "consumer/display.rs"]
mod display;
#[path = "consumer/forms.rs"]
mod forms;
#[path = "consumer/foundation.rs"]
mod foundation;
#[path = "consumer/navigation.rs"]
mod navigation;
#[path = "consumer/navigation_layout.rs"]
mod navigation_layout;
#[path = "consumer/overlays.rs"]
mod overlays;
#[path = "consumer/toasts.rs"]
mod toasts;

/// The color of the logical point (`x`, `y`) of the last frame as `0xRRGGBB`.
fn pixel(screen: &Screen, x: f32, y: f32) -> u32 {
    let scale = screen.scale();
    screen
        .screenshot()
        .pixel((x * scale) as u32, (y * scale) as u32)
        .expect("invariant: point inside the frame")
}

/// The color `inset` logical pixels inside the left edge of `id`, at mid height.
fn pixel_in(screen: &Screen, id: &str, inset: f32) -> u32 {
    let rect = screen.get_by_test_id(id).rect().expect("laid out");
    pixel(screen, rect.x + inset, rect.y + rect.height / 2.0)
}

/// The largest channel difference between two `0xRRGGBB` colors.
fn distance(a: u32, b: u32) -> u32 {
    [16, 8, 0]
        .iter()
        .map(|shift| ((a >> shift) & 0xff).abs_diff((b >> shift) & 0xff))
        .max()
        .unwrap_or(0)
}

fn assert_clean(screen: &Screen) {
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn options() -> RenderOptions {
    RenderOptions {
        theme: basic::theme(kivact::style::ThemeMode::Light),
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}
