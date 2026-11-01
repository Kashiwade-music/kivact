//! Role: Lowering of local scene layers commands.
use super::state::State;
use crate::context::Context;
use crate::sanitize;
use kivact_types::protocol::DrawCommand as D;
use kivact_wire::{Effects, PaintProps, Point, Rect, SceneCmd};
impl Context<'_> {
    pub(super) fn paint_layers(&self, cmd: &SceneCmd, state: &mut State<'_>) -> bool {
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
            SceneCmd::PushLayer(layer) => {
                let area = rect(layer.area);
                let outer = kivact_types::protocol::Layer {
                    clip: layer.clip.as_ref().map(|shape| {
                        crate::path::shifted(
                            crate::scene_shape::shape(shape),
                            Point {
                                x: origin.x,
                                y: origin.y,
                            },
                        )
                    }),
                    mask: layer
                        .mask
                        .as_ref()
                        .and_then(|mask| self.layer_paint(mask, area, origin_point))
                        .filter(|paint| {
                            let valid = sanitize::fill(paint);
                            if !valid {
                                self.drop_command();
                            }
                            valid
                        })
                        .map(|paint| kivact_types::protocol::Mask { rect: area, paint }),
                    ..kivact_types::protocol::Layer::default()
                };
                let props = PaintProps {
                    effects: Some(Box::new(Effects {
                        filter: layer.filter.clone(),
                        blend: layer.blend,
                        ..Effects::default()
                    })),
                    ..PaintProps::default()
                };
                let layers = crate::effects::layers(
                    &props,
                    outer,
                    layer.opacity.clamp(0.0, 1.0),
                    tint.matrix.as_ref(),
                );
                state.layer_stack.push((layers.len(), state.current_tint));
                state.current_tint.matrix = state
                    .current_tint
                    .then_own(crate::effects::color_matrix(&layer.filter));
                state
                    .out
                    .commands
                    .extend(layers.into_iter().map(D::PushLayer));
                return true;
            }
            SceneCmd::PopLayer => {
                if let Some((count, old)) = state.layer_stack.pop() {
                    state
                        .out
                        .commands
                        .extend(std::iter::repeat_n(D::PopLayer, count));
                    state.current_tint = old;
                }
                return true;
            }
            SceneCmd::PushTransform(m) => {
                let [a, b, c, d, e, f] = *m;
                let transform =
                    kivact_types::protocol::Transform { a, b, c, d, e, f }.about(Point {
                        x: origin.x,
                        y: origin.y,
                    });
                state.transforms += 1;
                state.raster_stack.push(self.raster.get());
                self.raster
                    .set(self.raster.get() * crate::scene::raster_zoom(transform.max_scale()));
                state.out.commands.push(D::PushTransform(transform));
                return true;
            }
            SceneCmd::PopTransform => {
                if state.transforms > 0 {
                    state.transforms -= 1;
                    if let Some(raster) = state.raster_stack.pop() {
                        self.raster.set(raster);
                    }
                    state.out.commands.push(D::PopTransform);
                }
                return true;
            }
            SceneCmd::PushClip(r) => D::PushClip(rect(*r)),
            SceneCmd::PopClip => {
                if state.clips > 0 {
                    state.clips -= 1;
                    state.out.commands.push(D::PopClip);
                }
                return true;
            }
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
