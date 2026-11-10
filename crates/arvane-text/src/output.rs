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

use std::collections::HashMap;
use std::sync::Arc;

use kivact_types::render::TextSpan;
use kivact_types::render::{Glyph, GlyphRun, TextDecorationRect, TextHighlight, TextLine};
use kivact_types::{ComputedTextStyle, TextDecoration};
use kivact_types::{DecorationLines, Rect, Size, TextAlign, TextOverflow};
use parley::{Layout, PositionedLayoutItem};

/// Slack for comparing positions that sum float advances.
const EPSILON: f64 = 0.01;

/// The vertical metrics of a run's font that decorations and bands use.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Metrics {
    ascent: f64,
    descent: f64,
    underline_offset: f64,
    underline_size: f64,
    strikethrough_offset: f64,
    strikethrough_size: f64,
}

impl Metrics {
    fn of(m: &parley::RunMetrics) -> Self {
        Self {
            ascent: f64::from(m.ascent),
            descent: f64::from(m.descent),
            underline_offset: f64::from(m.underline_offset),
            underline_size: f64::from(m.underline_size),
            strikethrough_offset: f64::from(m.strikethrough_offset),
            strikethrough_size: f64::from(m.strikethrough_size),
        }
    }
}

/// "…" shaped in a paragraph's style: its glyphs from pen 0 on the
/// baseline (y 0), uncolored.
#[derive(Clone, Debug)]
pub(crate) struct Ellipsis {
    run: Option<GlyphRun>,
    advance: f64,
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
                        x: pen + f64::from(g.x),
                        y: f64::from(g.y),
                        advance: f64::from(g.advance),
                    };
                    pen += f64::from(g.advance);
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
    x: f64,
    shift: f64,
    /// The kept text with the ellipsis.
    width: f64,
}

/// The lines a paragraph shows and how they are cut.
#[derive(Debug, Default)]
pub(crate) struct Plan {
    visible: usize,
    /// One per visible line when any line is cut, else empty.
    cuts: Vec<Option<Cut>>,
    pub(crate) size: Size,
    /// The first line's baseline.
    pub(crate) baseline: f64,
}

impl Plan {
    /// Vertical edges of the visible cap-to-baseline box in layout coordinates.
    pub(crate) fn cap_edges(
        &self,
        layout: &Layout<u32>,
        style: &ComputedTextStyle,
    ) -> Option<(f64, f64)> {
        if !style.capsize {
            return None;
        }
        let first = layout.get(0)?;
        let last = layout.get(self.visible.checked_sub(1)?)?;
        let cap = first
            .runs()
            .filter_map(|run| run.metrics().cap_height.map(f64::from))
            .fold(0.0f64, f64::max);
        (cap > 0.0).then(|| {
            (
                f64::from(first.metrics().baseline) - cap,
                f64::from(last.metrics().baseline),
            )
        })
    }
}

/// The lines `style` shows of `layout`.
fn visible_lines(layout: &Layout<u32>, style: &ComputedTextStyle) -> usize {
    let total = layout.len();
    style
        .max_lines
        .map_or(total, |n| (n.max(1) as usize).min(total))
}

/// The width of a line's content (trailing white space aside) from its
/// start.
fn content_width(line: &parley::Line<'_, u32>) -> f64 {
    let m = line.metrics();
    f64::from(m.advance) - f64::from(m.trailing_whitespace)
}

/// Import an uncut line's inline endpoints in Parley's native precision.
///
/// Alignment offsets and advances are rounded f32 metrics. Adding them only
/// after widening can expose their rounding residual as an apparent overflow
/// (for example, 91.000001907 pixels for a native 91-pixel right edge).
/// Establish both native edges first, then derive the shared f64 width from
/// those edges. Custom ellipsis shifts and widths stay in f64 in the caller.
fn native_inline_bounds(line: &parley::Line<'_, u32>, justified: bool) -> (f64, f64) {
    let m = line.metrics();
    let advance = if justified {
        // Justification widens glyph advances without updating the line's
        // pre-justification advance metric, so sum the native placed items.
        line.items()
            .map(|item| match item {
                PositionedLayoutItem::GlyphRun(run) => run.glyphs().map(|g| g.advance).sum::<f32>(),
                PositionedLayoutItem::InlineBox(item)
                    if item.kind == parley::InlineBoxKind::InFlow =>
                {
                    item.width
                }
                PositionedLayoutItem::InlineBox(_) => 0.0,
            })
            .sum::<f32>()
    } else {
        m.advance
    };
    let left = m.inline_min_coord + m.offset;
    let right = left + (advance - m.trailing_whitespace);
    let (left, right) = (f64::from(left), f64::from(right));
    (left, right - left)
}

