//! Role: The groups one local scene lowering has open in the output.
use crate::SceneOut;
use crate::effects::Tint;
use kivact_types::protocol::DrawCommand as D;
use kivact_wire::{ComputedText, Rect};
pub(super) struct State<'a> {
    pub(super) inherited: Option<&'a ComputedText>,
    pub(super) engine: &'a mut arvane::Engine,
    pub(super) out: &'a mut SceneOut,
    pub(super) origin: Rect,
    pub(super) clips: usize,
    pub(super) transforms: usize,
    pub(super) raster_stack: Vec<f32>,
    pub(super) layer_stack: Vec<(usize, Tint)>,
    pub(super) current_tint: Tint,
}
impl<'a> State<'a> {
    pub(super) fn new(
        inherited: Option<&'a ComputedText>,
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
            clips: 0,
            transforms: 0,
            raster_stack: Vec::new(),
            layer_stack: Vec::new(),
            current_tint: tint,
        }
    }
    pub(super) fn finish(self) {
        self.out.extend(std::iter::repeat_n(D::PopClip, self.clips));
        self.out
            .extend(std::iter::repeat_n(D::PopTransform, self.transforms));
        for (count, _) in self.layer_stack.into_iter().rev() {
            self.out.extend(std::iter::repeat_n(D::PopLayer, count));
        }
    }
}
