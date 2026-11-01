//! Role: Validate scene data before rasterization.
//! No invalid float, out-of-range
//! value or unmatched group enters the rasterizers.
//!
//! A scene is checked whole before anything is drawn ([`scene`]), so an
//! invalid one draws nothing and leaves the renderer usable: every number
//! finite, color components and gradient stop offsets in `0..=1` (stops
//! not decreasing), sizes and widths not negative, groups (clips,
//! transforms, layers) properly nested, each pop closing the innermost
//! group of its kind, at most [`MAX_SCENE_DEPTH`] open at once, blurs of at most
//! `MAX_BLUR` standard deviations, at most `MAX_DASHES` dash lengths, glyph
//! runs whose font skrifa reads and whose glyph ids it has. Images are
//! decoded straight-alpha RGBA8 whose bytes match their dimensions, edges
//! within 16,384 px ([`image`]); this crate decodes no encoded image.
//!
//! The frame budget (each side 16,384 physical px, 16,777,216 pixels: 64
//! MiB of RGBA8) is decided only here: [`dimensions`] says whether a frame
//! is drawn, and [`raster_plan`] picks, from a window's physical size and
//! scale, the logical extent (never rasterizing beyond the window) and the
//! raster scale, reduced for a window beyond the budget (its frame is then
//! enlarged to the window when it is presented).

use crate::error::RenderError;
mod fonts;
pub(crate) use fonts::Fonts;
use ui_types::{
    Backdrop, BoxShadow, ColorMatrix, DrawCommand, Gradient, GradientShape, ImageData, Layer,
    LayerFilter, Paint, PathCmd, PatternSource, Point, Rect, Rgba, RoundedRect, Scene, Shape, Size,
    Stroke, Transform,
};

use crate::limits::{MAX_BLUR, MAX_DASHES, MAX_PIXELS, MAX_SCENE_DEPTH, MAX_SIDE};

/// The pixel dimensions of a frame of logical `size` at `scale`, or why the
/// renderers do not draw it: the one place the frame budget is decided.
pub fn dimensions(size: Size, scale: f32) -> Result<(u16, u16), RenderError> {
    if !scale.is_finite()
        || scale <= 0.0
        || !size.width.is_finite()
        || !size.height.is_finite()
        || size.width <= 0.0
        || size.height <= 0.0
    {
        return Err(RenderError::InvalidSize);
    }
    let width = (size.width * scale).ceil();
    let height = (size.height * scale).ceil();
    if width < 1.0
        || height < 1.0
        || width > MAX_SIDE
        || height > MAX_SIDE
        || width * height > MAX_PIXELS
    {
        return Err(RenderError::InvalidSize);
    }
    Ok((width as u16, height as u16))
}

/// The logical extent and the physical scale to rasterize a `width` ×
/// `height` physical-pixel window at, whose display scale is `scale`: the
/// extent never rasterizes beyond the window, and a window beyond the
/// frame budget is drawn at a reduced scale (the frame is enlarged to the
/// window).
pub fn raster_plan(width: u32, height: u32, scale: f32) -> (Size, f32) {
    let size = raster_size(width, height, scale);
    let fits = |scale: f32| {
        let width = (size.width * scale).ceil();
        let height = (size.height * scale).ceil();
        width <= MAX_SIDE && height <= MAX_SIDE && width * height <= MAX_PIXELS
    };
    if !scale.is_finite() || scale <= 0.0 || fits(scale) {
        return (size, scale);
    }
    let (w, h) = (width as f32, height as f32);
    let mut reduced = scale
        * (MAX_SIDE / w)
            .min(MAX_SIDE / h)
            .min((MAX_PIXELS / (w * h)).sqrt());
    while !fits(reduced) {
        reduced *= 0.999;
    }
    (size, reduced)
}

/// Logical extent whose rasterization never exceeds the physical surface.
fn raster_size(width: u32, height: u32, scale: f32) -> Size {
    let extent = |physical: u32| {
        let mut logical = physical as f32 / scale;
        while logical * scale > physical as f32 {
            logical = logical.next_down();
        }
        logical
    };
    Size {
        width: extent(width),
        height: extent(height),
    }
}

fn finite(values: &[f32]) -> bool {
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
fn stroke(width: f32) -> bool {
    width.is_finite() && width >= 0.0
}
fn stops(stops: &[(f32, Rgba)]) -> bool {
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
        && (value.dash.is_empty() || value.dash.iter().sum::<f32>() > 0.0)
}
fn blur(std_dev: f32) -> bool {
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
pub(crate) fn image(value: &ImageData) -> Result<(), RenderError> {
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

pub(crate) fn scene(scene: &Scene, fonts: &mut Fonts) -> Result<SceneInfo, RenderError> {
    let mut groups: Vec<Group> = Vec::new();
    let mut backdrops = false;
    for command in &scene.commands {
        let valid = match command {
            DrawCommand::Rect {
                rect: bounds,
                fill: paint,
            } => rect(*bounds) && fill(paint)?,
            DrawCommand::RoundedRect {
                rect: bounds,
                radius,
                fill: paint,
            } => rect(*bounds) && stroke(*radius) && fill(paint)?,
            DrawCommand::RoundedRectCorners {
                rect: bounds,
                radii,
                fill: paint,
            } => rect(*bounds) && radii.iter().all(|radius| stroke(*radius)) && fill(paint)?,
            DrawCommand::StrokeRect {
                rect: bounds,
                width,
                color: paint,
                radius,
            } => rect(*bounds) && stroke(*width) && stroke(*radius) && color(*paint),
            DrawCommand::StrokeRoundedRectCorners {
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
            DrawCommand::Shadow {
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
            DrawCommand::Line {
                from,
                to,
                width,
                color: paint,
            } => point(*from) && point(*to) && stroke(*width) && color(*paint),
            DrawCommand::Path {
                cmds,
                fill,
                stroke: line,
            } => {
                path(cmds)
                    && fill.is_none_or(color)
                    && line.is_none_or(|(width, paint)| stroke(width) && color(paint))
            }
            DrawCommand::Glyphs {
                run,
                origin,
                color: override_color,
            } => {
                let count = fonts.glyphs(&run.font, run.font_index)?;
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
            DrawCommand::Image {
                image: value,
                rect: bounds,
                opacity,
                color_matrix,
            }
            | DrawCommand::ImageSampled {
                image: value,
                rect: bounds,
                opacity,
                color_matrix,
                ..
            } => {
                if let DrawCommand::ImageSampled {
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
            DrawCommand::External {
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
            DrawCommand::FillShape { shape: s, paint } => shape(s) && fill(paint)?,
            DrawCommand::StrokeShape {
                shape: s,
                stroke: style,
                paint,
            } => shape(s) && line_style(style) && fill(paint)?,
            DrawCommand::BoxShadow(value) => box_shadow(value),
            DrawCommand::Backdrop(value) => {
                backdrops = true;
                backdrop(value)
            }
            DrawCommand::PushClip(bounds) => {
                open(&mut groups, Group::Clip)?;
                rect(*bounds)
            }
            DrawCommand::PushTransform(value) => {
                open(&mut groups, Group::Transform)?;
                transform(value)
            }
            DrawCommand::PushLayer(value) => {
                open(&mut groups, Group::Layer)?;
                layer(value)?
            }
            DrawCommand::PopClip => close(&mut groups, Group::Clip)?,
            DrawCommand::PopTransform => close(&mut groups, Group::Transform)?,
            DrawCommand::PopLayer => close(&mut groups, Group::Layer)?,
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
