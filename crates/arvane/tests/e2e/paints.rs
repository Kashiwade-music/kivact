//! Role: Paint rendering contract tests.
//!
//! Paints: radial (circle and ellipse) and conic gradients, repeating and
//! reflected gradients with hard stops, and patterns tiling an image or a
//! gradient tile in a constant number of commands.

use std::sync::Arc;

use kivact_types::{
    DrawCommand, Extend, Gradient, GradientShape, ImageData, Paint, Pattern, PatternSource, Rgba,
    Shape,
};

use crate::support::{
    BLUE, RED, WHITE, assert_near, color, color_alpha, open, pixel, pt, rect, show,
};

fn fill_rect(x: f32, y: f32, width: f32, height: f32, paint: Paint) -> DrawCommand {
    DrawCommand::FillShape {
        shape: Shape::Rect(rect(x, y, width, height)),
        paint,
    }
}

fn gradient(shape: GradientShape, stops: Vec<(f32, Rgba)>, extend: Extend) -> Paint {
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

#[test]
fn a_two_circle_gradient_runs_from_one_circle_to_the_other() {
    let mut window = open(200.0, 100.0);
    let red_to_blue = vec![(0.0, color(RED)), (1.0, color(BLUE))];
    show(
        &mut window,
        vec![
            // Nested circles: red inside 20, blue outside 40.
            fill_rect(
                0.0,
                0.0,
                100.0,
                100.0,
                gradient(
                    GradientShape::RadialTwoCircle {
                        start_center: pt(50.0, 50.0),
                        start_radius: 20.0,
                        end_center: pt(50.0, 50.0),
                        end_radius: 40.0,
                    },
                    red_to_blue.clone(),
                    Extend::Pad,
                ),
            ),
            // A point growing into a circle to the right.
            fill_rect(
                100.0,
                0.0,
                100.0,
                100.0,
                gradient(
                    GradientShape::RadialTwoCircle {
                        start_center: pt(130.0, 50.0),
                        start_radius: 0.0,
                        end_center: pt(150.0, 50.0),
                        end_radius: 40.0,
                    },
                    red_to_blue,
                    Extend::Pad,
                ),
            ),
        ],
    );
    assert_near(pixel(&window, 50, 50), RED, 4, "inside the first circle");
    assert_near(pixel(&window, 80, 50), 0x800080, 10, "half way between");
    assert_near(pixel(&window, 95, 50), BLUE, 4, "outside the second");
    assert_near(pixel(&window, 130, 50), RED, 8, "the point");
    // Offset 2/3 is the circle at x = 143.3 of radius 26.7.
    assert_near(pixel(&window, 170, 50), 0x5500aa, 14, "along the cone");
}

#[test]
fn repeating_and_reflected_gradients_keep_hard_stops_hard() {
    let mut window = open(100.0, 40.0);
    let (black, white) = (color(0x000000), color(WHITE));
    let line = GradientShape::Linear {
        start: pt(0.0, 0.0),
        end: pt(10.0, 0.0),
    };
    show(
        &mut window,
        vec![
            // Black for 5 px, white for 5 px, over and over.
            fill_rect(
                0.0,
                0.0,
                100.0,
                20.0,
                gradient(
                    line,
                    vec![(0.0, black), (0.5, black), (0.5, white), (1.0, white)],
                    Extend::Repeat,
                ),
            ),
            // Black to white, then back.
            fill_rect(
                0.0,
                20.0,
                100.0,
                20.0,
                gradient(line, vec![(0.0, black), (1.0, white)], Extend::Reflect),
            ),
        ],
    );
    for period in 0..10 {
        let x = 10 * period;
        assert_eq!(pixel(&window, x, 10), 0x000000, "dark at {x}");
        assert_eq!(
            pixel(&window, x + 4, 10),
            0x000000,
            "dark up to the hard stop at {x}"
        );
        assert_eq!(
            pixel(&window, x + 5, 10),
            WHITE,
            "light right after it at {x}"
        );
        assert_eq!(
            pixel(&window, x + 9, 10),
            WHITE,
            "light to the period's end at {x}"
        );
    }
    let level = |x: u32| crate::support::channels(pixel(&window, x, 30))[0];
    assert!(level(1) < 40 && level(9) > 215, "black to white");
    assert!(level(11) > 215 && level(19) < 40, "then white to black");
    assert!(level(21) < 40, "and again");
}

/// An 8 × 8 image: red top-left and bottom-right quarters, blue others.
fn checker() -> ImageData {
    let mut rgba = Vec::with_capacity(8 * 8 * 4);
    for y in 0..8 {
        for x in 0..8 {
            let red = (x < 4) == (y < 4);
            rgba.extend_from_slice(if red {
                &[255, 0, 0, 255]
            } else {
                &[0, 0, 255, 255]
            });
        }
    }
    ImageData {
        width: 8,
        height: 8,
        rgba: Arc::from(rgba),
    }
}

#[test]
fn a_pattern_tiles_an_image_over_any_area_in_one_command() {
    let mut window = open(160.0, 80.0);
    let pattern = |repeat_y: bool| {
        Paint::Pattern(Pattern {
            source: PatternSource::Image {
                image: checker(),
                color_matrix: None,
            },
            tile: rect(0.0, 0.0, 8.0, 8.0),
            repeat_x: true,
            repeat_y,
        })
    };
    show(
        &mut window,
        vec![
            fill_rect(0.0, 0.0, 80.0, 80.0, pattern(true)),
            // Repeated along x only: one row of tiles.
            fill_rect(80.0, 0.0, 80.0, 80.0, pattern(false)),
        ],
    );
    for (x, y) in [(1, 1), (9, 1), (73, 73), (13, 5), (5, 13)] {
        assert_eq!(pixel(&window, x, y), RED, "red quarter at ({x}, {y})");
    }
    for (x, y) in [(5, 1), (1, 5), (77, 73), (9, 5)] {
        assert_eq!(pixel(&window, x, y), BLUE, "blue quarter at ({x}, {y})");
    }
    assert_eq!(pixel(&window, 81, 1), RED, "the single row");
    assert_eq!(pixel(&window, 85, 1), BLUE, "the single row");
    assert_eq!(pixel(&window, 81, 9), WHITE, "nothing below it");
}

#[test]
fn a_pattern_tiles_a_gradient_dot_and_follows_its_offset() {
    let mut window = open(120.0, 60.0);
    let dot = Gradient {
        shape: GradientShape::Radial {
            center: pt(6.0, 6.0),
            radius_x: 3.0,
            radius_y: 3.0,
        },
        stops: vec![
            (0.0, color(0x000000)),
            (1.0, color(0x000000)),
            (1.0, color_alpha(0x000000, 0.0)),
        ],
        extend: Extend::Pad,
    };
    show(
        &mut window,
        vec![fill_rect(
            0.0,
            0.0,
            120.0,
            60.0,
            Paint::Pattern(Pattern {
                source: PatternSource::Gradient(dot),
                tile: rect(2.0, 0.0, 12.0, 12.0),
                repeat_x: true,
                repeat_y: true,
            }),
        )],
    );
    // A dot at (8, 6) + 12 n each way.
    for (x, y) in [(8, 6), (20, 6), (8, 18), (104, 54)] {
        assert_eq!(pixel(&window, x, y), 0x000000, "a dot at ({x}, {y})");
    }
    for (x, y) in [(2, 0), (14, 12), (13, 6), (8, 11)] {
        assert_eq!(pixel(&window, x, y), WHITE, "between dots at ({x}, {y})");
    }
}

#[test]
fn a_pattern_image_goes_through_its_color_matrix() {
    let mut window = open(32.0, 16.0);
    show(
        &mut window,
        vec![fill_rect(
            0.0,
            0.0,
            32.0,
            16.0,
            Paint::Pattern(Pattern {
                source: PatternSource::Image {
                    image: checker(),
                    color_matrix: Some(kivact_types::ColorMatrix::grayscale(1.0)),
                },
                tile: rect(0.0, 0.0, 8.0, 8.0),
                repeat_x: true,
                repeat_y: true,
            }),
        )],
    );
    // Red is 0.2126 gray, blue 0.0722.
    assert_near(pixel(&window, 1, 1), 0x363636, 1, "gray red");
    assert_near(pixel(&window, 5, 1), 0x121212, 1, "gray blue");
}

#[test]
fn degenerate_gradients_paint_their_last_color_or_nothing_and_do_not_fail() {
    let mut window = open(60.0, 20.0);
    let stops = vec![(0.0, color(RED)), (1.0, color(BLUE))];
    show(
        &mut window,
        vec![
            fill_rect(
                0.0,
                0.0,
                20.0,
                20.0,
                gradient(
                    GradientShape::Radial {
                        center: pt(10.0, 10.0),
                        radius_x: 0.0,
                        radius_y: 0.0,
                    },
                    stops.clone(),
                    Extend::Pad,
                ),
            ),
            fill_rect(
                20.0,
                0.0,
                20.0,
                20.0,
                gradient(
                    GradientShape::Linear {
                        start: pt(30.0, 10.0),
                        end: pt(30.0, 10.0),
                    },
                    stops.clone(),
                    Extend::Pad,
                ),
            ),
        ],
    );
    assert_eq!(
        pixel(&window, 10, 10),
        BLUE,
        "a zero radius paints the last stop"
    );
    // A conic gradient ending where it starts is invalid.
    let invalid = vec![fill_rect(
        0.0,
        0.0,
        10.0,
        10.0,
        gradient(
            GradientShape::Conic {
                center: pt(5.0, 5.0),
                start_deg: 10.0,
                end_deg: 10.0,
            },
            stops,
            Extend::Pad,
        ),
    )];
    assert!(matches!(
        crate::support::present(&mut window, invalid),
        arvane::Presented::Dropped(_)
    ));
}
