//! Role: Board state shared through `provide` / `context`.
//! Every field is a
//! Copy handle, so components capture it with `move` and never clone it.

use crate::model::{Board, Lane};
use kivact::{Selector, Signal, Var};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(crate) struct BoardCx {
    pub board: Signal<Board>,
    /// The selected task (the editor is shown while it is `Some`).
    pub selected: Signal<Option<u64>>,
    /// `is_selected.is(&id)` re-runs only the two cards whose truth changed.
    pub is_selected: Selector<u64>,
    /// How often each card was selected. Card-local state lives on the data
    /// side, so it survives lane moves and virtual-list scrolling (rows are
    /// disposed when they leave the list or the viewport).
    pub visits: Signal<HashMap<u64, u32>>,
    pub title: Signal<String>,
    pub notes: Signal<String>,
    pub search: Signal<String>,
    /// Validation message shown as an alert.
    pub notice: Signal<Option<String>>,
    /// A card moved to another lane is a new row there (lists never adopt
    /// rows). The card that pressed "右へ" or "完了" asks its successor to
    /// take the keyboard focus when it mounts.
    pub refocus: Var<Option<(u64, Refocus)>>,
}

/// Which control of a moved card receives the focus.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Refocus {
    Right,
    Done,
}

impl BoardCx {
    /// Apply `edit` to a copy and publish it with `set`, which notifies only
    /// when the board really changed. The copy shares every page (`Arc`);
    /// the edit copies one page of 256 pointers and the edited task.
    fn change(self, edit: impl FnOnce(&mut Board) -> bool) -> bool {
        let mut next = self.board.get();
        let changed = edit(&mut next);
        if changed {
            self.board.set(next);
        }
        changed
    }

    pub(crate) fn add(self) {
        let title = self.title.get();
        let notes = self.notes.get();
        let mut added = None;
        self.change(|board| {
            added = board.add(title, notes);
            added.is_some()
        });
        match added {
            Some(id) => {
                self.selected.set(Some(id));
                self.title.set(String::new());
                self.notes.set(String::new());
                self.notice.set(None);
            }
            None => self.notice.set(Some("題名を入力してください".into())),
        }
    }

    /// Select a card and load it into the form.
    pub(crate) fn select(self, id: u64) {
        let task = self
            .board
            .with(|board| board.task(id).map(|t| (t.title.clone(), t.notes.clone())));
        if let Some((title, notes)) = task {
            self.title.set(title);
            self.notes.set(notes);
            self.selected.set(Some(id));
            self.visits
                .update(|visits| *visits.entry(id).or_default() += 1);
        }
    }

    pub(crate) fn move_task(self, id: u64, forward: bool) {
        self.change(|board| board.move_task(id, forward));
    }

    pub(crate) fn set_lane(self, id: u64, lane: Lane) {
        self.change(|board| board.set_lane(id, lane));
    }

    /// Checkbox: done moves to the Done lane, undone back to In progress.
    pub(crate) fn set_done(self, id: u64, done: bool) {
        let lane = if done { Lane::Done } else { Lane::InProgress };
        self.set_lane(id, lane);
    }

    pub(crate) fn edit_selected(self) {
        let Some(id) = self.selected.get() else {
            return;
        };
        let title = self.title.get();
        let notes = self.notes.get();
        if !self.change(move |board| board.edit(id, title, notes)) {
            self.notice.set(Some("題名を入力してください".into()));
        } else {
            self.notice.set(None);
        }
    }

    pub(crate) fn move_selected(self, forward: bool) {
        if let Some(id) = self.selected.get() {
            self.move_task(id, forward);
        }
    }

    pub(crate) fn set_selected_lane(self, lane: Lane) {
        if let Some(id) = self.selected.get() {
            self.set_lane(id, lane);
        }
    }

    pub(crate) fn delete_selected(self) {
        let Some(id) = self.selected.get() else {
            return;
        };
        self.change(|board| board.delete(id));
        self.selected.set(None);
        self.title.set(String::new());
        self.notes.set(String::new());
    }
}
