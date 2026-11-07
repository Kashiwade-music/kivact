//! Role: Image rendering contract tests.
//!
//! The GPU's frames of the whole drawing vocabulary against the CPU's:
//! commands, layers with their effects (clips, blends, masks, filters, a
//! backdrop), images through a color matrix (a layer filter), strokes,
//! shadows and patterns.

use std::sync::Arc;

use kivact_types::AssetBytes;
use kivact_types::paint::{MaskComposite, MaskMode};
use kivact_types::render::{
    BoxShadow, CompositeLayer, Composition, Gradient, GradientShape, LayerEffects, LayerFilter,
    LayerItem, Paint, Pattern, PatternSource, RenderBackdrop, RenderCmd, RenderMask, RenderScene,
};
use kivact_types::scene::{PathCmd, Radius, RoundedRect, Shape};
use kivact_types::{
    BlendMode, ColorMatrix, Extend, FillRule, LineCap, Pixels, Rect, Size, Stroke, Transform,
};

use crate::support::{
    BLUE, RED, WHITE, color, color_alpha, fill, gpu_against_cpu, layer, node_key, pt, raster, rect,
    root_layer, solid, text_runs,
};

/// The size of the vocabulary's window (logical px).
const SIZE: Size = Size {
    width: 240.0,
    height: 200.0,
};

/// An image of `rect` with no color change.
fn image(rect: Rect, image: Pixels) -> RenderCmd {
    RenderCmd::Image {
        image,
        rect,
        opacity: 1.0,
    }
}

/// A checkerboard of 2 × 2 pixels.
fn checker() -> Pixels {
    Pixels {
        width: 2,
        height: 2,
        rgba: AssetBytes::from_shared(Arc::from(
            &[
                255u8, 0, 0, 255, 0, 0, 255, 255, 0, 0, 255, 255, 255, 0, 0, 255,
            ][..],
        )),
    }
}

/// A frame of every new kind of drawing: the root layer draws the commands
/// and the layers in its order; each layer is a group with its effects.
fn vocabulary() -> Composition {
    let checker = checker();
    let mut layers = vec![vocabulary_root(&checker)];
    layers.push(masked_group());
    layers.push(shadow_group());
    layers.push(blur_group());
    layers.push(backdrop_group());
    layers.push(color_matrix_group(checker));
    Composition {
        layers,
        exports: Vec::new(),
    }
}

/// The root layer: the stripes, the scenes of strokes, shadows and tiles, and layers 1 to 5 in that order.
fn vocabulary_root(checker: &Pixels) -> CompositeLayer {
    let mut stripes: Vec<RenderCmd> = (0..60)
        .map(|i| fill(4.0 * i as f64, 0.0, 2.0, 200.0, 0x203040))
        .collect();
    stripes.insert(0, fill(0.0, 0.0, 240.0, 200.0, 0xf0e0d0));
    stripes.extend(transformed_scene());
    root_layer(
        SIZE,
        vec![
            raster(stripes),
            LayerItem::Layer {
                index: 1,
                clips: Vec::new(),
            },
            LayerItem::Layer {
                index: 2,
                clips: Vec::new(),
            },
            LayerItem::Layer {
                index: 3,
                clips: Vec::new(),
            },
            raster(stroke_scene(checker.clone())),
            raster(shadow_scene(checker.clone())),
            LayerItem::Layer {
                index: 4,
                clips: Vec::new(),
            },
            raster(vec![RenderCmd::FillShape {
                shape: Shape::RoundedRect(RoundedRect::new(rect(10.0, 150.0, 70.0, 40.0), 12.0)),
                paint: Paint::Solid(color_alpha(WHITE, 0.3)),
            }]),
            LayerItem::Layer {
                index: 5,
                clips: Vec::new(),
            },
            raster(tiled_scene(checker.clone())),
        ],
    )
}

/// Layer 1: a group clipped, faded and blended, and masked by a gradient.
fn masked_group() -> CompositeLayer {
    let black = color(0x000000);
    let whole = rect(0.0, 0.0, SIZE.width, SIZE.height);
    layer(
        node_key(1),
        whole,
        LayerEffects {
            opacity: 0.8,
            blend: BlendMode::Multiply,
            clip: Some(Shape::Polygon {
                points: vec![
                    pt(160.0, 10.0),
                    pt(230.0, 10.0),
                    pt(230.0, 50.0),
                    pt(170.0, 60.0),
                ],
                rule: FillRule::NonZero,
            }),
            masks: vec![RenderMask {
                scene: RenderScene {
                    commands: vec![RenderCmd::FillShape {
                        shape: Shape::Rect(rect(160.0, 10.0, 70.0, 50.0)),
                        paint: Paint::Gradient(Gradient {
                            shape: GradientShape::Linear {
                                start: pt(160.0, 35.0),
                                end: pt(230.0, 35.0),
                            },
                            stops: vec![(0.0, black), (1.0, color_alpha(0x000000, 0.0))],
                            extend: Extend::Pad,
                        }),
                    }],
                },
                mode: MaskMode::Alpha,
                composite: MaskComposite::Intersect,
            }],
            ..LayerEffects::default()
        },
        vec![raster(vec![fill(150.0, 0.0, 90.0, 70.0, 0x40c080)])],
    )
}

