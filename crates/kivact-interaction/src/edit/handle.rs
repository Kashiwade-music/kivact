//! Role: Edit sessions: the start, commit and cancel of one continuous edit.
//!
//! The start and end of one continuous edit, so an app can group it into one undo step.
//!
//! A component keeps the value an edit starts from in an [`EditSession`]:
//! `commit` keeps the writes, and `cancel` and the owner's disposal write the
//! starting value back before reporting `Cancel`. The history itself is the
//! application's; the session only reports [`EditPhase`]s. `restore` and the
//! `on_edit` observer are closures returning a `HandlerResult`. The observer
//! belongs to the setup owner that adds it, so when that owner (a child
//! component, say) is disposed first, the `Cancel` of the session owner's
//! later disposal is skipped for it rather than reported as a misuse.
//! [`EditSession::with_initial`] reads the starting value of the active edit
//! (tracked), for a component that shows who owns the edit.
//! [`EditSession::once`] wraps a single write in `Begin` and `Commit`, and
//! inside an edit already in progress joins it without another phase. The
//! scalar, the splitter, drags ([`DragApi::edit`](crate::DragApi::edit)) and zoom
//! and pan all report through it.

use kivact_core::ErrorKind;
use kivact_core::UiError;
use kivact_core::state::callback;
use kivact_core::state::{Callback, MachineHandle, Var, machine, on_cleanup, var};
use std::rc::Rc;

use super::machine;

use super::EditSessionState;
use kivact_core::view::{EditPhase, HandlerResult};

/// The state of the edit a component is in, and the phases it reports.
///
/// `begin` records the value the edit starts from and reports `Begin`; `commit`
/// reports `Commit`; `cancel` writes the recorded value back through `restore`
/// and reports `Cancel`. `commit` and `cancel` do nothing without an active
/// edit; `begin` does nothing while an edit is active. The owner's cleanup
/// cancels an active edit, restoring its starting value before reporting it.
///
/// Create the session once during setup. Begin before the first write, commit
/// on release, and cancel on Escape or pointer cancellation:
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// # pub use kivact_core::{view, state, style};
/// # pub mod prelude {
/// # //! Role: Supply the canonical imports used by this documentation example.
/// # pub use kivact_core::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Signal, Value, binding, effect, memo, on_cleanup, on_mount, signal};
/// # pub use kivact_core::view::{EditPhase, Component, IntoView, NodeBuilder, Orientation, Role, column, for_each, node, node_ref, row, show, text};
/// # pub use kivact_core::style::{ColorToken, Cursor, Style, style};
/// # }
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::prelude::*;
/// use kivact::interaction::EditSession;
/// use kivact::view::{ButtonPress, Handled, Key, MouseButton, PointerPhase};
///
/// let value = signal(100.0_f64);
/// let edit = EditSession::new(move |initial| value.set(initial));
/// let field = node().name("Width").focusable(true)
///     .on_pointer(move |event| {
///         match event.phase() {
///             PointerPhase::Down(ButtonPress { button: MouseButton::Primary, .. }) => {
///                 edit.begin(value.get_untracked());
///             }
///             PointerPhase::Move if edit.is_active() => value.set(event.pos().x),
///             PointerPhase::Up(_) => edit.commit(),
///             PointerPhase::Cancel => { edit.cancel(); }
///             _ => return Handled::No,
///         }
///         Handled::Yes
///     })
///     .on_key(move |event| {
///         if event.key() == Key::Escape && edit.cancel() {
///             Handled::Yes
///         } else {
///             Handled::No
///         }
///     });
/// # let _ = field;
/// ```
pub struct EditSession<T: 'static> {
    session: MachineHandle<machine::Machine<T>>,
    restore: Callback<T>,
    observer: Var<Option<Callback<EditPhase>>>,
    /// Passes an error of the observer on, as the session's owner reports it.
    failed: Callback<UiError>,
}

impl<T: 'static> Clone for EditSession<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for EditSession<T> {}

