//! Role: Converts gradient paints without renderer cache state.

use crate::convert;
use kivact_types::{ColorMatrix, Extend, Gradient, GradientShape, Rgba};
use vello_cpu::PaintType;
use vello_cpu::kurbo::Affine;
use vello_cpu::peniko::{self, Gradient as RasterGradient};

/// `color` through `matrix` when there is one.
pub(crate) fn color(color: Rgba, matrix: Option<&ColorMatrix>) -> Rgba {
    match matrix {
        Some(m) => m.apply(color),
        None => color,
    }
}

/// Stops as the rasterizers take them, colors through `matrix`.
///
/// Two stops at offset 0 (or 1) make a hard edge there, which the
/// rasterizers lose when padding: beyond the end they paint the first of
/// the stops at that offset. Then (`pad`) the stops move inwards by a
/// hair and the outer color ends the list.
pub(crate) fn stops_of(
    stops: &[(f32, Rgba)],
    matrix: Option<&ColorMatrix>,
    pad: bool,
) -> Vec<RasterStop> {
    let mut out: Vec<RasterStop> = stops
        .iter()
        .map(|(offset, c)| (*offset, convert::color(color(*c, matrix))))
        .collect();
    let n = out.len();
    if !pad || n < 2 {
        return out;
    }
    let hard_start = out[0].0 <= 0.0 && out[1].0 <= 0.0;
    let hard_end = out[n - 1].0 >= 1.0 && out[n - 2].0 >= 1.0;
    if hard_start || hard_end {
        const HAIR: f32 = 1e-4;
        let (first, last) = (out[0].1, out[n - 1].1);
        for stop in &mut out {
            stop.0 = HAIR + stop.0 * (1.0 - 2.0 * HAIR);
        }
        if hard_start {
            out.insert(0, (0.0, first));
        }
        if hard_end {
            out.push((1.0, last));
        }
    }
    out
}

/// A stop as the rasterizers take it.
type RasterStop = (f32, peniko::color::AlphaColor<peniko::color::Srgb>);

/// A gradient's brush and paint transform.
pub(crate) fn gradient(g: &Gradient, matrix: Option<&ColorMatrix>) -> (PaintType, Option<Affine>) {
    if let GradientShape::Radial {
        radius_x, radius_y, ..
    } = g.shape
        && (radius_x <= 0.0 || radius_y <= 0.0)
    {
        // Nothing to spread over: the last stop's color everywhere.
        let last = g.stops.last().map_or(Rgba::default(), |s| s.1);
        return (PaintType::Solid(convert::color(color(last, matrix))), None);
    }
    let (raster, transform) = gradient_shape(&g.shape);
    let extend = match g.extend {
        Extend::Pad => peniko::Extend::Pad,
        Extend::Repeat => peniko::Extend::Repeat,
        Extend::Reflect => peniko::Extend::Reflect,
    };
    (
        PaintType::Gradient(
            raster
                .with_extend(extend)
                .with_stops(stops_of(&g.stops, matrix, g.extend == Extend::Pad).as_slice()),
        ),
        transform,
    )
}

/// The raster gradient of a geometry, and its paint transform when the
/// geometry is not one vello has (an ellipse: a unit circle stretched).
fn gradient_shape(shape: &GradientShape) -> (RasterGradient, Option<Affine>) {
    match *shape {
        GradientShape::Linear { start, end } => (
            RasterGradient::new_linear(convert::point(start), convert::point(end)),
            None,
        ),
        GradientShape::Radial {
            center,
            radius_x,
            radius_y,
        } => {
            if radius_x == radius_y {
                (
                    RasterGradient::new_radial(convert::point(center), radius_x),
                    None,
                )
            } else {
                (
                    RasterGradient::new_radial((0.0, 0.0), 1.0),
                    Some(
                        Affine::translate((f64::from(center.x), f64::from(center.y)))
                            * Affine::scale_non_uniform(f64::from(radius_x), f64::from(radius_y)),
                    ),
                )
            }
        }
        GradientShape::RadialTwoCircle {
            start_center,
            start_radius,
            end_center,
            end_radius,
        } => (
            RasterGradient::new_two_point_radial(
                convert::point(start_center),
                start_radius,
                convert::point(end_center),
                end_radius,
            ),
            None,
        ),
        GradientShape::Conic {
            center,
            start_deg,
            end_deg,
        } => (
            RasterGradient::new_sweep(
                convert::point(center),
                start_deg.to_radians(),
                end_deg.to_radians(),
            ),
            None,
        ),
    }
}