/// Layer 2: a radial gradient under a drop shadow.
fn shadow_group() -> CompositeLayer {
    let whole = rect(0.0, 0.0, SIZE.width, SIZE.height);
    layer(
        node_key(2),
        whole,
        LayerEffects {
            filters: vec![LayerFilter::DropShadow {
                offset: pt(4.0, 4.0),
                std_dev: 3.0,
                color: color_alpha(0x000000, 0.6),
            }],
            ..LayerEffects::default()
        },
        vec![raster(vec![RenderCmd::FillShape {
            shape: Shape::Rect(rect(10.0, 80.0, 50.0, 30.0)),
            paint: Paint::Gradient(Gradient {
                shape: GradientShape::Radial {
                    center: pt(35.0, 95.0),
                    radius_x: 25.0,
                    radius_y: 15.0,
                },
                stops: vec![(0.0, color(RED)), (0.5, color(0x00ff00)), (1.0, color(RED))],
                extend: Extend::Pad,
            }),
        }])],
    )
}

/// Layer 3: a square blurred.
fn blur_group() -> CompositeLayer {
    let whole = rect(0.0, 0.0, SIZE.width, SIZE.height);
    layer(
        node_key(3),
        whole,
        LayerEffects {
            filters: vec![LayerFilter::Blur { std_dev: 2.0 }],
            ..LayerEffects::default()
        },
        vec![raster(vec![fill(80.0, 80.0, 30.0, 30.0, 0x8020a0)])],
    )
}

/// Layer 4: the backdrop: what lies behind the rounded rect, blurred and saturated.
fn backdrop_group() -> CompositeLayer {
    layer(
        node_key(4),
        rect(10.0, 150.0, 70.0, 40.0),
        LayerEffects {
            backdrop: Some(RenderBackdrop {
                shape: Shape::RoundedRect(RoundedRect::new(rect(10.0, 150.0, 70.0, 40.0), 12.0)),
                filters: vec![
                    LayerFilter::Blur { std_dev: 4.0 },
                    LayerFilter::ColorMatrix(ColorMatrix::saturate(1.6)),
                ],
            }),
            ..LayerEffects::default()
        },
        Vec::new(),
    )
}

/// Layer 5: an image through a color matrix: the matrix is the layer's filter.
fn color_matrix_group(checker: Pixels) -> CompositeLayer {
    layer(
        node_key(5),
        rect(200.0, 150.0, 30.0, 30.0),
        LayerEffects {
            filters: vec![LayerFilter::ColorMatrix(ColorMatrix::sepia(1.0))],
            ..LayerEffects::default()
        },
        vec![raster(vec![image(rect(200.0, 150.0, 30.0, 30.0), checker)])],
    )
}

/// [`vocabulary`] scaled down about the middle: tiles, glyphs and blurs at
/// other device sizes. The root keeps the identity transform (design §8.3),
/// so the vocabulary is a child layer of the root that is scaled instead.
fn scaled_vocabulary() -> Composition {
    let mut layers = vocabulary().layers;
    let mut vocab = layers.remove(0);
    // Layers 1 to 5 move to 2 to 6, and the scaled child takes index 1.
    for item in &mut vocab.items {
        if let LayerItem::Layer { index, .. } = item {
            *index += 1;
        }
    }
    vocab.key = node_key(6);
    vocab.transform = kivact_types::geometry3d::Matrix4::from_affine(
        Transform::scale(0.75).about(pt(120.0, 100.0)),
    );
    let root = root_layer(
        SIZE,
        vec![LayerItem::Layer {
            index: 1,
            clips: Vec::new(),
        }],
    );
    let mut all = vec![root, vocab];
    all.extend(layers);
    Composition {
        layers: all,
        exports: Vec::new(),
    }
}

#[test]
fn the_gpu_draws_the_whole_vocabulary_as_the_cpu_does() {
    if let Some(reason) = arvane::gpu_unavailable() {
        eprintln!("SKIPPED the_gpu_draws_the_whole_vocabulary_as_the_cpu_does: {reason}");
        return;
    }
    let frames = [vocabulary(), vocabulary(), scaled_vocabulary()];
    for (frame, diff) in gpu_against_cpu((240.0, 200.0), &frames).iter().enumerate() {
        diff.assert_within_tolerance(&format!("frame {frame}"));
    }
}

