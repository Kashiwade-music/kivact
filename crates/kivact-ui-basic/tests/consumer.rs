//! Role: External-consumer tests: no internal Kivact or renderer imports are available.

use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen};
use kivact_ui_basic as basic;

#[path = "consumer/builders.rs"]
mod builders;
#[path = "consumer/cleanup.rs"]
mod cleanup;
#[path = "consumer/code_block.rs"]
mod code_block;
#[path = "consumer/data.rs"]
mod data;
#[path = "consumer/data_table.rs"]
mod data_table;
#[path = "consumer/display.rs"]
mod display;
#[path = "consumer/display_details.rs"]
mod display_details;
#[path = "consumer/forms.rs"]
mod forms;
#[path = "consumer/forms_states.rs"]
mod forms_states;
#[path = "consumer/foundation.rs"]
mod foundation;
#[path = "consumer/motion.rs"]
mod motion;
#[path = "consumer/navigation.rs"]
mod navigation;
#[path = "consumer/navigation_layout.rs"]
mod navigation_layout;
#[path = "consumer/navigation_parts.rs"]
mod navigation_parts;
#[path = "consumer/overlays.rs"]
mod overlays;
#[path = "consumer/overlays_states.rs"]
mod overlays_states;
#[path = "consumer/recipes.rs"]
mod recipes;
#[path = "consumer/standard_overlays.rs"]
mod standard_overlays;
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

/// The width the caller-wins tests ask for.
const CALLER_WIDTH: f32 = 301.0;
/// The fill the caller-wins tests ask for: magenta, a color no kit token has.
const CALLER_FILL: u32 = 0xff00ff;

/// A caller's style: a width, side padding and a fill, all of which a kit
/// component also sets by default. The caller's `.style(..)` comes after the
/// constructor's own, so it has to win.
fn caller_style() -> kivact::style::Style {
    style()
        .width(CALLER_WIDTH)
        .padding_x(24)
        .background(rgb(CALLER_FILL))
}

/// Check that [`caller_style`] won on the element `id`: its fill, and its
/// width when `width` is true.
fn assert_caller_won(screen: &Screen, id: &str, width: bool) {
    if width {
        let wide = screen.get_by_test_id(id).rect().expect("laid out").width;
        assert!(
            (wide - CALLER_WIDTH).abs() < 0.6,
            "{id} is {wide} px wide, not {CALLER_WIDTH}"
        );
    }
    let fill = pixel_in(screen, id, 3.0);
    assert!(
        distance(fill, CALLER_FILL) <= 3,
        "{id} is filled {fill:06x}, not {CALLER_FILL:06x}"
    );
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
