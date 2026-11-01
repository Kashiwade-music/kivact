//! Role: Presentation contract tests.
//!
//! Presenting a scene: the frame's pixels on the CPU (deterministic), the
//! frame plan of a window beyond the frame budget, invalid scenes, and the
//! GPU's frame against the CPU's.

use std::sync::Arc;

use arvane::test_display::{VirtualWindow, VirtualWindowOptions, WindowMode, gpu_unavailable};
use arvane::{ErrorKind, FramePlan, Presented, Renderer, Window as _};
use kivact_types::{DrawCommand, ImageData, Paint, PathCmd, Point, SHADOW_REACH, Size, TextStyle};

use crate::support::{
    BLUE, RED, WHITE, assert_near, color, color_alpha, distance, engine, fill, gpu_against_cpu,
    ink, open, options, pixel, present, rect, show,
};

#[test]
fn cropped_pixelated_images_match_gpu() {
    let pixels = ImageData {
        width: 2,
        height: 1,
        rgba: Arc::from([255u8, 0, 0, 255, 0, 0, 255, 255]),
    };
    let command = DrawCommand::ImageSampled {
        image: pixels,
        rect: rect(8., 8., 32., 16.),
        source: Some(rect(1., 0., 1., 1.)),
        opacity: 0.5,
        color_matrix: None,
        pixelated: true,
    };
    let mut window = open(64., 48.);
    show(&mut window, vec![command.clone()]);
    assert_near(
        pixel(&window, 10, 10),
        0x8080ff,
        1,
        "source pixel enlarged with opacity",
    );
    let mut nearest = command.clone();
    if let DrawCommand::ImageSampled {
        source, opacity, ..
    } = &mut nearest
    {
        *source = None;
        *opacity = 1.0;
    }
    show(&mut window, vec![nearest.clone()]);
    assert_eq!(pixel(&window, 23, 10), RED, "last red pixel stays red");
    assert_eq!(
        pixel(&window, 24, 10),
        BLUE,
        "first blue pixel has no smoothing"
    );
    if let Some(reason) = gpu_unavailable() {
        eprintln!("SKIPPED cropped image GPU parity: {reason}");
        return;
    }
    for diff in gpu_against_cpu((64., 48.), &[vec![command], vec![nearest]]) {
        diff.assert_within_tolerance("cropped and whole nearest images");
    }
}

#[test]
fn empty_image_source_keeps_following_paint_on_cpu() {
    empty_image_source_keeps_following_paint(false);
}

#[test]
fn empty_image_source_keeps_following_paint_on_gpu() {
    if let Some(reason) = gpu_unavailable() {
        eprintln!("SKIPPED empty image source GPU regression: {reason}");
        return;
    }
    empty_image_source_keeps_following_paint(true);
}

fn empty_image_source_keeps_following_paint(gpu: bool) {
    let mut window = VirtualWindow::open(VirtualWindowOptions {
        gpu,
        ..options(32.0, 32.0)
    })
    .expect("a test window opens");
    for source in [rect(0., 0., 0., 1.), rect(0., 0., 1., 0.)] {
        show(
            &mut window,
            vec![
                DrawCommand::ImageSampled {
                    image: ImageData {
                        width: 1,
                        height: 1,
                        rgba: Arc::from([0u8, 0, 255, 255]),
                    },
                    rect: rect(8., 8., 16., 16.),
                    source: Some(source),
                    opacity: 0.5,
                    color_matrix: None,
                    pixelated: true,
                },
                fill(0., 0., 32., 32., RED),
            ],
        );
        assert_eq!(
            window.renderer(),
            Some(if gpu { Renderer::Gpu } else { Renderer::Cpu })
        );
        assert_eq!(
            pixel(&window, 16, 16),
            RED,
            "following paint keeps its opacity"
        );
        assert_eq!(
            pixel(&window, 2, 2),
            RED,
            "following paint keeps its full bounds"
        );
    }
}

#[test]
fn a_solid_rect_covers_exactly_its_pixels() {
    let mut window = open(64.0, 48.0);
    show(&mut window, vec![fill(10.0, 8.0, 20.0, 12.0, RED)]);
    let (width, height, _) = window.frame().expect("a frame");
    assert_eq!((width, height), (64, 48));
    assert_eq!(pixel(&window, 10, 8), RED);
    assert_eq!(pixel(&window, 29, 19), RED);
    assert_eq!(pixel(&window, 9, 8), WHITE);
    assert_eq!(pixel(&window, 30, 19), WHITE);
    assert_eq!(pixel(&window, 10, 20), WHITE);
    assert_eq!(window.renderer(), Some(Renderer::Cpu));
}

