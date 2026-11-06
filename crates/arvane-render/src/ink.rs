//! Role: The ink of a glyph run: the box its glyphs' outlines cover.
//!
//! Independent of the rasterizers (it reads the font file only), and not
//! cached: a caller asks when it needs the box.

use kivact_types::Rect;
use kivact_types::render::GlyphRun;
use skrifa::instance::{LocationRef, NormalizedCoord, Size};
use skrifa::{FontRef, MetadataProvider};

/// The smallest rectangle holding the outlines of `run`'s glyphs, in the
/// run's own coordinates (logical pixels from its layout origin, y down).
///
/// White space, glyphs without an outline (a bitmap or color-only glyph is
/// not approximated by its advance) and glyphs the font does not have add
/// nothing; `None` when no glyph has ink or the font cannot be read. This
/// reads the font's outlines, so call it only when needed, not for every
/// run drawn.
pub fn glyph_ink_bounds(run: &GlyphRun) -> Option<Rect> {
    let font = FontRef::from_index(&run.font, run.font_index).ok()?;
    let coords: Vec<NormalizedCoord> = run
        .normalized_coords
        .iter()
        .map(|c| NormalizedCoord::from_f32(*c as f32))
        .collect();
    let metrics = font.glyph_metrics(Size::new(run.font_size as f32), LocationRef::new(&coords));
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for glyph in &run.glyphs {
        let Some(b) = metrics.bounds(skrifa::GlyphId::new(glyph.id)) else {
            continue;
        };
        if !(b.x_max > b.x_min && b.y_max > b.y_min) {
            continue;
        }
        x0 = x0.min(glyph.x + f64::from(b.x_min));
        x1 = x1.max(glyph.x + f64::from(b.x_max));
        y0 = y0.min(glyph.y - f64::from(b.y_max));
        y1 = y1.max(glyph.y - f64::from(b.y_min));
    }
    (x0 < x1 && y0 < y1).then_some(Rect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}
