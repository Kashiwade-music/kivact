//! Role: Native editor ownership: the document, its edits and the states given back.

use crate::change::Damage;
use crate::edit_state::{EditState, Taken};
use crate::journal::{Incoming, Journal};

/// A native host's editor, owning the behavior-facing document. The host changes it by
/// recorded edits ([`Editor::journal`]), controlled values ([`Editor::set_controlled`]) and
/// states kivact gives back ([`Editor::incoming`]).
///
/// Behaviors receive only the state. Cloning copies the document, journal and damage
/// independently; no mutable editor state is shared.
#[derive(Clone, Debug)]
pub struct Editor {
    state: EditState,
}

impl Editor {
    /// Own a document without resetting its revision, session or pending changes.
    pub fn new(state: EditState) -> Self {
        Self { state }
    }

    /// The behavior-facing document.
    pub fn state(&self) -> &EditState {
        &self.state
    }

    /// The document's current revision.
    pub fn revision(&self) -> u64 {
        self.state.revision()
    }

    /// Take a controlled value unless the IME currently owns the document.
    pub fn set_controlled(&mut self, value: &str) -> bool {
        self.state.set_controlled(value)
    }

    /// Consume the damage accumulated since the line cache last refreshed.
    pub fn take_damage(&self) -> Option<Damage> {
        self.state.take_damage()
    }

    /// A copy of the document that records the changes made to it from here, so it can be
    /// given back by range ([`Editor::incoming`]).
    pub fn snapshot(&self) -> EditState {
        let mut copy = self.state.clone();
        copy.start_log();
        copy
    }

    /// Open the journal of one edit, its mutations held to `max_length` scalars.
    pub fn journal(&mut self, max_length: Option<usize>) -> Journal<'_> {
        self.state.max_length = max_length;
        self.state.start_log();
        let (revision, session) = (self.state.revision(), self.state.session().clone());
        Journal {
            state: &mut self.state,
            revision,
            session,
        }
    }

    /// `state`, given back by kivact: cut to `max_length` scalars unless composing, and
    /// classified against this editor's text.
    pub fn incoming(&self, mut state: EditState, max_length: Option<usize>) -> Incoming {
        state.max_length = max_length;
        if !state.is_composing()
            && let Some(max) = max_length
            && let Some((end, _)) = state.value().char_indices().nth(max)
        {
            state.truncate(end);
        }
        let revision = self.state.revision();
        let how = if state.revision() == revision {
            Taken::Same
        } else if state.recorded_since(revision) {
            Taken::Recorded
        } else if state.value() == self.state.value() {
            Taken::Same
        } else {
            Taken::Whole
        };
        Incoming {
            state,
            how,
            against: revision,
        }
    }
}
