//! Role: Conversion of a line-broken layout into measured and drawable output.
//!
//! From a layout broken into lines to what is measured and drawn: the
//! lines shown (a line clamp), where an ellipsis cuts them, the glyph runs
//! colored by their span, and the decoration lines and highlight bands
//! measured from the runs.
//!
//! Invariants:
//! - Measuring and drawing size a paragraph by the same [`Plan`]; without
//!   a cut it is parley's size, as before ellipses existed.
//! - A cut keeps whole clusters (a base character with its marks, a
//!   ligature) from the line's visual start while they leave room for the
//!   ellipsis, drops trailing white space, and draws "…" in the
//!   paragraph's style (CSS styles the ellipsis by its block) right after
//!   them; the cut line is aligned again by its new width. The visual end
//!   is the right (an RTL line is cut on the right too).
//! - Decorations and bands follow the glyphs drawn: a dropped glyph has
//!   none, the ellipsis has the paragraph's decoration.

use std::{collections::HashMap, sync::Arc};

use kivact_types::{
    DecorationLines, Glyph, GlyphRun, Rect, Size, TextAlign, TextDecoration, TextDecorationRect,
    TextHighlight, TextLine, TextOverflow, TextSpan, TextStyle,
};
use parley::{Layout, PositionedLayoutItem};

/// Slack for comparing positions that sum float advances.
const EPSILON: f32 = 0.01;

/// The vertical metrics of a run's font that decorations and bands use.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Metrics {
    ascent: f32,
    descent: f32,
    underline_offset: f32,
    underline_size: f32,
    strikethrough_offset: f32,
    strikethrough_size: f32,
}

impl Metrics {
    fn of(m: &parley::RunMetrics) -> Self {
        Self {
            ascent: m.ascent,
            descent: m.descent,
            underline_offset: m.underline_offset,
            underline_size: m.underline_size,
            strikethrough_offset: m.strikethrough_offset,
            strikethrough_size: m.strikethrough_size,
        }
    }
}

/// "…" shaped in a paragraph's style: its glyphs from pen 0 on the
/// baseline (y 0), uncolored.
#[derive(Clone, Debug)]
pub(crate) struct Ellipsis {
    run: Option<GlyphRun>,
    advance: f32,
    metrics: Metrics,
}

impl Ellipsis {
    /// The ellipsis of a layout of "…" broken into one line.
    pub(crate) fn of(layout: &Layout<u32>, fonts: &mut HashMap<u64, Arc<[u8]>>) -> Self {
        let mut out = Ellipsis {
            run: None,
            advance: 0.0,
            metrics: Metrics::default(),
        };
        let Some(line) = layout.lines().next() else {
            return out;
        };
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            let mut pen = 0.0;
            let glyphs = run
                .glyphs()
                .map(|g| {
                    let glyph = Glyph {
                        id: g.id,
                        x: pen + g.x,
                        y: g.y,
                        advance: g.advance,
                    };
                    pen += g.advance;
                    glyph
                })
                .collect();
            out.advance = pen;
            out.metrics = Metrics::of(run.run().metrics());
            out.run = Some(glyph_run(&run, glyphs, fonts));
            break;
        }
        out
    }
}

/// Where a line is cut: glyphs whose pen starts at `x` or later are
/// dropped and the ellipsis drawn at `x`; what is kept moves by `shift`
/// (aligned again by the cut width).
#[derive(Clone, Copy, Debug)]
struct Cut {
    x: f32,
    shift: f32,
    /// The kept text with the ellipsis.
    width: f32,
}

/// The lines a paragraph shows and how they are cut.
#[derive(Debug, Default)]
pub(crate) struct Plan {
    visible: usize,
    /// One per visible line when any line is cut, else empty.
    cuts: Vec<Option<Cut>>,
    pub(crate) size: Size,
    /// The first line's baseline.
    pub(crate) baseline: f32,
}

/// The lines `style` shows of `layout`.
fn visible_lines(layout: &Layout<u32>, style: &TextStyle) -> usize {
    let total = layout.len();
    style
        .max_lines
        .map_or(total, |n| (n.max(1) as usize).min(total))
}

/// The width of a line's content (trailing white space aside) from its
/// start.
fn content_width(line: &parley::Line<'_, u32>) -> f32 {
    let m = line.metrics();
    m.advance - m.trailing_whitespace
}

