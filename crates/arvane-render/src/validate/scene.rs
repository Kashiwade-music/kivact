//! Role: Checks every command of a scene before anything is drawn.

use super::Fonts;
use crate::error::RenderError;
use crate::limits::{MAX_BLUR, MAX_DASHES, MAX_SCENE_DEPTH};
use kivact_types::geometry::Transform;
use kivact_types::pixel::Pixels;
use kivact_types::render::{
    BoxShadow, GlyphStroke, Gradient, GradientShape, Group, Paint, PatternSource, RenderCmd,
    RenderScene,
};
use kivact_types::scene::{PathCmd, RoundedRect, Shape};
use kivact_types::{Point, Rect, Rgba, Stroke};

// The commands each validation category handles, as patterns: the dispatch and every
// category's match are exhaustive, so a new command fails to compile until one handles it.
macro_rules! primitive_commands {
    () => {
        RenderCmd::Rect { .. }
            | RenderCmd::RoundedRect { .. }
            | RenderCmd::RoundedRectCorners { .. }
            | RenderCmd::StrokeRect { .. }
            | RenderCmd::StrokeRoundedRectCorners { .. }
            | RenderCmd::Shadow { .. }
            | RenderCmd::Line { .. }
            | RenderCmd::Path { .. }
    };
}
macro_rules! content_commands {
    () => {
        RenderCmd::Glyphs { .. } | RenderCmd::Image { .. } | RenderCmd::ImageSampled { .. }
    };
}
macro_rules! group_commands {
    () => {
        RenderCmd::FillShape { .. }
            | RenderCmd::StrokeShape { .. }
            | RenderCmd::BoxShadow(_)
            | RenderCmd::PushClip(_)
            | RenderCmd::PushTransform(_)
            | RenderCmd::PushGroup(_)
            | RenderCmd::PopClip
            | RenderCmd::PopTransform
            | RenderCmd::PopGroup
    };
}

fn finite(values: &[f64]) -> bool {
    values.iter().all(|value| value.is_finite())
}

fn point(value: Point) -> bool {
    finite(&[value.x, value.y])
}

fn rect(value: Rect) -> bool {
    finite(&[
        value.x,
        value.y,
        value.width,
        value.height,
        value.x + value.width,
        value.y + value.height,
    ]) && value.width >= 0.0
        && value.height >= 0.0
}

fn color(value: Rgba) -> bool {
    [value.r, value.g, value.b, value.a]
        .iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
}

fn stroke(width: f64) -> bool {
    width.is_finite() && width >= 0.0
}

