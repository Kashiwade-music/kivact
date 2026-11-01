//! Role: Layer rendering contract tests.
//!
//! Layers: group opacity, clips to any shape, blend modes, blur and drop
//! shadow filters, gradient masks, and how groups must nest.

use arvane::{ErrorKind, Presented};
use kivact_types::{
    BlendMode, DrawCommand, Extend, FillRule, Gradient, GradientShape, Layer, LayerFilter, Mask,
    Paint, RoundedRect, Shape, Transform,
};

use crate::support::{
    BLUE, RED, WHITE, assert_near, channels, color, color_alpha, fill, gray, open, pixel, present,
    pt, rect, show,
};

fn layer(layer: Layer, content: Vec<DrawCommand>) -> Vec<DrawCommand> {
    let mut commands = vec![DrawCommand::PushLayer(layer)];
    commands.extend(content);
    commands.push(DrawCommand::PopLayer);
    commands
}

#[test]
fn group_opacity_fades_the_group_not_each_command() {
    let mut window = open(100.0, 40.0);
    let mut commands = layer(
        Layer {
            opacity: 0.5,
            ..Layer::default()
        },
        vec![
            fill(0.0, 0.0, 30.0, 40.0, RED),
            fill(20.0, 0.0, 30.0, 40.0, RED),
        ],
    );
    // The same two rects, each half transparent.
    for x in [50.0, 70.0] {
        commands.push(DrawCommand::Rect {
            rect: rect(x, 0.0, 30.0, 40.0),
            fill: Paint::Solid(color_alpha(RED, 0.5)),
        });
    }
    show(&mut window, commands);
    assert_near(pixel(&window, 10, 20), 0xff8080, 2, "one rect in the group");
    assert_near(
        pixel(&window, 25, 20),
        0xff8080,
        2,
        "the overlap is not darker",
    );
    assert_near(pixel(&window, 60, 20), 0xff8080, 2, "one faded rect");
    assert_near(
        pixel(&window, 75, 20),
        0xff4040,
        2,
        "faded rects darken where they overlap",
    );
}

#[test]
fn a_layer_clips_to_a_polygon_and_to_an_elliptical_rounded_rect() {
    let mut window = open(240.0, 60.0);
    let triangle = Shape::Polygon {
        points: vec![pt(10.0, 50.0), pt(50.0, 10.0), pt(90.0, 50.0)],
        rule: FillRule::NonZero,
    };
    let ellipse = Shape::RoundedRect(RoundedRect::ellipse(rect(110.0, 10.0, 100.0, 40.0)));
    let mut commands = layer(
        Layer {
            clip: Some(triangle),
            ..Layer::default()
        },
        vec![fill(0.0, 0.0, 100.0, 60.0, RED)],
    );
    commands.extend(layer(
        Layer {
            clip: Some(ellipse),
            ..Layer::default()
        },
        vec![fill(100.0, 0.0, 140.0, 60.0, BLUE)],
    ));
    show(&mut window, commands);
    assert_eq!(pixel(&window, 50, 40), RED, "inside the triangle");
    assert_eq!(pixel(&window, 15, 15), WHITE, "beside its slope");
    assert_eq!(pixel(&window, 50, 55), WHITE, "below its base");
    assert_eq!(pixel(&window, 160, 30), BLUE, "the ellipse's middle");
    assert_eq!(pixel(&window, 112, 30), BLUE, "near its left end");
    assert_eq!(pixel(&window, 115, 13), WHITE, "beyond its rounded corner");
    assert_eq!(pixel(&window, 160, 8), WHITE, "above it");
}

/// sRGB components of 0xRRGGBB in `0..=1`.
fn unit(rgb: u32) -> [f32; 3] {
    channels(rgb).map(|c| f32::from(c) / 255.0)
}

fn lum(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

fn clip_color(c: [f32; 3]) -> [f32; 3] {
    let l = lum(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);
    c.map(|v| {
        let mut v = v;
        if n < 0.0 {
            v = l + (v - l) * l / (l - n);
        }
        if x > 1.0 {
            v = l + (v - l) * (1.0 - l) / (x - l);
        }
        v
    })
}

fn set_lum(c: [f32; 3], l: f32) -> [f32; 3] {
    let d = l - lum(c);
    clip_color(c.map(|v| v + d))
}

fn sat(c: [f32; 3]) -> f32 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}

fn set_sat(c: [f32; 3], s: f32) -> [f32; 3] {
    let (max, min) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
    if max <= min {
        return [0.0; 3];
    }
    c.map(|v| (v - min) * s / (max - min))
}

