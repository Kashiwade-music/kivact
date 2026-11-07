//! Role: The boxes a node's background is painted in: content, text clip and frames.
//!
//! Invariants:
//! - The content box is layout's, inside the border and the padding; the
//!   padding box is the border box inset by the border widths (§9.3).
//! - A background clipped to the text is clipped to the glyph outlines of the
//!   node's own paragraph, drawn at the paragraph's place (§9.4). A node that
//!   is not a text node has none, so such a layer is not drawn.
//! - Each animated background layer shows the frame of its own playback
//!   (slot 1 + its index, §9.8).

use kivact_types::render::ShapedText;
use kivact_types::scene::Shape;
use kivact_types::{
    BackgroundClip, Edges, Fill, FillRule, NodeId, PaintProps, PathCmd, Point, Rect,
};

use crate::contract::NodeType;
use crate::lower::effects::Tint;
use crate::lower::fill::Boxes;
use crate::lower::shape::{self, BoxShape};
use crate::model::Model;
use crate::model::interact::visual;
use crate::model::playback::state::background_slot;

impl Model {
    /// The boxes of node `id` painted with `p`: its border box `outline`, its
    /// padding and content boxes, the glyph outlines of its text when a
    /// background clips to the text, and the frame of each background layer.
    pub(crate) fn node_boxes(
        &self,
        id: NodeId,
        p: &PaintProps,
        outline: &BoxShape,
        tint: &Tint,
        engine: &mut arvane::Engine,
    ) -> Boxes {
        let border = outline.rect();
        let layout = self.computed_layout(id).unwrap_or_default();
        let content = layout.content_box;
        // The content box is relative to the border box (as `visual` places it).
        let inside = Edges {
            top: content.y,
            right: border.width - content.x - content.width,
            bottom: border.height - content.y - content.height,
            left: content.x,
        };
        let layers = match p.background.as_ref() {
            Some(Fill::Layers(layers)) => layers.as_slice(),
            _ => &[],
        };
        let glyphs = if layers
            .iter()
            .any(|layer| layer.clip == BackgroundClip::Text)
        {
            self.text_glyphs(id, border, content, tint, engine)
        } else {
            None
        };
        Boxes {
            border: *outline,
            padding: outline.inset(shape::border_widths(p.border.as_ref())),
            content: outline.inset(inside),
            glyphs,
            frames: (0..layers.len())
                .map(|index| self.playback_frame_at(id, background_slot(index)))
                .collect(),
        }
    }

    /// The glyph outlines of the text of node `id`, laid out in its content box
    /// (`None`: it is not a text node, or its text cannot be shaped).
    fn text_glyphs(
        &self,
        id: NodeId,
        border: Rect,
        content: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
    ) -> Option<Shape> {
        if self.node(id)?.kind() != NodeType::Text {
            return None;
        }
        let area = visual::content_at(border, content);
        let shaped = self.shape_node_text(id, area, tint, engine)?;
        Some(glyph_clip(
            &shaped,
            Point {
                x: area.x,
                y: area.y,
            },
        ))
    }
}

/// The outlines of the glyphs of `shaped`, drawn at `origin`, as one path
/// filled by the non-zero rule.
fn glyph_clip(shaped: &ShapedText, origin: Point) -> Shape {
    let mut cmds = Vec::new();
    for run in &shaped.runs {
        cmds.extend(
            arvane::glyph_outline(run)
                .into_iter()
                .map(|cmd| shift(cmd, origin)),
        );
    }
    Shape::Path {
        cmds,
        rule: FillRule::NonZero,
    }
}

/// `cmd` moved by `by`. A glyph outline holds only moves, lines, curves and
/// closes (`arvane::glyph_outline`), so the other segments are not expected.
fn shift(cmd: PathCmd, by: Point) -> PathCmd {
    let moved = |p: Point| Point {
        x: p.x + by.x,
        y: p.y + by.y,
    };
    match cmd {
        PathCmd::MoveTo(p) => PathCmd::MoveTo(moved(p)),
        PathCmd::LineTo(p) => PathCmd::LineTo(moved(p)),
        PathCmd::QuadTo(a, b) => PathCmd::QuadTo(moved(a), moved(b)),
        PathCmd::CubicTo(a, b, c) => PathCmd::CubicTo(moved(a), moved(b), moved(c)),
        other => other,
    }
}
