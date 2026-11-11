//! Role: Native text geometry shared by layout reporting, editing and diagnostics.
//!
//! Text input geometry shared by caret painting, IME positioning and the
//! pointer: where the caret is, which caret the pointer is over, and the
//! horizontal scroll that keeps the caret inside the field. The editing
//! itself is the model's (`kivact_editing::EditState`).
//!
//! Invariants:
//! - Everything here reads the input's lines (`text_input`), shaped one
//!   hard line at a time: the caret, the pointer and the IME cost the
//!   lines they touch, not the text.
//! - Arvane's shaping carets carry Unicode scalar offsets; the
//!   editor's byte offsets are converted at this module's edges.
//! - The pointer only lands on extended grapheme cluster boundaries, of
//!   the line under it (the nearest line above or below the text).
//! - The IME is given the text around the caret, never more than
//!   2048 bytes (2048) on each side of the caret and
//!   the composition, whole lines while they fit, in offsets of that text:
//!   the selection and the composition are cut to it (a reconversion of a
//!   larger selection sees that part), and the IME's answers are moved
//!   back by where it starts in the value. A read-only input gives the IME
//!   nothing, a password input no text (its caret box only).
//! - Inputs never wrap (a multiline input breaks only at
//!   line feeds) and scroll horizontally to keep the caret within 2 px of
//!   the right edge, as far as the widest line shaped so far or the caret
//!   reaches. Text that cannot be shaped keeps the caret at the start.
//!   Only the inputs made so far are scrolled (`Edits::inputs`), no tree
//!   walk.
//! - An input is shaped in its inherited text style ([`Model::input_style`]):
//!   letter spacing and font features apply, decoration lines, ellipses,
//!   line clamps and case conversion do not (the caret is in the value as
//!   typed).
//! - The line height is the shaper's (the caret of an empty text), never a
//!   copy of its formula.
//! - A single-line input's line is centered vertically in its content box
//!   (like a browser's `<input>`; a box lower than the line overflows it
//!   evenly on both sides), a multiline input's starts at its top.
//!   `line_box` is the one place that decides it: painting the text, the
//!   selection, the caret and the placeholder, the pointer and the IME's
//!   character boxes all start from it. The IME's text area is the content
//!   box.

use std::ops::Range;

use crate::model::Model;
use kivact_types::{NodeId, Point, Rect};

use crate::model::interact::visual;

use crate::text_input::{line_height, line_offset};

/// Shape a non-editor text node with the same width allowance and
/// color-independent typography as the layout diagnostics.
pub(crate) fn shape_static_text(
    host: &Model,
    id: NodeId,
    engine: &mut arvane::Engine,
    area_width: f64,
) -> Option<kivact_types::ShapedTextSnapshot> {
    let node = host.node(id)?;
    let mut style = crate::lower::assets::text_style(engine, &host.text_props(id));
    style.color = kivact_types::ComputedTextStyle::default().color;
    style.decoration.color = None;
    let width = Some(area_width.max(0.0) + 1.0);
    let content = node.content().unwrap_or("");
    if node.spans().is_empty() {
        engine.shape_glyphs(content, &style, width).ok()
    } else {
        let spans = crate::measure::leaf::rich_spans(engine, node.spans(), &style);
        let text = kivact_types::render::RichText {
            text: content,
            spans: &spans,
        };
        engine.shape_glyphs(text, &style, width).ok()
    }
}

/// Whether native shaping cut any displayed line of an observed text node.
pub(crate) fn text_truncated(host: &Model, engine: &mut arvane::Engine, id: NodeId) -> bool {
    if !host
        .node(id)
        .is_some_and(|node| node.kind() == crate::contract::NodeType::Text)
    {
        return false;
    }
    let props = host.text_props(id);
    let more = &props;
    if more.max_lines.is_none() && more.overflow != kivact_types::TextOverflow::Ellipsis {
        return false;
    }
    shape_static_text(host, id, engine, visual::text_box(host, id).width)
        .is_some_and(|text| text.lines.iter().any(|line| line.cut))
}

