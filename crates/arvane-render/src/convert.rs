//! Role: Conversion of portable scene types into private rasterizer types.
//!
//! Convert portable scene vocabulary to private rasterizer types without leaking dependencies.
//!
//! Shapes reach the rasterizers as their one outline
//! (`kivact_types::Shape::outline`, arcs as cubics), the outline hit tests
//! use too. A zero-length contour, or a zero-length dash, with round or
//! square caps draws a dot: kurbo's stroker draws nothing for a
//! zero-length segment, so it becomes a segment of `DOT` (0.001) along x
//! ([`dots`], [`stroke`]). A path already holding such a tiny segment
//! (Tabler's `h.01` icons) is drawn as it is. Box shadows are drawn as
//! blurred rounded rectangles ([`blur_param`], [`shadow_quarters`]).
//!
//! Limits: a zero-length dash that starts exactly where the line ends
//! draws no dot (kurbo's dashing cuts it to nothing there).

use kivact_types::{
    BlendMode, FillRule, LineCap, LineJoin, PathCmd, Point, Rect, Rgba, Shape, Stroke, Transform,
};
use vello_cpu::color::{AlphaColor, Srgb};
use vello_cpu::kurbo::{
    self, Affine, BezPath, Point as RasterPoint, Rect as RasterRect, Shape as _,
};
use vello_cpu::peniko::{self, Compose, Mix};

pub(crate) fn point(point: Point) -> RasterPoint {
    RasterPoint::new(f64::from(point.x), f64::from(point.y))
}

pub(crate) fn rect(rect: Rect) -> RasterRect {
    RasterRect::new(
        f64::from(rect.x),
        f64::from(rect.y),
        f64::from(rect.x + rect.width),
        f64::from(rect.y + rect.height),
    )
}

pub(crate) fn rounded_rect(rectangle: Rect, radii: [f32; 4]) -> vello_cpu::kurbo::RoundedRect {
    vello_cpu::kurbo::RoundedRect::from_rect(
        rect(rectangle),
        (
            f64::from(radii[0]),
            f64::from(radii[1]),
            f64::from(radii[2]),
            f64::from(radii[3]),
        ),
    )
}

pub(crate) fn color(color: Rgba) -> AlphaColor<Srgb> {
    AlphaColor::new([color.r, color.g, color.b, color.a])
}

/// Path of a stroked line. Drawn at `scale` device pixels per unit (no
/// other transform; `None` under one), a horizontal or vertical line whose
/// width is a whole number of device pixels moves across its axis onto the
/// pixel grid (pixel centers for odd widths, pixel edges for even ones), so
/// a 1 px line covers one row or column at full coverage rather than two at
/// half. Other lines keep their exact geometry.
pub(crate) fn line(from: Point, to: Point, width: f32, scale: Option<f32>) -> BezPath {
    let (mut from, mut to) = (from, to);
    let Some(scale) = scale else {
        let mut path = BezPath::new();
        path.move_to(point(from));
        path.line_to(point(to));
        return path;
    };
    if from.x == to.x && from.y != to.y {
        from.x = snap(from.x, width, scale);
        to.x = from.x;
    } else if from.y == to.y && from.x != to.x {
        from.y = snap(from.y, width, scale);
        to.y = from.y;
    }
    let mut path = BezPath::new();
    path.move_to(point(from));
    path.line_to(point(to));
    path
}

/// The stroke center of a whole-pixel `width` nearest to `center`, in logical units.
fn snap(center: f32, width: f32, scale: f32) -> f32 {
    let device = width * scale;
    let pixels = device.round();
    if pixels < 1.0 || (device - pixels).abs() > 0.01 {
        return center;
    }
    let at = center * scale;
    let snapped = if pixels % 2.0 == 1.0 {
        at.floor() + 0.5
    } else {
        at.round()
    };
    snapped / scale
}

/// The blur parameter of vello's and vello_cpu's blurred rounded
/// rectangles for a gaussian of standard deviation `std_dev`: they fade by
/// erf(d / s), so s is √2 deviations. Both cut the blur off at 2.5 s (3.54
/// deviations, within `kivact_types::SHADOW_REACH`).
pub(crate) fn blur_param(std_dev: f32) -> f64 {
    f64::from(std_dev) * std::f64::consts::SQRT_2
}