/// The W3C compositing formula of `mode` for an opaque source `s` over an
/// opaque backdrop `b`.
fn blend(mode: BlendMode, b: [f32; 3], s: [f32; 3]) -> [f32; 3] {
    let each = |f: fn(f32, f32) -> f32| [f(b[0], s[0]), f(b[1], s[1]), f(b[2], s[2])];
    let hard = |b: f32, s: f32| {
        if s <= 0.5 {
            b * 2.0 * s
        } else {
            let s = 2.0 * s - 1.0;
            b + s - b * s
        }
    };
    match mode {
        BlendMode::Normal => s,
        BlendMode::Multiply => each(|b, s| b * s),
        BlendMode::Screen => each(|b, s| b + s - b * s),
        BlendMode::Overlay => [hard(s[0], b[0]), hard(s[1], b[1]), hard(s[2], b[2])],
        BlendMode::Darken => each(f32::min),
        BlendMode::Lighten => each(f32::max),
        BlendMode::ColorDodge => each(|b, s| {
            if b == 0.0 {
                0.0
            } else if s >= 1.0 {
                1.0
            } else {
                (b / (1.0 - s)).min(1.0)
            }
        }),
        BlendMode::ColorBurn => each(|b, s| {
            if b >= 1.0 {
                1.0
            } else if s == 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - b) / s).min(1.0)
            }
        }),
        BlendMode::HardLight => [hard(b[0], s[0]), hard(b[1], s[1]), hard(b[2], s[2])],
        BlendMode::SoftLight => each(|b, s| {
            if s <= 0.5 {
                b - (1.0 - 2.0 * s) * b * (1.0 - b)
            } else {
                let d = if b <= 0.25 {
                    ((16.0 * b - 12.0) * b + 4.0) * b
                } else {
                    b.sqrt()
                };
                b + (2.0 * s - 1.0) * (d - b)
            }
        }),
        BlendMode::Difference => each(|b, s| (b - s).abs()),
        BlendMode::Exclusion => each(|b, s| b + s - 2.0 * b * s),
        BlendMode::Hue => set_lum(set_sat(s, sat(b)), lum(b)),
        BlendMode::Saturation => set_lum(set_sat(b, sat(s)), lum(b)),
        BlendMode::Color => set_lum(s, lum(b)),
        BlendMode::Luminosity => set_lum(b, lum(s)),
        BlendMode::PlusLighter => each(|b, s| (b + s).min(1.0)),
    }
}

#[test]
fn every_blend_mode_mixes_a_layer_with_what_is_beneath_it() {
    const MODES: [BlendMode; 17] = [
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::Darken,
        BlendMode::Lighten,
        BlendMode::ColorDodge,
        BlendMode::ColorBurn,
        BlendMode::HardLight,
        BlendMode::SoftLight,
        BlendMode::Difference,
        BlendMode::Exclusion,
        BlendMode::Hue,
        BlendMode::Saturation,
        BlendMode::Color,
        BlendMode::Luminosity,
        BlendMode::PlusLighter,
    ];
    let (backdrop, source) = (0x4080c0, 0xcc4d99);
    let mut window = open(10.0 * MODES.len() as f32, 10.0);
    let mut commands = Vec::new();
    for (i, mode) in MODES.iter().enumerate() {
        let x = 10.0 * i as f32;
        commands.push(fill(x, 0.0, 10.0, 10.0, backdrop));
        commands.extend(layer(
            Layer {
                blend: *mode,
                ..Layer::default()
            },
            vec![fill(x, 0.0, 10.0, 10.0, source)],
        ));
    }
    show(&mut window, commands);
    for (i, mode) in MODES.iter().enumerate() {
        let expected = blend(*mode, unit(backdrop), unit(source))
            .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u32);
        let expected = (expected[0] << 16) | (expected[1] << 8) | expected[2];
        assert_near(
            pixel(&window, 10 * i as u32 + 5, 5),
            expected,
            3,
            &format!("{mode:?}"),
        );
    }
}

#[test]
fn blur_and_drop_shadow_layers_filter_their_content() {
    let mut window = open(200.0, 80.0);
    let mut commands = layer(
        Layer {
            filter: Some(LayerFilter::Blur { std_dev: 3.0 }),
            ..Layer::default()
        },
        vec![fill(20.0, 20.0, 40.0, 40.0, 0x000000)],
    );
    commands.extend(layer(
        Layer {
            filter: Some(LayerFilter::DropShadow {
                offset: pt(10.0, 10.0),
                std_dev: 2.0,
                color: color(0x000000),
            }),
            ..Layer::default()
        },
        vec![fill(110.0, 10.0, 40.0, 40.0, RED)],
    ));
    show(&mut window, commands);
    assert!(
        gray(pixel(&window, 40, 40)) < 8,
        "the blurred square's middle"
    );
    let edge = gray(pixel(&window, 20, 40));
    assert!((96..=160).contains(&edge), "half way at its edge: {edge}");
    let near = gray(pixel(&window, 16, 40));
    assert!(near > edge && near < 250, "the blur reaches beyond: {near}");
    assert_eq!(pixel(&window, 6, 40), WHITE, "and fades within its reach");
    assert_eq!(pixel(&window, 130, 30), RED, "the shadowed square");
    assert!(gray(pixel(&window, 155, 45)) < 40, "its shadow, offset");
    assert_eq!(pixel(&window, 105, 5), WHITE, "no shadow up and left");
}

