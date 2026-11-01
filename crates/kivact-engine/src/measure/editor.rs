//! Role: Optional native editor slot owned by Model, outside the structural tree.
use kivact_editing::{EditState, Editor};
use std::cell::RefCell;
#[derive(Clone, Debug)]
pub(crate) struct EditorNode {
    /// Document and host-only journal/damage controls.
    pub(crate) editor: Editor,
    /// Shaping cache, kept outside Editor to preserve the future crate DAG.
    pub(crate) lines: RefCell<kivact_text::EditLines>,
    /// A controlled value waits in node properties while the IME composes.
    pub(crate) held: bool,
    /// Last session sent to the selection listener.
    pub(crate) reported_session: Option<kivact_editing::TextSession>,
    /// Horizontal text scroll, independent of container scrolling.
    pub(crate) scroll_x: f32,
    /// Revision at which the current controlled Rc value was last considered.
    pub(crate) controlled_at: u64,
}
impl EditorNode {
    pub(crate) fn new(state: EditState) -> Self {
        Self {
            editor: Editor::new(state),
            lines: RefCell::default(),
            held: false,
            reported_session: None,
            scroll_x: 0.0,
            controlled_at: 0,
        }
    }
}
