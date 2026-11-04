//! Role: Optional native editor slot owned by Model, outside the structural tree.
use kivact_editing::{EditState, Editor, Journal};
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Clone, Debug)]
pub(crate) enum ControlledValue {
    InSync { at: u64 },
    HeldDuringComposition { value: Rc<str> },
}
#[derive(Clone, Debug)]
pub(crate) struct EditorNode {
    /// Document and host-only journal/damage controls.
    pub(crate) editor: Editor,
    // xtask-allow(state-fields): Shaping cache is independent of controlled-value state.
    /// Shaping cache (`None` until the input is first laid out), kept outside Editor to
    /// preserve the future crate DAG.
    pub(crate) lines: RefCell<Option<crate::text_input::EditLines>>,
    // xtask-allow(state-fields): Measurement cache is independent of controlled-value state.
    /// Width probes must not evict the geometry of the visible wrapped rows.
    pub(crate) measure_lines: RefCell<Option<crate::text_input::EditLines>>,
    /// A controlled value given while the IME composes, taken when the
    /// composition ends.
    pub(crate) controlled: ControlledValue,
    /// Last session sent to the selection listener.
    pub(crate) reported_session: Option<kivact_editing::TextSession>,
    /// Horizontal text scroll, independent of container scrolling.
    pub(crate) scroll_x: f64,
    /// Vertical scroll of an automatically sized or wrapped field.
    pub(crate) scroll_y: f64,
}
impl EditorNode {
    pub(crate) fn new(state: EditState) -> Self {
        Self {
            editor: Editor::new(state),
            lines: RefCell::default(),
            measure_lines: RefCell::default(),
            controlled: ControlledValue::InSync { at: 0 },
            reported_session: None,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }

    pub(crate) fn flush_damage(&mut self) {
        if let Some(damage) = self.editor.take_damage() {
            if let Some(lines) = self.lines.get_mut() {
                lines.hold_damage(damage);
            }
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
