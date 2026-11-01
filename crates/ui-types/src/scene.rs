//! Role: Portable display lists and shaped glyphs.
//! Frame bytes are premultiplied RGBA8.

use crate::gpu_interop::ExternalFrame;
use crate::image::ImageData;
use crate::{
    Backdrop, BoxShadow, ColorMatrix, Layer, Paint, PathCmd, Point, Rect, Rgba, Shape, Size,
    Stroke, TextDecorationRect, TextHighlight, Transform,
};
use std::{ops::Range, sync::Arc};

/// Positioned glyph, usable on any thread.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Glyph {
    /// Font-specific glyph identifier.
    pub id: u32,
    /// Horizontal baseline position.
    pub x: f32,
    /// Vertical baseline position.
    pub y: f32,
    /// Logical advance.
    pub advance: f32,
}

/// A shaped run retaining its font bytes; usable on any thread.
/// The shared bytes remain valid after the text engine or its caches are
/// dropped. Glyph positions use logical pixels from the text layout origin;
/// a scene places the immutable run with its glyph command's origin.
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphRun {
    /// Complete font file bytes.
    pub font: Arc<[u8]>,
    /// Font face index.
    pub font_index: u32,
    /// Logical font size.
    pub font_size: f32,
    /// Glyph positions relative to the run's layout origin.
    pub glyphs: Vec<Glyph>,
    /// Straight-alpha text color.
    pub color: Rgba,
    /// Normalized font variation coordinates.
    pub normalized_coords: Vec<f32>,
}

/// Owned shaping snapshot; usable on any thread, independent of the
/// engine's lifetime. All positions are logical pixels from its layout
/// origin, before any scene transform or device raster scale.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapedText {
    /// Logical measured extent.
    pub size: Size,
    /// Positioned glyph runs.
    pub runs: Vec<Arc<GlyphRun>>,
    /// Caret rectangles at Unicode scalar offsets, following shaped clusters.
    pub carets: Vec<TextCaret>,
    /// Sorted, unique shaping-cluster boundaries in Unicode scalar offsets.
    /// Includes zero and the final scalar offset; independent of caret positions.
    /// An empty list means every grapheme boundary is a stop.
    pub cluster_boundaries: Vec<u32>,
    /// The first line's baseline, from the top.
    pub baseline: f32,
    /// The lines drawn with the text (underlines and overlines before the
    /// glyphs, line-throughs after them).
    pub decorations: Vec<TextDecorationRect>,
    /// The bands of the highlighted spans, behind the glyphs.
    pub highlights: Vec<TextHighlight>,
    /// The lines shown, top to bottom (empty for empty text). A line
    /// clamp leaves out the lines it hides.
    pub lines: Vec<TextLine>,
}

/// The geometry of one shown line of a [`ShapedText`], in logical pixels
/// from its layout origin; usable on any thread.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextLine {
    /// The line's text as UTF-8 byte offsets in the string given to the
    /// engine, before any [`crate::TextTransform`]. It includes the
    /// trailing white space and the line break that end the line, and stays
    /// whole when the line is cut.
    pub range: Range<usize>,
    /// Where the line's content lies: its advance from the aligned start,
    /// without the trailing white space (a cut line: the kept text and the
    /// ellipsis), and the line box's height. The line box tiles the
    /// paragraph from top to bottom.
    pub rect: Rect,
    /// The baseline, from the top of the paragraph.
    pub baseline: f32,
    /// The distance from the baseline up to the top of the tallest font on
    /// the line.
    pub ascent: f32,
    /// The distance from the baseline down to the bottom of the deepest font
    /// on the line.
    pub descent: f32,
    /// The distance from the baseline up to the top of capital letters
    /// (the largest of the line's fonts); 0 for a line without glyphs or a
    /// font that gives none.
    pub cap_height: f32,
    /// The distance from the baseline up to the top of short lowercase
    /// letters (the largest of the line's fonts); 0 for a line without
    /// glyphs or a font that gives none.
    pub x_height: f32,
    /// Whether an ellipsis cut the line.
    pub cut: bool,
}

/// A cursor position in shaped text; usable on any thread.
///
/// Combining marks and ligatures can share a cluster and therefore a rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextCaret {
    /// Unicode scalar offset after [`crate::TextTransform`], including the
    /// final boundary. This is neither a UTF-8 byte nor a grapheme index;
    /// case conversion can change the number of scalars.
    pub offset: u32,
    /// Logical caret bounds relative to the text layout origin.
    pub rect: Rect,
}