/// The width of a line's content in a layout aligned with
/// [`parley::Alignment::Justify`]: the line's metrics keep the width it
/// had before its spaces widened, the glyphs' advances do not.
fn justified_width(line: &parley::Line<'_, u32>) -> f32 {
    let m = line.metrics();
    let advance: f32 = line
        .items()
        .filter_map(|item| match item {
            PositionedLayoutItem::GlyphRun(run) => {
                Some(run.glyphs().map(|g| g.advance).sum::<f32>())
            }
            _ => None,
        })
        .sum();
    advance - m.trailing_whitespace
}

/// Whether `style` cuts a line of `layout` broken at `max_width` (an
/// ellipsis is needed).
pub(crate) fn needs_cut(layout: &Layout<u32>, style: &TextStyle, max_width: Option<f32>) -> bool {
    if !style.ellipsizes() {
        return false;
    }
    let visible = visible_lines(layout, style);
    if visible < layout.len() {
        return true;
    }
    style.overflow == TextOverflow::Ellipsis
        && max_width.is_some_and(|w| {
            layout
                .lines()
                .any(|line| content_width(&line) > w + EPSILON)
        })
}

/// The lines `layout` (broken at `max_width`, aligned or not) shows in
/// `style`, cut for `ellipsis` (given when [`needs_cut`]).
pub(crate) fn plan(
    layout: &Layout<u32>,
    style: &TextStyle,
    max_width: Option<f32>,
    ellipsis: Option<&Ellipsis>,
) -> Plan {
    let baseline = layout.get(0).map_or(0.0, |l| l.metrics().baseline);
    let total = layout.len();
    let Some(ellipsis) = ellipsis else {
        return Plan {
            visible: total,
            cuts: Vec::new(),
            size: Size {
                width: layout.width(),
                height: layout.height(),
            },
            baseline,
        };
    };
    let visible = visible_lines(layout, style);
    let limit = max_width.unwrap_or(f32::INFINITY);
    let factor = match style.align {
        TextAlign::Start | TextAlign::Justify => 0.0,
        TextAlign::Center => 0.5,
        TextAlign::End => 1.0,
    };
    let mut cuts = Vec::with_capacity(visible);
    let mut width = 0.0f32;
    for (i, line) in layout.lines().take(visible).enumerate() {
        let m = *line.metrics();
        let full = content_width(&line);
        let clamped = i + 1 == visible && visible < total;
        let overflows = style.overflow == TextOverflow::Ellipsis && full > limit + EPSILON;
        if !(clamped || overflows) {
            cuts.push(None);
            width = width.max(m.inline_min_coord + full);
            continue;
        }
        // Keep whole clusters while they leave room for the ellipsis.
        let room = limit - ellipsis.advance;
        let (mut x, mut end) = (0.0f32, 0.0f32);
        'line: for run in line.runs() {
            for cluster in run.visual_clusters() {
                let advance = cluster.advance();
                if x + advance > room + EPSILON {
                    break 'line;
                }
                x += advance;
                if !cluster.is_space_or_nbsp() && cluster.is_line_break().is_none() {
                    end = x;
                }
            }
        }
        let cut_width = end + ellipsis.advance;
        let container = m.inline_max_coord - m.inline_min_coord;
        let offset = if container.is_finite() {
            factor * (container - cut_width).max(0.0)
        } else {
            0.0
        };
        let start = m.inline_min_coord + m.offset;
        cuts.push(Some(Cut {
            x: start + end,
            shift: m.inline_min_coord + offset - start,
            width: cut_width,
        }));
        width = width.max(m.inline_min_coord + cut_width);
    }
    let height = if visible < total {
        layout
            .get(visible - 1)
            .map_or(layout.height(), |l| l.metrics().block_max_coord)
    } else {
        layout.height()
    };
    Plan {
        visible,
        cuts,
        size: Size { width, height },
        baseline,
    }
}

