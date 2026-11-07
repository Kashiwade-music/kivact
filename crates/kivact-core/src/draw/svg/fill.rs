//! Role: Writes fills and strokes as SVG paints, defining the gradients they use.
//!
//! A fill is one color or one background layer: a solid image, a CSS linear
//! or radial gradient sized to the shape's bounds, or a coordinate gradient
//! (a canvas linear or two-circle radial gradient). A layer that is sized or
//! placed, a repeating or conic gradient, and an image are not supported.

use super::emitter::Emitter;
use super::encode::{hex, number, opacity_attribute};
use crate::Result;
use kivact_types::{
    CoordinateGradient, CoordinateShape, Extend, Fill, FillImage, FillLayer, FillSize, Gradient,
    GradientKind, GradientStop, Len, Point, RadialExtent, Rect, Rgba,
};

/// The SVG attributes of a paint `kind` (`fill` or `stroke`) for `fill`,
/// over a shape whose bounds are `bounds`. Gradients are defined in `defs`.
pub(super) fn paint_attributes(
    emitter: &mut Emitter<'_>,
    kind: &str,
    fill: &Fill,
    bounds: Rect,
) -> Result<String> {
    match fill {
        Fill::Solid(color) => Ok(solid(kind, *color)),
        Fill::Layers(layers) => match layers.as_slice() {
            [] => Ok(format!("{kind}=\"none\"")),
            [layer] => layer_paint(emitter, kind, layer, bounds),
            _ => Err(emitter.unsupported("more than one background layer in a fill")),
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
) -> Result<String> {
    if layer.size != FillSize::Auto || layer.position != [Len::ZERO; 2] {
        return Err(emitter.unsupported("a background layer sized or placed by hand"));
    }
    match &layer.image {
        FillImage::Solid(color) => Ok(solid(kind, *color)),
        FillImage::Gradient(gradient) => gradient_paint(emitter, kind, gradient, bounds),
        FillImage::Coordinate(gradient) => coordinate_paint(emitter, kind, gradient),
        FillImage::Image(_) => Err(emitter.unsupported("an image as a fill")),
    }
}

/// A CSS gradient over `bounds`: a linear one along its angle, or a radial
/// one with explicit radii.
fn gradient_paint(
    emitter: &mut Emitter<'_>,
    kind: &str,
    gradient: &Gradient,
    bounds: Rect,
) -> Result<String> {
    if gradient.repeating {
        return Err(emitter.unsupported("a repeating gradient"));
    }
    let Rect {
        x,
        y,
        width: w,
        height: h,
    } = bounds;
    match gradient.kind {
        GradientKind::Linear { angle_deg } => {
            // CSS: 0 points to the top, 90 to the right; the line is as long
            // as needed for the corners to take the end colors.
            let theta = angle_deg.to_radians();
            let (dx, dy) = (theta.sin(), -theta.cos());
            let length = (w * dx).abs() + (h * dy).abs();
            let center = Point {
                x: x + w / 2.0,
                y: y + h / 2.0,
            };
            let start = Point {
                x: center.x - dx * length / 2.0,
                y: center.y - dy * length / 2.0,
            };
            let end = Point {
                x: center.x + dx * length / 2.0,
                y: center.y + dy * length / 2.0,
            };
            let stops = css_stops(&gradient.stops, length);
            let id = emitter.id("grad");
            linear_def(emitter, &id, start, end, Extend::Pad, &stops);
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
            if !(radius > 0.0) {
                return Err(emitter.unsupported("a radial gradient of zero size"));
            }
            // An ellipse is the circle scaled vertically about its center.
            let scale = if circle { 1.0 } else { ry.of(h) / radius };
            let stops = css_stops(&gradient.stops, radius);
            let id = emitter.id("grad");
            radial_def(
                emitter,
                &id,
                Point { x: cx, y: cy },
                radius,
                scale,
                None,
                Extend::Pad,
                &stops,
            );
            Ok(format!("{kind}=\"url(#{id})\""))
        }
        GradientKind::Radial { .. } => {
            Err(emitter.unsupported("a radial gradient sized by a keyword"))
        }
        GradientKind::Conic { .. } => Err(emitter.unsupported("a conic gradient")),
    }
}

/// A canvas gradient placed by coordinates in the drawing.
fn coordinate_paint(
    emitter: &mut Emitter<'_>,
    kind: &str,
    gradient: &CoordinateGradient,
) -> Result<String> {
    let stops = gradient.stops.clone();
    let id = emitter.id("grad");
    match gradient.shape {
        CoordinateShape::Linear { start, end } => {
            linear_def(emitter, &id, start, end, gradient.extend, &stops);
        }
        CoordinateShape::RadialTwoCircle {
            start_center,
            start_radius,
            end_center,
            end_radius,
        } => {
            radial_def(
                emitter,
                &id,
                end_center,
                end_radius,
                1.0,
                Some((start_center, start_radius)),
                gradient.extend,
                &stops,
            );
        }
        CoordinateShape::Conic { .. } => {
            return Err(emitter.unsupported("a conic canvas gradient"));
        }
    }
    Ok(format!("{kind}=\"url(#{id})\""))
}

/// Stop offsets and colors of a CSS gradient whose line is `length` long.
/// A stop without a position takes the place CSS gives it: the first and
/// the last default to 0 and 1, the others are spread evenly between their
/// positioned neighbours. Offsets are clamped to `0..=1` and never decrease.
fn css_stops(stops: &[GradientStop], length: f64) -> Vec<(f64, Rgba)> {
    let mut at: Vec<Option<f64>> = stops
        .iter()
        .map(|stop| {
            stop.at.map(|len| {
                if length > 0.0 {
                    len.of(length) / length
                } else {
                    0.0
                }
            })
        })
        .collect();
    if let Some(first) = at.first_mut() {
        first.get_or_insert(0.0);
    }
    if let Some(last) = at.last_mut() {
        last.get_or_insert(1.0);
    }
    let mut known: Option<usize> = None;
    for i in 0..at.len() {
        let Some(known_at) = at[i] else { continue };
        if let Some(p) = known {
            let from = at[p].unwrap_or(0.0);
            let steps = (i - p) as f64;
            for k in p + 1..i {
                at[k] = Some(from + (known_at - from) * (k - p) as f64 / steps);
            }
        }
        known = Some(i);
    }
    let mut previous = 0.0_f64;
    stops
        .iter()
        .zip(at)
        .map(|(stop, offset)| {
            let offset = offset.unwrap_or(0.0).clamp(0.0, 1.0).max(previous);
            previous = offset;
            (offset, stop.color)
        })
        .collect()
}

fn linear_def(
    emitter: &mut Emitter<'_>,
    id: &str,
    start: Point,
    end: Point,
    extend: Extend,
    stops: &[(f64, Rgba)],
) {
    emitter.defs.push_str(&format!(
        "<linearGradient id=\"{id}\" gradientUnits=\"userSpaceOnUse\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"{}>{}</linearGradient>\n",
        number(start.x),
        number(start.y),
        number(end.x),
        number(end.y),
        spread(extend),
        stop_list(stops)
    ));
}

/// A radial gradient centered at `center` with radius `radius`, scaled
/// vertically by `scale` about its center. `focal` is the start circle of
/// a two-circle gradient (its center and radius).
#[allow(
    clippy::too_many_arguments,
    reason = "one definition takes every SVG radial attribute"
)]
fn radial_def(
    emitter: &mut Emitter<'_>,
    id: &str,
    center: Point,
    radius: f64,
    scale: f64,
    focal: Option<(Point, f64)>,
    extend: Extend,
    stops: &[(f64, Rgba)],
) {
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
        "<radialGradient id=\"{id}\" gradientUnits=\"userSpaceOnUse\" {attributes}{}>{}</radialGradient>\n",
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
