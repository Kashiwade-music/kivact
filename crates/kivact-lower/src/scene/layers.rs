//! Role: Lowering of local scene layers commands.
use super::state::{Frame, State};
use crate::context::Context;
use crate::sanitize;
use kivact_types::protocol::DrawCommand as D;
use kivact_types::{Effects, PaintProps, Point, Rect, SceneCmd};
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
        match cmd {
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
                state.frames.push(Frame::Layer {
                    pushed: layers.len(),
                    tint: state.current_tint,
                });
                state.current_tint.matrix = state
                    .current_tint
                    .then_own(crate::effects::color_matrix(&layer.filter));
                state.out.extend(layers.into_iter().map(D::PushLayer));
                true
            }
            SceneCmd::PopLayer => {
                if let Some(&Frame::Layer { pushed, tint }) = state.frames.last() {
                    state.frames.pop();
                    state.out.extend(std::iter::repeat_n(D::PopLayer, pushed));
                    state.current_tint = tint;
                } else {
                    self.drop_command();
                }
                true
            }
            SceneCmd::PushTransform(m) => {
                let [a, b, c, d, e, f] = *m;
                let transform =
                    kivact_types::protocol::Transform { a, b, c, d, e, f }.about(Point {
                        x: origin.x,
                        y: origin.y,
                    });
                state.frames.push(Frame::Transform {
                    raster: self.raster.get(),
                });
                self.raster
                    .set(self.raster.get() * crate::scene::raster_zoom(transform.max_scale()));
                state.out.push(D::PushTransform(transform));
                true
            }
            SceneCmd::PopTransform => {
                if let Some(&Frame::Transform { raster }) = state.frames.last() {
                    state.frames.pop();
                    self.raster.set(raster);
                    state.out.push(D::PopTransform);
                } else {
                    self.drop_command();
                }
                true
            }
            SceneCmd::PushClip(r) => {
                let command = D::PushClip(rect(*r));
                let drawn = sanitize::drawable(&command);
                if drawn {
                    state.out.push(command);
                } else {
                    self.drop_command();
                }
                state.frames.push(Frame::Clip { drawn });
                true
            }
            SceneCmd::PopClip => {
                if let Some(&Frame::Clip { drawn }) = state.frames.last() {
                    state.frames.pop();
                    if drawn {
                        state.out.push(D::PopClip);
                    }
                } else {
                    self.drop_command();
                }
                true
            }
            _ => false,
        }
    }
}
