//! Role: Checks every command of a scene before anything is drawn.

use super::Fonts;
use crate::error::RenderError;
use crate::limits::{MAX_BLUR, MAX_DASHES, MAX_SCENE_DEPTH};
use kivact_types::geometry::Transform;
use kivact_types::render::{
    Backdrop, BoxShadow, ColorMatrix, Gradient, GradientShape, Layer, LayerFilter, Paint,
    PatternSource, RenderCmd, RenderScene,
};
use kivact_types::scene::{PathCmd, Pixels, RoundedRect, Shape};
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
        RenderCmd::Glyphs { .. }
            | RenderCmd::Image { .. }
            | RenderCmd::ImageSampled { .. }
            | RenderCmd::External { .. }
    };
}
macro_rules! group_commands {
    () => {
        RenderCmd::FillShape { .. }
            | RenderCmd::StrokeShape { .. }
            | RenderCmd::BoxShadow(_)
            | RenderCmd::Backdrop(_)
            | RenderCmd::PushClip(_)
            | RenderCmd::PushTransform(_)
            | RenderCmd::PushLayer(_)
            | RenderCmd::PopClip
            | RenderCmd::PopTransform
            | RenderCmd::PopLayer
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

fn stops(stops: &[(f64, Rgba)]) -> bool {
    !stops.is_empty()
        && stops.iter().all(|(offset, value)| {
            offset.is_finite() && (0.0..=1.0).contains(offset) && color(*value)
        })
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

fn matrix(value: &ColorMatrix) -> bool {
    finite(&value.0)
}

fn fill(value: &Paint) -> Result<bool, RenderError> {
    Ok(match value {
        Paint::Solid(value) => color(*value),
        Paint::LinearGradient {
            angle_deg,
            stops: s,
        } => angle_deg.is_finite() && stops(s),
        Paint::Gradient(g) => gradient(g),
        Paint::Pattern(p) => {
            let source = match &p.source {
                PatternSource::Image {
                    image: value,
                    color_matrix,
                } => {
                    image(value)?;
                    color_matrix.as_ref().is_none_or(matrix)
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

fn layer(value: &Layer) -> Result<bool, RenderError> {
    let filter = match value.filter {
        None => true,
        Some(LayerFilter::Blur { std_dev }) => blur(std_dev),
        Some(LayerFilter::DropShadow {
            offset,
            std_dev,
            color: c,
        }) => point(offset) && blur(std_dev) && color(c),
    };
    let mask = match &value.mask {
        None => true,
        Some(mask) => rect(mask.rect) && fill(&mask.paint)?,
    };
    Ok(value.clip.as_ref().is_none_or(shape)
        && value.opacity.is_finite()
        && (0.0..=1.0).contains(&value.opacity)
        && filter
        && mask)
}

fn box_shadow(value: &BoxShadow) -> bool {
    rounded(&value.shape)
        && point(value.offset)
        && value.spread.is_finite()
        && blur(value.std_dev)
        && color(value.color)
}

fn backdrop(value: &Backdrop) -> bool {
    shape(&value.shape) && blur(value.std_dev) && value.color_matrix.as_ref().is_none_or(matrix)
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
/// layer's decode limit). There is no byte cap beyond that: the 64 MiB cap
/// mirrored the removed decoder's allocation limit, and
/// a photo the caller decoded must not fail the whole frame.
pub(crate) fn image(value: &Pixels) -> Result<(), RenderError> {
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

/// What drawing a valid scene needs to know.
pub(crate) struct SceneInfo {
    /// Whether it has a backdrop.
    pub(crate) backdrops: bool,
}

/// The kind of an open group.
#[derive(Clone, Copy, PartialEq)]
enum Group {
    Clip,
    Transform,
    Layer,
}

fn open(groups: &mut Vec<Group>, group: Group) -> Result<(), RenderError> {
    groups.push(group);
    if groups.len() > MAX_SCENE_DEPTH {
        return Err(RenderError::UnbalancedClip);
    }
    Ok(())
}

fn close(groups: &mut Vec<Group>, group: Group) -> Result<bool, RenderError> {
    match groups.pop() {
        Some(open) if open == group => Ok(true),
        _ => Err(RenderError::UnbalancedClip),
    }
}

pub(crate) fn scene(scene: &RenderScene, fonts: &mut Fonts) -> Result<SceneInfo, RenderError> {
    let mut groups: Vec<Group> = Vec::new();
    let mut backdrops = false;
    for command in &scene.commands {
        let valid = match command {
            primitive_commands!() => validate_primitive(command)?,
            content_commands!() => validate_content(command, fonts)?,
            group_commands!() => validate_group(command, &mut groups, &mut backdrops)?,
        };
        if !valid {
            return Err(RenderError::InvalidGeometry);
        }
    }
    if !groups.is_empty() {
        return Err(RenderError::UnbalancedClip);
    }
    Ok(SceneInfo { backdrops })
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

/// Validate content commands without changing their replay order.
fn validate_content(command: &RenderCmd, fonts: &mut Fonts) -> Result<bool, RenderError> {
    Ok(match command {
        RenderCmd::Glyphs {
            run,
            origin,
            color: override_color,
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
            point(*origin) && override_color.is_none_or(color) && color(run.color)
        }
        RenderCmd::Image {
            image: value,
            rect: bounds,
            opacity,
            color_matrix,
        }
        | RenderCmd::ImageSampled {
            image: value,
            rect: bounds,
            opacity,
            color_matrix,
            ..
        } => {
            if let RenderCmd::ImageSampled {
                source: Some(source),
                ..
            } = command
                && !rect(*source)
            {
                return Err(RenderError::InvalidGeometry);
            }
            image(value)?;
            rect(*bounds)
                && opacity.is_finite()
                && (0.0..=1.0).contains(opacity)
                && color_matrix.as_ref().is_none_or(matrix)
        }
        // A frame that is not imported or not ready draws nothing; its
        // values are checked as an image's.
        RenderCmd::External {
            rect: bounds,
            source,
            opacity,
            ..
        } => {
            rect(*bounds)
                && source.is_none_or(rect)
                && opacity.is_finite()
                && (0.0..=1.0).contains(opacity)
        }
        primitive_commands!() | group_commands!() => {
            unreachable!("invariant: the dispatch selects content commands")
        }
    })
}

/// Validate group commands without changing their replay order.
fn validate_group(
    command: &RenderCmd,
    groups: &mut Vec<Group>,
    backdrops: &mut bool,
) -> Result<bool, RenderError> {
    Ok(match command {
        RenderCmd::FillShape { shape: s, paint } => shape(s) && fill(paint)?,
        RenderCmd::StrokeShape {
            shape: s,
            stroke: style,
            paint,
        } => shape(s) && line_style(style) && fill(paint)?,
        RenderCmd::BoxShadow(value) => box_shadow(value),
        RenderCmd::Backdrop(value) => {
            *backdrops = true;
            backdrop(value)
        }
        RenderCmd::PushClip(bounds) => {
            open(groups, Group::Clip)?;
            rect(*bounds)
        }
        RenderCmd::PushTransform(value) => {
            open(groups, Group::Transform)?;
            transform(value)
        }
        RenderCmd::PushLayer(value) => {
            open(groups, Group::Layer)?;
            layer(value)?
        }
        RenderCmd::PopClip => close(groups, Group::Clip)?,
        RenderCmd::PopTransform => close(groups, Group::Transform)?,
        RenderCmd::PopLayer => close(groups, Group::Layer)?,
        primitive_commands!() | content_commands!() => {
            unreachable!("invariant: the dispatch selects group commands")
        }
    })
}
