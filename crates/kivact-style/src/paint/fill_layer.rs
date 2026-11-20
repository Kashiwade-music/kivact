//! Role: Resolve paints, gradients and shader paints with the theme into wire fills.

use super::canvas_gradient::{CanvasGradient, CanvasShape};
use super::gradient::{Gradient, GradientKind, RadialExtent, RadialSize};
use super::layer::{Paint, PaintImage, PaintSize, ShaderPaint};
use crate::resolve::Resolve;
use crate::shader::data::pack;
use crate::theme::value::Theme;
use kivact_types as p;
use kivact_types::Length;
use kivact_types::{
    FillImage, FillLayer, FillSize, GradientKind as WireKind, GradientStop, ShaderSpec,
};

/// A paint as the engine fills an area with it, its lengths resolved in `cx`.
pub(crate) fn fill_layer(l: &Paint, cx: &mut Resolve<'_>) -> FillLayer {
    let theme = cx.theme;
    let image = match &l.image {
        PaintImage::Color(c) => FillImage::Solid(c.resolve(theme)),
        PaintImage::Gradient(g) => FillImage::Gradient(gradient(g, cx)),
        PaintImage::Canvas(g) => FillImage::Coordinate(canvas_gradient(g, theme)),
        PaintImage::Image(a) => FillImage::Image(a.clone()),
        PaintImage::Shader(s) => FillImage::Shader(std::sync::Arc::new(shader_spec(s, cx))),
    };
    let size = match l.size {
        PaintSize::Auto => FillSize::Auto,
        PaintSize::Cover => FillSize::Cover,
        PaintSize::Contain => FillSize::Contain,
        PaintSize::Explicit(w, h) => FillSize::Explicit(explicit(w, cx), explicit(h, cx)),
    };
    FillLayer {
        image,
        size,
        position: point(l.position, cx),
        repeat_x: l.repeat.0,
        repeat_y: l.repeat.1,
        clip: l.clip,
        origin: l.origin,
        blend: l.blend,
    }
}

/// One side of an explicit size in its paint form; `auto` and the content
/// sizes (and an invalid sum, once resolved) are `None`.
fn explicit(l: Length, cx: &mut Resolve<'_>) -> Option<p::Len> {
    let l = cx.len(l);
    // `auto` and the content sizes are the unset size here, not a misplaced length.
    (l != Length::Auto && !l.is_content()).then(|| l.to_len(&mut 0))
}

/// A point in its paint form, each coordinate resolved in `cx`.
fn point((x, y): (Length, Length), cx: &mut Resolve<'_>) -> [p::Len; 2] {
    [cx.paint_len(x), cx.paint_len(y)]
}

/// A gradient by coordinates for the engine.
fn canvas_gradient(g: &CanvasGradient, theme: &Theme) -> p::CoordinateGradient {
    p::CoordinateGradient {
        shape: match g.shape {
            CanvasShape::Linear { from, to } => p::CoordinateShape::Linear {
                start: from,
                end: to,
            },
            CanvasShape::Radial { c0, r0, c1, r1 } => p::CoordinateShape::RadialTwoCircle {
                start_center: c0,
                start_radius: r0,
                end_center: c1,
                end_radius: r1,
            },
            CanvasShape::Conic { start_deg, center } => {
                p::CoordinateShape::Conic { center, start_deg }
            }
        },
        stops: g
            .stops
            .iter()
            .map(|(o, c)| (*o, c.resolve_space(theme)))
            .collect(),
        extend: g.extend,
        interpolation: g.interpolation,
    }
}

fn gradient(g: &Gradient, cx: &mut Resolve<'_>) -> p::GradientSpec {
    let theme = cx.theme;
    let stops: Vec<GradientStop> = g
        .stops
        .iter()
        .map(|(c, stop_at)| GradientStop {
            color: c.resolve_space(theme),
            at: stop_at.map(|l| cx.paint_len(l)),
        })
        .collect();
    // One hint per gap between stops; the last stop's hint has no gap.
    let gaps = stops.len().saturating_sub(1);
    let hints = g
        .hints
        .iter()
        .take(gaps)
        .map(|h| h.map(|l| cx.paint_len(l)))
        .collect();
    p::GradientSpec {
        kind: match g.kind {
            GradientKind::Linear { angle } => WireKind::Linear { angle_deg: angle },
            GradientKind::Radial {
                circle,
                size,
                at: c,
            } => WireKind::Radial {
                circle,
                extent: match size {
                    RadialSize::Extent(e) => match e {
                        RadialExtent::ClosestSide => p::RadialExtent::ClosestSide,
                        RadialExtent::ClosestCorner => p::RadialExtent::ClosestCorner,
                        RadialExtent::FarthestSide => p::RadialExtent::FarthestSide,
                        RadialExtent::FarthestCorner => p::RadialExtent::FarthestCorner,
                    },
                    RadialSize::Radii(x, y) => {
                        p::RadialExtent::Radii(cx.paint_len(x), cx.paint_len(y))
                    }
                },
                center: point(c, cx),
            },
            GradientKind::Conic { from, at: c } => WireKind::Conic {
                from_deg: from,
                center: point(c, cx),
            },
        },
        stops,
        hints,
        repeating: g.repeating,
        interpolation: g.interpolation,
    }
}

/// The engine's shader fill, with the theme's colors resolved and the
/// fallback lowered (a shader cannot be its own fallback: such a fallback
/// draws nothing).
fn shader_spec(s: &ShaderPaint, cx: &mut Resolve<'_>) -> ShaderSpec {
    let fallback = s
        .fallback
        .as_ref()
        .and_then(|p| match fill_layer(p, cx).image {
            FillImage::Shader(_) => None,
            image => Some(image),
        });
    ShaderSpec {
        program: s.data.program.clone(),
        params: pack(&s.data.program, &s.data.params, cx.theme),
        images: s.data.images.clone(),
        clock: s.data.clock,
        essential: s.data.essential,
        fallback,
        margin: 0.0,
    }
}
