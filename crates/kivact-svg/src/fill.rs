//! Role: Writes fills and strokes as SVG paints and the gradients they use.
//!
//! A fill is one color or one background layer: a solid image, a CSS linear
//! or radial gradient sized to the shape's bounds, or a coordinate gradient
//! (a canvas linear or two-circle radial gradient). A layer that is sized or
//! placed, a repeating or conic gradient, and an image are not supported.

use super::emitter::Emitter;
use super::encode::{hex, number, opacity_attribute};
use crate::SvgError;
use kivact_types::{
    CoordinateGradient, CoordinateShape, Extend, Fill, FillImage, FillLayer, FillSize,
    GradientKind, GradientSpec, Len, Point, RadialExtent, Rect, Rgba, normalized_stops,
};

/// The coordinate system a gradient's geometry is given in: the user space
/// of the drawing, or the box of the shape it paints (SVG
/// `gradientUnits="objectBoundingBox"`, the unit square over that box).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Units {
    UserSpace,
    ObjectBox,
}

impl Units {
    /// The SVG `gradientUnits` keyword of the system.
    fn name(self) -> &'static str {
        match self {
            Units::UserSpace => "userSpaceOnUse",
            Units::ObjectBox => "objectBoundingBox",
        }
    }
}

/// The SVG attributes of a paint `kind` (`fill` or `stroke`) for `fill`,
/// over a shape whose bounds are `bounds`, with gradient geometry in `units`.
/// Gradients are defined in `defs`.
pub(super) fn paint_attributes(
    emitter: &mut Emitter<'_>,
    kind: &str,
    fill: &Fill,
    bounds: Rect,
    units: Units,
) -> Result<String, SvgError> {
    match fill {
        Fill::Solid(color) => Ok(solid(kind, *color)),
        Fill::Layers(layers) => match layers.as_slice() {
            [] => Ok(format!("{kind}=\"none\"")),
            [layer] => layer_paint(emitter, kind, layer, bounds, units),
            _ => Err(SvgError::unsupported(
                "more than one background layer in a fill",
            )),
        },
    }
}

fn solid(kind: &str, color: Rgba) -> String {
    format!(
        "{kind}=\"{}\"{}",
        hex(color),
        opacity_attribute(kind, color)
    )
}

fn layer_paint(
    emitter: &mut Emitter<'_>,
    kind: &str,
    layer: &FillLayer,
    bounds: Rect,
    units: Units,
) -> Result<String, SvgError> {
    if layer.size != FillSize::Auto || layer.position != [Len::ZERO; 2] {
        return Err(SvgError::unsupported(
            "a background layer sized or placed by hand",
        ));
    }
    match &layer.image {
        FillImage::Solid(color) => Ok(solid(kind, *color)),
        FillImage::Gradient(gradient) => gradient_paint(emitter, kind, gradient, bounds, units),
        FillImage::Coordinate(gradient) => coordinate_paint(emitter, kind, gradient, units),
        FillImage::Image(_) => Err(SvgError::unsupported("an image as a fill")),
        FillImage::Shader(_) => Err(SvgError::unsupported("a shader as a fill")),
    }
}

/// A CSS gradient over `bounds`: a linear one along its angle, or a radial
/// one with explicit radii.
fn gradient_paint(
    emitter: &mut Emitter<'_>,
    kind: &str,
    gradient: &GradientSpec,
    bounds: Rect,
    units: Units,
) -> Result<String, SvgError> {
    if gradient.repeating {
        return Err(SvgError::unsupported("a repeating gradient"));
    }
    let Rect {
        x,
        y,
        width: w,
        height: h,
    } = bounds;
    match gradient.kind {
        GradientKind::Linear { .. } => {
            let Some((start, end)) = gradient.line(bounds) else {
                return Err(SvgError::unsupported("a linear gradient without a line"));
            };
            let length = (end.x - start.x).hypot(end.y - start.y);
            let stops = gradient.srgb_stops(length);
            let (Some(first), Some(last)) = (stops.first(), stops.last()) else {
                return Ok(format!("{kind}=\"none\""));
            };
            let (lo, hi) = (first.0.min(0.0), last.0.max(1.0));
            let at = |t: f64| Point {
                x: start.x + (end.x - start.x) * t,
                y: start.y + (end.y - start.y) * t,
            };
            let stops = normalized_stops(&stops, lo, hi);
            let id = emitter.id("grad");
            linear_def(emitter, &id, at(lo), at(hi), Extend::Pad, &stops, units);
            Ok(format!("{kind}=\"url(#{id})\""))
        }
        GradientKind::Radial {
            circle,
            extent: RadialExtent::Radii(rx, ry),
            center,
        } => {
            let cx = x + center[0].of(w);
            let cy = y + center[1].of(h);
            let radius = rx.of(w);
            if radius.is_nan() || radius <= 0.0 {
                return Err(SvgError::unsupported("a radial gradient of zero size"));
            }
            // An ellipse is the circle scaled vertically about its center.
            let scale = if circle { 1.0 } else { ry.of(h) / radius };
            let stops: Vec<(f64, Rgba)> = gradient
                .srgb_stops(radius)
                .into_iter()
                .map(|(t, c)| (t.clamp(0.0, 1.0), c))
                .collect();
            let id = emitter.id("grad");
            let shape = Radial {
                center: Point { x: cx, y: cy },
                radius,
                scale,
                focal: None,
            };
            radial_def(emitter, &id, &shape, Extend::Pad, &stops, units);
            Ok(format!("{kind}=\"url(#{id})\""))
        }
        GradientKind::Radial { .. } => Err(SvgError::unsupported(
            "a radial gradient sized by a keyword",
        )),
        GradientKind::Conic { .. } => Err(SvgError::unsupported("a conic gradient")),
    }
}

