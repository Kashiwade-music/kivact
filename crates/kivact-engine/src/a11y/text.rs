//! Role: Publish native editing selections and shaped text runs.
//!
//! An editor's text is published as one run per hard line, made from the
//! lines the editor draws (`kivact-text`) and kept with them: an edit makes
//! again the runs of the lines it changed (and of the lines it moved),
//! and gives the others again as they were, so the platform is handed
//! only those (`kivact_types::SemanticsText`). Only the lines around the caret
//! and the lines drawn are published (`EditLines::published`), and the
//! editor's value is not (assistive technology reads the runs), so a long
//! document costs assistive technology those lines, not its text.
//!
//! Invariants:
//! - The runs come from the same shaping the editor is drawn with, placed
//!   where its lines are drawn (its text line, less its horizontal
//!   scroll).
//! - A run is one hard line with its own line break (a CRLF included); it
//!   keeps its identity with its line, and an unchanged line that did not
//!   move gives the same `Arc` again. Runs carry no start offset:
//!   `SemanticsText::start` is the scalar offset of the first, and each
//!   other starts where the one before it ends.
//! - The selection and the caret are scalar offsets of the whole text, cut
//!   to the runs published.
//! - An empty text (the placeholder showing) is one empty run at the real
//!   caret.
//!
//! Limits: a run is a whole hard line, not split where mixed bidi text
//! changes direction.

use std::sync::Arc;

use crate::model::Model;
use crate::model::interact::text_geometry::{caret_rect, input_scroll, text_line};
use kivact_wire::{NodeId, Point, Rect};

use kivact_types::{SemanticsText, SemanticsTextRun};

impl Model {
    pub(super) fn editing_semantics(
        &mut self,
        id: NodeId,
        offset: Point,
        engine: &mut arvane::Engine,
    ) -> SemanticsText {
        let caret = shifted(caret_rect(self, id, engine), offset.x, offset.y);
        let line = text_line(self, id, engine);
        let at = Point {
            x: line.x - input_scroll(self, id) + offset.x,
            y: line.y + offset.y,
        };
        // A new line's run takes its identity from the group's counter.
        let (tree, a11y) = (&self.tree, &mut self.a11y);
        let edit = tree
            .get(id)
            .and_then(|n| {
                self.editors[n.index.0]
                    .as_deref()
                    .map(|editor| editor.editor.state())
            })
            .expect("invariant: editing node has editing state");
        let value = edit.value();
        let mut ids = || a11y.next_identity(tree);
        let (mut lines, _) = crate::measure::input::edit_lines(
            tree,
            &self.measure,
            &self.editors,
            id,
            crate::measure::input::LineViewport::Paint(line.width),
            engine,
        )
        .expect("invariant: editing node has lines");
        let (runs, start, end) = if lines.shows_value() {
            let published = lines.published(value, edit.caret(), at, line.width, &mut ids, engine);
            (published.runs, published.start, published.end)
        } else {
            // An empty text: one empty run at the caret.
            let run = Arc::new(SemanticsTextRun {
                id: kivact_types::TextRunId(lines.run_id(0, &mut ids)),
                value: String::new(),
                bounds: caret,
                carets: Vec::new(),
            });
            (Arc::from([run]), 0, 0)
        };
        // The selection is cut to the runs published.
        let char_of = |at: usize| lines.char_of_byte(value, at).clamp(start, end);
        let selection = (char_of(edit.anchor()), char_of(edit.caret()));
        SemanticsText {
            runs,
            start,
            selection,
            caret,
        }
    }
}

impl Model {
    /// The text runs a node published itself, as the platform gets them: the
    /// rectangles moved from the node's own coordinates to `bounds`'s origin
    /// (the window's, scrolls undone), each run with a native identity
    /// of its own that stays with the author's key for the run, and no retained accepted payload. `None` when the text is not
    /// valid (it would make the whole tree invalid).
    pub(super) fn authored_semantics(
        &mut self,
        id: NodeId,
        authored: &SemanticsText,
        bounds: Rect,
    ) -> Option<SemanticsText> {
        kivact_wire::check_text(authored).ok()?;
        let (dx, dy) = (bounds.x, bounds.y);
        let runs: Vec<Arc<SemanticsTextRun>> = authored
            .runs
            .iter()
            .map(|run| {
                let key = (id, run.id.0);
                let native = match self.a11y.authored.get(&key) {
                    Some(native) => *native,
                    None => self.a11y.next_identity(&self.tree),
                };
                self.a11y.authored.insert(key, native);
                let shown = SemanticsTextRun {
                    id: kivact_types::TextRunId(native),
                    value: run.value.clone(),
                    bounds: shifted(run.bounds, dx, dy),
                    carets: run
                        .carets
                        .iter()
                        .map(|c| kivact_types::TextCaret {
                            offset: c.offset,
                            rect: shifted(c.rect, dx, dy),
                        })
                        .collect(),
                };
                Arc::new(shown)
            })
            .collect();
        Some(SemanticsText {
            runs: runs.into(),
            start: authored.start,
            selection: authored.selection,
            caret: shifted(authored.caret, dx, dy),
        })
    }
}

fn shifted(rect: Rect, x: f32, y: f32) -> Rect {
    Rect {
        x: rect.x + x,
        y: rect.y + y,
        ..rect
    }
}
