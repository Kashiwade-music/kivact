//! Role: Canvas: the calls both rasterizers receive, and the glyph runs they draw.

use crate::paint::PatternFill;
use kivact_types::pixel::Pixels;
use vello_common::filter_effects::Filter;
use vello_cpu::kurbo::{Affine, BezPath, Rect, Stroke};
use vello_cpu::peniko::{self, BlendMode, FontData};
use vello_cpu::{Glyph, PaintType};

/// What [`draw`](super::draw) records into: a rasterizer's scene.
pub(crate) trait Canvas {
    /// The transform of what follows, to device pixels.
    fn set_transform(&mut self, transform: Affine);
    /// The paint of what follows.
    fn set_paint(&mut self, paint: PaintType);
    /// The paint's transform, for what follows until it is reset.
    fn set_paint_transform(&mut self, transform: Affine);
    /// Remove the paint transform.
    fn reset_paint_transform(&mut self);
    /// The stroke style of what follows.
    fn set_stroke(&mut self, stroke: Stroke);
    /// The fill rule of what follows.
    fn set_fill_rule(&mut self, rule: peniko::Fill);
    /// Only non-destructive modes, except destination out for box shadows.
    fn set_blend_mode(&mut self, mode: BlendMode);
    /// Fill a rectangle with the current paint.
    fn fill_rect(&mut self, rect: &Rect);
    /// Fill a path with the current paint.
    fn fill_path(&mut self, path: &BezPath);
    /// Stroke a path with the current paint and stroke.
    fn stroke_path(&mut self, path: &BezPath);
    /// Fill a rounded rectangle blurred by a gaussian of `std_dev`.
    fn fill_blurred_rounded_rect(&mut self, rect: &Rect, radius: f64, std_dev: f64);
    /// Clip what follows, in a layer popped by [`Canvas::pop_layer`].
    fn push_clip_layer(&mut self, path: &BezPath);
    /// Clip what follows without a layer (destination-out drawing inside
    /// still reaches the enclosing layer's pixels).
    fn push_clip_path(&mut self, path: &BezPath);
    /// Restore the clip pushed by [`Canvas::push_clip_path`].
    fn pop_clip_path(&mut self);
    /// A layer clipped to `clip` and faded by `opacity`.
    fn push_opacity_layer(&mut self, clip: &BezPath, opacity: f64);
    /// A layer with an optional clip, blend mode, opacity and filter.
    fn push_layer(
        &mut self,
        clip: Option<&BezPath>,
        blend: Option<BlendMode>,
        opacity: Option<f64>,
        filter: Option<Filter>,
    );
    /// Close the innermost layer.
    fn pop_layer(&mut self);
    /// Fill `glyphs` of `run`'s font, hinted and emboldened as `run` says.
    fn fill_glyphs(&mut self, run: GlyphRun<'_>, glyphs: impl Iterator<Item = Glyph> + Clone);
    /// Stroke the outlines of `glyphs` of `run`'s font with the current
    /// stroke and paint.
    fn stroke_glyphs(&mut self, run: GlyphRun<'_>, glyphs: impl Iterator<Item = Glyph> + Clone);
    /// Draw `image`'s pixels stretched over `rect` (bilinear sampling).
    fn draw_image(&mut self, image: &Pixels, rect: Rect, pixelated: bool);
    /// Fill `path` with `pattern`'s pixels mapped by its transform,
    /// repeated along the axes it says (bilinear sampling; the edge pixels
    /// extend along the others); `visible` bounds the frame in the current
    /// coordinates.
    fn fill_pattern(&mut self, path: &BezPath, pattern: &PatternFill, visible: Rect);
}

/// How a run of glyphs is drawn.
pub(crate) struct GlyphRun<'a> {
    /// The font the glyph ids index.
    pub(crate) font: &'a FontData,
    /// Pixels per em.
    pub(crate) font_size: f64,
    /// Normalized variation coordinates, in 2.14 fixed point.
    pub(crate) coords: &'a [i16],
    /// Stem darkening of the run.
    pub(crate) embolden: glifo::FontEmbolden,
    // xtask-allow(state-fields): drawing options - Hint vertically (only under a translation and a uniform scale)
    /// Hint vertically (only under a translation and a uniform scale).
    pub(crate) hint: bool,
    // xtask-allow(state-fields): drawing options - cached
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
            .font_size(self.font_size as f32)
            .normalized_coords(self.coords)
            .hint(self.hint)
            .font_embolden(self.embolden)
    }
}
