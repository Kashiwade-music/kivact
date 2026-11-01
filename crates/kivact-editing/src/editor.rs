//! Role: Native editor ownership and controlled-value change bookkeeping.

use super::{Damage, EditState, Editor, Taken, TextChange, TextSession};

impl Editor {
    /// Own a document without resetting its revision, session or pending changes.
    pub fn new(state: EditState) -> Self {
        Self { state }
    }
    /// The behavior-facing document.
    pub fn state(&self) -> &EditState {
        &self.state
    }
    /// Lend the document to a behavior, preserving the host's current limit.
    pub fn state_mut(&mut self) -> &mut EditState {
        &mut self.state
    }
    /// Return the document with its revision and journal intact.
    pub fn into_state(self) -> EditState {
        self.state
    }
    /// The document's current revision.
    pub fn revision(&self) -> u64 {
        self.state.revision()
    }
    /// The sole limit enforced by behavior-facing document mutations.
    pub fn max_length(&self) -> Option<usize> {
        self.state.max_length
    }
    /// Set the document's mutation limit without truncating existing content.
    pub fn set_max_length(&mut self, limit: Option<usize>) {
        self.state.max_length = limit;
    }
    /// Take a controlled value unless the IME currently owns the document.
    pub fn set_controlled(&mut self, value: &str) -> bool {
        self.state.set_controlled(value)
    }
    /// Adopt a formerly held controlled value as a recorded minimal edit.
    pub fn adopt(&mut self, value: &str) -> bool {
        self.state.adopt(value)
    }
    /// Trim the document as a recorded edit, clamping `end` to its length
    /// and rounding down to a UTF-8 character boundary. The session is
    /// clamped into the remaining text.
    pub fn truncate(&mut self, end: usize) {
        self.state.truncate(end);
    }
    /// Start the host's change journal at the current revision.
    pub fn start_log(&mut self) {
        self.state.start_log();
    }
    /// Whether the journal contains every change since the supplied revision.
    pub fn recorded_since(&self, revision: u64) -> bool {
        self.state.recorded_since(revision)
    }
    /// Drain recorded changes and begin a new journal at the current revision.
    pub fn take_changes(&mut self) -> Vec<TextChange> {
        self.state.take_changes()
    }
    /// Revert the journal and restore the session captured before a behavior.
    /// `revision` and `session` must be those captured when [`Self::start_log`]
    /// began this journal; the journal must not have been drained since then.
    pub fn revert(&mut self, revision: u64, session: TextSession) {
        self.state.revert(revision, session);
    }
    /// Adopt a behavior's document using the host's revision classification.
    /// With [`Taken::Recorded`], `state` must have recorded every change
    /// since this editor's current revision (`state` was read from it).
    pub fn take(&mut self, state: EditState, how: Taken) {
        self.state.take(state, how);
    }
    /// Consume the damage accumulated since the line cache last refreshed.
    pub fn take_damage(&self) -> Option<Damage> {
        self.state.take_damage()
    }
}
