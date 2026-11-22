//! Role: Render options, pixel probes and the error check shared by consumer tests.

use kivact::test::Screen;

/// The color of the logical point (`x`, `y`) of the last frame as `0xRRGGBB`.
pub(crate) fn pixel(screen: &Screen, x: f64, y: f64) -> u32 {
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

/// The color `inset` logical pixels inside the left edge of `id`, at mid height.
pub(crate) fn pixel_in(screen: &Screen, id: &str, inset: f64) -> u32 {
    let rect = screen.get_by_test_id(id).rect().expect("laid out");
    pixel(screen, rect.x + inset, rect.y + rect.height / 2.0)
}
