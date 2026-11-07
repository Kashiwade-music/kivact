//! Role: Project the focused editor caret after scrolling its text.
use crate::model::Model;
use kivact_types::Rect;
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
        Some(self.drawn_rect(id, caret))
    }
}
