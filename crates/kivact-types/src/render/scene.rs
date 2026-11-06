//! Role: Portable display lists, CPU frames and external GPU frames.
//! Frame bytes are premultiplied RGBA8.

use super::color::ColorMatrix;
use super::effect::{Backdrop, BoxShadow, Layer};
use super::paint::Paint;
use super::shaped::GlyphRunSnapshot;
use crate::color::Rgba;
use crate::geometry::Transform;
use crate::geometry::{Point, Rect};
use crate::gpu::ExternalFrame;
use crate::scene::Pixels;
use crate::scene::Stroke;
use crate::scene::{PathCmd, Shape};

/// Portable drawing operation, usable on any thread.
///
/// Coordinates are logical pixels, mapped by the open transforms. Groups
/// nest: [`RenderCmd::PushClip`], [`RenderCmd::PushTransform`] and
/// [`RenderCmd::PushLayer`] each open a group that the next pop of the
/// same kind closes, and every pop must close the innermost open group (at
/// most 256 open at once); groups that cross are an invalid scene.
/// `arvane-render`'s `draw.rs` gives every command its meaning, the same on
/// the CPU and the GPU.
///
/// A pushed transform applies inside the enclosing ones and maps
/// everything up to its pop: rectangles, paths, glyphs, images, clips,
/// shadows, layers and the paints' own coordinates. Under a transform that
/// flattens everything to a line or a point (determinant 0), and inside a
/// layer of opacity 0, nothing is drawn up to the matching pop. Every
/// value is Arvane's own type: no rasterizer vocabulary (vello, peniko,
/// kurbo) appears here.
#[derive(Clone, Debug, PartialEq)]
pub enum RenderCmd {
    /// An image with a source rectangle in natural pixels and optional nearest-neighbor sampling.
    ImageSampled {
        /// Pixels.
        image: Pixels,
        /// Destination.
        rect: Rect,
        /// Source pixels; None draws the whole image.
        source: Option<Rect>,
        /// Group opacity.
        opacity: f64,
        /// Pixel color transform.
        color_matrix: Option<ColorMatrix>,
        /// Disable smoothing.
        pixelated: bool,
    },
    /// A frame of an image an external render engine shares with the
    /// window ([`crate::SharedTextures`]), drawn as
    /// [`RenderCmd::ImageSampled`] draws its pixels (no color matrix:
    /// the frame is already premultiplied RGBA8). Only the GPU renderer
    /// draws it, and only in the window the image was imported into, once
    /// the frame is ready; otherwise nothing is drawn, and the scene is
    /// still valid (the CPU renderer never draws it).
    External {
        /// The frame.
        frame: ExternalFrame,
        /// Destination.
        rect: Rect,
        /// Source pixels of the frame; None draws the whole frame.
        source: Option<Rect>,
        /// Group opacity.
        opacity: f64,
        /// Disable smoothing.
        pixelated: bool,
    },
    /// Filled rectangular bounds.
    Rect {
        /// Bounds.
        rect: Rect,
        /// Paint.
        fill: Paint,
    },
    /// Filled bounds with uniform corner radius.
    RoundedRect {
        /// Bounds.
        rect: Rect,
        /// Radius.
        radius: f64,
        /// Paint.
        fill: Paint,
    },
    /// Filled bounds with independent corner radii.
    RoundedRectCorners {
        /// Bounds.
        rect: Rect,
        /// Top-left, top-right, bottom-right, bottom-left radii.
        radii: [f64; 4],
        /// Paint.
        fill: Paint,
    },
    /// Stroked bounds with uniform corner radius.
    StrokeRect {
        /// Bounds.
        rect: Rect,
        /// Stroke width.
        width: f64,
        /// Color.
        color: Rgba,
        /// Radius.
        radius: f64,
    },
    /// Stroked bounds with independent corner radii.
    StrokeRoundedRectCorners {
        /// Bounds.
        rect: Rect,
        /// Stroke width.
        width: f64,
        /// Color.
        color: Rgba,
        /// Top-left, top-right, bottom-right, bottom-left radii.
        radii: [f64; 4],
    },
    /// Stroked line. Renderers move a horizontal or vertical line whose
    /// width is a whole number of device pixels onto the pixel grid.
    Line {
        /// Start.
        from: Point,
        /// End.
        to: Point,
        /// Stroke width.
        width: f64,
        /// Color.
        color: Rgba,
    },
    /// Filled or stroked path.
    Path {
        /// Path geometry.
        cmds: Vec<PathCmd>,
        /// Optional fill color.
        fill: Option<Rgba>,
        /// Optional stroke width and color.
        stroke: Option<(f64, Rgba)>,
    },
    /// A box shadow: `rect` with rounded corners, blurred by a gaussian of
    /// standard deviation `std_dev` (more than zero). Renderers draw it
    /// within `rect` grown by [`SHADOW_REACH`] × `std_dev` on each side.
    /// Each corner's radius shapes the quarter of the shadow around it
    /// (split at the center of `rect`).
    Shadow {
        /// The shadow's shape before blurring.
        rect: Rect,
        /// Top-left, top-right, bottom-right, bottom-left radii.
        radii: [f64; 4],
        /// Standard deviation of the gaussian blur.
        std_dev: f64,
        /// Color.
        color: Rgba,
    },
    /// A run of already shaped glyphs.
    Glyphs {
        /// Immutable layout-local glyph payload.
        run: GlyphRunSnapshot,
        /// Logical translation before the active scene transform.
        origin: Point,
        /// Override the shaped color; `None` preserves it.
        color: Option<Rgba>,
    },
    /// An image stretched to bounds.
    Image {
        /// Pixels.
        image: Pixels,
        /// Destination.
        rect: Rect,
        /// Alpha multiplier in `[0, 1]`, applied by the renderer.
        opacity: f64,
        /// Color matrix applied to the pixels (CSS `filter: grayscale(1)`
        /// on an image); the renderer converts the pixels once and keeps
        /// the result while frames draw the same image with the same
        /// matrix.
        color_matrix: Option<ColorMatrix>,
    },
    /// Intersect current clip with these bounds.
    PushClip(Rect),
    /// Restore the preceding clip; lists must be balanced.
    PopClip,
    /// A shape filled with a paint.
    FillShape {
        /// Shape.
        shape: Shape,
        /// Paint.
        paint: Paint,
    },
    /// A shape's outline stroked with a paint. A zero-length contour (or
    /// dash) with round or square caps draws a dot.
    StrokeShape {
        /// Shape (polygons and rounded rects are closed; a path's open
        /// contours stay open).
        shape: Shape,
        /// Line style.
        stroke: Stroke,
        /// Paint.
        paint: Paint,
    },
    /// A CSS box shadow, outer or inset, of a box with elliptical corners.
    BoxShadow(BoxShadow),
    /// Map the coordinates of every command up to the matching
    /// [`RenderCmd::PopTransform`] by this transform, then by the
    /// enclosing ones. Glyphs under a rotation, skew or non-uniform scale
    /// are drawn from unhinted outlines.
    PushTransform(Transform),
    /// Close the innermost transform.
    PopTransform,
    /// Draw the commands up to the matching [`RenderCmd::PopLayer`] as
    /// one group: filtered, clipped, masked, faded and blended as the
    /// [`Layer`] says.
    PushLayer(Layer),
    /// Close the innermost layer.
    PopLayer,
    /// What the commands before this one drew behind a shape, blurred and
    /// color-filtered (CSS `backdrop-filter`); see [`Backdrop`].
    Backdrop(Backdrop),
}

