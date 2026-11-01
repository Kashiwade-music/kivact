//! Role: Lowering of local scene geometry commands.
use super::state::State;
use crate::context::Context;
use crate::sanitize;
use crate::shape::BoxShape;
use kivact_types::protocol::DrawCommand as D;
use kivact_wire::{DrawingShape, Fill, Point, Rect, SceneCmd};
impl Context<'_> {
    pub(super) fn paint_geometry(&self, cmd: &SceneCmd, state: &mut State<'_>) -> bool {
        let origin = state.origin;
        let point = |p: Point| Point {
            x: origin.x + p.x,
            y: origin.y + p.y,
        };
        let rect = |r: Rect| Rect {
            x: origin.x + r.x,
            y: origin.y + r.y,
            width: r.width,
            height: r.height,
        };
        let origin_point = Point {
            x: origin.x,
            y: origin.y,
        };
        let tint = &state.current_tint;
        let command = match cmd {
            SceneCmd::Shape(painted) => {
                let shape = crate::scene_shape::shape(&painted.shape);
                // Before lowering arcs and before any paint resource.
                if let kivact_types::protocol::Shape::Path { cmds, .. } = &shape
                    && !sanitize::path(cmds)
                {
                    self.drop_command();
                    return true;
                }
                let shape = crate::path::shifted(
                    shape,
                    Point {
                        x: origin.x,
                        y: origin.y,
                    },
                );
                let mut area = match &painted.shape {
                    DrawingShape::Basic { rect: bounds, .. }
                    | DrawingShape::Rounded { rect: bounds, .. } => rect(*bounds),
                    DrawingShape::Path { .. } => shape.bounds(),
                };
                if let Some(stroke) = &painted.stroke {
                    // A straight line still has a nonzero paint area.
                    let pad = stroke.width.max(0.0) / 2.0;
                    if area.width == 0.0 {
                        area.x -= pad;
                        area.width = 2.0 * pad;
                    }
                    if area.height == 0.0 {
                        area.y -= pad;
                        area.height = 2.0 * pad;
                    }
                }
                let mut push = |paint| {
                    state.out.commands.push(match &painted.stroke {
                        Some(stroke) => D::StrokeShape {
                            shape: shape.clone(),
                            stroke: stroke.clone(),
                            paint,
                        },
                        None => D::FillShape {
                            shape: shape.clone(),
                            paint,
                        },
                    })
                };
                match &painted.paint {
                    Fill::Solid(color) => {
                        push(kivact_types::protocol::Paint::Solid(tint.color(*color)))
                    }
                    Fill::Layers(layers) => {
                        for layer in layers.iter().rev() {
                            match self.layer_paint(layer, area, origin_point) {
                                Some(paint) if sanitize::fill(&paint) => {
                                    push(tint.paint(paint));
                                }
                                Some(_) => self.drop_command(),
                                None => {}
                            }
                        }
                    }
                }
                return true;
            }
            SceneCmd::Rect { rect: r, fill } => {
                let r = rect(*r);
                if !sanitize::drawable(&D::PushClip(r)) {
                    self.drop_command();
                } else if let Fill::Solid(color) = fill {
                    // As before fills had layers.
                    state.out.commands.push(D::Rect {
                        rect: r,
                        fill: kivact_types::protocol::Paint::Solid(tint.color(*color)),
                    });
                } else {
                    self.fill(state.out, &BoxShape::Rect(r), fill, tint, origin_point);
                }
                return true;
            }
            SceneCmd::RoundedRect {
                rect: r,
                radius,
                fill,
            } => {
                let r = rect(*r);
                if sanitize::drawable(&D::PushClip(r)) && radius.is_finite() && *radius >= 0.0 {
                    self.fill(
                        state.out,
                        &BoxShape::Circular(r, [*radius; 4]),
                        fill,
                        tint,
                        origin_point,
                    );
                } else {
                    self.drop_command();
                }
                return true;
            }
            SceneCmd::StrokeRect {
                rect: r,
                width,
                color,
                radius,
            } => D::StrokeRect {
                rect: rect(*r),
                width: *width,
                color: tint.color(*color),
                radius: *radius,
            },
            SceneCmd::Line {
                from,
                to,
                width,
                color,
            } => D::Line {
                from: point(*from),
                to: point(*to),
                width: *width,
                color: tint.color(*color),
            },

            _ => return false,
        };
        if sanitize::drawable(&command) {
            state.clips += usize::from(matches!(command, D::PushClip(_)));
            state.out.commands.push(command);
        } else {
            self.drop_command();
        }
        true
    }
}
