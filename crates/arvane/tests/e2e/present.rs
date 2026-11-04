//! Role: Presentation contract tests.
//!
//! Presenting a scene: the frame's pixels on the CPU (deterministic), the
//! frame plan of a window beyond the frame budget, invalid scenes, and the
//! GPU's frame against the CPU's.

use std::sync::Arc;

use crate::support::Harness;
use arvane::Renderer;
use arvane::gpu_unavailable;
use kivact_types::render::{Paint, RenderCmd};
use kivact_types::{ComputedTextStyle, Pixels};
use kivact_types::{Point, Size};

use crate::support::{
    BLUE, RED, WHITE, color, color_alpha, engine, fill, gpu_against_cpu, ink, open, pixel, rect,
    show,
};

#[test]
fn a_solid_rect_covers_exactly_its_pixels() {
    let mut window = open(64.0, 48.0);
    show(&mut window, vec![fill(10.0, 8.0, 20.0, 12.0, RED)]);
    let (width, height, _) = window.probe().frame().expect("a frame");
    assert_eq!((width, height), (64, 48));
    assert_eq!(pixel(&window, 10, 8), RED);
    assert_eq!(pixel(&window, 29, 19), RED);
    assert_eq!(pixel(&window, 9, 8), WHITE);
    assert_eq!(pixel(&window, 30, 19), WHITE);
    assert_eq!(pixel(&window, 10, 20), WHITE);
    assert_eq!(window.renderer(), Some(Renderer::Cpu));
}

#[test]
fn a_clip_bounds_what_it_encloses_until_it_is_popped() {
    let mut window = open(64.0, 32.0);
    show(
        &mut window,
        vec![
            RenderCmd::PushClip(rect(8.0, 4.0, 16.0, 16.0)),
            fill(0.0, 0.0, 64.0, 32.0, RED),
            RenderCmd::PopClip,
            fill(40.0, 0.0, 8.0, 8.0, BLUE),
        ],
    );
    assert_eq!(pixel(&window, 8, 4), RED);
    assert_eq!(pixel(&window, 23, 19), RED);
    assert_eq!(pixel(&window, 7, 4), WHITE);
    assert_eq!(pixel(&window, 24, 10), WHITE);
    assert_eq!(pixel(&window, 10, 20), WHITE);
    // After the clip is popped, painting reaches the whole frame again.
    assert_eq!(pixel(&window, 44, 4), BLUE);
}

/// Shape `text` and draw its runs with the layout's origin at `origin`;
/// returns the shaped size and the first line's baseline.
fn draw_text(window: &mut Harness, text: &str, origin: Point) -> (Size, f64) {
    let style = ComputedTextStyle {
        font_size: 20.0,
        ..ComputedTextStyle::default()
    };
    let shaped = engine().shape(text, &style, None).expect("text shapes");
    let mut baseline = f64::NAN;
    let commands = shaped
        .runs
        .iter()
        .map(|run| {
            if let Some(glyph) = run.glyphs.last() {
                baseline = glyph.y + origin.y;
            }
            RenderCmd::Glyphs {
                run: std::sync::Arc::clone(run),
                origin,
                color: None,
            }
        })
        .collect();
    show(window, commands);
    (shaped.size, baseline)
}

#[test]
fn a_glyph_run_inks_inside_its_shaped_box_on_its_baseline() {
    let origin = Point { x: 10.0, y: 6.0 };
    let mut window = open(160.0, 48.0);
    let (size, baseline) = draw_text(&mut window, "HILT", origin);
    let ((left, top, right, bottom), count) = ink(&window).expect("the text inks");
    // Coverage, not a golden: the ink lies within the shaped box ...
    assert!(
        left as f64 >= origin.x - 1.0,
        "ink starts in the box: {left}"
    );
    assert!(
        top as f64 >= origin.y,
        "ink starts below the box's top: {top}"
    );
    assert!(
        right as f64 <= origin.x + size.width + 1.0,
        "ink ends in the box: {right} > {}",
        origin.x + size.width
    );
    assert!(
        (bottom as f64) < origin.y + size.height,
        "ink ends above the box's bottom: {bottom}"
    );
    // ... capitals stand on the baseline (hinted to the pixel grid) ...
    assert!(
        (bottom as f64 - (baseline - 1.0)).abs() <= 1.0,
        "capitals end on the baseline {baseline}: {bottom}"
    );
    // ... and cover a fair share of it (four stems and bars).
    assert!(count > 60, "the glyphs are drawn: {count} pixels");
    // A descender goes below the baseline.
    let mut window = open(160.0, 48.0);
    let (_, baseline) = draw_text(&mut window, "Hgy", origin);
    let ((_, _, _, bottom), _) = ink(&window).expect("the text inks");
    assert!(
        bottom as f64 > baseline + 2.0,
        "descenders reach below the baseline {baseline}: {bottom}"
    );
}

/// Text, a rounded rect, a shadow, a gradient, translucency, a clip and a
/// stretched image: the paint kinds of a typical frame.
fn representative() -> Vec<RenderCmd> {
    let style = ComputedTextStyle {
        font_size: 18.0,
        ..ComputedTextStyle::default()
    };
    let shaped = engine().shape("Arvane frames", &style, None).expect("text");
    let mut commands = vec![
        RenderCmd::Shadow {
            rect: rect(20.0, 20.0, 120.0, 60.0),
            radii: [8.0; 4],
            std_dev: 6.0,
            color: color_alpha(0x000000, 0.4),
        },
        RenderCmd::RoundedRect {
            rect: rect(20.0, 20.0, 120.0, 60.0),
            radius: 8.0,
            fill: Paint::LinearGradient {
                angle_deg: 0.0,
                stops: vec![(0.0, color(0x3366cc)), (1.0, color(0xcc6633))],
            },
        },
        RenderCmd::PushClip(rect(160.0, 20.0, 60.0, 60.0)),
        RenderCmd::Rect {
            rect: rect(150.0, 10.0, 90.0, 90.0),
            fill: Paint::Solid(color_alpha(RED, 0.5)),
        },
        RenderCmd::PopClip,
        RenderCmd::Image {
            image: Pixels {
                width: 2,
                height: 2,
                rgba: Arc::from(
                    &[
                        255u8, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 255, 255, 255, 255, 64,
                    ][..],
                ),
            },
            rect: rect(20.0, 100.0, 80.0, 40.0),
            opacity: 0.8,
            color_matrix: None,
        },
    ];
    commands.extend(shaped.runs.iter().map(|run| RenderCmd::Glyphs {
        run: std::sync::Arc::clone(run),
        origin: Point { x: 120.0, y: 110.0 },
        color: None,
    }));
    commands
}

#[test]
fn the_gpu_frame_matches_the_cpu_frame_within_the_tolerance() {
    if let Some(reason) = gpu_unavailable() {
        eprintln!("SKIPPED the_gpu_frame_matches_the_cpu_frame_within_the_tolerance: {reason}");
        return;
    }
    gpu_against_cpu((260.0, 160.0), &[representative()])[0]
        .assert_within_tolerance("the representative scene");
}
