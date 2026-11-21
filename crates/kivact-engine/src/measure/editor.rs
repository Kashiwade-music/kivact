//! Role: Optional native editor slot owned by Model, outside the structural tree.
use kivact_editing::EditState;
use kivact_editing::host::{Editor, Journal};
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Clone, Debug)]
pub(crate) enum ControlledValue {
    InSync { at: u64 },
    HeldDuringComposition { value: Rc<str> },
}
// xtask-allow(state): cache: lines (paint width) and measure_lines (width probes) are separate shaping caches filled lazily by refresh_lines, each on its own path; sync empties both when fonts are added
#[derive(Clone, Debug)]
pub(crate) struct EditorNode {
    /// Document and host-only journal controls.
    pub(crate) editor: Editor,
    /// Shaping cache (`None` until the input is first laid out), kept outside Editor to
    /// preserve the future crate DAG. Shared, for one request, with an editing behavior's
    /// context (`text_input::SharedLines`).
    pub(crate) lines: Rc<RefCell<Option<crate::text_input::EditLines>>>,
    /// Width probes must not evict the geometry of the visible wrapped rows.
    pub(crate) measure_lines: RefCell<Option<crate::text_input::EditLines>>,
    /// A controlled value given while the IME composes, taken when the
    /// composition ends.
    pub(crate) controlled: ControlledValue,
    /// Last session sent to the selection listener.
    reported_session: kivact_types::cell::Sent<kivact_editing::TextSession>,
    /// Horizontal text scroll, independent of container scrolling.
    pub(crate) scroll_x: f64,
    /// Vertical scroll of an automatically sized or wrapped field.
    pub(crate) scroll_y: f64,
}
impl EditorNode {
    /// Note `session` as the one last sent to the selection listener;
    /// whether it differs from the one before (the only writer of
    /// `reported_session`).
    pub(crate) fn report_session(&mut self, session: &kivact_editing::TextSession) -> bool {
        self.reported_session.update(session.clone()).is_some()
    }

    pub(crate) fn new(state: EditState) -> Self {
        Self {
            editor: Editor::new(state),
            lines: Rc::default(),
            measure_lines: RefCell::default(),
            controlled: ControlledValue::InSync { at: 0 },
            reported_session: kivact_types::cell::Sent::new(),
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }

    /// Take the controlled value held during a composition that ended, as a
    /// recorded change of `journal` (`Journal::adopt`); whether its text
    /// changed. The fields are passed apart because `journal` borrows the
    /// editor.
    pub(crate) fn adopt_pending_controlled(
        controlled: &mut ControlledValue,
        journal: &mut Journal<'_>,
    ) -> bool {
        if journal.state().is_composing() {
            return false;
        }
        let ControlledValue::HeldDuringComposition { value } = controlled else {
            return false;
        };
        let changed = journal.adopt(value);
        *controlled = ControlledValue::InSync {
            at: journal.revision(),
        };
        changed
    }
}
