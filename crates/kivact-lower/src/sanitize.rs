//! Role: CSS paint correction and graceful local-command adaptation.
//!
//! Correct supported CSS paint inputs and select drawable local commands.
//! Colors are clamped before lowering. These adaptations do not certify a
//! complete scene: Arvane rendering validation is the final boundary.

use kivact_wire::*;
use ui_types::DrawCommand as D;

/// A fraction in `[0, 1]`, or `fallback` when not finite.
pub fn unit(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        fallback
    }
}

/// Components are clamped to `[0, 1]`; non-finite components become zero.
pub fn color(p: Rgba) -> ui_types::Rgba {
    let unit = |v| unit(v, 0.0);
    ui_types::Rgba {
        r: unit(p.r),
        g: unit(p.g),
        b: unit(p.b),
        a: unit(p.a),
    }
}

/// A non-negative length; non-finite and negative values become zero.
pub fn extent(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// Preserve a finite signed offset; non-finite values become zero.
pub fn offset(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

/// A paint-time length with non-finite parts made zero.
fn len(l: &mut Len) {
    l.px = offset(l.px);
    l.pct = offset(l.pct);
}

fn corners(c: &mut Corners) {
    for radius in [
        &mut c.top_left,
        &mut c.top_right,
        &mut c.bottom_right,
        &mut c.bottom_left,
    ] {
        len(&mut radius.x);
        len(&mut radius.y);
    }
}

/// A fill layer's lengths made finite (colors are clamped by `color`).
pub(crate) fn fill_layer(l: &mut FillLayer) {
    l.position.iter_mut().for_each(len);
    if let FillSize::Explicit(w, h) = &mut l.size {
        w.iter_mut().chain(h.iter_mut()).for_each(len);
    }
    if let FillImage::Coordinate(g) = &mut l.image {
        let point = |p: &mut Point| {
            p.x = offset(p.x);
            p.y = offset(p.y);
        };
        match &mut g.shape {
            CoordinateShape::Linear { start, end } => {
                point(start);
                point(end);
            }
            CoordinateShape::RadialTwoCircle {
                start_center,
                start_radius,
                end_center,
                end_radius,
            } => {
                point(start_center);
                point(end_center);
                *start_radius = extent(*start_radius);
                *end_radius = extent(*end_radius);
            }
            CoordinateShape::Conic { center, start_deg } => {
                point(center);
                *start_deg = offset(*start_deg);
            }
        }
    }
    if let FillImage::Gradient(g) = &mut l.image {
        match &mut g.kind {
            GradientKind::Linear { angle_deg } => *angle_deg = offset(*angle_deg),
            GradientKind::Radial { extent, center, .. } => {
                center.iter_mut().for_each(len);
                if let RadialExtent::Radii(x, y) = extent {
                    len(x);
                    len(y);
                }
            }
            GradientKind::Conic { from_deg, center } => {
                *from_deg = offset(*from_deg);
                center.iter_mut().for_each(len);
            }
        }
        for stop in &mut g.stops {
            stop.at.iter_mut().for_each(len);
        }
    }
}

/// Clamp evaluated paint props into renderable ranges.
pub fn paint(p: &mut PaintProps) {
    p.opacity = unit(p.opacity, 1.0);
    if let Some(b) = &mut p.border {
        let w = &mut b.widths;
        for width in [&mut w.top, &mut w.right, &mut w.bottom, &mut w.left] {
            *width = extent(*width);
        }
    }
    if let Some(ring) = &mut p.ring {
        ring.width = extent(ring.width);
    }
    corners(&mut p.radius);
    if let Some(Fill::Layers(layers)) = &mut p.background {
        layers.iter_mut().for_each(fill_layer);
    }
    if let Some(e) = &mut p.effects {
        if let Some(m) = &mut e.mask {
            fill_layer(m);
        }
        match &mut e.clip_path {
            Some(ClipShape::Path { .. }) => {}
            Some(ClipShape::Inset { edges, radius }) => {
                for side in [
                    &mut edges.top,
                    &mut edges.right,
                    &mut edges.bottom,
                    &mut edges.left,
                ] {
                    len(side);
                }
                corners(radius);
            }
            Some(ClipShape::Circle { radius, center }) => {
                len(radius);
                center.iter_mut().for_each(len);
            }
            Some(ClipShape::Ellipse {
                radius_x,
                radius_y,
                center,
            }) => {
                len(radius_x);
                len(radius_y);
                center.iter_mut().for_each(len);
            }
            Some(ClipShape::Polygon { points, .. }) => {
                points.iter_mut().flatten().for_each(len);
            }
            None => {}
        }
    }
    for shadow in &mut p.shadows {
        shadow.offset.x = offset(shadow.offset.x);
        shadow.offset.y = offset(shadow.offset.y);
        shadow.blur = extent(shadow.blur);
        shadow.spread = offset(shadow.spread);
    }
    // A transform with a value that is not finite is none
    // (`model::transform::resolve`).
}

fn finite(values: &[f32]) -> bool {
    values.iter().all(|value| value.is_finite())
}
fn point(p: Point) -> bool {
    finite(&[p.x, p.y])
}
fn rect(r: Rect) -> bool {
    finite(&[r.x, r.y, r.width, r.height, r.x + r.width, r.y + r.height])
        && r.width >= 0.0
        && r.height >= 0.0
}
fn length(value: f32) -> bool {
    value.is_finite() && value >= 0.0
}
/// Gradient stops Arvane draws: at least one, offsets finite, within
/// `[0, 1]` and not decreasing. Colors are clamped here before lowering.
fn stops(stops: &[(f32, Rgba)]) -> bool {
    !stops.is_empty()
        && stops
            .iter()
            .all(|(offset, _)| offset.is_finite() && (0.0..=1.0).contains(offset))
        && stops.windows(2).all(|pair| pair[0].0 <= pair[1].0)
}
/// A gradient Arvane draws: valid stops and finite geometry (a conic one
/// sweeps forward).
fn gradient(value: &ui_types::Gradient) -> bool {
    stops(&value.stops)
        && match value.shape {
            ui_types::GradientShape::Linear { start, end } => point(start) && point(end),
            ui_types::GradientShape::Radial {
                center,
                radius_x,
                radius_y,
            } => point(center) && length(radius_x) && length(radius_y),
            ui_types::GradientShape::RadialTwoCircle {
                start_center,
                start_radius,
                end_center,
                end_radius,
            } => {
                point(start_center)
                    && point(end_center)
                    && length(start_radius)
                    && length(end_radius)
            }
            ui_types::GradientShape::Conic {
                center,
                start_deg,
                end_deg,
            } => point(center) && finite(&[start_deg, end_deg]) && end_deg > start_deg,
        }
}
pub(crate) fn fill(value: &ui_types::Paint) -> bool {
    match value {
        ui_types::Paint::Solid(_) => true,
        ui_types::Paint::Pattern(p) => {
            (match &p.source {
                ui_types::PatternSource::Gradient(g) => gradient(g),
                ui_types::PatternSource::Image { .. } => true,
            }) && rect(p.tile)
                && p.tile.width > 0.0
                && p.tile.height > 0.0
        }
        ui_types::Paint::LinearGradient {
            angle_deg,
            stops: s,
        } => angle_deg.is_finite() && stops(s),
        ui_types::Paint::Gradient(g) => gradient(g),
    }
}
/// Whether the commands form contours with finite points and arcs.
pub(crate) fn path(cmds: &[ui_types::PathCmd]) -> bool {
    let mut open = false;
    cmds.iter().all(|cmd| match cmd {
        ui_types::PathCmd::MoveTo(p) => {
            open = true;
            point(*p)
        }
        ui_types::PathCmd::LineTo(p) => open && point(*p),
        ui_types::PathCmd::QuadTo(p, q) => open && point(*p) && point(*q),
        ui_types::PathCmd::CubicTo(p, q, r) => open && point(*p) && point(*q) && point(*r),
        ui_types::PathCmd::ArcTo {
            to,
            rx,
            ry,
            x_rotation_deg,
            ..
        } => open && point(*to) && finite(&[*rx, *ry, *x_rotation_deg]),
        ui_types::PathCmd::Arc {
            center,
            rx,
            ry,
            x_rotation_deg,
            start_deg,
            sweep_deg,
        } => {
            open = true;
            point(*center) && finite(&[*rx, *ry, *x_rotation_deg, *start_deg, *sweep_deg])
        }
        ui_types::PathCmd::Close => std::mem::replace(&mut open, false),
    })
}

/// Whether a converted client command can be rendered; clips are balanced
/// by the caller.
pub(crate) fn drawable(cmd: &D) -> bool {
    match cmd {
        D::Rect { rect: r, fill: f } => rect(*r) && fill(f),
        D::RoundedRect {
            rect: r,
            radius,
            fill: f,
        } => rect(*r) && length(*radius) && fill(f),
        D::StrokeRect {
            rect: r,
            width,
            radius,
            ..
        } => rect(*r) && length(*width) && length(*radius),
        D::Line {
            from, to, width, ..
        } => point(*from) && point(*to) && length(*width),
        D::Path { cmds, stroke, .. } => path(cmds) && stroke.is_none_or(|(width, _)| length(width)),
        D::PushClip(r) => rect(*r),
        D::External {
            rect: r,
            source,
            opacity,
            ..
        } => rect(*r) && source.is_none_or(rect) && opacity.is_finite(),
        _ => true,
    }
}