#[test]
fn a_mask_keeps_the_alpha_of_its_paint_and_hides_the_rest() {
    let mut window = open(140.0, 40.0);
    let black = color(0x000000);
    let gradient = Paint::Gradient(Gradient {
        shape: GradientShape::Linear {
            start: pt(0.0, 0.0),
            end: pt(100.0, 0.0),
        },
        stops: vec![(0.0, black), (1.0, color_alpha(0x000000, 0.0))],
        extend: Extend::Pad,
    });
    show(
        &mut window,
        layer(
            Layer {
                mask: Some(Mask {
                    rect: rect(0.0, 0.0, 100.0, 40.0),
                    paint: gradient,
                }),
                ..Layer::default()
            },
            vec![fill(0.0, 0.0, 140.0, 40.0, BLUE)],
        ),
    );
    assert_near(pixel(&window, 0, 20), BLUE, 4, "opaque at the start");
    assert_near(pixel(&window, 50, 20), 0x8080ff, 6, "half at the middle");
    assert_near(pixel(&window, 99, 20), WHITE, 6, "clear at the end");
    assert_eq!(pixel(&window, 120, 20), WHITE, "hidden beyond the mask");
    // With a filter and a clip: the mask applies after the blur.
    show(
        &mut window,
        layer(
            Layer {
                clip: Some(Shape::Rect(rect(0.0, 0.0, 70.0, 40.0))),
                filter: Some(LayerFilter::Blur { std_dev: 1.0 }),
                mask: Some(Mask {
                    rect: rect(0.0, 0.0, 50.0, 40.0),
                    paint: Paint::Solid(black),
                }),
                ..Layer::default()
            },
            vec![fill(0.0, 0.0, 140.0, 40.0, BLUE)],
        ),
    );
    assert_near(pixel(&window, 25, 20), BLUE, 2, "inside the mask");
    assert_eq!(
        pixel(&window, 60, 20),
        WHITE,
        "masked out, though clipped in"
    );
}

#[test]
fn groups_nest_and_every_pop_closes_its_own_kind() {
    let mut window = open(32.0, 32.0);
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, BLUE)]);
    let invalid = [
        vec![
            DrawCommand::PushLayer(Layer::default()),
            DrawCommand::PushClip(rect(0.0, 0.0, 4.0, 4.0)),
            DrawCommand::PopLayer,
            DrawCommand::PopClip,
        ],
        vec![
            DrawCommand::PushTransform(Transform::IDENTITY),
            DrawCommand::PopLayer,
        ],
        vec![DrawCommand::PushLayer(Layer::default())],
        vec![
            DrawCommand::PushLayer(Layer {
                opacity: 1.5,
                ..Layer::default()
            }),
            DrawCommand::PopLayer,
        ],
        vec![
            DrawCommand::PushLayer(Layer {
                filter: Some(LayerFilter::Blur { std_dev: f32::NAN }),
                ..Layer::default()
            }),
            DrawCommand::PopLayer,
        ],
    ];
    for commands in invalid {
        let what = format!("{commands:?}");
        match present(&mut window, commands) {
            Presented::Dropped(error) => {
                assert_eq!(error.kind(), ErrorKind::Content, "{what}: {error}");
            }
            other => panic!("{what} is not dropped: {other:?}"),
        }
        assert_eq!(pixel(&window, 16, 16), BLUE, "the picture stays");
    }
    // A transparent layer draws nothing, nested groups included.
    let mut commands = layer(
        Layer {
            opacity: 0.0,
            ..Layer::default()
        },
        vec![
            DrawCommand::PushClip(rect(0.0, 0.0, 32.0, 32.0)),
            fill(0.0, 0.0, 32.0, 32.0, RED),
            DrawCommand::PopClip,
        ],
    );
    commands.push(fill(0.0, 0.0, 8.0, 8.0, RED));
    show(&mut window, commands);
    assert_eq!(pixel(&window, 16, 16), WHITE);
    assert_eq!(pixel(&window, 4, 4), RED, "drawing resumes after the pop");
}