/// The horizontal text scroll of a text input.
pub(crate) fn input_scroll(host: &Model, id: NodeId) -> f64 {
    host.editor_node(id).map_or(0.0, |n| n.scroll_x)
}

/// The caret of the text input `id` relative to its unscrolled text
/// origin (the start of its box and its first line's top).
fn caret_local(host: &Model, id: NodeId, engine: &mut arvane::Engine) -> Rect {
    let width = visual::text_box(host, id).width;
    let caret = host
        .node(id)
        .and_then(|n| {
            host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
        })
        .map_or(0, |e| e.caret());
    match host.edit_lines(id, engine) {
        Some((mut lines, text)) => lines.caret(text, caret, width, engine),
        None => Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: {
                let style = host.input_style(id, engine);
                line_height(engine, &style)
            },
        },
    }
}

/// Where the text of the text input `id` starts when its content box is
/// drawn at `area`: `area` itself for a multiline input, else the box of one
/// line (the shaper's line height) centered vertically in `area`. A content
/// box lower than the line overflows it evenly on both sides.
pub(crate) fn line_box(host: &Model, id: NodeId, engine: &mut arvane::Engine, area: Rect) -> Rect {
    if host.node(id).is_none_or(|n| n.multiline()) {
        let scroll_y = host.editor_node(id).map_or(0.0, |editor| editor.scroll_y);
        return Rect {
            y: area.y - scroll_y,
            ..area
        };
    }
    let style = host.input_style(id, engine);
    let height = line_height(engine, &style);
    Rect {
        y: area.y + line_offset(area.height, height),
        height,
        ..area
    }
}

/// Where the text of the text input `id` starts in its own space
/// (`line_box` of its content box).
pub(crate) fn text_line(host: &Model, id: NodeId, engine: &mut arvane::Engine) -> Rect {
    line_box(host, id, engine, visual::text_box(host, id))
}

/// The caret of the text input `id` in its own space, as painted.
pub(crate) fn caret_rect(host: &Model, id: NodeId, engine: &mut arvane::Engine) -> Rect {
    let line = text_line(host, id, engine);
    caret_in(host, id, engine, line)
}

/// The caret of the text input `id` whose text starts at the `line` box
/// (`line_box`).
pub(crate) fn caret_in(host: &Model, id: NodeId, engine: &mut arvane::Engine, line: Rect) -> Rect {
    let local = caret_local(host, id, engine);
    Rect {
        x: line.x + local.x - input_scroll(host, id),
        y: line.y + local.y,
        ..local
    }
}

/// The byte offset in the value of the text input `id` nearest the window
/// point `at` (mapped into its own space through its transforms), on a
/// grapheme boundary (`None`: it is no text input).
pub(crate) fn offset_at(
    host: &Model,
    id: NodeId,
    at: Point,
    engine: &mut arvane::Engine,
) -> Option<usize> {
    let origin = host.rect(id)?;
    let local = host.to_local(id, at)?;
    let at = Point {
        x: origin.x + local.x,
        y: origin.y + local.y,
    };
    let bounds = text_line(host, id, engine);
    let (mut lines, text) = host.edit_lines(id, engine)?;
    let x = at.x - bounds.x + input_scroll(host, id);
    Some(lines.offset_at(text, x, at.y - bounds.y, bounds.width, engine))
}