/// A canvas gradient placed by coordinates in the drawing.
fn coordinate_paint(
    emitter: &mut Emitter<'_>,
    kind: &str,
    gradient: &CoordinateGradient,
    units: Units,
) -> Result<String, SvgError> {
    let stops: Vec<(f64, Rgba)> = gradient
        .stops
        .iter()
        .map(|(offset, color)| (*offset, color.to_srgb()))
        .collect();
    let id = emitter.id("grad");
    match gradient.shape {
        CoordinateShape::Linear { start, end } => {
            linear_def(emitter, &id, start, end, gradient.extend, &stops, units);
        }
        CoordinateShape::RadialTwoCircle {
            start_center,
            start_radius,
            end_center,
            end_radius,
        } => {
            let shape = Radial {
                center: end_center,
                radius: end_radius,
                scale: 1.0,
                focal: Some((start_center, start_radius)),
            };
            radial_def(emitter, &id, &shape, gradient.extend, &stops, units);
        }
        CoordinateShape::Conic { .. } => {
            return Err(SvgError::unsupported("a conic canvas gradient"));
        }
    }
    Ok(format!("{kind}=\"url(#{id})\""))
}

fn linear_def(
    emitter: &mut Emitter<'_>,
    id: &str,
    start: Point,
    end: Point,
    extend: Extend,
    stops: &[(f64, Rgba)],
    units: Units,
) {
    emitter.defs.push_str(&format!(
        "<linearGradient id=\"{id}\" gradientUnits=\"{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"{}>{}</linearGradient>\n",
        units.name(),
        number(start.x),
        number(start.y),
        number(end.x),
        number(end.y),
        spread(extend),
        stop_list(stops)
    ));
}

/// The geometry of a radial gradient: a circle of `radius` at `center`,
/// scaled vertically by `scale` about its center. `focal` is the start
/// circle of a two-circle gradient (its center and radius).
#[derive(Clone, Copy)]
struct Radial {
    center: Point,
    radius: f64,
    scale: f64,
    focal: Option<(Point, f64)>,
}

/// Defines the radial gradient `id` over `shape` in `defs`.
fn radial_def(
    emitter: &mut Emitter<'_>,
    id: &str,
    shape: &Radial,
    extend: Extend,
    stops: &[(f64, Rgba)],
    units: Units,
) {
    let Radial {
        center,
        radius,
        scale,
        focal,
    } = *shape;
    let mut attributes = format!(
        "cx=\"{}\" cy=\"{}\" r=\"{}\"",
        number(center.x),
        number(center.y),
        number(radius)
    );
    if let Some((start, start_radius)) = focal {
        attributes.push_str(&format!(
            " fx=\"{}\" fy=\"{}\" fr=\"{}\"",
            number(start.x),
            number(start.y),
            number(start_radius)
        ));
    }
    if scale != 1.0 {
        // Maps the circle's y about its center: (x, y) -> (x, s·y + cy·(1 − s)).
        attributes.push_str(&format!(
            " gradientTransform=\"matrix(1 0 0 {} 0 {})\"",
            number(scale),
            number(center.y * (1.0 - scale))
        ));
    }
    emitter.defs.push_str(&format!(
        "<radialGradient id=\"{id}\" gradientUnits=\"{}\" {attributes}{}>{}</radialGradient>\n",
        units.name(),
        spread(extend),
        stop_list(stops)
    ));
}

fn spread(extend: Extend) -> &'static str {
    match extend {
        Extend::Pad => "",
        Extend::Repeat => " spreadMethod=\"repeat\"",
        Extend::Reflect => " spreadMethod=\"reflect\"",
    }
}

fn stop_list(stops: &[(f64, Rgba)]) -> String {
    stops
        .iter()
        .map(|(offset, color)| {
            format!(
                "<stop offset=\"{}\" stop-color=\"{}\"{}/>",
                number(*offset),
                hex(*color),
                opacity_attribute("stop", *color)
            )
        })
        .collect()
}
