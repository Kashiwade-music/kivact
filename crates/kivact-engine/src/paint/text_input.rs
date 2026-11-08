//! Role: Render prepared native editor text, selection, caret and decoration facts.

use crate::lower::effects::Tint;
use crate::lower::input_paint::{InputPaintColor, InputPaintOp};
use crate::model::Model;
use kivact_types::render::{Paint, RenderCmd as D};
use kivact_types::{NodeId, Rect, Rgba};

impl Model {
    pub(super) fn paint_input(
        &self,
        id: NodeId,
        bounds: Rect,
        clip: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut crate::lower::SceneOut,
    ) {
        let Some(prepared) = self.prepare_input_paint(id, bounds, clip, engine) else {
            return;
        };
        out.push(D::PushClip(prepared.area));
        for op in prepared.ops {
            paint_op(op, tint, out);
        }
        out.push(D::PopClip);
    }
}

fn paint_op(op: InputPaintOp, tint: &Tint, out: &mut crate::lower::SceneOut) {
    match op {
        InputPaintOp::Rect { rect, color } => out.push(D::Rect {
            rect,
            fill: kivact_types::render::Paint::Solid(match color {
                InputPaintColor::Source(color) => tint.color(color),
                InputPaintColor::Render(color) => tint.arvane(color),
            }),
        }),
        InputPaintOp::Line {
            from,
            to,
            width,
            color,
        } => out.push(D::Line {
            from,
            to,
            width,
            color: tint.color(color),
        }),
        InputPaintOp::RangeDecoration { rect, kind, color } => {
            paint_decoration(out, rect, kind, tint.color(color))
        }
        InputPaintOp::Decoration { rect, color, style } => crate::lower::glyphs::decoration(
            out,
            rect,
            kivact_types::render::Paint::Solid(tint.color(color)),
            style,
        ),
        InputPaintOp::Glyphs {
            shaped,
            origin,
            color,
        } => {
            for run in &shaped.runs {
                out.push(D::Glyphs {
                    run: run.clone(),
                    origin,
                    fill: Some(Paint::Solid(match color {
                        Some(color) => tint.color(color),
                        None => tint.arvane(run.color),
                    })),
                    stroke: None,
                    order: kivact_types::PaintOrder::default(),
                });
            }
        }
    }
}

fn paint_decoration(
    out: &mut crate::lower::SceneOut,
    rect: Rect,
    kind: kivact_editing::RangeDecorationKind,
    color: Rgba,
) {
    use kivact_editing::RangeDecorationKind;
    let fill = Paint::Solid(color);
    match kind {
        RangeDecorationKind::Background => out.push(D::Rect { rect, fill }),
        RangeDecorationKind::Underline(style) => crate::lower::glyphs::decoration(
            out,
            Rect {
                y: rect.bottom() - 1.0,
                height: 1.0,
                ..rect
            },
            fill,
            style,
        ),
        RangeDecorationKind::Outline => {
            for band in [
                Rect {
                    height: 1.0,
                    ..rect
                },
                Rect {
                    y: rect.bottom() - 1.0,
                    height: 1.0,
                    ..rect
                },
                Rect { width: 1.0, ..rect },
                Rect {
                    x: rect.right() - 1.0,
                    width: 1.0,
                    ..rect
                },
            ] {
                out.push(D::Rect {
                    rect: band,
                    fill: fill.clone(),
                });
            }
        }
    }
}
