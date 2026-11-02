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
//! [`EditSession::once`] wraps a single write in `Begin` and `Commit`, and
//! inside an edit already in progress joins it without another phase. The
//! scalar, the splitter, drags ([`Drag::edit`](crate::Drag::edit)) and zoom
//! and pan all report through it.

use kivact_core::diagnostics::ErrorKind;
use kivact_core::view::{HandlerResult, callback};
use kivact_core::{Callback, UiError, Var, on_cleanup, var};

/// The phase of one continuous edit that a component reports to the app.
///
/// A drag, or one key press, writes the value many times or once; an app that
/// records undo steps wants those writes as a single step. A component that
/// takes `on_edit(f)` calls `f` with these phases around its writes, the same
/// way for every kit. They play the role of `onPointerDown` and `onPointerUp`
/// in a React handler that opens and closes an undo transaction, and also cover
/// keys and assistive technology, which have no pointer events.
///
/// The rules are the same for every component:
///
/// - `Begin` comes before the first write of the edit to the binding. Take the
///   snapshot for the undo step here.
/// - `Commit` comes after the last write. Close the undo step here.
/// - `Cancel` comes after the component wrote the value the edit started
///   from back to the binding. Drop the undo step here; the binding already
///   holds the old value.
/// - A `Commit` or a `Cancel` is sent only for an edit that sent `Begin`, and
///   an edit sends `Begin` once and then exactly one of `Commit` and `Cancel`.
/// - If the component is disposed in the middle of an edit (its owner is
///   cleaned up), the edit ends as `Cancel`.
///
/// A drag is one edit from the press to the release. A key press, a wheel
/// notch or an assistive technology action that writes once is its own edit:
/// `Begin`, the write, `Commit`.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::{control, prelude};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{EditPhase, ScalarRange, use_splitter};
/// use kivact::prelude::*;
/// let width = signal(240.0);
/// let started_at = signal(240.0);
/// let undo = signal(Vec::<(f64, f64)>::new());
/// let on_edit = move |phase| match phase {
///     EditPhase::Begin => started_at.set(width.get_untracked()),
///     EditPhase::Commit => {
///         undo.update(|steps| steps.push((started_at.get_untracked(), width.get_untracked())))
///     }
///     EditPhase::Cancel => {}
/// };
/// let range = ScalarRange::new(120.0, 480.0).unwrap();
/// let handle = use_splitter(width, range, Orientation::Horizontal)
///     .on_edit(on_edit)
///     .handle();
/// let view = node().part(handle).name("Sidebar width");
/// # let _ = view;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditPhase {
    /// The edit starts; no write has happened yet.
    Begin,
    /// The edit ended with its writes kept.
    Commit,
    /// The edit was abandoned and the value it started from is back in the binding.
    Cancel,
}

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
/// #     pub use kivact_core::{control, prelude, view, Callback};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::prelude::*;
/// use kivact::interaction::EditSession;
/// use kivact::view::{ButtonPress, Handled, MouseButton, PointerPhase};
///
/// let value = signal(100.0_f64);
/// let edit = EditSession::new(move |initial| value.set(initial));
/// let field = node().name("Width").focusable(true)
///     .on_pointer(move |event| {
///         match event.phase {
///             PointerPhase::Down(ButtonPress { button: MouseButton::Primary, .. }) => {
///                 edit.begin(value.get_untracked());
///             }
///             PointerPhase::Move if edit.is_active() => value.set(f64::from(event.pos.x)),
///             PointerPhase::Up(_) => edit.commit(),
///             PointerPhase::Cancel => { edit.cancel(); }
///             _ => return Handled::No,
///         }
///         Handled::Yes
///     })
///     .on_key(move |event| {
///         if event.key == Key::Escape && edit.cancel() {
///             Handled::Yes
///         } else {
///             Handled::No
///         }
///     });
/// # let _ = field;
/// ```
pub struct EditSession<T: 'static> {
    initial: Var<Option<T>>,
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
            initial: var(None),
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
        // An observer of a disposed owner is skipped, not a misuse (G-01).
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
        self.initial.set(Some(initial));
        self.report(EditPhase::Begin);
    }

    /// End the active edit and report `Commit`.
    pub fn commit(self) {
        if self.initial.update(Option::take).is_some() {
            self.report(EditPhase::Commit);
        }
    }

    /// Write the starting value back, end the active edit and report `Cancel`.
    /// Returns whether an edit was active.
    pub fn cancel(self) -> bool {
        let Some(initial) = self.initial.update(Option::take) else {
            return false;
        };
        self.restore.call(initial);
        self.report(EditPhase::Cancel);
        true
    }

    /// One whole edit around `write`: `Begin` with `initial`, the write, then
    /// `Commit`. While an edit is already active the write joins it, `initial`
    /// is ignored and no further phase is reported.
    ///
    /// ```no_run
    /// use kivact_core::prelude::*;
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

    /// Whether an edit has begun and not ended.
    pub fn is_active(self) -> bool {
        self.initial.with(Option::is_some)
    }
}
