//! Role: The groups one local scene lowering has open in the output.
use crate::SceneOut;
use crate::effects::Tint;
use kivact_types::render::RenderCmd as D;
use kivact_types::{ComputedTextStyle, Rect};
/// A group a local scene opened in the output and has not closed yet.
#[derive(Clone, Copy, Debug)]
pub(super) enum Frame {
    /// A rect clip; `drawn: false` when its rect could not be drawn (its pop draws nothing).
    Clip { drawn: bool },
    /// A transform, with the raster scale outside it.
    Transform { raster: f64 },
    /// A drawing layer: the output layers it pushed and the tint outside it.
    Layer { pushed: usize, tint: Tint },
}

pub(super) struct State<'a> {
    pub(super) inherited: Option<&'a ComputedTextStyle>,
    pub(super) engine: &'a mut arvane::Engine,
    pub(super) out: &'a mut SceneOut,
    pub(super) origin: Rect,
    pub(super) frames: Vec<Frame>,
    pub(super) current_tint: Tint,
}
impl<'a> State<'a> {
    pub(super) fn new(
        inherited: Option<&'a ComputedTextStyle>,
        engine: &'a mut arvane::Engine,
        out: &'a mut SceneOut,
        origin: Rect,
        tint: Tint,
    ) -> Self {
        Self {
            inherited,
            engine,
            out,
            origin,
            frames: Vec::new(),
            current_tint: tint,
        }
    }
    pub(super) fn finish(self) {
        // The groups left open are closed innermost first; the caller restores the raster scale.
        for frame in self.frames.into_iter().rev() {
            match frame {
                Frame::Clip { drawn: true } => self.out.push(D::PopClip),
                Frame::Clip { drawn: false } => {}
                Frame::Transform { .. } => self.out.push(D::PopTransform),
                Frame::Layer { pushed, .. } => {
                    self.out.extend(std::iter::repeat_n(D::PopLayer, pushed));
                }
            }
        }
    }
}
