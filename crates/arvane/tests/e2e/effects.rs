//! Role: Effect rendering contract tests.
//!
//! Box shadows (outer with elliptical corners and a knockout, inset) and
//! backdrops (what lies behind a shape, blurred and color-filtered, drawn
//! again without reading pixels back).

use kivact_types::render::{Backdrop, RenderCmd};
use kivact_types::scene::{RoundedRect, Shape};

use crate::support::{RED, WHITE, fill, gray, open, pixel, rect, show};

/// Black and white vertical stripes, 2 px each, over `width` × `height`.
fn stripes(width: f64, height: f64) -> Vec<RenderCmd> {
    (0..(width / 4.0) as u32)
        .map(|i| fill(4.0 * i as f64, 0.0, 2.0, height, 0x000000))
        .collect()
}

#[test]
fn a_backdrop_blurs_what_is_behind_its_shape_and_nothing_else() {
    let mut window = open(200.0, 100.0);
    let mut commands = stripes(200.0, 100.0);
    commands.push(RenderCmd::Backdrop(Backdrop {
        shape: Shape::RoundedRect(RoundedRect::new(rect(50.0, 20.0, 100.0, 60.0), 10.0)),
        std_dev: 4.0,
        color_matrix: None,
    }));
    // Drawn after the backdrop: not in it.
    commands.push(fill(0.0, 90.0, 200.0, 10.0, RED));
    show(&mut window, commands);
    for (x, y) in [(60, 50), (100, 30), (101, 50), (140, 70)] {
        let g = gray(pixel(&window, x, y));
        assert!(
            (96..=160).contains(&g),
            "blurred to gray at ({x}, {y}): {g}"
        );
    }
    // Outside the shape (its corner included), the stripes are untouched.
    for (x, y) in [(40, 50), (41, 50), (100, 10), (101, 10), (50, 20), (51, 21)] {
        let p = pixel(&window, x, y);
        assert!(
            p == WHITE || p == 0x000000,
            "a crisp stripe at ({x}, {y}): {p:06x}"
        );
    }
    assert_eq!(
        pixel(&window, 100, 95),
        RED,
        "what comes after is drawn as is"
    );
}
