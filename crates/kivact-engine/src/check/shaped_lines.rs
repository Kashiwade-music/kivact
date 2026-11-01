//! Role: Shape a text as it is painted and read its lines' advances and ink.

use crate::model::Model;
use crate::model::interact::visual;
use kivact_wire::{NodeId, NodeType, Point, Rect};

/// A shaped line, placed in the layout space.
pub(super) struct Line {
    /// Its ink: the advance across, the glyphs' outlines (or, without
    /// them, the capitals' top to the descent) down.
    pub(super) ink: Rect,
    /// Its baseline.
    pub(super) baseline: f32,
    /// The height of the capitals above the baseline.
    pub(super) cap_height: f32,
    /// The height of its line box.
    pub(super) height: f32,
    /// An ellipsis cut it.
    pub(super) cut: bool,
}

/// A text shaped as it is painted, with its lines in the layout space.
pub(super) struct Shaped {
    pub(super) lines: Vec<Line>,
    pub(super) font_size: f32,
}

impl Shaped {
    /// The rect the text covers: its lines' advances across, their ink
    /// down.
    pub(super) fn rect(&self) -> Rect {
        self.lines
            .iter()
            .map(|l| l.ink)
            .reduce(|a, b| a.union(b))
            .unwrap_or_default()
    }
}

/// Shape the text `id` as it is painted (`None`: it draws nothing, or is
/// not checked: a multiline input scrolls its lines).
pub(super) fn shape(
    host: &Model,
    engine: &mut arvane::Engine,
    id: NodeId,
    ink: bool,
) -> Option<Shaped> {
    let n = host.node(id)?;
    let props = host.text_props(id);
    debug_assert_eq!(
        *props,
        crate::measure::text_props(host.tree(), id),
        "node {} keeps a stale computed text style",
        id.0
    );
    let area = visual::text_box(host, id);
    let (shaped, origin) = match n.kind() {
        _ if host.editors[n.index.0]
            .as_deref()
            .map(|editor| editor.editor.state())
            .is_some()
            && !n.multiline() =>
        {
            let line = crate::model::interact::text_geometry::line_box(host, id, engine, area);
            let (mut lines, text) = host.edit_lines(id, engine)?;
            (
                lines.shaped(text, 0, engine)?,
                Point::new(area.x + lines.offset(text, 0, area.width, engine), line.y),
            )
        }
        NodeType::Text
            if host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .is_none() =>
        {
            let mut style = kivact_lower::assets::text_style(engine, &props);
            // As the paint shapes it: colors never change a shape.
            style.color = kivact_types::protocol::TextStyle::default().color;
            style.decoration.color = None;
            let width = Some(area.width.max(0.0) + 1.0);
            let content = n.content().unwrap_or("");
            let shaped = if n.spans().is_empty() {
                engine.shape_glyphs(content, &style, width)
            } else {
                let spans = crate::measure::leaf::rich_spans(engine, n.spans(), &style);
                let text = kivact_types::protocol::RichText {
                    text: content,
                    spans: &spans,
                };
                engine.shape_glyphs(text, &style, width)
            };
            (shaped.ok()?, Point::new(area.x, area.y))
        }
        _ => return None,
    };
    let mut inks: Vec<Option<Rect>> = vec![None; shaped.lines.len()];
    if ink {
        for run in &shaped.runs {
            let (Some(b), Some(g)) = (arvane::glyph_ink_bounds(run), run.glyphs.first()) else {
                continue;
            };
            // The line whose baseline this run sits on.
            let nearest = (0..shaped.lines.len()).min_by(|a, b| {
                let d = |i: usize| (shaped.lines[i].baseline - g.y).abs();
                d(*a).total_cmp(&d(*b))
            });
            if let Some(i) = nearest {
                let b = Rect::new(origin.x + b.x, origin.y + b.y, b.width, b.height);
                inks[i] = Some(inks[i].map_or(b, |a| a.union(b)));
            }
        }
    }
    let lines = shaped
        .lines
        .iter()
        .zip(inks)
        .map(|(l, ink)| {
            let x = origin.x + l.rect.x;
            // From the capitals' top to the descent: where a line's ink
            // lies without looking at the glyphs.
            let top = if l.cap_height > 0.0 {
                l.cap_height
            } else {
                l.ascent
            };
            let metrics = Rect::new(
                x,
                origin.y + l.baseline - top,
                l.rect.width,
                top + l.descent,
            );
            // The advance across, the ink down.
            let ink = ink.map_or(metrics, |i| Rect::new(x, i.y, l.rect.width, i.height));
            Line {
                ink,
                baseline: origin.y + l.baseline,
                cap_height: l.cap_height,
                height: l.rect.height,
                cut: l.cut,
            }
        })
        .collect();
    Some(Shaped {
        lines,
        font_size: props.font_size,
    })
}