/// How a box shadow (`DrawCommand::Shadow`) is drawn.
pub(crate) enum ShadowParts {
    /// Whole, with one radius.
    Whole(f64),
    /// Each quarter (an area to paint) with its corner's radius.
    Quarters([(RasterRect, f64); 4]),
}

/// How a box shadow (`DrawCommand::Shadow`) is drawn: with one radius
/// when its corners agree, else per corner, each in its quarter of
/// the blur's reach (split at the center of `rect`), so each quadrant
/// has its own radius. Radii are limited to half the
/// shorter side.
pub(crate) fn shadow_quarters(rect: Rect, radii: [f32; 4], std_dev: f32) -> ShadowParts {
    let limit = f64::from(rect.width.min(rect.height)) / 2.0;
    let radii = radii.map(|r| f64::from(r).min(limit));
    if radii.iter().all(|r| *r == radii[0]) {
        return ShadowParts::Whole(radii[0]);
    }
    // The renderers cut the blur off at 2.5 of their parameter.
    let reach = 2.5 * blur_param(std_dev);
    let whole = self::rect(rect).inflate(reach, reach);
    let c = whole.center();
    ShadowParts::Quarters([
        (RasterRect::new(whole.x0, whole.y0, c.x, c.y), radii[0]),
        (RasterRect::new(c.x, whole.y0, whole.x1, c.y), radii[1]),
        (RasterRect::new(c.x, c.y, whole.x1, whole.y1), radii[2]),
        (RasterRect::new(whole.x0, c.y, c.x, whole.y1), radii[3]),
    ])
}

/// A path's segments, arcs as cubics (`kivact_types`'s, the outline hit
/// tests use too).
pub(crate) fn path(cmds: &[PathCmd]) -> BezPath {
    let has_arcs = cmds
        .iter()
        .any(|c| matches!(c, PathCmd::ArcTo { .. } | PathCmd::Arc { .. }));
    if has_arcs {
        let lowered = Shape::Path {
            cmds: cmds.to_vec(),
            rule: FillRule::NonZero,
        }
        .outline();
        return outline(&lowered);
    }
    outline(cmds)
}

/// An outline without arcs (`Shape::outline`) as a raster path.
fn outline(cmds: &[PathCmd]) -> BezPath {
    let mut path = BezPath::new();
    for cmd in cmds {
        match cmd {
            PathCmd::MoveTo(p) => path.move_to(point(*p)),
            PathCmd::LineTo(p) => path.line_to(point(*p)),
            PathCmd::QuadTo(a, b) => path.quad_to(point(*a), point(*b)),
            PathCmd::CubicTo(a, b, c) => path.curve_to(point(*a), point(*b), point(*c)),
            PathCmd::Close => path.close_path(),
            PathCmd::ArcTo { .. } | PathCmd::Arc { .. } => {
                unreachable!("invariant: outlines have no arcs")
            }
        }
    }
    path
}

/// A shape's outline as a raster path.
pub(crate) fn shape(shape: &Shape) -> BezPath {
    match shape {
        Shape::Rect(r) => rect(*r).to_path(0.1),
        Shape::Path { cmds, .. } => path(cmds),
        _ => outline(&shape.outline()),
    }
}

pub(crate) fn fill_rule(rule: FillRule) -> peniko::Fill {
    match rule {
        FillRule::NonZero => peniko::Fill::NonZero,
        FillRule::EvenOdd => peniko::Fill::EvenOdd,
    }
}

pub(crate) fn affine(t: Transform) -> Affine {
    Affine::new([t.a, t.b, t.c, t.d, t.e, t.f].map(f64::from))
}