fn unit(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn stops(stops: &[(f64, Rgba)]) -> bool {
    !stops.is_empty()
        && stops
            .iter()
            .all(|(offset, value)| unit(*offset) && color(*value))
        && stops.windows(2).all(|pair| pair[0].0 <= pair[1].0)
}

fn gradient(value: &Gradient) -> bool {
    stops(&value.stops)
        && match value.shape {
            GradientShape::Linear { start, end } => point(start) && point(end),
            GradientShape::Radial {
                center,
                radius_x,
                radius_y,
            } => point(center) && stroke(radius_x) && stroke(radius_y),
            GradientShape::RadialTwoCircle {
                start_center,
                start_radius,
                end_center,
                end_radius,
            } => {
                point(start_center)
                    && point(end_center)
                    && stroke(start_radius)
                    && stroke(end_radius)
            }
            GradientShape::Conic {
                center,
                start_deg,
                end_deg,
            } => point(center) && finite(&[start_deg, end_deg]) && end_deg > start_deg,
        }
}

fn fill(value: &Paint) -> Result<bool, RenderError> {
    Ok(match value {
        Paint::Solid(value) => color(*value),
        Paint::Gradient(g) => gradient(g),
        Paint::Pattern(p) => {
            let source = match &p.source {
                PatternSource::Image { image } => {
                    image_is_valid(image)?;
                    true
                }
                PatternSource::Gradient(g) => gradient(g),
            };
            source && rect(p.tile) && p.tile.width > 0.0 && p.tile.height > 0.0
        }
    })
}

fn rounded(value: &RoundedRect) -> bool {
    rect(value.rect) && value.radii.iter().all(|r| finite(&[r.x, r.y]))
}

fn shape(value: &Shape) -> bool {
    match value {
        Shape::Rect(r) => rect(*r),
        Shape::RoundedRect(r) => rounded(r),
        Shape::Polygon { points, .. } => points.iter().all(|p| point(*p)),
        Shape::Path { cmds, .. } => path(cmds),
    }
}

fn line_style(value: &Stroke) -> bool {
    stroke(value.width)
        && value.miter_limit.is_finite()
        && value.miter_limit >= 1.0
        && value.dash_offset.is_finite()
        && value.dash.len() <= MAX_DASHES
        && value.dash.iter().all(|d| stroke(*d))
        && (value.dash.is_empty() || value.dash.iter().sum::<f64>() > 0.0)
}

fn blur(std_dev: f64) -> bool {
    std_dev.is_finite() && (0.0..=MAX_BLUR).contains(&std_dev)
}

fn transform(value: &Transform) -> bool {
    finite(&[value.a, value.b, value.c, value.d, value.e, value.f])
}

fn group(value: &Group) -> bool {
    value.clip.as_ref().is_none_or(shape) && unit(value.opacity) && value.blur.is_none_or(blur)
}

fn box_shadow(value: &BoxShadow) -> bool {
    rounded(&value.shape)
        && point(value.offset)
        && value.spread.is_finite()
        && blur(value.std_dev)
        && color(value.color)
}

fn path(commands: &[PathCmd]) -> bool {
    let mut contour = false;
    for command in commands {
        let valid = match command {
            PathCmd::MoveTo(value) => {
                contour = true;
                point(*value)
            }
            PathCmd::LineTo(value) => contour && point(*value),
            PathCmd::QuadTo(a, b) => contour && point(*a) && point(*b),
            PathCmd::CubicTo(a, b, c) => contour && point(*a) && point(*b) && point(*c),
            PathCmd::ArcTo {
                rx,
                ry,
                x_rotation_deg,
                to,
                ..
            } => contour && finite(&[*rx, *ry, *x_rotation_deg]) && point(*to),
            PathCmd::Arc {
                center,
                rx,
                ry,
                x_rotation_deg,
                start_deg,
                sweep_deg,
            } => {
                contour = true;
                point(*center) && finite(&[*rx, *ry, *x_rotation_deg, *start_deg, *sweep_deg])
            }
            PathCmd::Close => {
                let valid = contour;
                contour = false;
                valid
            }
        };
        if !valid {
            return false;
        }
    }
    true
}

/// Dimensions and byte count agree, edges within 16,384 px (the asset
/// layer's decode limit). There is no byte cap beyond that: a photo the
/// caller decoded must not fail the whole frame.
fn image_is_valid(value: &Pixels) -> Result<(), RenderError> {
    let bytes = usize::try_from(value.width)
        .ok()
        .and_then(|w| {
            usize::try_from(value.height)
                .ok()
                .and_then(|h| w.checked_mul(h))
        })
        .and_then(|p| p.checked_mul(4));
    if value.width == 0
        || value.height == 0
        || value.width > 16_384
        || value.height > 16_384
        || bytes.is_none_or(|n| n != value.rgba.len())
    {
        return Err(RenderError::InvalidImage);
    }
    Ok(())
}

/// The kind of an open group.
#[derive(Clone, Copy, PartialEq)]
enum Open {
    Clip,
    Transform,
    Group,
}

fn open(groups: &mut Vec<Open>, group: Open) -> Result<(), RenderError> {
    groups.push(group);
    if groups.len() > MAX_SCENE_DEPTH {
        return Err(RenderError::UnbalancedClip);
    }
    Ok(())
}

fn close(groups: &mut Vec<Open>, group: Open) -> Result<bool, RenderError> {
    match groups.pop() {
        Some(open) if open == group => Ok(true),
        _ => Err(RenderError::UnbalancedClip),
    }
}

/// Checks `scene` whole: every command valid, groups balanced and nested.
pub(crate) fn scene(scene: &RenderScene, fonts: &mut Fonts) -> Result<(), RenderError> {
    let mut groups: Vec<Open> = Vec::new();
    for command in &scene.commands {
        let valid = match command {
            primitive_commands!() => validate_primitive(command)?,
            content_commands!() => validate_content(command, fonts)?,
            group_commands!() => validate_group(command, &mut groups)?,
        };
        if !valid {
            return Err(RenderError::InvalidGeometry);
        }
    }
    if !groups.is_empty() {
        return Err(RenderError::UnbalancedClip);
    }
    Ok(())
}

/// Validate primitive commands without changing their replay order.
fn validate_primitive(command: &RenderCmd) -> Result<bool, RenderError> {
    Ok(match command {
        RenderCmd::Rect {
            rect: bounds,
            fill: paint,
        } => rect(*bounds) && fill(paint)?,
        RenderCmd::RoundedRect {
            rect: bounds,
            radius,
            fill: paint,
        } => rect(*bounds) && stroke(*radius) && fill(paint)?,
        RenderCmd::RoundedRectCorners {
            rect: bounds,
            radii,
            fill: paint,
        } => rect(*bounds) && radii.iter().all(|radius| stroke(*radius)) && fill(paint)?,
        RenderCmd::StrokeRect {
            rect: bounds,
            width,
            color: paint,
            radius,
        } => rect(*bounds) && stroke(*width) && stroke(*radius) && color(*paint),
        RenderCmd::StrokeRoundedRectCorners {
            rect: bounds,
            radii,
            width,
            color: paint,
        } => {
            rect(*bounds)
                && radii.iter().all(|radius| stroke(*radius))
                && stroke(*width)
                && color(*paint)
        }
        RenderCmd::Shadow {
            rect: bounds,
            radii,
            std_dev,
            color: paint,
        } => {
            rect(*bounds)
                && radii.iter().all(|radius| stroke(*radius))
                && std_dev.is_finite()
                && *std_dev > 0.0
                && color(*paint)
        }
        RenderCmd::Line {
            from,
            to,
            width,
            color: paint,
        } => point(*from) && point(*to) && stroke(*width) && color(*paint),
        RenderCmd::Path {
            cmds,
            fill,
            stroke: line,
        } => {
            path(cmds)
                && fill.is_none_or(color)
                && line.is_none_or(|(width, paint)| stroke(width) && color(paint))
        }
        content_commands!() | group_commands!() => {
            unreachable!("invariant: the dispatch selects primitive commands")
        }
    })
}

/// Validate a glyph run's outline stroke.
fn glyph_stroke(value: &GlyphStroke) -> Result<bool, RenderError> {
    Ok(line_style(&value.stroke) && fill(&value.paint)?)
}

/// Validate content commands without changing their replay order.
fn validate_content(command: &RenderCmd, fonts: &mut Fonts) -> Result<bool, RenderError> {
    Ok(match command {
        RenderCmd::Glyphs {
            run,
            origin,
            fill: paint,
            stroke: outline,
            order: _,
        } => {
            let count = fonts.glyphs(run.font.as_shared(), run.font_index)?;
            if !run.font_size.is_finite()
                || run.font_size <= 0.0
                || run.glyphs.iter().any(|glyph| {
                    !finite(&[
                        glyph.x,
                        glyph.y,
                        glyph.advance,
                        glyph.x + origin.x,
                        glyph.y + origin.y,
                    ]) || glyph.id >= count
                })
                || run
                    .normalized_coords
                    .iter()
                    .any(|coord| !coord.is_finite() || !(-1.0..=1.0).contains(coord))
            {
                return Err(RenderError::InvalidFont);
            }
            let paint_valid = match paint {
                Some(value) => fill(value)?,
                None => true,
            };
            let outline_valid = match outline {
                Some(value) => glyph_stroke(value)?,
                None => true,
            };
            point(*origin) && color(run.color) && paint_valid && outline_valid
        }
        RenderCmd::Image {
            image: value,
            rect: bounds,
            opacity,
        } => {
            image_is_valid(value)?;
            rect(*bounds) && unit(*opacity)
        }
        RenderCmd::ImageSampled {
            image: value,
            rect: bounds,
            source,
            opacity,
            ..
        } => {
            if let Some(source) = source
                && !rect(*source)
            {
                return Err(RenderError::InvalidGeometry);
            }
            image_is_valid(value)?;
            rect(*bounds) && source.is_none_or(rect) && unit(*opacity)
        }
        primitive_commands!() | group_commands!() => {
            unreachable!("invariant: the dispatch selects content commands")
        }
    })
}

/// Validate group commands without changing their replay order.
fn validate_group(command: &RenderCmd, groups: &mut Vec<Open>) -> Result<bool, RenderError> {
    Ok(match command {
        RenderCmd::FillShape { shape: s, paint } => shape(s) && fill(paint)?,
        RenderCmd::StrokeShape {
            shape: s,
            stroke: style,
            paint,
        } => shape(s) && line_style(style) && fill(paint)?,
        RenderCmd::BoxShadow(value) => box_shadow(value),
        RenderCmd::PushClip(bounds) => {
            open(groups, Open::Clip)?;
            rect(*bounds)
        }
        RenderCmd::PushTransform(value) => {
            open(groups, Open::Transform)?;
            transform(value)
        }
        RenderCmd::PushGroup(value) => {
            open(groups, Open::Group)?;
            group(value)
        }
        RenderCmd::PopClip => close(groups, Open::Clip)?,
        RenderCmd::PopTransform => close(groups, Open::Transform)?,
        RenderCmd::PopGroup => close(groups, Open::Group)?,
        primitive_commands!() | content_commands!() => {
            unreachable!("invariant: the dispatch selects group commands")
        }
    })
}