/// What the focused text input shows the IME: the text around its caret
/// (all of it when short: `EditLines::excerpt`), the selection (cut to
/// that text) and the composition in UTF-16 code units of it, the boxes
/// of the composed characters (the caret's without a composition) and its
/// text area, in window coordinates as painted: the bounds of each box
/// mapped by the input's transforms; with the byte offset the text starts
/// at in the value (`None`: no text input has the focus).
pub(crate) fn ime_text(
    host: &Model,
    engine: &mut arvane::Engine,
) -> Option<(arvane::ShellImeText, usize)> {
    let id = host.focused_input()?;
    let node = host.node(id)?;
    if node.read_only() {
        return None;
    }
    let edit = host.editors[node.index.0]
        .as_deref()
        .map(|editor| editor.editor.state())?;
    let value = edit.value();
    let area = visual::text_box(host, id);
    let line = line_box(host, id, engine, area);
    let left = area.x - input_scroll(host, id);
    let (mut lines, _) = host.edit_lines(id, engine)?;
    let (from, to) = edit
        .marked()
        .map_or((edit.caret(), edit.caret()), |m| (m.start, m.end));
    let mut place = |at: usize| {
        let r = lines.caret(value, at, area.width, engine);
        Rect {
            x: left + r.x,
            y: line.y + r.y,
            ..r
        }
    };
    let mut rects = Vec::new();
    let mut at = from;
    for c in value[from..to].chars() {
        let (start, end) = (place(at), place(at + c.len_utf8()));
        let rect = Rect {
            x: start.x.min(end.x),
            y: start.y,
            width: (end.x - start.x).abs().max(start.width),
            height: start.height,
        };
        rects.extend(std::iter::repeat_n(rect, c.len_utf16()));
        at += c.len_utf8();
    }
    rects.push(place(to));
    // Into the window, as drawn.
    for r in &mut rects {
        *r = host.drawn_rect(id, *r);
    }
    let area = host.drawn_rect(id, area);
    if node.password() {
        let text = arvane::ShellImeText {
            text: String::new(),
            text_start: 0,
            selection: (0, 0),
            caret: 0,
            composition: None,
            rects_from: 0,
            rects: vec![*rects.last().expect("invariant: the caret was appended")],
            area,
        };
        return Some((text, 0));
    }
    let excerpt = lines.excerpt(value, from.min(edit.caret())..to.max(edit.caret()));
    let start = lines.utf16_of_byte(value, excerpt.start);
    let units =
        |byte: usize| lines.utf16_of_byte(value, byte.clamp(excerpt.start, excerpt.end)) - start;
    let selection = edit.selection();
    let text = arvane::ShellImeText {
        text: value[excerpt.clone()].to_string(),
        text_start: start,
        selection: (units(selection.start), units(selection.end)),
        caret: units(edit.caret()),
        composition: edit.marked().map(|m| (units(m.start), units(m.end))),
        rects_from: units(from),
        rects,
        area,
    };
    Some((text, excerpt.start))
}

impl Model {
    /// Scroll every text input so its caret stays inside
    /// the field: past the right edge less 2 px, or at the left edge.
    pub(crate) fn scroll_inputs(&mut self, engine: &mut arvane::Engine) {
        for id in self.input_ids() {
            let area = visual::text_box(self, id);
            let width = area.width;
            // Hidden or zero-width fields have no viewport to scroll into.
            if width <= 0.0 {
                continue;
            }
            let caret = caret_local(self, id, engine);
            // Reveal or resize can fit the text again. Bound any retained
            // offset to the remaining overflow before keeping the caret visible.
            let (extent_x, extent_y) = self
                .edit_lines(id, engine)
                .filter(|(lines, _)| lines.shows_value())
                .map_or((0.0, 0.0), |(lines, _)| {
                    (lines.width(), lines.len() as f64 * lines.height())
                });
            let max_scroll = (extent_x.max(caret.x) - width + 2.0).max(0.0);
            let mut scroll = input_scroll(self, id).clamp(0.0, max_scroll);
            if caret.x - scroll > width - 2.0 {
                scroll = caret.x - width + 2.0;
            } else if caret.x - scroll < 0.0 {
                scroll = caret.x.max(0.0);
            }
            let vertical = self.node(id).is_some_and(|node| {
                let layout = node.edit_line_layout();
                layout.wrap || layout.rows.is_some()
            });
            if let Some(x) = self.editor_node_mut(id) {
                let scroll_y = if vertical {
                    let max = (extent_y - area.height).max(0.0);
                    x.scroll_y
                        .max(caret.bottom() - area.height)
                        .min(caret.y)
                        .clamp(0.0, max)
                } else {
                    0.0
                };
                if x.scroll_x != scroll || x.scroll_y != scroll_y {
                    x.scroll_x = scroll;
                    x.scroll_y = scroll_y;
                    self.invalidate_paint();
                }
            }
        }
        self.sync();
    }
}