/// The raster stroke style. Zero-length dashes with round or square caps
/// become tiny ones, so that they draw dots as SVG says (the stroker draws
/// nothing for a zero-length segment).
pub(crate) fn stroke(style: &Stroke) -> kurbo::Stroke {
    let cap = match style.cap {
        LineCap::Butt => kurbo::Cap::Butt,
        LineCap::Round => kurbo::Cap::Round,
        LineCap::Square => kurbo::Cap::Square,
    };
    let join = match style.join {
        LineJoin::Miter => kurbo::Join::Miter,
        LineJoin::Round => kurbo::Join::Round,
        LineJoin::Bevel => kurbo::Join::Bevel,
    };
    let mut out = kurbo::Stroke::new(f64::from(style.width))
        .with_caps(cap)
        .with_join(join)
        .with_miter_limit(f64::from(style.miter_limit));
    if !style.dash.is_empty() {
        let mut dash: Vec<f64> = style.dash.iter().map(|d| f64::from(*d)).collect();
        if dash.len() % 2 == 1 {
            dash.extend_from_within(..);
        }
        if style.cap != LineCap::Butt {
            for i in (0..dash.len()).step_by(2) {
                if dash[i] == 0.0 {
                    dash[i] = DOT;
                    dash[i + 1] = (dash[i + 1] - DOT).max(0.0);
                }
            }
        }
        out = out.with_dashes(f64::from(style.dash_offset), dash);
    }
    out
}

/// The length of a dot's segment.
const DOT: f64 = 1e-3;

/// `path` with each zero-length contour made a tiny segment along x, so a
/// round or square cap draws a dot there (SVG: a zero-length subpath with
/// such caps is drawn, aligned with the x axis).
pub(crate) fn dots(path: BezPath) -> BezPath {
    use kurbo::PathEl;
    let elements = path.elements();
    let mut out: Option<BezPath> = None;
    let mut start = 0;
    while start < elements.len() {
        let end = elements[start + 1..]
            .iter()
            .position(|e| matches!(e, PathEl::MoveTo(_)))
            .map_or(elements.len(), |i| start + 1 + i);
        let contour = &elements[start..end];
        let degenerate = match contour.first() {
            Some(PathEl::MoveTo(p)) => contour[1..].iter().all(|e| match e {
                PathEl::LineTo(q) => q == p,
                PathEl::QuadTo(a, b) => a == p && b == p,
                PathEl::CurveTo(a, b, c) => a == p && b == p && c == p,
                PathEl::ClosePath => true,
                PathEl::MoveTo(_) => false,
            }),
            _ => false,
        };
        if degenerate {
            let out = out.get_or_insert_with(|| elements[..start].iter().copied().collect());
            if let Some(PathEl::MoveTo(p)) = contour.first() {
                out.move_to(*p);
                out.line_to((p.x + DOT, p.y));
            }
        } else if let Some(out) = out.as_mut() {
            out.extend(contour.iter().copied());
        }
        start = end;
    }
    out.unwrap_or(path)
}

pub(crate) fn blend(mode: BlendMode) -> Option<peniko::BlendMode> {
    let mix = match mode {
        BlendMode::Normal => return None,
        BlendMode::PlusLighter => return Some(peniko::BlendMode::new(Mix::Normal, Compose::Plus)),
        BlendMode::Multiply => Mix::Multiply,
        BlendMode::Screen => Mix::Screen,
        BlendMode::Overlay => Mix::Overlay,
        BlendMode::Darken => Mix::Darken,
        BlendMode::Lighten => Mix::Lighten,
        BlendMode::ColorDodge => Mix::ColorDodge,
        BlendMode::ColorBurn => Mix::ColorBurn,
        BlendMode::HardLight => Mix::HardLight,
        BlendMode::SoftLight => Mix::SoftLight,
        BlendMode::Difference => Mix::Difference,
        BlendMode::Exclusion => Mix::Exclusion,
        BlendMode::Hue => Mix::Hue,
        BlendMode::Saturation => Mix::Saturation,
        BlendMode::Color => Mix::Color,
        BlendMode::Luminosity => Mix::Luminosity,
    };
    Some(peniko::BlendMode::new(mix, Compose::SrcOver))
}

/// Removing what it covers from what is beneath (Porter-Duff destination
/// out): masks and inset shadows.
pub(crate) fn dest_out() -> peniko::BlendMode {
    peniko::BlendMode::new(Mix::Normal, Compose::DestOut)
}