/// The geometry of the lines `plan` shows of `layout`, with `range` mapping
/// a line's byte range in the layout's text to the caller's.
pub(crate) fn lines(
    layout: &Layout<u32>,
    plan: &Plan,
    justified: bool,
    range: impl Fn(std::ops::Range<usize>) -> std::ops::Range<usize>,
) -> Vec<TextLine> {
    layout
        .lines()
        .take(plan.visible)
        .enumerate()
        .map(|(i, line)| {
            let m = *line.metrics();
            let cut = plan.cuts.get(i).copied().flatten();
            let (x, width) = match cut {
                Some(c) => (m.inline_min_coord + m.offset + c.shift, c.width),
                None if justified => (m.inline_min_coord + m.offset, justified_width(&line)),
                None => (m.inline_min_coord + m.offset, content_width(&line)),
            };
            // The tallest letters of the line's fonts.
            let (mut cap_height, mut x_height) = (0.0f32, 0.0f32);
            for run in line.runs() {
                let run = run.metrics();
                cap_height = cap_height.max(run.cap_height.unwrap_or(0.0));
                x_height = x_height.max(run.x_height.unwrap_or(0.0));
            }
            TextLine {
                range: range(line.text_range()),
                rect: Rect {
                    x,
                    y: m.block_min_coord,
                    width: width.max(0.0),
                    height: m.block_max_coord - m.block_min_coord,
                },
                baseline: m.baseline,
                ascent: m.ascent,
                descent: m.descent,
                cap_height,
                x_height,
                cut: cut.is_some(),
            }
        })
        .collect()
}

/// A drawn piece of a glyph run: where its kept glyphs lie on a line.
struct Segment {
    line: usize,
    /// Its span (`None`: the paragraph's style).
    span: Option<usize>,
    x0: f32,
    x1: f32,
    baseline: f32,
    font_size: f32,
    color: kivact_types::Rgba,
    metrics: Metrics,
}

/// What a planned paragraph draws.
pub(crate) struct Drawn {
    pub(crate) runs: Vec<Arc<GlyphRun>>,
    pub(crate) decorations: Vec<TextDecorationRect>,
    pub(crate) highlights: Vec<TextHighlight>,
}

/// The glyph runs, decorations and highlight bands `layout` draws as
/// `plan` shows it.
pub(crate) fn draw(
    layout: &Layout<u32>,
    style: &TextStyle,
    spans: &[TextSpan],
    plan: &Plan,
    ellipsis: Option<&Ellipsis>,
    fonts: &mut HashMap<u64, Arc<[u8]>>,
) -> Drawn {
    let mut runs = Vec::new();
    let mut segments = Vec::new();
    for (i, line) in layout.lines().take(plan.visible).enumerate() {
        let cut = plan.cuts.get(i).copied().flatten();
        let shift = cut.map_or(0.0, |c| c.shift);
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            let span = (run.style().brush as usize).checked_sub(1);
            let span_style = span.and_then(|s| spans.get(s)).map(|s| &s.style);
            let color = span_style.and_then(|s| s.color).unwrap_or(style.color);
            let baseline = run.baseline();
            let mut pen = run.offset();
            let (mut x0, mut x1) = (None, pen);
            let mut glyphs = Vec::new();
            for g in run.glyphs() {
                let start = pen;
                pen += g.advance;
                if cut.is_some_and(|c| start >= c.x - EPSILON) {
                    continue;
                }
                glyphs.push(Glyph {
                    id: g.id,
                    x: start + g.x + shift,
                    y: baseline + g.y,
                    advance: g.advance,
                });
                x0.get_or_insert(start + shift);
                x1 = pen + shift;
            }
            let Some(x0) = x0 else { continue };
            let mut glyph_run = glyph_run(&run, glyphs, fonts);
            glyph_run.color = color;
            segments.push(Segment {
                line: i,
                span,
                x0,
                x1,
                baseline,
                font_size: glyph_run.font_size,
                color,
                metrics: Metrics::of(run.run().metrics()),
            });
            runs.push(Arc::new(glyph_run));
        }
        if let (Some(cut), Some(ellipsis)) = (cut, ellipsis)
            && let Some(template) = &ellipsis.run
        {
            let x = cut.x + cut.shift;
            let baseline = line.metrics().baseline;
            let mut run = template.clone();
            run.color = style.color;
            for glyph in &mut run.glyphs {
                glyph.x += x;
                glyph.y += baseline;
            }
            segments.push(Segment {
                line: i,
                span: None,
                x0: x,
                x1: x + ellipsis.advance,
                baseline,
                font_size: run.font_size,
                color: style.color,
                metrics: ellipsis.metrics,
            });
            runs.push(Arc::new(run));
        }
    }
    Drawn {
        runs,
        decorations: decorations(&segments, style, spans),
        highlights: highlights(&segments, spans),
    }
}

