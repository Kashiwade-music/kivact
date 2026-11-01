//! Role: Shape rendering contract tests.
//!
//! Shapes and strokes: elliptical corners, polygons and paths with arcs by
//! their fill rule, what is drawn agreeing with `Shape::contains`, and
//! stroke caps, joins, dashes and dots.

use kivact_types::{
    DrawCommand, FillRule, LineCap, LineJoin, PathCmd, Radius, RoundedRect, Shape, Stroke,
};

use crate::support::{RED, WHITE, gray, open, pixel, pt, rect, show, solid};

fn fill_shape(shape: Shape) -> DrawCommand {
    DrawCommand::FillShape {
        shape,
        paint: solid(RED),
    }
}

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

/// Every fully covered pixel's center is inside `shape` and every
/// untouched one outside: what is drawn and what is hit agree.
#[track_caller]
fn drawn_as_contained(window: &arvane::test_display::VirtualWindow, shape: &Shape) {
    let (width, height, pixels) = window.frame().expect("a frame");
    let (mut inside, mut outside) = (0, 0);
    for y in 0..height {
        for x in 0..width {
            let center = pt(x as f32 + 0.5, y as f32 + 0.5);
            match pixels[(y * width + x) as usize] {
                RED => {
                    inside += 1;
                    assert!(shape.contains(center), "drawn at ({x}, {y}) but not inside");
                }
                WHITE => {
                    outside += 1;
                    assert!(
                        !shape.contains(center),
                        "inside at ({x}, {y}) but not drawn"
                    );
                }
                _ => {}
            }
        }
    }
    assert!(inside > 100 && outside > 100, "{inside} in, {outside} out");
}

#[test]
fn elliptical_corners_are_drawn_where_they_are_hit() {
    let shapes = [
        Shape::RoundedRect(RoundedRect {
            rect: rect(10.0, 10.0, 120.0, 60.0),
            radii: [
                Radius { x: 40.0, y: 20.0 },
                Radius { x: 10.0, y: 30.0 },
                Radius { x: 0.0, y: 0.0 },
                Radius { x: 60.0, y: 30.0 },
            ],
        }),
        // Radii adding up to more than a side shrink to fit.
        Shape::RoundedRect(RoundedRect::new(rect(10.0, 10.0, 120.0, 60.0), 50.0)),
        Shape::RoundedRect(RoundedRect::ellipse(rect(10.0, 10.0, 120.0, 60.0))),
    ];
    for shape in shapes {
        let mut window = open(140.0, 80.0);
        show(&mut window, vec![fill_shape(shape.clone())]);
        drawn_as_contained(&window, &shape);
    }
}

#[test]
fn polygons_and_paths_fill_by_their_rule() {
    let star = |rule| Shape::Polygon {
        points: vec![
            pt(50.0, 5.0),
            pt(76.0, 85.0),
            pt(8.0, 35.0),
            pt(92.0, 35.0),
            pt(24.0, 85.0),
        ],
        rule,
    };
    let mut window = open(100.0, 90.0);
    show(&mut window, vec![fill_shape(star(FillRule::EvenOdd))]);
    assert_eq!(
        pixel(&window, 50, 50),
        WHITE,
        "even-odd leaves the middle out"
    );
    assert_eq!(pixel(&window, 50, 20), RED, "and fills a point");
    drawn_as_contained(&window, &star(FillRule::EvenOdd));
    show(&mut window, vec![fill_shape(star(FillRule::NonZero))]);
    assert_eq!(pixel(&window, 50, 50), RED, "non-zero fills it");
    drawn_as_contained(&window, &star(FillRule::NonZero));
    // Arcs: a full circle (canvas `arc`) and a half disc closed by an SVG
    // arc.
    let shape = path(vec![
        PathCmd::Arc {
            center: pt(25.0, 45.0),
            rx: 20.0,
            ry: 20.0,
            x_rotation_deg: 0.0,
            start_deg: 0.0,
            sweep_deg: 360.0,
        },
        PathCmd::Close,
        PathCmd::MoveTo(pt(55.0, 45.0)),
        PathCmd::ArcTo {
            rx: 20.0,
            ry: 20.0,
            x_rotation_deg: 0.0,
            large_arc: false,
            sweep: true,
            to: pt(95.0, 45.0),
        },
        PathCmd::Close,
    ]);
    show(&mut window, vec![fill_shape(shape.clone())]);
    assert_eq!(pixel(&window, 25, 45), RED, "the circle's middle");
    assert_eq!(
        pixel(&window, 75, 35),
        RED,
        "the half disc, above the chord (clockwise)"
    );
    assert_eq!(pixel(&window, 75, 55), WHITE, "nothing below the chord");
    drawn_as_contained(&window, &shape);
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
    let style = |width: f32, cap: LineCap, join: LineJoin| Stroke {
        cap,
        join,
        ..Stroke::new(width)
    };
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
                Stroke {
                    dash: vec![10.0, 10.0],
                    dash_offset: 5.0,
                    ..Stroke::new(4.0)
                },
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
                Stroke {
                    dash: vec![0.0, 10.0],
                    ..style(4.0, LineCap::Round, LineJoin::Miter)
                },
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
