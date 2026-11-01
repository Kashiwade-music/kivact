//! Role: Effect rendering contract tests.
//!
//! Box shadows (outer with elliptical corners and a knockout, inset) and
//! backdrops (what lies behind a shape, blurred and color-filtered, drawn
//! again without reading pixels back).

use kivact_types::{
    Backdrop, BoxShadow, ColorMatrix, DrawCommand, Radius, RoundedRect, Shape, Transform,
};

use crate::support::{
    BLUE, RED, WHITE, assert_near, channels, color, color_alpha, fill, gray, open, options, pixel,
    pt, rect, show,
};

fn shadow(
    shape: RoundedRect,
    std_dev: f32,
    spread: f32,
    inset: bool,
    knockout: bool,
) -> DrawCommand {
    DrawCommand::BoxShadow(BoxShadow {
        shape,
        offset: pt(0.0, 0.0),
        spread,
        std_dev,
        color: color(0x000000),
        inset,
        knockout,
    })
}

#[test]
fn inset_shadows_darken_the_inside_of_the_box_only() {
    let mut window = open(240.0, 80.0);
    let rounded = RoundedRect::new(rect(10.0, 10.0, 60.0, 60.0), 8.0);
    let ellipse = RoundedRect::ellipse(rect(90.0, 10.0, 60.0, 60.0));
    let ring = RoundedRect::new(rect(170.0, 10.0, 60.0, 60.0), 0.0);
    show(
        &mut window,
        vec![
            shadow(rounded, 4.0, 0.0, true, false),
            shadow(ellipse, 4.0, 0.0, true, false),
            // `box-shadow: inset 0 0 0 4px`: a hard inner ring.
            shadow(ring, 0.0, 4.0, true, false),
        ],
    );
    // Half the color at the box's edge (half the blurred hole), fading
    // inwards.
    let edge = gray(pixel(&window, 10, 40));
    assert!(
        (96..=160).contains(&edge),
        "half dark at the inner edge: {edge}"
    );
    assert!(gray(pixel(&window, 13, 40)) > edge, "lighter inwards");
    assert_eq!(pixel(&window, 40, 40), WHITE, "clear in the middle");
    assert_eq!(pixel(&window, 8, 40), WHITE, "nothing outside");
    assert!(
        gray(pixel(&window, 90, 40)) < 170,
        "dark at the ellipse's edge"
    );
    assert_eq!(pixel(&window, 120, 40), WHITE, "clear in its middle");
    assert_eq!(pixel(&window, 92, 12), WHITE, "nothing beyond its curve");
    assert_eq!(pixel(&window, 171, 40), 0x000000, "the ring");
    assert_eq!(pixel(&window, 173, 40), 0x000000, "the ring, 4 wide");
    assert_eq!(pixel(&window, 175, 40), WHITE, "inside the ring");
}

#[test]
fn outer_shadows_take_elliptical_corners_and_can_leave_the_box_out() {
    let mut window = open(240.0, 100.0);
    let ellipse = |x: f32| RoundedRect {
        rect: rect(x, 20.0, 80.0, 60.0),
        radii: [Radius { x: 40.0, y: 30.0 }; 4],
    };
    show(
        &mut window,
        vec![
            shadow(ellipse(20.0), 4.0, 0.0, false, false),
            shadow(ellipse(140.0), 4.0, 0.0, false, true),
        ],
    );
    assert!(
        gray(pixel(&window, 60, 50)) < 8,
        "beneath the box without a knockout"
    );
    assert_eq!(pixel(&window, 180, 50), WHITE, "left out with one");
    for (x, y) in [(60, 82), (180, 82), (17, 50), (137, 50)] {
        let g = gray(pixel(&window, x, y));
        assert!(g > 64 && g < 240, "the falloff at ({x}, {y}): {g}");
    }
    // The ellipse's corners: the rect's corner is not shadowed.
    assert_eq!(pixel(&window, 21, 21), WHITE);
    assert_eq!(pixel(&window, 141, 21), WHITE);
    // A circular box shadow draws as the fast `Shadow` does.
    let circular = RoundedRect::new(rect(20.0, 20.0, 80.0, 60.0), 10.0);
    show(&mut window, vec![shadow(circular, 4.0, 0.0, false, false)]);
    let fast = window.frame().expect("a frame").2.to_vec();
    show(
        &mut window,
        vec![DrawCommand::Shadow {
            rect: rect(20.0, 20.0, 80.0, 60.0),
            radii: [10.0; 4],
            std_dev: 4.0,
            color: color(0x000000),
        }],
    );
    assert_eq!(window.frame().expect("a frame").2, &fast[..]);
}