/// An Arvane glyph run of a parley one holding `glyphs`, in the default
/// color.
fn glyph_run(
    run: &parley::GlyphRun<'_, u32>,
    glyphs: Vec<Glyph>,
    fonts: &mut HashMap<u64, Arc<[u8]>>,
) -> GlyphRun {
    let run = run.run();
    let font = fonts
        .entry(run.font().data.id())
        .or_insert_with(|| Arc::from(run.font().data.as_ref()));
    GlyphRun {
        font: Arc::clone(font),
        font_index: run.font().index,
        font_size: run.font_size(),
        color: TextStyle::default().color,
        normalized_coords: run
            .normalized_coords()
            .iter()
            .map(|value| f32::from(*value) / 16384.0)
            .collect(),
        glyphs,
    }
}

/// The decoration lines of `segments`, each line's adjacent pieces of the
/// same kind, color and position joined (no seam between spans).
fn decorations(
    segments: &[Segment],
    style: &TextStyle,
    spans: &[TextSpan],
) -> Vec<TextDecorationRect> {
    let mut out: Vec<TextDecorationRect> = Vec::new();
    for s in segments {
        let decoration: TextDecoration = s
            .span
            .and_then(|i| spans.get(i))
            .and_then(|span| span.style.decoration)
            .unwrap_or(style.decoration);
        if decoration.lines.is_empty() || s.x1 <= s.x0 {
            continue;
        }
        let color = decoration.color.unwrap_or(s.color);
        let m = s.metrics;
        // Fonts without the metrics get a line of a fourteenth of their size.
        let fallback = (s.font_size / 14.0).max(0.5);
        let thickness = |font: f32| {
            decoration
                .thickness
                .filter(|t| t.is_finite() && *t > 0.0)
                .unwrap_or(if font > 0.0 { font } else { fallback })
        };
        for (line, top, height) in [
            (
                DecorationLines::OVERLINE,
                s.baseline - m.ascent,
                thickness(m.underline_size),
            ),
            (
                DecorationLines::UNDERLINE,
                s.baseline - m.underline_offset + decoration.underline_offset,
                thickness(m.underline_size),
            ),
            (
                DecorationLines::LINE_THROUGH,
                s.baseline
                    - if m.strikethrough_offset > 0.0 {
                        m.strikethrough_offset
                    } else {
                        s.font_size * 0.3
                    },
                thickness(m.strikethrough_size),
            ),
        ] {
            if !decoration.lines.contains(line) {
                continue;
            }
            let rect = Rect {
                x: s.x0,
                y: top,
                width: s.x1 - s.x0,
                height,
            };
            let joins = out.iter_mut().rev().take(4).find(|d| {
                d.line == line
                    && d.color == color
                    && (d.rect.y - rect.y).abs() < EPSILON
                    && (d.rect.height - rect.height).abs() < EPSILON
                    && (d.rect.x + d.rect.width - rect.x).abs() < 0.5
            });
            match joins {
                Some(d) => d.rect.width = rect.x + rect.width - d.rect.x,
                None => out.push(TextDecorationRect { rect, color, line }),
            }
        }
    }
    out
}

/// The bands of the highlighted spans: one per span and line, over its
/// glyphs' advance and their fonts' ascent and descent.
fn highlights(segments: &[Segment], spans: &[TextSpan]) -> Vec<TextHighlight> {
    let mut out: Vec<(usize, TextHighlight)> = Vec::new();
    for s in segments {
        let Some(span) = s
            .span
            .filter(|i| spans.get(*i).is_some_and(|s| s.style.highlight))
        else {
            continue;
        };
        let rect = Rect {
            x: s.x0,
            y: s.baseline - s.metrics.ascent,
            width: s.x1 - s.x0,
            height: s.metrics.ascent + s.metrics.descent,
        };
        match out
            .iter_mut()
            .find(|(line, h)| *line == s.line && h.span == span)
        {
            Some((_, h)) => h.rect = union(h.rect, rect),
            None => out.push((s.line, TextHighlight { rect, span })),
        }
    }
    out.into_iter().map(|(_, h)| h).collect()
}

fn union(a: Rect, b: Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rect {
        x,
        y,
        width: (a.x + a.width).max(b.x + b.width) - x,
        height: (a.y + a.height).max(b.y + b.height) - y,
    }
}