/// Portable drawing operation, usable on any thread.
///
/// Coordinates are logical pixels, mapped by the open transforms. Groups
/// nest: [`DrawCommand::PushClip`], [`DrawCommand::PushTransform`] and
/// [`DrawCommand::PushLayer`] each open a group that the next pop of the
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
pub enum DrawCommand {
    /// An image with a source rectangle in natural pixels and optional nearest-neighbor sampling.
    ImageSampled {
        /// Pixels.
        image: ImageData,
        /// Destination.
        rect: Rect,
        /// Source pixels; None draws the whole image.
        source: Option<Rect>,
        /// Group opacity.
        opacity: f32,
        /// Pixel color transform.
        color_matrix: Option<ColorMatrix>,
        /// Disable smoothing.
        pixelated: bool,
    },
    /// A frame of an image an external render engine shares with the
    /// window ([`crate::SharedTextures`]), drawn as
    /// [`DrawCommand::ImageSampled`] draws its pixels (no color matrix:
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
        opacity: f32,
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
        radius: f32,
        /// Paint.
        fill: Paint,
    },
    /// Filled bounds with independent corner radii.
    RoundedRectCorners {
        /// Bounds.
        rect: Rect,
        /// Top-left, top-right, bottom-right, bottom-left radii.
        radii: [f32; 4],
        /// Paint.
        fill: Paint,
    },
    /// Stroked bounds with uniform corner radius.
    StrokeRect {
        /// Bounds.
        rect: Rect,
        /// Stroke width.
        width: f32,
        /// Color.
        color: Rgba,
        /// Radius.
        radius: f32,
    },
    /// Stroked bounds with independent corner radii.
    StrokeRoundedRectCorners {
        /// Bounds.
        rect: Rect,
        /// Stroke width.
        width: f32,
        /// Color.
        color: Rgba,
        /// Top-left, top-right, bottom-right, bottom-left radii.
        radii: [f32; 4],
    },
    /// Stroked line. Renderers move a horizontal or vertical line whose
    /// width is a whole number of device pixels onto the pixel grid.
    Line {
        /// Start.
        from: Point,
        /// End.
        to: Point,
        /// Stroke width.
        width: f32,
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
        stroke: Option<(f32, Rgba)>,
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
        radii: [f32; 4],
        /// Standard deviation of the gaussian blur.
        std_dev: f32,
        /// Color.
        color: Rgba,
    },
    /// A run of already shaped glyphs.
    Glyphs {
        /// Immutable layout-local glyph payload.
        run: Arc<GlyphRun>,
        /// Logical translation before the active scene transform.
        origin: Point,
        /// Override the shaped color; `None` preserves it.
        color: Option<Rgba>,
    },
    /// An image stretched to bounds.
    Image {
        /// Pixels.
        image: ImageData,
        /// Destination.
        rect: Rect,
        /// Alpha multiplier in `[0, 1]`, applied by the renderer.
        opacity: f32,
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
    /// [`DrawCommand::PopTransform`] by this transform, then by the
    /// enclosing ones. Glyphs under a rotation, skew or non-uniform scale
    /// are drawn from unhinted outlines.
    PushTransform(Transform),
    /// Close the innermost transform.
    PopTransform,
    /// Draw the commands up to the matching [`DrawCommand::PopLayer`] as
    /// one group: filtered, clipped, masked, faded and blended as the
    /// [`Layer`] says.
    PushLayer(Layer),
    /// Close the innermost layer.
    PopLayer,
    /// What the commands before this one drew behind a shape, blurred and
    /// color-filtered (CSS `backdrop-filter`); see [`Backdrop`].
    Backdrop(Backdrop),
}

/// How many standard deviations beyond its rect a [`DrawCommand::Shadow`]
/// can be drawn (renderers cut the gaussian off within it, where it is
/// below 0.1%).
pub const SHADOW_REACH: f32 = 3.6;

/// Owned drawing snapshot, usable on any thread. Commands are in logical
/// window pixels before their nested transforms and the frame's raster
/// scale. Source image rectangles explicitly use image pixels instead.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    /// Drawing operations in paint order.
    pub commands: Vec<DrawCommand>,
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
