//! Role: Lowering of local scene geometry commands.
use super::state::State;
use crate::lower::border_image::lower_border_image;
use crate::lower::context::Context;
use crate::lower::fill::Boxes;
use crate::lower::sanitize;
use crate::lower::shape::BoxShape;
use arvane_protocol::RenderCmd as D;
use kivact_types::{DrawCmd, Edges, Fill, Point, Rect};
impl Context<'_> {
    pub(super) fn paint_geometry(&self, cmd: &DrawCmd, state: &mut State<'_>) -> bool {
        let origin = state.origin;
        let rect = |r: Rect| Rect {
            x: origin.x + r.x,
            y: origin.y + r.y,
            width: r.width,
            height: r.height,
        };
        let tint = &state.current_tint;
        let command = match cmd {
            DrawCmd::Shape(_) => {
                self.paint_shape(cmd, state);
                return true;
            }
            DrawCmd::Rect { rect: r, fill } => {
                self.paint_rect(rect(*r), fill, state);
                return true;
            }
            DrawCmd::RoundedRect {
                rect: r,
                radius,
                fill,
            } => {
                self.paint_rounded_rect(rect(*r), *radius, fill, state);
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

    /// Paint the filled rectangle `r`, already in scene space, with `fill`.
    fn paint_rect(&self, r: Rect, fill: &Fill, state: &mut State<'_>) {
        let tint = &state.current_tint;
        if !sanitize::drawable(&D::PushClip(r)) {
            self.drop_command();
        } else if let Fill::Solid(color) = fill {
            // As before fills had layers.
            state.out.push(D::Rect {
                rect: r,
                fill: arvane_protocol::RasterPaint::Solid(tint.color(*color)),
            });
        } else {
            let boxes = Boxes::uniform(BoxShape::Rect(r));
            self.fill_items(state.out, &boxes, fill, tint, origin_point_of(state.origin));
        }
    }

    /// Paint the rounded rectangle `r`, already in scene space, with corners of `radius` and `fill`.
    fn paint_rounded_rect(&self, r: Rect, radius: f64, fill: &Fill, state: &mut State<'_>) {
        let tint = &state.current_tint;
        if sanitize::drawable(&D::PushClip(r)) && radius.is_finite() && radius >= 0.0 {
            let boxes = Boxes::uniform(BoxShape::Circular(r, [radius; 4]));
            self.fill_items(state.out, &boxes, fill, tint, origin_point_of(state.origin));
        } else {
            self.drop_command();
        }
    }

    fn paint_shape(&self, cmd: &DrawCmd, state: &mut State<'_>) {
        let DrawCmd::Shape(painted) = cmd else {
            unreachable!("invariant: matching command family");
        };
        let origin = state.origin;
        let origin_point = Point {
            x: origin.x,
            y: origin.y,
        };
        let tint = &state.current_tint;

        let shape = painted.shape.to_shape();
        // Before lowering arcs and before any paint resource.
        if let kivact_types::scene::Shape::Path { cmds, .. } = &shape
            && !sanitize::path(cmds)
        {
            self.drop_command();
            return;
        }
        let shape = shape.translated(Point {
            x: origin.x,
            y: origin.y,
        });
        let local = painted.paint_area();
        let area = Rect {
            x: origin.x + local.x,
            y: origin.y + local.y,
            width: local.width,
            height: local.height,
        };
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
            Fill::Solid(color) => push(arvane_protocol::RasterPaint::Solid(tint.color(*color))),
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

/// The origin of a layer as a point.
fn origin_point_of(origin: Rect) -> Point {
    Point {
        x: origin.x,
        y: origin.y,
    }
}