#[test]
fn a_frame_is_drawn_at_the_window_scale() {
    let mut window = VirtualWindow::open(VirtualWindowOptions {
        scale: 2.0,
        ..options(32.0, 16.0)
    })
    .expect("a test window opens");
    show(&mut window, vec![fill(0.0, 0.0, 8.0, 8.0, BLUE)]);
    let (width, height, _) = window.frame().expect("a frame");
    assert_eq!((width, height), (64, 32));
    assert_eq!(pixel(&window, 15, 15), BLUE);
    assert_eq!(pixel(&window, 16, 15), WHITE);
}

#[test]
fn a_rounded_rect_leaves_its_corners_out() {
    let mut window = open(64.0, 64.0);
    show(
        &mut window,
        vec![DrawCommand::RoundedRect {
            rect: rect(8.0, 8.0, 48.0, 48.0),
            radius: 16.0,
            fill: Paint::Solid(color(RED)),
        }],
    );
    // The corner pixel lies outside the arc; the edges' middles and the
    // center are inside.
    assert_eq!(pixel(&window, 8, 8), WHITE);
    assert_eq!(pixel(&window, 55, 55), WHITE);
    assert_eq!(pixel(&window, 32, 8), RED);
    assert_eq!(pixel(&window, 8, 32), RED);
    assert_eq!(pixel(&window, 32, 32), RED);
}

