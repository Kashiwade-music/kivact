//! Role: One edit session shared by several sources of a component, owned by the source that began it.
use kivact::Callback;
use kivact::interaction::EditSession;
use kivact::ui::EditPhase;

/// An [`EditSession`] whose value carries who began the edit (`O`) beside the
/// snapshot it restores (`S`), so a source can end only its own edit and a
/// write of another source is refused while an edit runs.
pub(crate) struct OwnedEdit<O: Copy + PartialEq + 'static, S: Clone + 'static> {
    session: EditSession<(O, S)>,
}

impl<O: Copy + PartialEq + 'static, S: Clone + 'static> Clone for OwnedEdit<O, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O: Copy + PartialEq + 'static, S: Clone + 'static> Copy for OwnedEdit<O, S> {}

impl<O: Copy + PartialEq + 'static, S: Clone + 'static> OwnedEdit<O, S> {
    /// An edit that writes `restore` the snapshot when canceled and reports
    /// its phases to `observer`. Setup only.
    pub(crate) fn new(
        restore: impl Fn(S) + 'static,
        observer: Option<Callback<EditPhase>>,
    ) -> Self {
        Self {
            session: crate::editors::shared::model::observed(
                EditSession::new(move |(_, snapshot): (O, S)| restore(snapshot)),
                observer,
            ),
        }
    }

    /// The session, for [`Drag::edit`](kivact::interaction::Drag::edit).
    pub(crate) fn session(self) -> EditSession<(O, S)> {
        self.session
    }

    /// Who began the active edit; `None` while no edit is active. Tracked.
    pub(crate) fn owner(self) -> Option<O> {
        self.session.with_initial(|(owner, _)| *owner)
    }

    /// Whether an edit is active.
    pub(crate) fn is_active(self) -> bool {
        self.session.is_active()
    }

    /// Begin an edit of `owner`; nothing while an edit is active.
    pub(crate) fn begin(self, owner: O, snapshot: S) {
        self.session.begin((owner, snapshot));
    }

    /// Commit the active edit when `owner` began it.
    pub(crate) fn commit(self, owner: O) {
        if self.owner() == Some(owner) {
            self.session.commit();
        }
    }

    /// Cancel the active edit when `owner` began it; whether it was canceled.
    pub(crate) fn cancel_owned(self, owner: O) -> bool {
        self.owner() == Some(owner) && self.session.cancel()
    }

    /// Cancel the active edit, whoever began it; whether one was active.
    pub(crate) fn cancel(self) -> bool {
        self.session.cancel()
    }

    /// With no edit active: begin an edit of `owner`, write, commit. While
    /// `owner`'s edit is active the write joins it. While another source's
    /// edit is active nothing is written.
    pub(crate) fn once(self, owner: O, snapshot: impl FnOnce() -> S, write: impl FnOnce()) {
        match self.owner() {
            None => self.session.once((owner, snapshot()), write),
            Some(current) if current == owner => write(),
            Some(_) => {}
        }
    }
}
