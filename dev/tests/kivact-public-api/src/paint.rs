//! Role: Pixel helpers the E2E binaries share: colors, boxes on a canvas and comparisons.

use kivact::prelude::*;
use kivact::style::Position;
use kivact::test::Screenshot;

pub const RED: u32 = 0xff0000;
pub const BLUE: u32 = 0x0000ff;
pub const GREEN: u32 = 0x00ff00;
pub const WHITE: u32 = 0xffffff;
pub const BLACK: u32 = 0x000000;

/// A white 800 × 600 canvas holding `children`, each placed with [`at`].
pub fn canvas_of(children: impl IntoView) -> Node {
    node()
        .child(children)
        .style(style().size(800, 600).background(Color::WHITE))
}

/// A box of `w × h` at `(x, y)` of the canvas.
pub fn at(x: i32, y: i32, w: i32, h: i32) -> Style {
    style()
        .position(Position::Absolute)
        .left(x)
        .top(y)
        .size(w, h)
}

/// The color of a pixel (0xRRGGBB).
#[track_caller]
pub fn pixel(shot: &Screenshot, x: u32, y: u32) -> u32 {
    shot.pixel(x, y).expect("in the frame")
}

/// The largest channel difference between two colors.
pub fn distance(a: u32, b: u32) -> u32 {
    (0..3)
        .map(|i| ((a >> (8 * i)) & 0xff).abs_diff((b >> (8 * i)) & 0xff))
        .max()
        .unwrap_or(0)
}

/// The pixel at `(x, y)` is `want`, within `tolerance` per channel.
#[track_caller]
pub fn expect(shot: &Screenshot, x: u32, y: u32, want: u32, tolerance: u32, what: &str) {
    let got = pixel(shot, x, y);
    assert!(
        distance(got, want) <= tolerance,
        "{what}: pixel ({x}, {y}) is {got:06x}, want {want:06x} (±{tolerance})"
    );
}
