//! Role: Project the focused editor caret after scrolling its text.
use crate::model::Model;
use kivact_wire::Rect;
impl Model {
    /// After client turns: the IME caret area of the focused text input
    /// in the window, as drawn (its text scrolled to the caret first).
    pub(crate) fn caret(&mut self, engine: &mut arvane::Engine) -> Option<Rect> {
        self.scroll_inputs(engine);
        let id = self.focused_input()?;
        if self.node(id).is_some_and(|n| n.read_only()) {
            return None;
        }
        let caret = crate::model::interact::text_geometry::caret_rect(self, id, engine);
        let t = self.transform_between(id, None);
        Some(crate::model::transform::map_rect(t.as_ref(), caret))
    }
}