#[test]
fn a_clip_bounds_what_it_encloses_until_it_is_popped() {
    let mut window = open(64.0, 32.0);
    show(
        &mut window,
        vec![
            DrawCommand::PushClip(rect(8.0, 4.0, 16.0, 16.0)),
            fill(0.0, 0.0, 64.0, 32.0, RED),
            DrawCommand::PopClip,
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

#[test]
fn translucent_paint_blends_over_what_is_below() {
    let mut window = open(64.0, 16.0);
    let image = ImageData {
        width: 1,
        height: 1,
        rgba: Arc::from(&[0u8, 0, 255, 255][..]),
    };
    show(
        &mut window,
        vec![
            DrawCommand::Rect {
                rect: rect(0.0, 0.0, 16.0, 16.0),
                fill: Paint::Solid(color_alpha(RED, 0.5)),
            },
            // Half a red over blue.
            fill(16.0, 0.0, 16.0, 16.0, BLUE),
            DrawCommand::Rect {
                rect: rect(16.0, 0.0, 16.0, 16.0),
                fill: Paint::Solid(color_alpha(RED, 0.5)),
            },
            // An opaque image drawn at half opacity.
            DrawCommand::Image {
                image,
                rect: rect(32.0, 0.0, 16.0, 16.0),
                opacity: 0.5,
                color_matrix: None,
            },
        ],
    );
    assert_near(pixel(&window, 8, 8), 0xff8080, 1, "half red over white");
    assert_near(pixel(&window, 24, 8), 0x800080, 1, "half red over blue");
    assert_near(pixel(&window, 40, 8), 0x8080ff, 1, "a half-opaque image");
}

#[test]
fn images_are_straight_alpha_and_drawn_premultiplied() {
    let mut window = open(40.0, 20.0);
    // Two texels: a half-transparent mid gray and an opaque green, each
    // stretched over 20 × 20 pixels.
    let image = ImageData {
        width: 2,
        height: 1,
        rgba: Arc::from(&[128u8, 128, 128, 128, 0, 255, 0, 255][..]),
    };
    show(
        &mut window,
        vec![DrawCommand::Image {
            image,
            rect: rect(0.0, 0.0, 40.0, 20.0),
            opacity: 1.0,
            color_matrix: None,
        }],
    );
    // Straight (128, 128, 128, 0.5) over white is 64 + 127 = 191 per
    // channel; taking it for premultiplied would give 255.
    assert_near(pixel(&window, 4, 10), 0xbfbfbf, 2, "the gray texel");
    assert_near(pixel(&window, 36, 10), 0x00ff00, 2, "the green texel");
}

#[test]
fn a_path_fills_and_strokes() {
    let mut window = open(48.0, 48.0);
    let triangle = vec![
        PathCmd::MoveTo(Point { x: 4.0, y: 4.0 }),
        PathCmd::LineTo(Point { x: 44.0, y: 4.0 }),
        PathCmd::LineTo(Point { x: 4.0, y: 44.0 }),
        PathCmd::Close,
    ];
    show(
        &mut window,
        vec![DrawCommand::Path {
            cmds: triangle,
            fill: Some(color(RED)),
            stroke: None,
        }],
    );
    assert_eq!(pixel(&window, 10, 10), RED);
    assert_eq!(pixel(&window, 40, 40), WHITE);
}

/// Shape `text` and draw its runs with the layout's origin at `origin`;
/// returns the shaped size and the first line's baseline.
fn draw_text(window: &mut VirtualWindow, text: &str, origin: Point) -> (Size, f32) {
    let style = TextStyle {
        font_size: 20.0,
        ..TextStyle::default()
    };
    let shaped = engine().shape(text, &style, None).expect("text shapes");
    let mut baseline = f32::NAN;
    let commands = shaped
        .runs
        .iter()
        .map(|run| {
            if let Some(glyph) = run.glyphs.last() {
                baseline = glyph.y + origin.y;
            }
            DrawCommand::Glyphs {
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
        left as f32 >= origin.x - 1.0,
        "ink starts in the box: {left}"
    );
    assert!(
        top as f32 >= origin.y,
        "ink starts below the box's top: {top}"
    );
    assert!(
        right as f32 <= origin.x + size.width + 1.0,
        "ink ends in the box: {right} > {}",
        origin.x + size.width
    );
    assert!(
        (bottom as f32) < origin.y + size.height,
        "ink ends above the box's bottom: {bottom}"
    );
    // ... capitals stand on the baseline (hinted to the pixel grid) ...
    assert!(
        (bottom as f32 - (baseline - 1.0)).abs() <= 1.0,
        "capitals end on the baseline {baseline}: {bottom}"
    );
    // ... and cover a fair share of it (four stems and bars).
    assert!(count > 60, "the glyphs are drawn: {count} pixels");
    // A descender goes below the baseline.
    let mut window = open(160.0, 48.0);
    let (_, baseline) = draw_text(&mut window, "Hgy", origin);
    let ((_, _, _, bottom), _) = ink(&window).expect("the text inks");
    assert!(
        bottom as f32 > baseline + 2.0,
        "descenders reach below the baseline {baseline}: {bottom}"
    );
}

#[test]
fn a_shadow_reaches_as_far_as_its_blur_says() {
    let mut window = open(120.0, 120.0);
    let std_dev = 4.0;
    show(
        &mut window,
        vec![DrawCommand::Shadow {
            rect: rect(40.0, 40.0, 40.0, 40.0),
            radii: [0.0; 4],
            std_dev,
            color: color(0x000000),
        }],
    );
    let reach = (SHADOW_REACH * std_dev).ceil() as u32;
    assert_near(pixel(&window, 60, 60), 0x000000, 4, "the shadow's middle");
    // Just outside the rect, the blur still darkens ...
    assert!(distance(pixel(&window, 60, 82), WHITE) > 32, "the falloff");
    // ... and nothing is drawn beyond the reach.
    for (x, y) in [
        (60, 80 + reach),
        (60, 39 - reach),
        (80 + reach, 60),
        (39 - reach, 60),
    ] {
        assert_eq!(
            pixel(&window, x, y),
            WHITE,
            "beyond the reach at ({x}, {y})"
        );
    }
}

#[test]
fn a_window_beyond_the_frame_budget_is_drawn_at_a_reduced_scale() {
    // Wider than the renderers' longest side (16,384 px). The OS clamps
    // real windows to the screen, so this one is headless: the frame is
    // the same either way.
    let plan = FramePlan::new((17_000, 40), 1.0);
    assert_eq!(
        plan.size,
        Size {
            width: 17_000.0,
            height: 40.0
        }
    );
    assert!(
        plan.scale < 1.0 && plan.scale > 0.9,
        "reduced: {}",
        plan.scale
    );
    let mut window = VirtualWindow::open(VirtualWindowOptions {
        mode: WindowMode::Headless,
        ..options(17_000.0, 40.0)
    })
    .expect("a test window opens");
    assert_eq!(window.frame_plan(), plan);
    show(
        &mut window,
        vec![
            fill(0.0, 0.0, 17_000.0, 40.0, BLUE),
            fill(16_900.0, 0.0, 100.0, 40.0, RED),
        ],
    );
    // The reduced frame is enlarged to the whole window.
    let (width, height, _) = window.frame().expect("a frame");
    assert_eq!((width, height), (17_000, 40));
    assert_eq!(pixel(&window, 100, 20), BLUE);
    assert_eq!(pixel(&window, 16_999, 20), RED);
}

#[test]
fn an_invalid_scene_is_dropped_as_content_and_keeps_the_picture() {
    let mut window = VirtualWindow::open(VirtualWindowOptions {
        mode: WindowMode::Real,
        ..options(32., 32.)
    })
    .expect("real test window");
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, BLUE)]);
    let mut invalid = vec![
        vec![fill(f32::NAN, 0.0, 4.0, 4.0, RED)],
        vec![DrawCommand::PopClip],
        vec![DrawCommand::PushClip(rect(0.0, 0.0, 4.0, 4.0))],
        vec![DrawCommand::Image {
            image: ImageData {
                width: 2,
                height: 2,
                rgba: Arc::from(&[0u8; 4][..]),
            },
            rect: rect(0.0, 0.0, 4.0, 4.0),
            opacity: 1.0,
            color_matrix: None,
        }],
    ];
    let shaped = engine()
        .shape_glyphs("H", &TextStyle::default(), None)
        .expect("shaped");
    let run = Arc::clone(&shaped.runs[0]);
    invalid.push(vec![DrawCommand::Glyphs {
        run: Arc::clone(&run),
        origin: Point {
            x: f32::NAN,
            y: 0.0,
        },
        color: None,
    }]);
    invalid.push(vec![DrawCommand::Glyphs {
        run: Arc::clone(&run),
        origin: Point::default(),
        color: Some(kivact_types::Rgba {
            r: f32::NAN,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
    }]);
    let mut bad_id = (*run).clone();
    bad_id.glyphs[0].id = u32::MAX;
    invalid.push(vec![DrawCommand::Glyphs {
        run: Arc::new(bad_id),
        origin: Point::default(),
        color: None,
    }]);
    let mut bad_variation = (*run).clone();
    bad_variation.normalized_coords.push(f32::NAN);
    invalid.push(vec![DrawCommand::Glyphs {
        run: Arc::new(bad_variation),
        origin: Point::default(),
        color: None,
    }]);
    let mut overflowing = (*run).clone();
    overflowing.glyphs[0].x = f32::MAX;
    invalid.push(vec![DrawCommand::Glyphs {
        run: Arc::new(overflowing),
        origin: Point {
            x: f32::MAX,
            y: 0.0,
        },
        color: None,
    }]);
    for (case, commands) in invalid.into_iter().enumerate() {
        let what = format!("malformed scene case {case}");
        match present(&mut window, commands) {
            Presented::Dropped(error) => {
                assert_eq!(error.kind(), ErrorKind::Content, "{what}: {error}");
            }
            other => panic!("{what} is not dropped: {other:?}"),
        }
        assert_eq!(pixel(&window, 16, 16), BLUE, "the picture stays");
    }
    // The renderer is still usable.
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, RED)]);
    assert_eq!(pixel(&window, 16, 16), RED);
}

/// Text, a rounded rect, a shadow, a gradient, translucency, a clip and a
/// stretched image: the paint kinds of a typical frame.
fn representative() -> Vec<DrawCommand> {
    let style = TextStyle {
        font_size: 18.0,
        ..TextStyle::default()
    };
    let shaped = engine().shape("Arvane frames", &style, None).expect("text");
    let mut commands = vec![
        DrawCommand::Shadow {
            rect: rect(20.0, 20.0, 120.0, 60.0),
            radii: [8.0; 4],
            std_dev: 6.0,
            color: color_alpha(0x000000, 0.4),
        },
        DrawCommand::RoundedRect {
            rect: rect(20.0, 20.0, 120.0, 60.0),
            radius: 8.0,
            fill: Paint::LinearGradient {
                angle_deg: 0.0,
                stops: vec![(0.0, color(0x3366cc)), (1.0, color(0xcc6633))],
            },
        },
        DrawCommand::PushClip(rect(160.0, 20.0, 60.0, 60.0)),
        DrawCommand::Rect {
            rect: rect(150.0, 10.0, 90.0, 90.0),
            fill: Paint::Solid(color_alpha(RED, 0.5)),
        },
        DrawCommand::PopClip,
        DrawCommand::Image {
            image: ImageData {
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
    commands.extend(shaped.runs.iter().map(|run| DrawCommand::Glyphs {
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

#[test]
fn fixed_resolution_rasterizer_preserves_premultiplied_alpha_and_matches_window() {
    let command = DrawCommand::Rect {
        rect: rect(0.0, 0.0, 8.0, 8.0),
        fill: Paint::Solid(color_alpha(RED, 0.5)),
    };
    let mut window = open(16.0, 16.0);
    show(&mut window, vec![command.clone()]);
    let mut rasterizer = arvane::Rasterizer::new();
    let scene = kivact_types::Scene {
        commands: vec![command],
    };
    for (width, height) in [(0, 16), (16, 0), (16_385, 1), (4097, 4096)] {
        assert_eq!(
            arvane::Rasterizer::check_size(width, height)
                .unwrap_err()
                .kind(),
            ErrorKind::Content,
            "{width}x{height}"
        );
        assert_eq!(
            rasterizer.render(&scene, width, height).unwrap_err().kind(),
            ErrorKind::Content
        );
    }
    arvane::Rasterizer::check_size(16_384, 1).expect("the budget's own limit is drawn");
    let frame = rasterizer.render(&scene, 16, 16).unwrap();
    assert_eq!(
        (frame.width, frame.height, frame.rgba.len()),
        (16, 16, 16 * 16 * 4)
    );
    let inside = &frame.rgba[(4 * 16 + 4) * 4..][..4];
    assert!(inside[0].abs_diff(128) <= 1);
    assert_eq!(&inside[1..3], &[0, 0]);
    assert!(inside[3].abs_diff(128) <= 1);
    assert_eq!(&frame.rgba[(12 * 16 + 12) * 4..][..4], &[0, 0, 0, 0]);
    for (index, rgb) in frame.to_softbuffer().into_iter().enumerate() {
        assert_eq!(rgb, pixel(&window, index as u32 % 16, index as u32 / 16));
    }
    drop(rasterizer);
    assert_eq!(frame.rgba.len(), 16 * 16 * 4);
}

#[test]
fn shared_glyph_placements_and_color_overrides_preserve_the_payload_after_cache_release() {
    let shaped = engine()
        .shape_glyphs(
            "H",
            &TextStyle {
                color: color(BLUE),
                ..Default::default()
            },
            None,
        )
        .expect("shaped");
    let run = Arc::clone(&shaped.runs[0]);
    let original = run.glyphs.clone();
    let commands = vec![
        DrawCommand::Glyphs {
            run: Arc::clone(&run),
            origin: Point { x: 5., y: 3. },
            color: None,
        },
        DrawCommand::Glyphs {
            run: Arc::clone(&run),
            origin: Point { x: 45., y: 3. },
            color: Some(color(RED)),
        },
    ];
    let mut window = VirtualWindow::open(VirtualWindowOptions {
        mode: WindowMode::Real,
        ..options(90., 40.)
    })
    .expect("real test window");
    show(&mut window, commands.clone());
    let before = window.frame().expect("frame").2.to_vec();
    assert!(ink(&window).is_some());
    for y in 0..35 {
        for x in 0..30 {
            let blue = pixel(&window, x + 5, y);
            let red = pixel(&window, x + 45, y);
            assert_eq!(blue & 255, 255);
            assert_eq!((red >> 16) & 255, 255);
            assert_eq!(
                (blue >> 16) & 255,
                red & 255,
                "placement and color at {x},{y}"
            );
        }
    }
    window.release_caches();
    show(&mut window, commands.clone());
    assert_eq!(window.frame().expect("after release").2, &before);
    assert_eq!(
        run.glyphs, original,
        "placement never mutates the shared run"
    );
    if gpu_unavailable().is_none() {
        gpu_against_cpu((90., 40.), &[commands])[0]
            .assert_within_tolerance("shared glyph placements");
    }
}
