//! Role: Transform contract tests.
//!
//! Transforms: every command between `PushTransform` and `PopTransform`
//! is mapped (rects, clips, images, glyphs), nested transforms compose,
//! and a transform that flattens everything draws nothing.

use std::sync::Arc;

use arvane::Presented;
use kivact_types::{DrawCommand, ImageData, Point, Shape, Transform};

use crate::support::{
    BLUE, RED, WHITE, assert_near, fill, ink, open, pixel, present, pt, rect, show, solid,
    text_runs,
};

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

#[test]
fn transforms_nest_and_apply_to_clips_and_images() {
    let mut window = open(120.0, 60.0);
    // Two halves of a quarter turn about (30, 30): a quarter turn.
    let half = Transform::rotate_deg(45.0).about(pt(30.0, 30.0));
    let image = ImageData {
        width: 2,
        height: 1,
        rgba: Arc::from(&[255u8, 0, 0, 255, 0, 0, 255, 255][..]),
    };
    show(
        &mut window,
        vec![
            DrawCommand::PushTransform(half),
            DrawCommand::PushTransform(half),
            // Red left, blue right; turned a quarter: red above, blue below.
            DrawCommand::Image {
                image,
                rect: rect(10.0, 20.0, 40.0, 20.0),
                opacity: 1.0,
                color_matrix: None,
            },
            DrawCommand::PopTransform,
            DrawCommand::PopTransform,
            // A clip turned with its content: a diamond.
            DrawCommand::PushTransform(Transform::rotate_deg(45.0).about(pt(90.0, 30.0))),
            DrawCommand::PushClip(rect(75.0, 15.0, 30.0, 30.0)),
            DrawCommand::PushTransform(Transform::IDENTITY),
            fill(60.0, 0.0, 60.0, 60.0, RED),
            DrawCommand::PopTransform,
            DrawCommand::PopClip,
            DrawCommand::PopTransform,
        ],
    );
    assert_eq!(pixel(&window, 30, 15), RED, "the image's left half, above");
    assert_eq!(pixel(&window, 30, 45), BLUE, "its right half, below");
    assert_eq!(pixel(&window, 15, 30), WHITE, "nothing where it was");
    assert_eq!(pixel(&window, 90, 30), RED, "the diamond's middle");
    assert_eq!(pixel(&window, 90, 12), RED, "its top vertex region");
    assert_eq!(pixel(&window, 76, 16), WHITE, "the unturned clip's corner");
}

#[test]
fn a_transform_that_flattens_everything_draws_nothing_inside() {
    let mut window = open(40.0, 20.0);
    show(
        &mut window,
        vec![
            DrawCommand::PushTransform(Transform::scale_xy(1.0, 0.0)),
            fill(0.0, 0.0, 20.0, 20.0, RED),
            DrawCommand::PushClip(rect(0.0, 0.0, 10.0, 10.0)),
            fill(0.0, 0.0, 20.0, 20.0, RED),
            DrawCommand::PopClip,
            DrawCommand::PopTransform,
            fill(20.0, 0.0, 20.0, 20.0, BLUE),
        ],
    );
    assert_eq!(pixel(&window, 5, 5), WHITE);
    assert_eq!(
        pixel(&window, 30, 10),
        BLUE,
        "drawing resumes after the pop"
    );
}

#[test]
fn rotated_text_inks_along_its_rotation_and_scaled_text_grows() {
    let mut window = open(200.0, 120.0);
    let origin = pt(30.0, 10.0);
    show(&mut window, text_runs("HILT", 20.0, origin));
    let ((l, t, r, b), upright) = ink(&window).expect("the text inks");
    let (width, height) = (r - l, b - t);
    assert!(width > height, "upright text is wide: {width} × {height}");
    // A quarter turn about the layout's origin: tall instead of wide,
    // drawn from unhinted outlines, about as much ink.
    let mut commands = vec![DrawCommand::PushTransform(
        Transform::rotate_deg(90.0).about(origin),
    )];
    commands.extend(text_runs("HILT", 20.0, origin));
    commands.push(DrawCommand::PopTransform);
    show(&mut window, commands);
    let ((l, t, r, b), turned) = ink(&window).expect("the turned text inks");
    assert!(
        (r - l).abs_diff(height) <= 3 && (b - t).abs_diff(width) <= 3,
        "turned text is {} × {} for {width} × {height}",
        r - l,
        b - t
    );
    assert!(r <= 31, "it lies left of its origin: {r}");
    assert!(
        turned.abs_diff(upright) * 4 <= upright,
        "about as much ink: {turned} for {upright}"
    );
    // Twice the size under a uniform scale (still hinted).
    let mut commands = vec![DrawCommand::PushTransform(
        Transform::scale(2.0).about(origin),
    )];
    commands.extend(text_runs("HILT", 20.0, origin));
    commands.push(DrawCommand::PopTransform);
    show(&mut window, commands);
    let ((l, t, r, b), _) = ink(&window).expect("the scaled text inks");
    assert!(
        (r - l).abs_diff(2 * width) <= 3 && (b - t).abs_diff(2 * height) <= 3,
        "scaled text is {} × {} for {width} × {height}",
        r - l,
        b - t
    );
}

#[test]
fn paints_follow_the_transform() {
    let mut window = open(80.0, 40.0);
    // A left-to-right gradient turned a half turn runs right to left.
    show(
        &mut window,
        vec![
            DrawCommand::PushTransform(Transform::rotate_deg(180.0).about(pt(40.0, 20.0))),
            DrawCommand::FillShape {
                shape: Shape::Rect(rect(0.0, 0.0, 80.0, 40.0)),
                paint: kivact_types::Paint::LinearGradient {
                    angle_deg: 0.0,
                    stops: vec![
                        (0.0, crate::support::color(RED)),
                        (1.0, crate::support::color(BLUE)),
                    ],
                },
            },
            DrawCommand::PopTransform,
        ],
    );
    assert_near(pixel(&window, 1, 20), BLUE, 8, "blue on the left");
    assert_near(pixel(&window, 78, 20), RED, 8, "red on the right");
    // Pushing more transforms than the limit is invalid.
    let mut deep =
        vec![DrawCommand::PushTransform(Transform::IDENTITY); arvane::MAX_SCENE_DEPTH + 1];
    deep.extend(vec![DrawCommand::PopTransform; arvane::MAX_SCENE_DEPTH + 1]);
    assert!(matches!(present(&mut window, deep), Presented::Dropped(_)));
    let mut ok =
        vec![DrawCommand::PushTransform(Transform::translate(1.0, 0.0)); arvane::MAX_SCENE_DEPTH];
    ok.push(DrawCommand::FillShape {
        shape: Shape::Rect(rect(0.0, 0.0, 10.0, 10.0)),
        paint: solid(RED),
    });
    ok.extend(vec![DrawCommand::PopTransform; arvane::MAX_SCENE_DEPTH]);
    // 256 nested moves by 1: the square lies beyond the frame.
    show(&mut window, ok);
    assert_eq!(pixel(&window, 5, 5), WHITE);
}