/// How many standard deviations beyond its rect a [`RenderCmd::Shadow`]
/// can be drawn (renderers cut the gaussian off within it, where it is
/// below 0.1%).
pub const SHADOW_REACH: f64 = 3.6;

/// Owned drawing snapshot, usable on any thread. Commands are in logical
/// window pixels before their nested transforms and the frame's raster
/// scale. Source image rectangles explicitly use image pixels instead.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderScene {
    /// Drawing operations in paint order.
    pub commands: Vec<RenderCmd>,
}

/// Owned CPU raster output; usable on any thread. Its dimensions are
/// physical raster pixels, which may be fewer than the window's physical
/// pixels when the frame budget reduces its raster scale.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CpuFrame {
    /// Physical width.
    pub width: u32,
    /// Physical height.
    pub height: u32,
    /// Exactly width * height * 4 premultiplied RGBA8 bytes, row-major
    /// from the top left, with no row padding.
    pub rgba: Vec<u8>,
}

impl CpuFrame {
    /// Composite over white and encode softbuffer's 0x00RRGGBB pixels.
    pub fn to_softbuffer(&self) -> Vec<u32> {
        self.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| {
                let white = 255 - u32::from(p[3]);
                ((u32::from(p[0]) + white).min(255) << 16)
                    | ((u32::from(p[1]) + white).min(255) << 8)
                    | (u32::from(p[2]) + white).min(255)
            })
            .collect()
    }
}