impl<T: Clone + 'static> EditSession<T> {
    /// A session under the current setup owner. `restore` writes the value the
    /// edit started from back; it may return `()` or `Result<(), UiError>`, and
    /// an `Err` goes to the nearest error handler. Setup only.
    #[track_caller]
    pub fn new<R: HandlerResult>(restore: impl Fn(T) -> R + 'static) -> Self {
        let session = Self {
            session: machine(
                EditSessionState::Idle,
                || machine::Machine::new(),
                |_, ()| {},
            ),
            restore: callback(restore),
            observer: var(None),
            failed: callback(Err::<(), UiError>),
        };
        on_cleanup(move || {
            session.cancel();
        });
        session
    }

    /// Report each [`EditPhase`] of the session's edits to `f`. A later call
    /// replaces the earlier one. `f` belongs to the owner whose setup calls
    /// this, so an observer added by a child component stops hearing the
    /// session when that child is disposed: the `Cancel` that ends an edit
    /// when the session's owner is cleaned up afterwards is not reported to
    /// it. Setup only.
    pub fn on_edit<R: HandlerResult>(self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.observer.set(Some(callback(f)));
        self
    }

    fn report(self, phase: EditPhase) {
        let Some(observer) = self.observer.get() else {
            return;
        };
        // An observer of a disposed owner is skipped, not a misuse (G-01: no
        // callback outlives the component that registered it).
        match observer.try_call(phase) {
            Err(error) if !matches!(error.kind(), ErrorKind::Disposed { .. }) => {
                self.failed.call(error);
            }
            _ => {}
        }
    }

    /// Start an edit from `initial` and report `Begin`. Does nothing while an
    /// edit is active, so one edit reports `Begin` once.
    pub fn begin(self, initial: T) {
        if self.is_active() {
            return;
        }
        self.session.send(machine::Event::Begin(Rc::new(initial)));
        self.report(EditPhase::Begin);
    }

    /// End the active edit and report `Commit`.
    pub fn commit(self) {
        if self.is_active() {
            self.session.send(machine::Event::Finish);
            self.report(EditPhase::Commit);
        }
    }

    /// Write the starting value back, end the active edit and report `Cancel`.
    /// Returns whether an edit was active.
    pub fn cancel(self) -> bool {
        if !self.is_active() {
            return false;
        }
        let initial = self.session.state().with_untracked(|state| match state {
            EditSessionState::Idle => None,
            EditSessionState::Editing { initial } => Some(initial.as_ref().clone()),
        });
        let Some(initial) = initial else {
            return false;
        };
        self.session.send(machine::Event::Finish);
        self.restore.call(initial);
        self.report(EditPhase::Cancel);
        true
    }

    /// One whole edit around `write`: `Begin` with `initial`, the write, then
    /// `Commit`. While an edit is already active the write joins it, `initial`
    /// is ignored and no further phase is reported.
    ///
    /// ```no_run
    /// use kivact_core::state::{signal, Signal};
    /// use kivact_interaction::EditSession;
    /// // The session must be created during component setup.
    /// let value = signal(10.0);
    /// let edit = EditSession::new(move |initial| value.set(initial));
    /// edit.once(value.get_untracked(), || value.set(20.0));
    /// ```
    pub fn once(self, initial: T, write: impl FnOnce()) {
        if self.is_active() {
            write();
            return;
        }
        self.begin(initial);
        write();
        self.commit();
    }

    /// Whether an edit has begun and not ended (untracked: an effect that
    /// calls it does not follow the edit; use
    /// [`with_initial`](Self::with_initial) for that).
    pub fn is_active(self) -> bool {
        self.session
            .state()
            .with_untracked(|state| matches!(state, EditSessionState::Editing { .. }))
    }

    /// Read the value the active edit started from; `None` while no edit is
    /// active. Tracked: a memo, an effect or a style that calls it follows the
    /// edit's begin and end, so `with_initial(|_| ()).is_some()` is a tracked
    /// "is active".
    pub fn with_initial<R>(self, read: impl FnOnce(&T) -> R) -> Option<R> {
        self.session.state().with(|state| match state {
            EditSessionState::Idle => None,
            EditSessionState::Editing { initial } => Some(read(initial.as_ref())),
        })
    }
}
