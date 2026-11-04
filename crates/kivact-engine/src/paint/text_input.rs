//! Role: Paint a native text input: its text, selection, caret and preedit.

use crate::model::Model;
use crate::model::interact::{text_geometry, visual};
use kivact_lower::effects::Tint;
use kivact_types::render::RenderCmd as D;
use kivact_types::{NodeId, Point, Rect, Rgba};

impl Model {
    /// A text input: the lines of its value (the placeholder in its color,
    /// else at half opacity, while empty) that reach `clip` within its
    /// content box, scrolled to its caret, and while focused its selection,
    /// caret and composition underline on them. Its lines are shaped one at
    /// a time and shared with the caret, the pointer and the IME
    /// (`kivact-text`); its line colors only change the color of its glyph
    /// runs (a password and a placeholder have none), and are taken only
    /// as far as the lines drawn.
    pub(super) fn paint_input(
        &self,
        id: NodeId,
        bounds: Rect,
        clip: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut kivact_lower::SceneOut,
    ) {
        let host = &self;
        let Some(n) = host.node(id) else {
            return;
        };
        let props = host.text_style(id);
        let color = props.color;
        let layout = host.computed_layout(id).unwrap_or_default();
        let area = visual::content_at(bounds, layout.content_box);
        let line = text_geometry::line_box(host, id, engine, area);
        let left = area.x - text_geometry::input_scroll(host, id);
        let Some((mut lines, shown)) = host.edit_lines(id, engine) else {
            return;
        };
        let view = clip.intersect(&area);
        let rows = lines.rows(view.y - line.y, view.bottom() - line.y);
        lines.drawn(rows.clone());
        let place = |r: Rect| Rect {
            x: left + r.x,
            y: line.y + r.y,
            ..r
        };
        out.push(D::PushClip(area));
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
            let fill = kivact_types::render::Paint::Solid(match props.selection_color {
                Some(own) => tint.color(own),
                None => tint.arvane(Rgba {
                    r: 0.2,
                    g: 0.45,
                    b: 0.85,
                    a: 0.3,
                }),
            });
            for rect in spans(edit.selection(), engine) {
                out.push(D::Rect {
                    rect,
                    fill: fill.clone(),
                });
            }
            let marked = edit.marked().map(|m| spans(m, engine));
            out.push(D::Rect {
                rect: place(lines.caret(value, edit.caret(), area.width, engine)),
                fill: kivact_types::render::Paint::Solid(
                    tint.color(props.caret_color.unwrap_or(color)),
                ),
            });
            if let Some(marked) = marked {
                for band in marked {
                    out.push(D::Line {
                        from: Point {
                            x: band.x,
                            y: band.bottom(),
                        },
                        to: Point {
                            x: band.right(),
                            y: band.bottom(),
                        },
                        width: 1.0,
                        color: tint.color(color),
                    });
                }
            }
        }
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
            let (x, y) = (
                left + lines.offset(shown, i, area.width, engine),
                line.y + i as f64 * lines.height(),
            );
            match colors
                .as_ref()
                .and_then(|_| lines.painted(shown, i, base, engine))
            {
                // Each run keeps its span's color, drawn through the tint.
                Some(painted) => push_runs(out, Some(&painted), x, y, |run| tint.arvane(run)),
                None => {
                    let color = tint.color(color);
                    let shaped = lines.shaped(shown, i, engine);
                    push_runs(out, shaped.as_deref(), x, y, |_| color);
                }
            }
        }
        out.push(D::PopClip);
    }
}

/// Glyph runs of `shaped` at `(x, y)`, colored `color` of their shaped
/// color.
fn push_runs(
    out: &mut kivact_lower::SceneOut,
    shaped: Option<&kivact_types::render::ShapedText>,
    x: f64,
    y: f64,
    color: impl Fn(kivact_types::Rgba) -> kivact_types::Rgba,
) {
    for run in shaped.iter().flat_map(|shaped| &shaped.runs) {
        out.push(D::Glyphs {
            run: std::sync::Arc::clone(run),
            origin: Point { x, y },
            color: Some(color(run.color)),
        });
    }
}
