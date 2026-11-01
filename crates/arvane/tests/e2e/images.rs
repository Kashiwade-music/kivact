//! Role: Image rendering contract tests.
//!
//! Images through a color matrix, and raw pixels a host changes every
//! frame (on the CPU and the GPU); and the GPU's frames of the whole
//! drawing vocabulary against the CPU's.

use std::sync::Arc;

use kivact_types::{
    Backdrop, BlendMode, BoxShadow, ColorMatrix, DrawCommand, Extend, FillRule, Gradient,
    GradientShape, ImageData, Layer, LayerFilter, LineCap, Mask, Paint, PathCmd, Pattern,
    PatternSource, Radius, RoundedRect, Shape, Stroke, Transform,
};

use crate::support::{
    BLUE, RED, WHITE, color, color_alpha, fill, gpu_against_cpu, pt, rect, solid, text_runs,
};

fn image(
    rect: kivact_types::Rect,
    image: ImageData,
    color_matrix: Option<ColorMatrix>,
) -> DrawCommand {
    DrawCommand::Image {
        image,
        rect,
        opacity: 1.0,
        color_matrix,
    }
}

/// A scene of every new command: transforms with text, layers with clips,
/// blends, filters and masks, gradients, patterns, strokes, shadows, a
/// backdrop and a filtered image.
fn vocabulary() -> Vec<DrawCommand> {
    let black = color(0x000000);
    let mut stripes: Vec<DrawCommand> = (0..60)
        .map(|i| fill(4.0 * i as f32, 0.0, 2.0, 200.0, 0x203040))
        .collect();
    stripes.insert(0, fill(0.0, 0.0, 240.0, 200.0, 0xf0e0d0));
    let mut commands = stripes;
    let checker = ImageData {
        width: 2,
        height: 2,
        rgba: Arc::from(
            &[
                255u8, 0, 0, 255, 0, 0, 255, 255, 0, 0, 255, 255, 255, 0, 0, 255,
            ][..],
        ),
    };
    commands.extend([
        DrawCommand::PushTransform(Transform::rotate_deg(20.0).about(pt(40.0, 30.0))),
        DrawCommand::BoxShadow(BoxShadow {
            shape: RoundedRect::new(rect(10.0, 10.0, 60.0, 40.0), 6.0),
            offset: pt(0.0, 4.0),
            spread: 2.0,
            std_dev: 3.0,
            color: color_alpha(0x000000, 0.5),
            inset: false,
            knockout: false,
        }),
        DrawCommand::FillShape {
            shape: Shape::RoundedRect(RoundedRect {
                rect: rect(10.0, 10.0, 60.0, 40.0),
                radii: [Radius { x: 20.0, y: 10.0 }; 4],
            }),
            paint: Paint::Gradient(Gradient {
                shape: GradientShape::Radial {
                    center: pt(40.0, 30.0),
                    radius_x: 30.0,
                    radius_y: 20.0,
                },
                stops: vec![
                    (0.0, color(RED)),
                    (0.5, color(RED)),
                    (0.5, color(BLUE)),
                    (1.0, color(WHITE)),
                ],
                extend: Extend::Repeat,
            }),
        },
    ]);
    commands.extend(text_runs("Turn", 14.0, pt(20.0, 50.0)));
    commands.push(DrawCommand::PopTransform);
    commands.push(DrawCommand::PushTransform(
        Transform::skew_deg(-15.0, 0.0).about(pt(120.0, 30.0)),
    ));
    commands.extend(text_runs("Skew", 14.0, pt(90.0, 20.0)));
    commands.push(DrawCommand::PopTransform);
    commands.extend([
        DrawCommand::PushLayer(Layer {
            clip: Some(Shape::Polygon {
                points: vec![
                    pt(160.0, 10.0),
                    pt(230.0, 10.0),
                    pt(230.0, 50.0),
                    pt(170.0, 60.0),
                ],
                rule: FillRule::NonZero,
            }),
            opacity: 0.8,
            blend: BlendMode::Multiply,
            filter: None,
            mask: Some(Mask {
                rect: rect(160.0, 10.0, 70.0, 50.0),
                paint: Paint::LinearGradient {
                    angle_deg: 90.0,
                    stops: vec![(0.0, black), (1.0, color_alpha(0x000000, 0.0))],
                },
            }),
        }),
        fill(150.0, 0.0, 90.0, 70.0, 0x40c080),
        DrawCommand::PopLayer,
        DrawCommand::PushLayer(Layer {
            filter: Some(LayerFilter::DropShadow {
                offset: pt(4.0, 4.0),
                std_dev: 3.0,
                color: color_alpha(0x000000, 0.6),
            }),
            ..Layer::default()
        }),
        DrawCommand::FillShape {
            shape: Shape::Rect(rect(10.0, 80.0, 50.0, 30.0)),
            paint: Paint::Gradient(Gradient {
                shape: GradientShape::Conic {
                    center: pt(35.0, 95.0),
                    start_deg: -90.0,
                    end_deg: 270.0,
                },
                stops: vec![(0.0, color(RED)), (0.5, color(0x00ff00)), (1.0, color(RED))],
                extend: Extend::Pad,
            }),
        },
        DrawCommand::PopLayer,
        DrawCommand::PushLayer(Layer {
            filter: Some(LayerFilter::Blur { std_dev: 2.0 }),
            ..Layer::default()
        }),
        fill(80.0, 80.0, 30.0, 30.0, 0x8020a0),
        DrawCommand::PopLayer,
        DrawCommand::FillShape {
            shape: Shape::Rect(rect(120.0, 80.0, 50.0, 30.0)),
            paint: Paint::Pattern(Pattern {
                source: PatternSource::Image {
                    image: checker.clone(),
                    color_matrix: None,
                },
                tile: rect(120.0, 80.0, 6.0, 6.0),
                repeat_x: true,
                repeat_y: true,
            }),
        },
        DrawCommand::StrokeShape {
            shape: Shape::RoundedRect(RoundedRect::new(rect(180.0, 80.0, 50.0, 30.0), 8.0)),
            stroke: Stroke {
                cap: LineCap::Round,
                dash: vec![6.0, 4.0],
                ..Stroke::new(3.0)
            },
            paint: solid(0x102030),
        },
        DrawCommand::StrokeShape {
            shape: Shape::Path {
                cmds: vec![
                    PathCmd::MoveTo(pt(10.0, 130.0)),
                    PathCmd::ArcTo {
                        rx: 20.0,
                        ry: 10.0,
                        x_rotation_deg: 0.0,
                        large_arc: false,
                        sweep: true,
                        to: pt(60.0, 130.0),
                    },
                    PathCmd::MoveTo(pt(70.0, 130.0)),
                    PathCmd::LineTo(pt(70.01, 130.0)),
                ],
                rule: FillRule::NonZero,
            },
            stroke: Stroke {
                cap: LineCap::Round,
                ..Stroke::new(4.0)
            },
            paint: solid(0x000000),
        },
        DrawCommand::BoxShadow(BoxShadow {
            shape: RoundedRect::new(rect(90.0, 130.0, 50.0, 40.0), 10.0),
            offset: pt(2.0, 2.0),
            spread: 1.0,
            std_dev: 3.0,
            color: color_alpha(0x000000, 0.7),
            inset: true,
            knockout: false,
        }),
        DrawCommand::BoxShadow(BoxShadow {
            shape: RoundedRect::ellipse(rect(85.0, 45.0, 50.0, 28.0)),
            offset: pt(-2.0, 2.0),
            spread: 0.0,
            std_dev: 3.0,
            color: color_alpha(0x000040, 0.8),
            inset: true,
            knockout: false,
        }),
        // Strokes painted with a gradient and with a pattern.
        DrawCommand::StrokeShape {
            shape: Shape::Path {
                cmds: vec![
                    PathCmd::MoveTo(pt(150.0, 66.0)),
                    PathCmd::LineTo(pt(230.0, 66.0)),
                ],
                rule: FillRule::NonZero,
            },
            stroke: Stroke::new(5.0),
            paint: Paint::LinearGradient {
                angle_deg: 0.0,
                stops: vec![(0.0, color(RED)), (1.0, color(BLUE))],
            },
        },
        DrawCommand::StrokeShape {
            shape: Shape::Path {
                cmds: vec![
                    PathCmd::MoveTo(pt(150.0, 74.0)),
                    PathCmd::LineTo(pt(230.0, 74.0)),
                ],
                rule: FillRule::NonZero,
            },
            stroke: Stroke {
                cap: LineCap::Square,
                ..Stroke::new(5.0)
            },
            paint: Paint::Pattern(Pattern {
                source: PatternSource::Image {
                    image: checker.clone(),
                    color_matrix: None,
                },
                tile: rect(0.0, 0.0, 4.0, 4.0),
                repeat_x: true,
                repeat_y: true,
            }),
        },
        DrawCommand::BoxShadow(BoxShadow {
            shape: RoundedRect::ellipse(rect(150.0, 130.0, 40.0, 30.0)),
            offset: pt(0.0, 3.0),
            spread: 0.0,
            std_dev: 4.0,
            color: color_alpha(0x000000, 0.5),
            inset: false,
            knockout: true,
        }),
        DrawCommand::Backdrop(Backdrop {
            shape: Shape::RoundedRect(RoundedRect::new(rect(10.0, 150.0, 70.0, 40.0), 12.0)),
            std_dev: 4.0,
            color_matrix: Some(ColorMatrix::saturate(1.6)),
        }),
        DrawCommand::FillShape {
            shape: Shape::RoundedRect(RoundedRect::new(rect(10.0, 150.0, 70.0, 40.0), 12.0)),
            paint: Paint::Solid(color_alpha(WHITE, 0.3)),
        },
        image(
            rect(200.0, 150.0, 30.0, 30.0),
            checker.clone(),
            Some(ColorMatrix::sepia(1.0)),
        ),
        // A tile larger than the atlas takes: drawn copy by copy.
        DrawCommand::FillShape {
            shape: Shape::Rect(rect(90.0, 180.0, 150.0, 20.0)),
            paint: Paint::Pattern(Pattern {
                source: PatternSource::Image {
                    image: checker,
                    color_matrix: None,
                },
                tile: rect(0.0, 0.0, 300.0, 300.0),
                repeat_x: true,
                repeat_y: true,
            }),
        },
    ]);
    commands
}

/// [`vocabulary`] scaled down about the middle: tiles, glyphs and blurs at
/// other device sizes.
fn scaled_vocabulary() -> Vec<DrawCommand> {
    let mut commands = vec![DrawCommand::PushTransform(
        Transform::scale(0.75).about(pt(120.0, 100.0)),
    )];
    commands.extend(vocabulary());
    commands.push(DrawCommand::PopTransform);
    commands
}

#[test]
fn the_gpu_draws_the_whole_vocabulary_as_the_cpu_does() {
    if let Some(reason) = arvane::test_display::gpu_unavailable() {
        eprintln!("SKIPPED the_gpu_draws_the_whole_vocabulary_as_the_cpu_does: {reason}");
        return;
    }
    let frames = [vocabulary(), vocabulary(), scaled_vocabulary()];
    for (frame, diff) in gpu_against_cpu((240.0, 200.0), &frames).iter().enumerate() {
        diff.assert_within_tolerance(&format!("frame {frame}"));
    }
}