/// Whether `style` cuts a line of `layout` broken at `max_width` (an
/// ellipsis is needed).
pub(crate) fn needs_cut(
    layout: &Layout<u32>,
    style: &ComputedTextStyle,
    max_width: Option<f64>,
) -> bool {
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
    style: &ComputedTextStyle,
    max_width: Option<f64>,
    ellipsis: Option<&Ellipsis>,
) -> Plan {
    let baseline = layout
        .get(0)
        .map_or(0.0, |l| f64::from(l.metrics().baseline));
    let total = layout.len();
    let Some(ellipsis) = ellipsis else {
        return Plan {
            visible: total,
            cuts: Vec::new(),
            size: Size {
                width: f64::from(layout.width()),
                height: f64::from(layout.height()),
            },
            baseline,
        };
    };
    let visible = visible_lines(layout, style);
    let limit = max_width.unwrap_or(f64::INFINITY);
    let factor = match style.align {
        TextAlign::Start | TextAlign::Justify => 0.0,
        TextAlign::Center => 0.5,
        TextAlign::End => 1.0,
    };
    let mut cuts = Vec::with_capacity(visible);
    let mut width = 0.0f64;
    for (i, line) in layout.lines().take(visible).enumerate() {
        let m = *line.metrics();
        let full = content_width(&line);
        let clamped = i + 1 == visible && visible < total;
        let overflows = style.overflow == TextOverflow::Ellipsis && full > limit + EPSILON;
        if !(clamped || overflows) {
            cuts.push(None);
            width = width.max(f64::from(m.inline_min_coord) + full);
            continue;
        }
        // Keep whole clusters while they leave room for the ellipsis.
        let room = limit - ellipsis.advance;
        let (mut x, mut end) = (0.0f64, 0.0f64);
        'line: for run in line.runs() {
            for cluster in run.visual_clusters() {
                let advance = f64::from(cluster.advance());
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
        let container = f64::from(m.inline_max_coord) - f64::from(m.inline_min_coord);
        let offset = if container.is_finite() {
            factor * (container - cut_width).max(0.0)
        } else {
            0.0
        };
        let start = f64::from(m.inline_min_coord) + f64::from(m.offset);
        cuts.push(Some(Cut {
            x: start + end,
            shift: f64::from(m.inline_min_coord) + offset - start,
            width: cut_width,
        }));
        width = width.max(f64::from(m.inline_min_coord) + cut_width);
    }
    let height = if visible < total {
        layout
            .get(visible - 1)
            .map_or(f64::from(layout.height()), |l| {
                f64::from(l.metrics().block_max_coord)
            })
    } else {
        f64::from(layout.height())
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
                Some(c) => (
                    f64::from(m.inline_min_coord) + f64::from(m.offset) + c.shift,
                    c.width,
                ),
                None => native_inline_bounds(&line, justified),
            };
            // The tallest letters of the line's fonts.
            let (mut cap_height, mut x_height) = (0.0f64, 0.0f64);
            for run in line.runs() {
                let run = run.metrics();
                cap_height = cap_height.max(f64::from(run.cap_height.unwrap_or(0.0)));
                x_height = x_height.max(f64::from(run.x_height.unwrap_or(0.0)));
            }
            TextLine {
                range: range(line.text_range()),
                rect: Rect {
                    x,
                    y: f64::from(m.block_min_coord),
                    width: width.max(0.0),
                    height: f64::from(m.block_max_coord) - f64::from(m.block_min_coord),
                },
                baseline: f64::from(m.baseline),
                ascent: f64::from(m.ascent),
                descent: f64::from(m.descent),
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
    x0: f64,
    x1: f64,
    baseline: f64,
    font_size: f64,
    color: kivact_types::Rgba,
    metrics: Metrics,
}

/// What a planned paragraph draws.
pub(crate) struct Drawn {
    pub(crate) runs: Vec<kivact_types::render::GlyphRunSnapshot>,
    pub(crate) decorations: Vec<TextDecorationRect>,
    pub(crate) highlights: Vec<TextHighlight>,
}

/// The glyph runs, decorations and highlight bands `layout` draws as
/// `plan` shows it.
pub(crate) fn draw(
    layout: &Layout<u32>,
    style: &ComputedTextStyle,
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
            let baseline = f64::from(run.baseline());
            let mut pen = f64::from(run.offset());
            let (mut x0, mut x1) = (None, pen);
            let mut glyphs = Vec::new();
            for g in run.glyphs() {
                let start = pen;
                pen += f64::from(g.advance);
                if cut.is_some_and(|c| start >= c.x - EPSILON) {
                    continue;
                }
                glyphs.push(Glyph {
                    id: g.id,
                    x: start + f64::from(g.x) + shift,
                    y: baseline + f64::from(g.y),
                    advance: f64::from(g.advance),
                });
                x0.get_or_insert(start + shift);
                x1 = pen + shift;
            }
            let Some(x0) = x0 else { continue };
            let mut glyph_run = glyph_run(&run, glyphs, fonts);
            glyph_run.color = color;
            glyph_run.stroke = span_style.and_then(|s| s.stroke);
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
            runs.push(glyph_run.into());
        }
        if let (Some(cut), Some(ellipsis)) = (cut, ellipsis)
            && let Some(template) = &ellipsis.run
        {
            let x = cut.x + cut.shift;
            let baseline = f64::from(line.metrics().baseline);
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
            runs.push(run.into());
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
        font: kivact_types::AssetBytes::from_shared(Arc::clone(font)),
        font_index: run.font().index,
        font_size: f64::from(run.font_size()),
        color: ComputedTextStyle::default().color,
        stroke: None,
        normalized_coords: run
            .normalized_coords()
            .iter()
            .map(|value| f64::from(*value) / 16384.0)
            .collect(),
        glyphs,
    }
}

/// The decoration lines of `segments`, each line's adjacent pieces of the
/// same kind, color and position joined (no seam between spans).
fn decorations(
    segments: &[Segment],
    style: &ComputedTextStyle,
    spans: &[TextSpan],
) -> Vec<TextDecorationRect> {
    let mut out: Vec<TextDecorationRect> = Vec::new();
    for s in segments {
        let span_decoration = s
            .span
            .and_then(|i| spans.get(i))
            .and_then(|span| span.style.decoration);
        let decoration: TextDecoration = span_decoration.unwrap_or(style.decoration);
        if decoration.lines.is_empty() || s.x1 <= s.x0 {
            continue;
        }
        let color = decoration.color.unwrap_or(s.color);
        let m = s.metrics;
        // Fonts without the metrics get a line of a fourteenth of their size.
        let fallback = (s.font_size / 14.0).max(0.5);
        // Inherited decoration lengths keep the paragraph's font basis.
        let own = ComputedTextStyle {
            font_size: if span_decoration.is_some() {
                s.font_size
            } else {
                style.font_size
            },
            decoration,
            ..style.clone()
        };
        let thickness = |font: f64| {
            own.decoration_thickness_px()
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
                s.baseline - m.underline_offset + own.resolve(decoration.underline_offset),
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
                d.style == decoration.style
                    && d.line == line
                    && d.color == color
                    && (d.rect.y - rect.y).abs() < EPSILON
                    && (d.rect.height - rect.height).abs() < EPSILON
                    && (d.rect.x + d.rect.width - rect.x).abs() < 0.5
            });
            match joins {
                Some(d) => d.rect.width = rect.x + rect.width - d.rect.x,
                None => out.push(TextDecorationRect {
                    rect,
                    color,
                    line,
                    style: decoration.style,
                }),
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