fn transformed_scene() -> Vec<RenderCmd> {
    let mut commands = Vec::new();
    commands.extend([
        RenderCmd::PushTransform(Transform::rotate_deg(20.0).about(pt(40.0, 30.0))),
        RenderCmd::BoxShadow(BoxShadow {
            shape: RoundedRect::new(rect(10.0, 10.0, 60.0, 40.0), 6.0),
            offset: pt(0.0, 4.0),
            spread: 2.0,
            std_dev: 3.0,
            color: color_alpha(0x000000, 0.5),
            inset: false,
            knockout: false,
        }),
        RenderCmd::FillShape {
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
    commands.push(RenderCmd::PopTransform);
    commands.push(RenderCmd::PushTransform(
        Transform::skew_deg(-15.0, 0.0).about(pt(120.0, 30.0)),
    ));
    commands.extend(text_runs("Skew", 14.0, pt(90.0, 20.0)));
    commands.push(RenderCmd::PopTransform);
    commands
}

/// Patterns: an image tile, and a tile larger than the atlas takes.
fn tiled_scene(checker: Pixels) -> Vec<RenderCmd> {
    vec![
        // A tile larger than the atlas takes: drawn copy by copy.
        RenderCmd::FillShape {
            shape: Shape::Rect(rect(90.0, 180.0, 150.0, 20.0)),
            paint: Paint::Pattern(Pattern {
                source: PatternSource::Image { image: checker },
                tile: rect(0.0, 0.0, 300.0, 300.0),
                repeat_x: true,
                repeat_y: true,
            }),
        },
    ]
}

fn stroke_scene(checker: Pixels) -> Vec<RenderCmd> {
    vec![
        RenderCmd::FillShape {
            shape: Shape::Rect(rect(120.0, 80.0, 50.0, 30.0)),
            paint: Paint::Pattern(Pattern {
                source: PatternSource::Image {
                    image: checker.clone(),
                },
                tile: rect(120.0, 80.0, 6.0, 6.0),
                repeat_x: true,
                repeat_y: true,
            }),
        },
        RenderCmd::StrokeShape {
            shape: Shape::RoundedRect(RoundedRect::new(rect(180.0, 80.0, 50.0, 30.0), 8.0)),
            stroke: Stroke {
                cap: LineCap::Round,
                dash: vec![6.0, 4.0],
                ..Stroke::new(3.0)
            },
            paint: solid(0x102030),
        },
        RenderCmd::StrokeShape {
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
    ]
}

fn shadow_scene(checker: Pixels) -> Vec<RenderCmd> {
    vec![
        RenderCmd::BoxShadow(BoxShadow {
            shape: RoundedRect::new(rect(90.0, 130.0, 50.0, 40.0), 10.0),
            offset: pt(2.0, 2.0),
            spread: 1.0,
            std_dev: 3.0,
            color: color_alpha(0x000000, 0.7),
            inset: true,
            knockout: false,
        }),
        RenderCmd::BoxShadow(BoxShadow {
            shape: RoundedRect::ellipse(rect(85.0, 45.0, 50.0, 28.0)),
            offset: pt(-2.0, 2.0),
            spread: 0.0,
            std_dev: 3.0,
            color: color_alpha(0x000040, 0.8),
            inset: true,
            knockout: false,
        }),
        // Strokes painted with a gradient and with a pattern.
        RenderCmd::StrokeShape {
            shape: Shape::Path {
                cmds: vec![
                    PathCmd::MoveTo(pt(150.0, 66.0)),
                    PathCmd::LineTo(pt(230.0, 66.0)),
                ],
                rule: FillRule::NonZero,
            },
            stroke: Stroke::new(5.0),
            paint: Paint::Gradient(Gradient {
                shape: GradientShape::Linear {
                    start: pt(150.0, 66.0),
                    end: pt(230.0, 66.0),
                },
                stops: vec![(0.0, color(RED)), (1.0, color(BLUE))],
                extend: Extend::Pad,
            }),
        },
        RenderCmd::StrokeShape {
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
                source: PatternSource::Image { image: checker },
                tile: rect(0.0, 0.0, 4.0, 4.0),
                repeat_x: true,
                repeat_y: true,
            }),
        },
        RenderCmd::BoxShadow(BoxShadow {
            shape: RoundedRect::ellipse(rect(150.0, 130.0, 40.0, 30.0)),
            offset: pt(0.0, 3.0),
            spread: 0.0,
            std_dev: 4.0,
            color: color_alpha(0x000000, 0.5),
            inset: false,
            knockout: true,
        }),
    ]
}
