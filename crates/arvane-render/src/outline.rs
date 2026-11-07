//! Role: The outlines of a glyph run as path commands, for strokes and pattern fills.
//!
//! Unhinted outlines read from the font file, independent of the
//! rasterizers: the glyph outlines a glyph outline stroke and a pattern
//! fill of text are drawn from (the rasterizers' own glyph paths are
//! hinted and cached by glifo).

use kivact_types::Point;
use kivact_types::render::GlyphRun;
use kivact_types::scene::PathCmd;
use skrifa::instance::{LocationRef, NormalizedCoord, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};

/// The outlines of `run`'s glyphs as path commands in run space (font size
/// applied, y down, each glyph moved to its x and y), with the run's
/// normalized coordinates. Unhinted.
///
/// Glyphs without an outline (a bitmap or color-only glyph) and glyphs the
/// font does not have add nothing; an unreadable font yields nothing.
pub fn glyph_outline(run: &GlyphRun) -> Vec<PathCmd> {
    let mut out = Vec::new();
    let Ok(font) = FontRef::from_index(&run.font, run.font_index) else {
        return out;
    };
    let coords: Vec<NormalizedCoord> = run
        .normalized_coords
        .iter()
        .map(|c| NormalizedCoord::from_f32(*c as f32))
        .collect();
    let outlines = font.outline_glyphs();
    let size = Size::new(run.font_size as f32);
    let location = LocationRef::new(&coords);
    for glyph in &run.glyphs {
        let Some(outline) = outlines.get(GlyphId::new(glyph.id)) else {
            continue;
        };
        let mut pen = Pen {
            out: &mut out,
            dx: glyph.x,
            dy: glyph.y,
        };
        let _ = outline.draw(DrawSettings::unhinted(size, location), &mut pen);
    }
    out
}

/// Collects one glyph's outline, moved to its place and flipped to y down.
struct Pen<'a> {
    /// The commands collected so far.
    out: &'a mut Vec<PathCmd>,
    /// The glyph's x in run space.
    dx: f64,
    /// The glyph's y in run space (its baseline).
    dy: f64,
}

impl Pen<'_> {
    /// The run-space point of font coordinates `(x, y)` (font y is up).
    fn at(&self, x: f32, y: f32) -> Point {
        Point {
            x: self.dx + f64::from(x),
            y: self.dy - f64::from(y),
        }
    }
}

impl OutlinePen for Pen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        let to = self.at(x, y);
        self.out.push(PathCmd::MoveTo(to));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let to = self.at(x, y);
        self.out.push(PathCmd::LineTo(to));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let (control, to) = (self.at(cx0, cy0), self.at(x, y));
        self.out.push(PathCmd::QuadTo(control, to));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let (first, second, to) = (self.at(cx0, cy0), self.at(cx1, cy1), self.at(x, y));
        self.out.push(PathCmd::CubicTo(first, second, to));
    }

    fn close(&mut self) {
        self.out.push(PathCmd::Close);
    }
}
