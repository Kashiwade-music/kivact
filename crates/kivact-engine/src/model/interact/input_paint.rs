//! Role: Prepare native editor drawing facts from a single borrowed line snapshot.

use super::{text_geometry, visual};
use crate::lower::input_paint::{InputPaintColor, InputPaintOp, PreparedInputPaint};
use crate::model::Model;
use kivact_types::{NodeId, Point, Rect, Rgba};

impl Model {
    pub(crate) fn prepare_input_paint(
        &self,
        id: NodeId,
        bounds: Rect,
        clip: Rect,
        engine: &mut arvane::Engine,
    ) -> Option<PreparedInputPaint> {
        let host = &self;
        let n = host.node(id)?;
        let props = host.text_style(id);
        let layout = host.computed_layout(id).unwrap_or_default();
        let area = visual::content_at(bounds, layout.content_box);
        let line = text_geometry::line_box(host, id, engine, area);
        let left = area.x - text_geometry::input_scroll(host, id);
        let (mut lines, shown) = host.edit_lines(id, engine)?;
        let view = clip.intersect(&area);
        let rows = lines.rows(view.y - line.y, view.bottom() - line.y);
        lines.drawn(rows.clone());
        let mut ops = Vec::new();
        let frame = InputPaint {
            host,
            id,
            n,
            props: &props,
            area,
            line,
            left,
        };
        frame.selection(&mut lines, rows.clone(), engine, &mut ops);
        frame.glyphs(&mut lines, shown, rows, engine, &mut ops);
        Some(PreparedInputPaint { area, ops })
    }
}

fn clamp_range(text: &str, range: &std::ops::Range<usize>) -> std::ops::Range<usize> {
    let floor = |offset: usize| {
        let mut offset = offset.min(text.len());
        while !text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    };
    floor(range.start)..floor(range.end)
}

