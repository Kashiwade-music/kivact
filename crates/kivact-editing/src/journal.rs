//! Role: One recorded edit of an editor's document, finished or taken back as a whole.

use crate::change::TextChange;
use crate::edit_state::{EditState, Taken};
use crate::session::TextSession;

/// The host's recording of one edit of an editor's document: every change made through it
/// since it opened ([`Editor::journal`](crate::Editor::journal)). It ends with [`Journal::finish`], which returns the
/// changes to report; dropping it unfinished takes its changes back (a change the host does
/// not report is not kept).
#[must_use = "finish the journal to report its changes; dropping it takes them back"]
pub struct Journal<'a> {
    /// The document the journal records changes of.
    pub(crate) state: &'a mut EditState,
    /// The revision the journal opened on.
    pub(crate) revision: u64,
    /// The session the journal opened on.
    pub(crate) session: TextSession,
}

impl Journal<'_> {
    /// The document as the edit left it so far.
    pub fn state(&self) -> &EditState {
        self.state
    }

    /// Lend the document to a behavior; its changes are recorded.
    pub fn state_mut(&mut self) -> &mut EditState {
        self.state
    }

    /// The document's current revision.
    pub fn revision(&self) -> u64 {
        self.state.revision()
    }

    /// Whether the text changed since the journal opened.
    pub fn text_changed(&self) -> bool {
        self.state.revision() != self.revision
    }

    /// Take every change back: the text, revision and session the journal opened on. The
    /// journal stays open and records from there.
    pub fn revert(&mut self) {
        self.state.revert(self.revision, self.session.clone());
    }

    /// Trim the document after byte `end` (clamped, rounded down to a char boundary) as a
    /// recorded change; the session is clamped into what is left.
    pub fn truncate(&mut self, end: usize) {
        self.state.truncate(end);
    }

    /// Take a controlled value held while composing as one recorded minimal change; whether
    /// the text changed.
    pub fn adopt(&mut self, value: &str) -> bool {
        self.state.adopt(value)
    }

    /// Take a state kivact gave back, as [`Editor::incoming`](crate::Editor::incoming) classified it.
    pub fn take(&mut self, incoming: Incoming) {
        assert_eq!(
            incoming.against,
            self.state.revision(),
            "invariant: an incoming state is taken at the revision it was classified against"
        );
        self.state.take(incoming.state, incoming.how);
    }

    /// End the journal: the changes recorded since it opened, in order.
    pub fn finish(self) -> Vec<TextChange> {
        // The log is empty afterwards, so `Drop` takes nothing back.
        self.state.take_changes()
    }
}

impl Drop for Journal<'_> {
    fn drop(&mut self) {
        if self.state.has_changes() {
            self.revert();
        }
    }
}

/// A state kivact gave back to an editor, cut to the editor's length limit and classified
/// against the editor's text ([`Editor::incoming`](crate::Editor::incoming)); taken by [`Journal::take`].
pub struct Incoming {
    pub(crate) state: EditState,
    pub(crate) how: Taken,
    /// The editor revision it was classified against.
    pub(crate) against: u64,
}

impl Incoming {
    /// How it changes the editor's text.
    pub fn how(&self) -> Taken {
        self.how
    }

    /// The state as it will be taken (after the cut).
    pub fn state(&self) -> &EditState {
        &self.state
    }
}
