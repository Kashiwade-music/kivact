//! Role: Optional native editor slot owned by Model, outside the structural tree.
use kivact_editing::{EditState, Editor, Journal};
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Clone, Debug)]
pub(crate) struct EditorNode {
    /// Document and host-only journal/damage controls.
    pub(crate) editor: Editor,
    /// Shaping cache (`None` until the input is first laid out), kept outside Editor to
    /// preserve the future crate DAG.
    pub(crate) lines: RefCell<Option<kivact_text::EditLines>>,
    /// A controlled value given while the IME composes, taken when the
    /// composition ends.
    pub(crate) pending_controlled: Option<Rc<str>>,
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
            pending_controlled: None,
            reported_session: None,
            scroll_x: 0.0,
            controlled_at: 0,
        }
    }

    /// Take the controlled value held during a composition that ended, as a
    /// recorded change of `journal` (`Journal::adopt`); whether its text
    /// changed. The fields are passed apart because `journal` borrows the
    /// editor.
    pub(crate) fn adopt_pending_controlled(
        pending_controlled: &mut Option<Rc<str>>,
        controlled_at: &mut u64,
        journal: &mut Journal<'_>,
    ) -> bool {
        if journal.state().is_composing() {
            return false;
        }
        let Some(value) = pending_controlled.take() else {
            return false;
        };
        let changed = journal.adopt(&value);
        *controlled_at = journal.revision();
        changed
    }
}