struct InputPaint<'a> {
    host: &'a Model,
    id: NodeId,
    n: &'a crate::tree::node::HostNode,
    props: &'a kivact_types::ComputedTextStyle,
    area: Rect,
    line: Rect,
    left: f64,
}
impl InputPaint<'_> {
    fn selection(
        &self,
        lines: &mut crate::text_input::EditLines,
        rows: std::ops::Range<usize>,
        engine: &mut arvane::Engine,
        out: &mut Vec<InputPaintOp>,
    ) {
        let Self {
            host,
            id,
            n,
            props,
            area,
            line,
            left,
        } = *self;
        let color = props.color;
        let place = |r: Rect| Rect {
            x: left + r.x,
            y: line.y + r.y,
            ..r
        };
        if host.focused()
            == Some(kivact_types::FocusTarget {
                node: id,
                region: None,
            })
            && let Some(edit) = host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
        {
            let value = edit.value();
            // The shared selection bands of `range` on the drawn lines.
            let mut spans = |range: std::ops::Range<usize>, engine: &mut arvane::Engine| {
                let mut out = Vec::new();
                for i in rows.clone() {
                    let reached = lines
                        .content(i)
                        .is_some_and(|l| range.start <= l.end && range.end >= l.start);
                    if reached {
                        let bands =
                            lines.spans(value, i, (range.start, range.end), area.width, engine);
                        out.extend(bands.into_iter().map(place));
                    }
                }
                out
            };
            let fill = match props.selection_color {
                Some(own) => InputPaintColor::Source(own),
                None => InputPaintColor::Render(Rgba {
                    r: 0.2,
                    g: 0.45,
                    b: 0.85,
                    a: 0.3,
                }),
            };
            for rect in spans(edit.selection(), engine) {
                out.push(InputPaintOp::Rect { rect, color: fill });
            }
            let marked = edit.marked().map(|m| spans(m, engine));
            out.push(InputPaintOp::Rect {
                rect: place(lines.caret(value, edit.caret(), area.width, engine)),
                color: InputPaintColor::Source(props.caret_color.unwrap_or(color)),
            });
            if let Some(marked) = marked {
                for band in marked {
                    out.push(InputPaintOp::Line {
                        from: Point {
                            x: band.x,
                            y: band.bottom(),
                        },
                        to: Point {
                            x: band.right(),
                            y: band.bottom(),
                        },
                        width: 1.0,
                        color,
                    });
                }
            }
        }
    }
    fn glyphs(
        &self,
        lines: &mut crate::text_input::EditLines,
        shown: &str,
        rows: std::ops::Range<usize>,
        engine: &mut arvane::Engine,
        out: &mut Vec<InputPaintOp>,
    ) {
        let Self {
            n,
            props,
            area,
            line,
            left,
            ..
        } = *self;
        let color = props.color;
        // Placeholders are drawn in their color, else at half opacity.
        let color = if lines.shows_value() {
            color
        } else {
            props
                .placeholder_color
                .unwrap_or_else(|| color.with_alpha_mul(0.5))
        };
        let colors = n
            .input_data()
            .and_then(|data| data.colors.clone())
            .filter(|_| lines.shows_value() && !n.password());
        match &colors {
            Some(colors) => lines.color(shown, colors, rows.end),
            None => lines.uncolor(),
        }
        let base = kivact_types::Rgba::sanitize(color);
        for i in rows {
            self.line_decorations(lines, shown, i, engine, out);
            let (x, y) = (
                left + lines.offset(shown, i, area.width, engine),
                line.y + i as f64 * lines.height(),
            );
            match colors
                .as_ref()
                .and_then(|_| lines.painted(shown, i, base, engine))
            {
                // Each run keeps its span's color, drawn through the tint.
                Some(painted) => {
                    out.push(InputPaintOp::Glyphs {
                        shaped: painted.clone(),
                        origin: Point { x, y },
                        color: None,
                    });
                    // Decoration geometry comes from the same shaped ranges as the glyphs.
                    for decoration in &painted.decorations {
                        let rect = Rect {
                            x: x + decoration.rect.x,
                            y: y + decoration.rect.y,
                            ..decoration.rect
                        };
                        out.push(InputPaintOp::Decoration {
                            rect,
                            color: decoration.color,
                            style: decoration.style,
                        });
                    }
                }
                None => {
                    let shaped = lines.shaped(shown, i, engine);
                    if let Some(shaped) = shaped {
                        out.push(InputPaintOp::Glyphs {
                            shaped,
                            origin: Point { x, y },
                            color: Some(color),
                        });
                    }
                }
            }
        }
    }
    fn line_decorations(
        &self,
        lines: &mut crate::text_input::EditLines,
        shown: &str,
        i: usize,
        engine: &mut arvane::Engine,
        out: &mut Vec<InputPaintOp>,
    ) {
        let Self {
            n,
            area,
            line,
            left,
            ..
        } = *self;
        let place = |r: Rect| Rect {
            x: left + r.x,
            y: line.y + r.y,
            ..r
        };
        let content = lines.content(i).unwrap_or(0..0);
        let decorations = n
            .input_data()
            .and_then(|data| data.decorations.as_deref())
            .unwrap_or(&[]);
        let backgrounds = lines
            .line(i)
            .and_then(|(line, _)| line.colors())
            .map(|colors| colors.spans.clone())
            .unwrap_or_default();
        for style in backgrounds {
            if let Some(color) = style.background {
                for rect in lines.spans(
                    shown,
                    i,
                    (
                        content.start + style.range.start,
                        content.start + style.range.end,
                    ),
                    area.width,
                    engine,
                ) {
                    out.push(InputPaintOp::Rect {
                        rect: place(rect),
                        color: InputPaintColor::Source(color),
                    });
                }
            }
        }
        if lines.shows_value() && !n.password() {
            for decoration in decorations {
                let range = clamp_range(shown, &decoration.range);
                if range.start >= range.end
                    || range.start > content.end
                    || range.end < content.start
                {
                    continue;
                }
                for rect in lines.spans(shown, i, (range.start, range.end), area.width, engine) {
                    out.push(InputPaintOp::RangeDecoration {
                        rect: place(rect),
                        kind: decoration.kind,
                        color: decoration.color,
                    });
                }
            }
        }
    }
}
