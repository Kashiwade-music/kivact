//! Role: Attach custom painting that reads native editing geometry.

use kivact_editing::TextLayout;
use kivact_style::draw::Draw;
use std::rc::Rc;

impl super::Node {
    /// Paint over the native editor, using geometry in the canvas coordinate space.
    /// Reads made by the callback are reactive. Geometry refreshes with layout,
    /// text and selection changes. This replaces the preceding layout callback.
    pub fn draw_with_layout(mut self, f: impl Fn(&mut Draw, &TextLayout) + 'static) -> Self {
        self.common.draw_with_layout = Some(Rc::new(f));
        self
    }
}
