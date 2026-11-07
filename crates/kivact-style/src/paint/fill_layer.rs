//! Role: Resolve paints, gradients and shader paints with the theme into wire fills.

use super::canvas_gradient::{CanvasGradient, CanvasShape};
use super::gradient::{Gradient, GradientKind, RadialExtent, RadialSize};
use super::layer::{Paint, PaintImage, PaintSize};
use crate::length::Length;
use crate::shader::{ShaderPaint, pack};
use crate::theme::value::Theme;
use kivact_types as p;
use kivact_types::{
    FillImage, FillLayer, FillSize, GradientKind as WireKind, GradientStop, ShaderSpec,
};

/// A paint as the engine fills an area with it.
pub(crate) fn fill_layer(l: &Paint, theme: &Theme) -> FillLayer {
    let len = |l: Length| l.to_len();
    let auto = |l: Length| (l != Length::Auto && !l.is_content()).then(|| l.to_len());
    FillLayer {
        image: match &l.image {
            PaintImage::Color(c) => FillImage::Solid(c.resolve(theme)),
            PaintImage::Gradient(g) => FillImage::Gradient(gradient(g, theme)),
            PaintImage::Canvas(g) => FillImage::Coordinate(canvas_gradient(g, theme)),
            PaintImage::Image(a) => FillImage::Image(a.clone()),
            PaintImage::Shader(s) => FillImage::Shader(std::sync::Arc::new(shader_spec(s, theme))),
        },
        size: match l.size {
            PaintSize::Auto => FillSize::Auto,
            PaintSize::Cover => FillSize::Cover,
            PaintSize::Contain => FillSize::Contain,
            PaintSize::Explicit(w, h) => FillSize::Explicit(auto(w), auto(h)),
        },
        position: [len(l.position.0), len(l.position.1)],
        repeat_x: l.repeat.0,
        repeat_y: l.repeat.1,
        clip: l.clip,
        origin: l.origin,
        blend: l.blend,
    }
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

fn gradient(g: &Gradient, theme: &Theme) -> p::Gradient {
    let at = |(x, y): (Length, Length)| [x.to_len(), y.to_len()];
    let stops: Vec<GradientStop> = g
        .stops
        .iter()
        .map(|(c, at)| GradientStop {
            color: c.resolve_space(theme),
            at: at.map(Length::to_len),
        })
        .collect();
    // One hint per gap between stops; the last stop's hint has no gap.
    let gaps = stops.len().saturating_sub(1);
    let hints = g
        .hints
        .iter()
        .take(gaps)
        .map(|h| h.map(Length::to_len))
        .collect();
    p::Gradient {
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
                    RadialSize::Radii(x, y) => p::RadialExtent::Radii(x.to_len(), y.to_len()),
                },
                center: at(c),
            },
            GradientKind::Conic { from, at: c } => WireKind::Conic {
                from_deg: from,
                center: at(c),
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
fn shader_spec(s: &ShaderPaint, theme: &Theme) -> ShaderSpec {
    let fallback = s
        .fallback
        .as_ref()
        .and_then(|p| match fill_layer(p, theme).image {
            FillImage::Shader(_) => None,
            image => Some(image),
        });
    ShaderSpec {
        program: s.program.clone(),
        params: pack(&s.program, &s.params, theme),
        images: s.images.clone(),
        clock: s.clock,
        essential: s.essential,
        fallback,
        margin: 0.0,
    }
}
