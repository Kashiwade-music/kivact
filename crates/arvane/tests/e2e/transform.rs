//! Role: Transform contract tests.
//!
//! Transforms: every command between `PushTransform` and `PopTransform`
//! is mapped (rects, clips, images, glyphs), nested transforms compose,
//! and a transform that flattens everything draws nothing.

use kivact_types::Point;
use kivact_types::protocol::{DrawCommand, Transform};

use crate::support::{BLUE, RED, WHITE, fill, open, pixel, pt, show};

/// The pixel whose center is nearest `p` (logical = physical at scale 1).
fn at(p: Point) -> (u32, u32) {
    (p.x.floor() as u32, p.y.floor() as u32)
}

#[test]
fn a_transform_rotates_and_skews_what_it_encloses_until_it_is_popped() {
    let mut window = open(160.0, 80.0);
    let rotate = Transform::rotate_deg(45.0).about(pt(40.0, 40.0));
    let skew = Transform::skew_deg(30.0, 0.0).about(pt(110.0, 40.0));
    show(
        &mut window,
        vec![
            DrawCommand::PushTransform(rotate),
            fill(20.0, 20.0, 40.0, 40.0, RED),
            DrawCommand::PopTransform,
            DrawCommand::PushTransform(skew),
            fill(90.0, 20.0, 40.0, 40.0, BLUE),
            DrawCommand::PopTransform,
            // After the pops, nothing is transformed.
            fill(150.0, 0.0, 10.0, 10.0, RED),
        ],
    );
    // Points well inside the squares land inside the drawn shapes ...
    for (x, y) in [
        (25.0, 25.0),
        (55.0, 25.0),
        (40.0, 40.0),
        (25.0, 55.0),
        (55.0, 55.0),
    ] {
        let (px, py) = at(rotate.apply(pt(x, y)));
        assert_eq!(pixel(&window, px, py), RED, "rotated ({x}, {y})");
    }
    for (x, y) in [(95.0, 25.0), (125.0, 25.0), (95.0, 55.0), (125.0, 55.0)] {
        let (px, py) = at(skew.apply(pt(x, y)));
        assert_eq!(pixel(&window, px, py), BLUE, "skewed ({x}, {y})");
    }
    // ... and the untransformed squares' corners are outside.
    assert_eq!(pixel(&window, 21, 21), WHITE, "the rotated square's corner");
    assert_eq!(
        pixel(&window, 91, 58),
        WHITE,
        "the unskewed square's bottom-left corner"
    );
    assert_eq!(pixel(&window, 155, 5), RED, "after the pop");
}
