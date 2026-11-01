//! Role: Shape rendering contract tests.
//!
//! Shapes and strokes: elliptical corners, polygons and paths with arcs by
//! their fill rule, what is drawn agreeing with `Shape::contains`, and
//! stroke caps, joins, dashes and dots.

use kivact_types::protocol::{DrawCommand, PathCmd, Shape};
use kivact_types::{FillRule, LineCap, LineJoin, Stroke};

use crate::support::{gray, open, pixel, pt, show, solid};

fn stroke_shape(shape: Shape, stroke: Stroke) -> DrawCommand {
    DrawCommand::StrokeShape {
        shape,
        stroke,
        paint: solid(0x000000),
    }
}

fn path(cmds: Vec<PathCmd>) -> Shape {
    Shape::Path {
        cmds,
        rule: FillRule::NonZero,
    }
}

#[test]
fn strokes_have_caps_joins_dashes_and_dots() {
    let mut window = open(200.0, 140.0);
    let line = |y: f32| {
        path(vec![
            PathCmd::MoveTo(pt(20.0, y)),
            PathCmd::LineTo(pt(40.0, y)),
        ])
    };
    let open_corner = |x: f32| {
        path(vec![
            PathCmd::MoveTo(pt(x, 70.0)),
            PathCmd::LineTo(pt(x + 30.0, 70.0)),
            PathCmd::LineTo(pt(x + 30.0, 100.0)),
        ])
    };
    let style = |width: f32, cap: LineCap, join: LineJoin| Stroke::new(width).cap(cap).join(join);
    show(
        &mut window,
        vec![
            stroke_shape(line(10.0), style(6.0, LineCap::Butt, LineJoin::Miter)),
            stroke_shape(line(25.0), style(6.0, LineCap::Square, LineJoin::Miter)),
            stroke_shape(line(40.0), style(6.0, LineCap::Round, LineJoin::Miter)),
            stroke_shape(
                open_corner(10.0),
                style(10.0, LineCap::Butt, LineJoin::Miter),
            ),
            stroke_shape(
                open_corner(60.0),
                style(10.0, LineCap::Butt, LineJoin::Bevel),
            ),
            stroke_shape(
                open_corner(110.0),
                style(10.0, LineCap::Butt, LineJoin::Round),
            ),
            // Dashes of 10 with gaps of 10, starting 5 into the pattern.
            stroke_shape(
                path(vec![
                    PathCmd::MoveTo(pt(60.0, 20.0)),
                    PathCmd::LineTo(pt(190.0, 20.0)),
                ]),
                Stroke::new(4.0).dash([10.0, 10.0]).dash_offset(5.0),
            ),
            // Tabler's `h.01` dot and a zero-length one, round and square.
            stroke_shape(
                path(vec![
                    PathCmd::MoveTo(pt(20.0, 120.0)),
                    PathCmd::LineTo(pt(20.01, 120.0)),
                ]),
                style(6.0, LineCap::Round, LineJoin::Miter),
            ),
            stroke_shape(
                path(vec![
                    PathCmd::MoveTo(pt(40.0, 120.0)),
                    PathCmd::LineTo(pt(40.0, 120.0)),
                ]),
                style(6.0, LineCap::Round, LineJoin::Miter),
            ),
            stroke_shape(
                path(vec![PathCmd::MoveTo(pt(60.0, 120.0)), PathCmd::Close]),
                style(6.0, LineCap::Square, LineJoin::Miter),
            ),
            stroke_shape(
                path(vec![
                    PathCmd::MoveTo(pt(80.0, 120.0)),
                    PathCmd::LineTo(pt(80.0, 120.0)),
                ]),
                style(6.0, LineCap::Butt, LineJoin::Miter),
            ),
            // A dotted line: zero-length dashes with round caps.
            stroke_shape(
                path(vec![
                    PathCmd::MoveTo(pt(100.0, 120.0)),
                    PathCmd::LineTo(pt(190.0, 120.0)),
                ]),
                style(4.0, LineCap::Round, LineJoin::Miter).dash([0.0, 10.0]),
            ),
        ],
    );
    let dark = |x: u32, y: u32| gray(pixel(&window, x, y)) < 64;
    let light = |x: u32, y: u32| gray(pixel(&window, x, y)) > 192;
    // Caps: butt ends at the end point, square and round go 3 beyond.
    assert!(dark(21, 10) && light(18, 10) && light(41, 10), "butt");
    assert!(
        dark(17, 25) && dark(42, 25) && dark(17, 22),
        "square, corners included"
    );
    assert!(
        dark(17, 40) && dark(42, 40) && light(17, 37),
        "round, corners left out"
    );
    // Joins at the right angle (outer corner at (x + 35, 65)).
    assert!(dark(44, 66), "a miter fills the outer corner");
    assert!(light(94, 66) && dark(91, 69), "a bevel cuts it");
    assert!(light(144, 66) && dark(142, 68), "a round join rounds it");
    // Dashes: on from 60 to 65, off to 75, on to 85 ...
    assert!(
        dark(62, 20) && light(70, 20) && dark(80, 20) && light(90, 20),
        "dashes"
    );
    // Dots.
    assert!(
        dark(20, 120) && dark(17, 120) && light(17, 117),
        "the h.01 dot"
    );
    assert!(
        dark(40, 120) && dark(42, 120) && light(42, 117),
        "a round dot"
    );
    assert!(dark(60, 120) && dark(62, 117), "a square dot, square");
    assert!(light(80, 120), "a butt cap draws no dot");
    // (The dash pattern ends exactly at the line's end: no dot there.)
    for x in (100..=180).step_by(10) {
        assert!(dark(x, 120), "a dot at {x}");
        assert!(light(x + 5, 120), "a gap at {}", x + 5);
    }
}
