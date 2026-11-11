//! Role: One editing transaction: a Begin snapshot dropped at Commit or Cancel.

use kivact::state::{Signal, signal};

/// The editing transaction of one editor: idle, or editing from a snapshot.
#[derive(Clone, PartialEq)]
pub(super) enum Txn<T> {
    Idle,
    Editing { snapshot: T },
}

impl<T: Clone + PartialEq + 'static> Txn<T> {
    /// The source of every transaction: idle until an edit begins.
    pub(super) fn signal() -> Signal<Self> {
        signal(Txn::Idle)
    }

    /// Update the editing snapshot, if an edit is running.
    pub(super) fn update_snapshot<R>(
        state: Signal<Self>,
        change: impl FnOnce(&mut T) -> R,
    ) -> Option<R> {
        let mut result = None;
        state.update(|txn| {
            if let Txn::Editing { snapshot } = txn {
                result = Some(change(snapshot));
            }
        });
        result
    }

    /// Begin editing from `snapshot`.
    pub(super) fn begin(state: Signal<Self>, snapshot: T) {
        state.set(Txn::Editing { snapshot });
    }

    /// The snapshot of the running edit, if any.
    pub(super) fn snapshot(state: Signal<Self>) -> Option<T> {
        state.with_untracked(|txn| match txn {
            Txn::Idle => None,
            Txn::Editing { snapshot } => Some(snapshot.clone()),
        })
    }

    /// Read the snapshot of the running edit without cloning it.
    pub(super) fn read<R>(state: Signal<Self>, read: impl FnOnce(&T) -> R) -> Option<R> {
        state.with_untracked(|txn| match txn {
            Txn::Idle => None,
            Txn::Editing { snapshot } => Some(read(snapshot)),
        })
    }

    /// End the edit, keeping the application's current values.
    pub(super) fn commit(state: Signal<Self>) {
        state.set(Txn::Idle);
    }

    /// End the edit and return the snapshot to restore.
    pub(super) fn cancel(state: Signal<Self>) -> Option<T> {
        let snapshot = Self::snapshot(state);
        state.set(Txn::Idle);
        snapshot
    }
}
