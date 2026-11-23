//! Role: Shape a text as it is painted and read its lines' advances and line boxes.

use crate::contract::NodeType;
use crate::model::Model;
use crate::model::interact::visual;
use kivact_types::{NodeId, Point, Rect};

/// A shaped line, placed in the layout space.
pub(super) struct Line {
    /// Its box: the advance across, the line box down (the line-height
    /// included). The glyphs' outlines do not count.
    pub(super) bounds: Rect,
    /// Its baseline.
    pub(super) baseline: f64,
    /// The height of the capitals above the baseline.
    pub(super) cap_height: f64,
    /// The height of its line box.
    pub(super) height: f64,
    /// An ellipsis cut it.
    pub(super) cut: bool,
}

/// A text shaped as it is painted, with its lines in the layout space.
pub(super) struct Shaped {
    pub(super) lines: Vec<Line>,
    pub(super) font_size: f64,
}

impl Shaped {
    /// The rect the text covers: its lines' advances across, their line
    /// boxes down.
    pub(super) fn rect(&self) -> Rect {
        self.lines
            .iter()
            .map(|l| l.bounds)
            .reduce(|a, b| a.union(b))
            .unwrap_or_default()
    }
}

/// Shape the text `id` as it is painted (`None`: it draws nothing, or is
/// not checked: a multiline input scrolls its lines).
pub(super) fn shape(
    host: &Model,
    engine: &mut arvane_text::TextEngine,
    id: NodeId,
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
        _ if host.slots.editors[n.index.0].is_some() && !n.multiline() => {
            let line = crate::model::interact::text_geometry::line_box(host, id, engine, area);
            let (mut lines, text) = host.edit_lines(id, engine)?;
            (
                lines.shaped(text, 0, engine)?,
                Point {
                    x: area.x + lines.offset(text, 0, area.width, engine),
                    y: line.y,
                },
            )
        }
        NodeType::Text if host.slots.editors[n.index.0].is_none() => (
            crate::model::interact::text_geometry::shape_static_text(host, id, engine, area.width)?,
            Point {
                x: area.x,
                y: area.y,
            },
        ),
        _ => return None,
    };
    let lines = shaped
        .lines
        .iter()
        .map(|l| Line {
            // The advance across, the line box down: the line's tile of the
            // paragraph, from its top.
            bounds: Rect {
                x: origin.x + l.line_box.x,
                y: origin.y + l.line_box.y,
                width: l.line_box.width,
                height: l.line_box.height,
            },
            baseline: origin.y + l.baseline,
            cap_height: l.cap_height,
            height: l.line_box.height,
            cut: l.cut,
        })
        .collect();
    Some(Shaped {
        lines,
        font_size: props.font_size,
    })
}
