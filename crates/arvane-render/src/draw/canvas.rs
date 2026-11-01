//! Role: Canvas: the calls both rasterizers receive, and the glyph runs they draw.

use crate::paint::PatternFill;
use kivact_types::{ExternalFrame, ImageData};
use vello_common::filter_effects::Filter;
use vello_cpu::kurbo::{Affine, BezPath, Rect, Stroke};
use vello_cpu::peniko::{self, BlendMode, FontData};
use vello_cpu::{Glyph, PaintType};

/// What [`draw`] records into: a rasterizer's scene.
pub(crate) trait Canvas {
    /// The transform of what follows, to device pixels.
    fn set_transform(&mut self, transform: Affine);
    fn set_paint(&mut self, paint: PaintType);
    fn set_paint_transform(&mut self, transform: Affine);
    fn reset_paint_transform(&mut self);
    fn set_stroke(&mut self, stroke: Stroke);
    fn set_fill_rule(&mut self, rule: peniko::Fill);
    /// Only non-destructive modes (destination out for masks).
    fn set_blend_mode(&mut self, mode: BlendMode);
    fn fill_rect(&mut self, rect: &Rect);
    fn fill_path(&mut self, path: &BezPath);
    fn stroke_path(&mut self, path: &BezPath);
    fn fill_blurred_rounded_rect(&mut self, rect: &Rect, radius: f32, std_dev: f32);
    fn push_clip_layer(&mut self, path: &BezPath);
    /// Clip what follows without a layer (destination-out drawing inside
    /// still reaches the enclosing layer's pixels).
    fn push_clip_path(&mut self, path: &BezPath);
    fn pop_clip_path(&mut self);
    fn push_opacity_layer(&mut self, clip: &BezPath, opacity: f32);
    fn push_layer(
        &mut self,
        clip: Option<&BezPath>,
        blend: Option<BlendMode>,
        opacity: Option<f32>,
        filter: Option<Filter>,
    );
    fn pop_layer(&mut self);
    /// Fill `glyphs` of `run`'s font, hinted and emboldened as `run` says.
    fn fill_glyphs(&mut self, run: GlyphRun<'_>, glyphs: impl Iterator<Item = Glyph> + Clone);
    /// Draw `image`'s pixels stretched over `rect` (bilinear sampling).
    fn draw_image(&mut self, image: &ImageData, rect: Rect, pixelated: bool);
    /// The pixel size of the external frame `frame` when this frame can
    /// draw it (imported here and ready), else `None` (nothing is drawn).
    fn external_size(&self, frame: &ExternalFrame) -> Option<(u32, u32)>;
    /// Draw the external frame `frame` stretched over `rect`, as
    /// [`Canvas::draw_image`] draws an image.
    fn draw_external(&mut self, frame: &ExternalFrame, rect: Rect, pixelated: bool);
    /// Fill `path` with `pattern`'s pixels mapped by its transform,
    /// repeated along the axes it says (bilinear sampling; the edge pixels
    /// extend along the others); `visible` bounds the frame in the current
    /// coordinates.
    fn fill_pattern(&mut self, path: &BezPath, pattern: &PatternFill, visible: Rect);
}

/// How a run of glyphs is drawn.
pub(crate) struct GlyphRun<'a> {
    pub(crate) font: &'a FontData,
    pub(crate) font_size: f32,
    pub(crate) coords: &'a [i16],
    pub(crate) embolden: glifo::FontEmbolden,
    /// Hint vertically (only under a translation and a uniform scale).
    pub(crate) hint: bool,
    /// May come from a glyph atlas (only under a translation and a uniform
    /// scale; rotated glyphs are drawn from their outlines).
    #[cfg_attr(
        not(feature = "gpu"),
        expect(dead_code, reason = "the GPU's glyph atlas")
    )]
    pub(crate) cached: bool,
}

impl GlyphRun<'_> {
    /// `builder` set up for this run: vertical hinting and stem darkening
    /// (`text.rs`).
    pub(crate) fn apply<'a, B: glifo::GlyphRunBackend<'a>>(
        &self,
        builder: glifo::GlyphRunBuilder<'a, B>,
    ) -> glifo::GlyphRunBuilder<'a, B>
    where
        Self: 'a,
    {
        builder
            .font_size(self.font_size)
            .normalized_coords(self.coords)
            .hint(self.hint)
            .font_embolden(self.embolden)
    }
}