#[test]
fn offset_and_spread_move_and_grow_the_shadow() {
    let mut window = open(120.0, 120.0);
    show(
        &mut window,
        vec![DrawCommand::BoxShadow(BoxShadow {
            shape: RoundedRect::new(rect(20.0, 20.0, 40.0, 40.0), 0.0),
            offset: pt(20.0, 10.0),
            spread: 10.0,
            std_dev: 0.0,
            color: color(RED),
            inset: false,
            knockout: true,
        })],
    );
    // The shadow covers (30, 20)..(90, 80), minus the box.
    assert_eq!(pixel(&window, 31, 61), RED);
    assert_eq!(pixel(&window, 88, 78), RED);
    assert_eq!(pixel(&window, 91, 50), WHITE);
    assert_eq!(pixel(&window, 40, 40), WHITE, "the box is left out");
}

/// Black and white vertical stripes, 2 px each, over `width` × `height`.
fn stripes(width: f32, height: f32) -> Vec<DrawCommand> {
    (0..(width / 4.0) as u32)
        .map(|i| fill(4.0 * i as f32, 0.0, 2.0, height, 0x000000))
        .collect()
}

#[test]
fn a_backdrop_blurs_what_is_behind_its_shape_and_nothing_else() {
    let mut window = open(200.0, 100.0);
    let mut commands = stripes(200.0, 100.0);
    commands.push(DrawCommand::Backdrop(Backdrop {
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

#[test]
fn a_backdrop_filters_colors_and_follows_its_groups() {
    let mut window = open(200.0, 60.0);
    let mut commands = vec![
        fill(0.0, 0.0, 200.0, 30.0, RED),
        fill(0.0, 30.0, 200.0, 30.0, BLUE),
        // Inside a moved, clipped group: the replayed background is not
        // moved by it.
        DrawCommand::PushTransform(Transform::translate(100.0, 0.0)),
        DrawCommand::PushClip(rect(0.0, 0.0, 100.0, 60.0)),
        DrawCommand::Backdrop(Backdrop {
            shape: Shape::Rect(rect(20.0, 10.0, 60.0, 40.0)),
            std_dev: 0.0,
            color_matrix: Some(ColorMatrix::grayscale(1.0)),
        }),
        // Glass over it.
        DrawCommand::Rect {
            rect: rect(20.0, 10.0, 60.0, 40.0),
            fill: kivact_types::Paint::Solid(color_alpha(WHITE, 0.0)),
        },
        DrawCommand::PopClip,
        DrawCommand::PopTransform,
    ];
    // A second backdrop beneath which nothing reaches far away.
    commands.push(DrawCommand::Backdrop(Backdrop {
        shape: Shape::Rect(rect(20.0, 10.0, 40.0, 40.0)),
        std_dev: 2.0,
        color_matrix: Some(ColorMatrix::saturate(0.0)),
    }));
    show(&mut window, commands);
    let gray_red = 0x363636;
    let gray_blue = 0x121212;
    assert_near(
        pixel(&window, 130, 20),
        gray_red,
        1,
        "red behind the gray backdrop",
    );
    assert_near(pixel(&window, 130, 40), gray_blue, 1, "blue behind it");
    assert_eq!(pixel(&window, 110, 20), RED, "the backdrop's outside");
    let [r, g, b] = channels(pixel(&window, 40, 15));
    assert!(r == g && g == b, "desaturated: {r} {g} {b}");
    assert_eq!(pixel(&window, 10, 20), RED, "outside the second");
}

/// A blurred backdrop over the whole frame, its left half red and its
/// right half blue, on a CPU or GPU window.
fn edge_backdrop(gpu: bool) {
    let mut window =
        arvane::test_display::VirtualWindow::open(arvane::test_display::VirtualWindowOptions {
            gpu,
            ..options(200.0, 100.0)
        })
        .expect("a test window opens");
    let commands = vec![
        fill(0.0, 0.0, 100.0, 100.0, RED),
        fill(100.0, 0.0, 100.0, 100.0, BLUE),
        DrawCommand::Backdrop(Backdrop {
            shape: Shape::Rect(rect(0.0, 0.0, 200.0, 100.0)),
            std_dev: 8.0,
            color_matrix: None,
        }),
    ];
    show(&mut window, commands);
    // The blur reads the frame mirrored across its edges, not the
    // transparent outside: the edges and corners keep their colors.
    for (x, y) in [(0, 0), (0, 50), (0, 99), (50, 0), (50, 99)] {
        assert_near(
            pixel(&window, x, y),
            RED,
            6,
            &format!("red at ({x}, {y}) (gpu: {gpu})"),
        );
    }
    for (x, y) in [(199, 0), (199, 50), (199, 99), (150, 0), (150, 99)] {
        assert_near(
            pixel(&window, x, y),
            BLUE,
            6,
            &format!("blue at ({x}, {y}) (gpu: {gpu})"),
        );
    }
    // Across the halves, the blur mixes them.
    let [r, _, b] = channels(pixel(&window, 100, 50));
    assert!(
        r > 64 && b > 64,
        "mixed at the middle (gpu: {gpu}): {r} {b}"
    );
}

#[test]
fn a_backdrop_at_the_frame_edges_blurs_the_frame_mirrored_there() {
    edge_backdrop(false);
    if let Some(reason) = arvane::test_display::gpu_unavailable() {
        eprintln!("SKIPPED the GPU half: {reason}");
        return;
    }
    edge_backdrop(true);
}