/// The text input `id`'s lines `rows` (clamped to its lines) and their
/// carets in the window, as drawn: the lines of its value (none while it
/// is empty) and their caret stops, one per scalar offset (the CR of a
/// CRLF at its line's end). Only those lines are shaped and read.
pub(crate) fn text_layout(
    host: &Model,
    id: NodeId,
    rows: Range<usize>,
    engine: &mut arvane::Engine,
) -> Option<kivact_editing::TextLayout> {
    layout_snapshot(host, id, rows, engine, false)
}

pub(crate) fn layout_snapshot(
    host: &Model,
    id: NodeId,
    rows: Range<usize>,
    engine: &mut arvane::Engine,
    canvas_coordinates: bool,
) -> Option<kivact_editing::TextLayout> {
    use kivact_editing::{EditCaret, EditLine, TextLayout};
    let value = host.edit(id)?.value();
    let origin = text_line(host, id, engine);
    let scroll = input_scroll(host, id);
    let canvas = visual::padding_box(host.rect(id)?, &visual::paint(host, id));
    let mapped = |rect| map_snapshot_rect(host, id, rect, canvas_coordinates, canvas);
    let caret = mapped(caret_rect(host, id, engine));
    let (mut lines, _) = host.edit_lines(id, engine)?;
    let mut out = TextLayout {
        caret,
        selection: Vec::new(),
        line_count: 0,
        lines: Vec::new(),
        carets: Vec::new(),
    };
    if !lines.shows_value() {
        out.carets.push(EditCaret {
            offset: 0,
            rect: caret,
        });
        return Some(out);
    }
    out.line_count = lines.len();
    let rows = rows.start.min(lines.len())..rows.end.min(lines.len());
    let mut offset = lines.char_of_line(rows.start);
    for i in rows {
        let x = origin.x - scroll + lines.offset(value, i, origin.width, engine);
        let y = origin.y + i as f64 * lines.height();
        let shift = |r: Rect| {
            mapped(Rect {
                x: x + r.x,
                y: y + r.y,
                ..r
            })
        };
        let selection = host.edit(id)?.selection();
        for rect in lines.spans(
            value,
            i,
            (selection.start, selection.end),
            origin.width,
            engine,
        ) {
            out.selection.push(mapped(Rect {
                x: origin.x - scroll + rect.x,
                y: origin.y + rect.y,
                ..rect
            }));
        }
        let range = lines.range(i);
        let chars = value[range.clone()].chars().count() as u32;
        let shaped = lines.shaped(value, i, engine);
        let rect = shaped.as_ref().and_then(|s| s.lines.first()).map_or(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: lines.height(),
            },
            |l| l.rect,
        );
        out.lines.push(EditLine {
            range: range.clone(),
            rect: shift(rect),
            baseline: mapped(Rect {
                x,
                y: y + lines.baseline(value, i, engine),
                width: 0.0,
                height: 0.0,
            })
            .y,
        });
        if let Some(shaped) = shaped {
            append_snapshot_carets(&mut out, &shaped, shift, offset, lines.crlf(i));
        }
        offset += chars;
    }
    Some(out)
}

fn append_snapshot_carets(
    out: &mut kivact_editing::TextLayout,
    shaped: &kivact_types::render::ShapedText,
    shift: impl Fn(Rect) -> Rect,
    offset: u32,
    crlf: bool,
) {
    use kivact_editing::EditCaret;
    let mut end = None;
    for c in &shaped.carets {
        let rect = shift(c.rect);
        end = Some(rect);
        out.carets.push(EditCaret {
            offset: offset + c.offset,
            rect,
        });
    }
    let content = shaped.carets.len().saturating_sub(1) as u32;
    if crlf && let Some(rect) = end {
        out.carets.push(EditCaret {
            offset: offset + content + 1,
            rect,
        });
    }
}

fn map_snapshot_rect(
    host: &Model,
    id: NodeId,
    rect: Rect,
    canvas_coordinates: bool,
    canvas: Rect,
) -> Rect {
    if canvas_coordinates {
        Rect {
            x: rect.x - canvas.x,
            y: rect.y - canvas.y,
            ..rect
        }
    } else {
        host.drawn_rect(id, rect)
    }
}
