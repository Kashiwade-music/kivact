//! Role: Lowering of local scene geometry commands.
use super::state::State;
use crate::lower::border_image::lower_border_image;
use crate::lower::context::Context;
use crate::lower::fill::Boxes;
use crate::lower::sanitize;
use crate::lower::shape::BoxShape;
use kivact_types::render::RenderCmd as D;
use kivact_types::{DrawCmd, DrawingShape, Edges, Fill, Point, Rect};
impl Context<'_> {
    pub(super) fn paint_geometry(&self, cmd: &DrawCmd, state: &mut State<'_>) -> bool {
        let origin = state.origin;
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
            DrawCmd::Shape(_) => {
                self.paint_shape(cmd, state);
                return true;
            }
            DrawCmd::Rect { rect: r, fill } => {
                let r = rect(*r);
                if !sanitize::drawable(&D::PushClip(r)) {
                    self.drop_command();
                } else if let Fill::Solid(color) = fill {
                    // As before fills had layers.
                    state.out.push(D::Rect {
                        rect: r,
                        fill: kivact_types::render::Paint::Solid(tint.color(*color)),
                    });
                } else {
                    let boxes = Boxes::uniform(BoxShape::Rect(r));
                    self.fill_items(state.out, &boxes, fill, tint, origin_point);
                }
                return true;
            }
            DrawCmd::RoundedRect {
                rect: r,
                radius,
                fill,
            } => {
                let r = rect(*r);
                if sanitize::drawable(&D::PushClip(r)) && radius.is_finite() && *radius >= 0.0 {
                    self.fill_items(
                        state.out,
                        &Boxes::uniform(BoxShape::Circular(r, [*radius; 4])),
                        fill,
                        tint,
                        origin_point,
                    );
                } else {
                    self.drop_command();
                }
                return true;
            }
            DrawCmd::StrokeRect {
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
            DrawCmd::Line {
                from,
                to,
                width,
                color,
            } => D::Line {
                from: shifted_point(*from, origin),
                to: shifted_point(*to, origin),
                width: *width,
                color: tint.color(*color),
            },

            DrawCmd::BorderImage { rect: r, image } => {
                lower_border_image(state.out, image, rect(*r), Edges::all(0.0), self);
                return true;
            }

            _ => return false,
        };
        if sanitize::drawable(&command) {
            state.out.push(command);
        } else {
            self.drop_command();
        }
        true
    }
    fn paint_shape(&self, cmd: &DrawCmd, state: &mut State<'_>) {
        let DrawCmd::Shape(painted) = cmd else {
            unreachable!("invariant: matching command family");
        };
        let origin = state.origin;
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

        let shape = crate::lower::scene_shape::shape(&painted.shape);
        // Before lowering arcs and before any paint resource.
        if let kivact_types::scene::Shape::Path { cmds, .. } = &shape
            && !sanitize::path(cmds)
        {
            self.drop_command();
            return;
        }
        let shape = crate::lower::path::shifted(
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
            state.out.push(match &painted.stroke {
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
            Fill::Solid(color) => push(kivact_types::render::Paint::Solid(tint.color(*color))),
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
    }
}

fn shifted_point(p: Point, origin: Rect) -> Point {
    Point {
        x: origin.x + p.x,
        y: origin.y + p.y,
    }
}
