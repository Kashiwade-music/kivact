//! Role: Paint rendering contract tests.
//!
//! Paints: radial (circle and ellipse) and conic gradients, repeating and
//! reflected gradients with hard stops, and patterns tiling an image or a
//! gradient tile in a constant number of commands.

use kivact_types::render::{Gradient, GradientShape, Paint, RenderCmd};
use kivact_types::scene::Shape;
use kivact_types::{Extend, Rgba};

use crate::support::{BLUE, RED, assert_near, color, open, pixel, pt, rect, show};

fn fill_rect(x: f64, y: f64, width: f64, height: f64, paint: Paint) -> RenderCmd {
    RenderCmd::FillShape {
        shape: Shape::Rect(rect(x, y, width, height)),
        paint,
    }
}

fn gradient(shape: GradientShape, stops: Vec<(f64, Rgba)>, extend: Extend) -> Paint {
    Paint::Gradient(Gradient {
        shape,
        stops,
        extend,
    })
}

#[test]
fn radial_and_conic_gradients_spread_from_their_center() {
    let mut window = open(300.0, 100.0);
    let red_to_blue = vec![(0.0, color(RED)), (1.0, color(BLUE))];
    show(
        &mut window,
        vec![
            fill_rect(
                0.0,
                0.0,
                100.0,
                100.0,
                gradient(
                    GradientShape::Radial {
                        center: pt(50.0, 50.0),
                        radius_x: 40.0,
                        radius_y: 40.0,
                    },
                    red_to_blue.clone(),
                    Extend::Pad,
                ),
            ),
            fill_rect(
                100.0,
                0.0,
                100.0,
                100.0,
                gradient(
                    GradientShape::Radial {
                        center: pt(150.0, 50.0),
                        radius_x: 40.0,
                        radius_y: 10.0,
                    },
                    red_to_blue.clone(),
                    Extend::Pad,
                ),
            ),
            fill_rect(
                200.0,
                0.0,
                100.0,
                100.0,
                gradient(
                    GradientShape::Conic {
                        center: pt(250.0, 50.0),
                        start_deg: 0.0,
                        end_deg: 360.0,
                    },
                    red_to_blue,
                    Extend::Pad,
                ),
            ),
        ],
    );
    assert_near(pixel(&window, 50, 50), RED, 8, "the circle's center");
    assert_near(pixel(&window, 70, 50), 0x800080, 10, "half way out");
    assert_near(pixel(&window, 50, 30), 0x800080, 10, "half way up");
    assert_near(pixel(&window, 95, 50), BLUE, 4, "padded beyond the radius");
    assert_near(
        pixel(&window, 170, 50),
        0x800080,
        10,
        "the ellipse half way along x",
    );
    // (150.5, 55.5) is 0.55 of the way.
    assert_near(pixel(&window, 150, 55), 0x73008c, 6, "and along y, sooner");
    assert_near(pixel(&window, 150, 70), BLUE, 4, "padded beyond it");
    // Clockwise from the positive x axis.
    assert_near(
        pixel(&window, 280, 52),
        RED,
        16,
        "the conic's start, to the right",
    );
    assert_near(
        pixel(&window, 250, 80),
        0xbf0040,
        14,
        "a quarter turn, below",
    );
    assert_near(
        pixel(&window, 220, 50),
        0x800080,
        14,
        "half a turn, to the left",
    );
    assert_near(
        pixel(&window, 250, 20),
        0x4000bf,
        14,
        "three quarters, above",
    );
}
