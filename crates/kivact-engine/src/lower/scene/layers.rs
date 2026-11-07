//! Role: Lowering of local scene layers commands.
use super::state::{Frame, State};
use crate::lower::context::Context;
use crate::lower::sanitize;
use kivact_types::render::{Group, RenderCmd as D};
use kivact_types::{DrawCmd, Point, Rect};
impl Context<'_> {
    pub(super) fn paint_layers(&self, cmd: &DrawCmd, state: &mut State<'_>) -> bool {
        let origin = state.origin;
        let rect = |r: Rect| Rect {
            x: origin.x + r.x,
            y: origin.y + r.y,
            width: r.width,
            height: r.height,
        };
        match cmd {
            DrawCmd::PushLayer(_) => {
                self.push_layer(cmd, state);
                true
            }
            DrawCmd::PopLayer => {
                if let Some(Frame::Layer) = state.frames.last() {
                    state.frames.pop();
                    state.out.push(D::PopGroup);
                } else {
                    self.drop_command();
                }
                true
            }
            DrawCmd::PushTransform(m) => {
                let transform = m.about(Point {
                    x: origin.x,
                    y: origin.y,
                });
                state.frames.push(Frame::Transform {
                    raster: self.raster.get(),
                });
                self.raster.set(
                    self.raster.get() * crate::lower::scene::raster_zoom(transform.max_scale()),
                );
                state.out.push(D::PushTransform(transform));
                true
            }
            DrawCmd::PopTransform => {
                if let Some(&Frame::Transform { raster }) = state.frames.last() {
                    state.frames.pop();
                    self.raster.set(raster);
                    state.out.push(D::PopTransform);
                } else {
                    self.drop_command();
                }
                true
            }
            DrawCmd::PushClip(r) => {
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
            DrawCmd::PopClip => {
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
    /// Open the group of a drawing layer: its clip, opacity and blend. Its
    /// filters, masks, Porter-Duff operator and 3D transform need a
    /// compositor layer and are not lowered here.
    fn push_layer(&self, cmd: &DrawCmd, state: &mut State<'_>) {
        let DrawCmd::PushLayer(layer) = cmd else {
            unreachable!("invariant: matching command family");
        };
        let origin = state.origin;
        let clip = layer.clip.as_ref().map(|shape| {
            crate::lower::path::shifted(
                crate::lower::scene_shape::shape(shape),
                Point {
                    x: origin.x,
                    y: origin.y,
                },
            )
        });
        state.frames.push(Frame::Layer);
        state.out.push(D::PushGroup(Group {
            clip,
            opacity: layer.opacity.clamp(0.0, 1.0),
            blend: layer.blend,
            blur: None,
        }));
    }
}
