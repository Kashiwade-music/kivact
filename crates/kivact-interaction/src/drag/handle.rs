//! Role: Owner-scoped pointer drag sessions attached to sources as parts.
//!
//! [`use_drag`] keeps only the pointer identity and the start of one session
//! in its owner; the caller takes any snapshot of its own value in
//! `on_start`, and an external binding write does not take it again. A drag
//! is one edit of an [`EditSession`] through [`DragApi::edit`] (begun at the
//! press) or [`DragApi::edit_on_change`] (begun at the first [`DragApi::change`],
//! so a press that only selects reports nothing). The splitter and the
//! slider's track use `edit`; Creative's domain editors use `edit_on_change`
//! with a source on the canvas and on each region, and take a middle-button
//! pan in the same drag with [`DragApi::buttons`]. Zoom and pan does not use a
//! drag: the middle button, Space with the primary button and a two-finger
//! pinch do not fit one primary-button session, so it drives the same
//! `EditSession` itself (see `crate::zoom_pan`).
//!
//! Invariants:
//! - Escape reaching any source of an active drag (or a focused descendant
//!   of one) stops it like an engine Cancel and is consumed; without an
//!   active drag Escape passes on. A component that wants the Escape the drag
//!   passes on puts its key handler in a part attached after the source,
//!   since the node's own handlers run before its parts.
//! - A drag commits or cancels only an edit it began itself: an edit already
//!   in progress when the drag would begin one (a field being typed in) is
//!   joined and left as it is.
//! - The disabled effect, `cancel` and owner disposal drop the session
//!   without a callback; the engine Cancel that disabling the node sends is
//!   observed by `on_end`. A drag stopped by the disabled value commits its
//!   edit.

use super::data::{
    Active, BeginAt, DragCoordinates, DragEdit, DragOptions, DragPhase, DragStart, DragState,
    DragUpdate, EditOwnership, bit,
};
use super::machine;
use crate::edit::EditSession;
use crate::options_cell::OptionsCell;
use kivact_core::custom::clock_now;
use kivact_core::custom::{Part, PartState, part};
use kivact_core::geometry::Point;
use kivact_core::state::{IntoValue, callback};
use kivact_core::state::{MachineHandle, machine};
use kivact_core::state::{Signal, Value, Var, memo, signal, var};
use kivact_core::view::HandlerResult;
use kivact_core::view::{
    Handled, Key, KeyEvent, Modifiers, MouseButton, PointerEvent, PointerPhase,
};
use kivact_types::cell::Generation;
use machine::State as DragSession;
use std::rc::Rc;

impl<T: Clone> Active<T> {
    fn update(&self, pos: Point, modifiers: Modifiers, phase: DragPhase) -> Option<DragUpdate<T>> {
        let mut update = update(self.start.as_ref().clone(), pos, modifiers, phase)?;
        update.delta = self.axis.constrain(update.delta);
        update.velocity = self.axis.constrain(self.velocity.at(clock_now()));
        Some(update)
    }
}

/// How a drag ends.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stop {
    /// The starting button was released: `on_end(End)`, then the drag's own
    /// edit commits.
    Release,
    /// An engine Cancel or Escape: `on_end(Cancel)`, then the drag's own edit
    /// cancels (commits while disabled).
    Cancel,
    /// The mounted source disabled and the engine canceled its capture.
    CancelDisabled,
    /// The disabled value became true: no callback, the drag's own edit
    /// commits where it is.
    Disabled,
    /// [`DragApi::cancel`]: no callback, the drag's own edit cancels.
    Dropped,
}

struct SessionEdit<V: 'static> {
    session: EditSession<V>,
    start: Rc<dyn Fn() -> V>,
}

impl<V: Clone + 'static> DragEdit for SessionEdit<V> {
    fn is_active(&self) -> bool {
        self.session.is_active()
    }
    fn begin(&self) {
        self.session.begin((self.start)());
    }
    fn commit(&self) {
        self.session.commit();
    }
    fn cancel(&self) {
        self.session.cancel();
    }
}

/// What one pointer event did to the session.
enum Step<T> {
    Pending,
    Started {
        start: DragUpdate<T>,
        moved: DragUpdate<T>,
    },
    /// A drag started or moved: report it to its callback.
    Update(DragUpdate<T>),
    /// The drag ended; `update` is what `on_end` gets (none for an event with
    /// invalid coordinates).
    Stop {
        stop: Stop,
        owns_edit: bool,
        update: Option<DragUpdate<T>>,
    },
}

/// A Copy handle to one temporary drag session owned by the current setup owner.
///
/// Attach a [`source`](Self::source) to each element or canvas region a drag
/// can start on. Start requires the primary button (including touch/pen
/// contact), or another of the [`buttons`](Self::buttons), on an enabled
/// source while no drag is active. Subsequent events must have the same
/// pointer identity, and the release the same button; unrelated
/// pointers/buttons do not end or replace a session. Each accepted update clones the source's payload; use
/// an `Rc` or small value for a large one. External value changes do not
/// re-anchor or cancel a session: take any snapshot the drag needs in
/// [`on_start`](Self::on_start). Up/Cancel clear the session. Disposing the
/// owner drops it. Like other owner handles, use after disposal is a located
/// misuse.
///
/// The source part uses the engine's existing pointer capture, so moving
/// outside the box does not end a drag. Caller pointer handlers run before the
/// source and can stop it by consuming the event. Disabling the source's
/// element already produces a captured Cancel. [`disabled`](Self::disabled)
/// additionally supports a separately supplied disabled source. The session
/// never writes application state itself. Give it the [`EditSession`] of the
/// value the drag edits with [`edit`](Self::edit) (or
/// [`edit_on_change`](Self::edit_on_change)) to make each drag one edit that
/// Escape and an engine Cancel undo. Escape on a source stops an active drag
/// the way an engine Cancel does, with or without an edit. The disabled
/// effect, [`cancel`](Self::cancel) and owner disposal discard the session
/// without calling a callback. The setters work in any order, before or
/// after the sources are made.
pub struct DragApi<T: 'static> {
    observed: Signal<Option<Value<DragState>>>,
    session: MachineHandle<machine::Machine<T>>,
    /// The source of the active drag, derived from `session`.
    active_source: Value<Option<Generation>>,
    sources: Var<Generation>,
    coordinates: DragCoordinates,
    settings: OptionsCell<DragOptions<T>>,
    disabled: Signal<Option<Value<bool>>>,
}

impl<T: 'static> Clone for DragApi<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for DragApi<T> {}

/// Create a drag session under the current setup owner, reporting positions in
/// `coordinates`. Setup only.
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
/// use kivact::interaction::{DragCoordinates, DragUpdate, use_drag};
/// use kivact::prelude::*;
/// let offset = signal(0.0_f64);
/// let started = signal(0.0_f64);
/// let session = use_drag::<()>(DragCoordinates::Window)
///     .on_start(move |_| started.set(offset.get_untracked()))
///     .on_update(move |update: DragUpdate<()>| offset.set(started.get_untracked() + update.delta.x));
/// let source = session.source(());
/// let dragging = source.state();
/// let handle = node()
///     .part(source)
///     .style(move || style().size(40, 20).opacity(if dragging.get().active { 0.5 } else { 1.0 }));
/// ```
#[track_caller]
pub fn use_drag<T: Clone + 'static>(coordinates: DragCoordinates) -> DragApi<T> {
    let settings = OptionsCell::new(DragOptions::<T>::default());
    let session = machine(
        DragSession::<T>::Idle,
        move || machine::Machine::new(settings.get()),
        move |_, machine::Effect::Commit| {
            if let Some((_, edit)) = settings.get().edit {
                edit.commit();
            }
        },
    );
    let active_source = memo(move || {
        session.state().with(|session| match session {
            DragSession::Active(active) => Some(active.source),
            DragSession::Idle | DragSession::Pending { .. } | DragSession::Releasing { .. } => None,
        })
    });
    let drag = DragApi {
        observed: signal(None),
        session,
        active_source,
        sources: var(Generation::new()),
        coordinates,
        settings,
        disabled: signal(None),
    };
    session.feed(
        move || {
            drag.disabled
                .with(|disabled| disabled.is_some_and(|value| value.get()))
        },
        machine::Event::Disabled,
    );
    drag
}

impl<T: Clone + 'static> DragApi<T> {
    /// The frozen payload of the pending or active gesture, read reactively.
    ///
    /// This is `None` while idle or releasing. A pending press already exposes
    /// its press-time snapshot, even before the slop threshold is exceeded.
    /// The projection never calls a source's payload factory. Setup only.
    pub fn active_payload(self) -> Value<Option<T>>
    where
        T: PartialEq,
    {
        let state = self.session.state();
        memo(move || {
            state.with(|state| match state {
                DragSession::Pending { active, .. } | DragSession::Active(active) => {
                    Some(active.start.payload.clone())
                }
                DragSession::Idle | DragSession::Releasing { .. } => None,
            })
        })
    }

    /// Wait until the pointer moves farther than `distance` logical pixels before starting.
    /// The default is zero, which starts at the press. Setup only.
    pub fn slop(self, distance: impl Into<f64>) -> Self {
        let distance = distance.into().max(0.0);
        self.settings.update(|options| options.slop = distance);
        self
    }

    /// Lock delta and velocity to the first movement axis after the slop is exceeded. Setup only.
    pub fn axis_lock(self) -> Self {
        self.settings.update(|options| options.axis_lock = true);
        self
    }

    /// Report velocity over the latest 100 milliseconds in logical pixels per millisecond. Setup only.
    pub fn track_velocity(self) -> Self {
        self.settings
            .update(|options| options.track_velocity = true);
        self
    }

    /// Drawing state of the actual attached input target.
    pub fn source_state(self) -> Value<DragState> {
        kivact_core::state::memo(move || {
            self.observed
                .get()
                .map(|state| state.get())
                .unwrap_or(DragState {
                    input: PartState::default(),
                    active: self.is_active(),
                })
        })
    }

    /// Cancel when this source becomes disabled, and refuse new starts while disabled.
    ///
    /// The source's element stays enabled. Setup only. This is an
    /// owner-scoped effect, not an additional input/capture manager.
    #[track_caller]
    pub fn disabled(self, disabled: impl IntoValue<bool>) -> Self {
        let disabled = disabled.into_value();
        self.disabled.update(|current| *current = Some(disabled));
        self
    }

    /// Make each drag one edit of `session`: it begins with the value
    /// `start` reads before [`on_start`](Self::on_start) runs, and commits
    /// after the release's [`on_end`](Self::on_end). Escape on any source of
    /// the drag (or on a focused descendant of one), an engine Cancel and
    /// [`cancel`](Self::cancel) cancel it, which writes the starting value
    /// back; Escape and an engine Cancel do so after `on_end`. To begin at
    /// the first change instead, use [`edit_on_change`](Self::edit_on_change).
    /// A drag that the
    /// [`disabled`](Self::disabled) value stops, including the engine Cancel
    /// that disabling the source's element sends while it is true, commits
    /// instead. If the session already has an edit in progress when the drag
    /// would begin one (a field being typed in), the drag joins it and
    /// neither commits nor cancels it. Setup only.
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
    /// use kivact::interaction::{DragCoordinates, DragUpdate, EditSession, use_drag};
    /// use kivact::prelude::*;
    /// let x = signal(0.0_f64);
    /// let origin = signal(0.0_f64);
    /// let edit = EditSession::new(move |start| x.set(start))
    ///     .on_edit(|phase: EditPhase| println!("{phase:?}"));
    /// let source = use_drag::<()>(DragCoordinates::Window)
    ///     .edit(edit, move || x.get_untracked())
    ///     .on_start(move |_| origin.set(x.get_untracked()))
    ///     .on_update(move |update: DragUpdate<()>| x.set(origin.get_untracked() + update.delta.x))
    ///     .source(());
    /// let handle = node().part(source).focusable(true).name("Handle");
    /// ```
    pub fn edit<V: Clone + 'static>(
        self,
        session: EditSession<V>,
        start: impl Fn() -> V + 'static,
    ) -> Self {
        self.set_edit(BeginAt::Press, session, start)
    }

    /// Make each drag that changes the value one edit of `session`, as
    /// [`edit`](Self::edit) does, except that the edit begins at the drag's
    /// first [`change`](Self::change) instead of the press, with the value
    /// `start` reads then. A drag that never changes the value (a press that
    /// selects or opens something, a move that snaps back to the start)
    /// reports nothing. The release, Escape, an engine Cancel,
    /// [`cancel`](Self::cancel) and the [`disabled`](Self::disabled) value
    /// end a begun edit as they do with [`edit`](Self::edit). Escape while no
    /// drag is active passes on, so the component's own Escape (clearing a
    /// selection) still runs. If the session already has an edit in progress
    /// when the drag would begin one (a field being typed in), the drag joins
    /// it and neither commits nor cancels it. Setup only.
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
    /// use kivact::interaction::{DragCoordinates, DragUpdate, EditSession, use_drag};
    /// use kivact::prelude::*;
    /// let x = signal(0.0_f64);
    /// let origin = signal(0.0_f64);
    /// let edit = EditSession::new(move |start| x.set(start));
    /// let drag = use_drag::<()>(DragCoordinates::Window)
    ///     .edit_on_change(edit, move || x.get_untracked())
    ///     .on_start(move |_| origin.set(x.get_untracked()));
    /// let drag = drag.on_update(move |update: DragUpdate<()>| {
    ///     // Whole steps of 10: a small move changes nothing and begins no edit.
    ///     let next = origin.get_untracked() + (update.delta.x / 10.0).round() * 10.0;
    ///     if next != x.get_untracked() {
    ///         drag.change(|| x.set(next));
    ///     }
    /// });
    /// let handle = node().part(drag.source(())).focusable(true).name("Handle");
    /// ```
    pub fn edit_on_change<V: Clone + 'static>(
        self,
        session: EditSession<V>,
        start: impl Fn() -> V + 'static,
    ) -> Self {
        self.set_edit(BeginAt::Change, session, start)
    }

    fn set_edit<V: Clone + 'static>(
        self,
        at: BeginAt,
        session: EditSession<V>,
        start: impl Fn() -> V + 'static,
    ) -> Self {
        let edit: Rc<dyn DragEdit> = Rc::new(SessionEdit {
            session,
            start: Rc::new(start),
        });
        self.settings
            .update(|settings| settings.edit = Some((at, edit)));
        self
    }

    /// Run `write`, one change of the value the active drag edits. The first
    /// change of a drag given to [`edit_on_change`](Self::edit_on_change)
    /// begins its edit before `write` runs; the edit of [`edit`](Self::edit)
    /// began at the press. A change in the [`on_end`](Self::on_end) of a
    /// release (the last position) belongs to the drag too. Outside a drag
    /// `write` just runs.
    pub fn change<R>(self, write: impl FnOnce() -> R) -> R {
        self.begin_on_change();
        write()
    }

    /// Also start a drag with these buttons, besides the primary one, such as
    /// a middle-button pan on a canvas whose primary drag edits. The update's
    /// [`button`](DragUpdate::button) says which button started the drag, and
    /// only that button's release ends it. Setup only.
    pub fn buttons(self, buttons: impl IntoIterator<Item = MouseButton>) -> Self {
        let set = buttons
            .into_iter()
            .fold(bit(MouseButton::Primary), |set, button| set | bit(button));
        self.settings.update(|settings| settings.buttons = set);
        self
    }

    fn is_disabled(self) -> bool {
        self.disabled
            .with_untracked(|disabled| disabled.is_some_and(|value| value.get_untracked()))
    }

    /// The edit of the drag, if the drag was given one that begins `at`.
    fn edit_at(self, at: BeginAt) -> Option<Rc<dyn DragEdit>> {
        match &self.settings.get().edit {
            Some((when, edit)) if *when == at => Some(edit.clone()),
            _ => None,
        }
    }

    /// Mark the active (or releasing) drag as the owner of its edit.
    fn own_edit(self) {
        self.session.send(machine::Event::OwnEdit);
    }

    /// Whether the drag in progress (active or releasing) has no edit of its
    /// own yet; `None` outside a drag.
    fn lacks_edit(self) -> Option<bool> {
        self.session
            .state()
            .with_untracked(|session| match session {
                DragSession::Active(active) => Some(active.edit == EditOwnership::Borrowed),
                DragSession::Releasing { edit } => Some(*edit == EditOwnership::Borrowed),
                DragSession::Idle | DragSession::Pending { .. } => None,
            })
    }

    /// The press began the drag: begin its edit unless one is in progress.
    fn begin_on_press(self) {
        let Some(edit) = self.edit_at(BeginAt::Press) else {
            return;
        };
        if !edit.is_active() {
            edit.begin();
            self.own_edit();
        }
    }

    /// The drag is about to write a change: begin its edit unless it has
    /// one of its own or one is in progress.
    fn begin_on_change(self) {
        let Some(edit) = self.edit_at(BeginAt::Change) else {
            return;
        };
        if self.lacks_edit() == Some(true) && !edit.is_active() {
            edit.begin();
            self.own_edit();
        }
    }

    /// Call `f` with the Start update when a drag begins. Setup only.
    pub fn on_start<R: HandlerResult>(self, f: impl Fn(DragUpdate<T>) -> R + 'static) -> Self {
        let f = callback(f);
        self.settings.update(|settings| settings.start = Some(f));
        self
    }

    /// Call `f` with each Move update of an active drag. Setup only.
    pub fn on_update<R: HandlerResult>(self, f: impl Fn(DragUpdate<T>) -> R + 'static) -> Self {
        let f = callback(f);
        self.settings.update(|settings| settings.update = Some(f));
        self
    }

    /// Call `f` with the End or Cancel update that finishes a drag; the
    /// update's [`phase`](DragUpdate::phase) says which. Escape on a source
    /// reports Cancel at the last position. Setup only.
    pub fn on_end<R: HandlerResult>(self, f: impl Fn(DragUpdate<T>) -> R + 'static) -> Self {
        let f = callback(f);
        self.settings.update(|settings| settings.end = Some(f));
        self
    }

    /// Whether a primary drag is active.
    ///
    /// This is an untracked query; calling it in a style or memo does not
    /// subscribe that reader to drag updates. A source's
    /// [`DragState::active`] is the reactive form.
    #[track_caller]
    pub fn is_active(self) -> bool {
        self.session
            .state()
            .with_untracked(|session| matches!(session, DragSession::Active(_)))
    }

    /// Drop the session explicitly without a callback, returning what was
    /// captured at its start. The edit the drag began is canceled.
    #[track_caller]
    pub fn cancel(self) -> Option<DragStart<T>> {
        if self.clear_pending() {
            return None;
        }
        let active = self.take_active()?;
        let _ = self.stop(Stop::Dropped, active.edit == EditOwnership::Owned, None);
        Some(active.start.as_ref().clone())
    }

    fn clear_pending(self) -> bool {
        let pending = self
            .session
            .state()
            .with_untracked(|state| matches!(state, DragSession::Pending { .. }));
        if pending {
            self.session.send(machine::Event::Stop);
        }
        pending
    }

    /// Take the active session out, leaving it idle; a releasing session is
    /// left alone.
    fn take_active(self) -> Option<Active<T>> {
        if !self.is_active() {
            return None;
        }
        let active = self.session.state().with_untracked(|state| match state {
            DragSession::Active(active) => Some(active.clone()),
            _ => None,
        });
        self.session.send(machine::Event::Stop);
        active
    }

    /// End a drag the caller has already taken: run `on_end` for a release or
    /// an engine Cancel, then commit or cancel the edit the drag began.
    fn stop(
        self,
        stop: Stop,
        mut owns_edit: bool,
        update: Option<DragUpdate<T>>,
    ) -> kivact_core::Result<()> {
        let end = self.settings.get().end;
        let mut result = Ok(());
        match (stop, update, end) {
            (Stop::Release, Some(update), end) => {
                self.session.send(machine::Event::Release(if owns_edit {
                    EditOwnership::Owned
                } else {
                    EditOwnership::Borrowed
                }));
                if let Some(end) = end {
                    result = end.try_call(update);
                }
                // A `change` in `on_end` may have begun the edit.
                owns_edit = self
                    .session
                    .state()
                    .with_untracked(|session| match session {
                        DragSession::Releasing { edit } => *edit == EditOwnership::Owned,
                        DragSession::Active(_)
                        | DragSession::Pending { .. }
                        | DragSession::Idle => owns_edit,
                    });
                self.session.send(machine::Event::Stop);
            }
            (Stop::Cancel | Stop::CancelDisabled, Some(update), Some(end)) => {
                result = end.try_call(update)
            }
            _ => {}
        }
        if owns_edit && let Some((_, edit)) = self.settings.get().edit {
            match stop {
                Stop::Release | Stop::Disabled | Stop::CancelDisabled => edit.commit(),
                Stop::Dropped => edit.cancel(),
                Stop::Cancel if self.is_disabled() => edit.commit(),
                Stop::Cancel => edit.cancel(),
            }
        }
        result
    }

    /// Create the input attachment of one place a drag can start, carrying
    /// `payload` into every update of a drag that starts there. Read its state
    /// before moving it.
    ///
    /// Accepted updates consume the pointer event, so later parts on the same
    /// target do not see it. Enter/Leave are ignored. End/Cancel are delivered
    /// even when the disabled source is true, so a caller still observes an
    /// engine cancellation before the effect runs. Escape here during a drag
    /// stops it like an engine Cancel and is consumed, whichever source of the
    /// drag it started on: the keyboard focus may sit on another source of the
    /// same component (a canvas whose regions are the other sources). Without
    /// an active drag Escape passes on. The node's own key handlers run before
    /// its parts, so a component's Escape that should only see what the drag
    /// passes on goes in a part attached after the source. Setup only.
    #[track_caller]
    pub fn source(self, payload: T) -> Part<DragState> {
        self.source_with(move || payload.clone())
    }

    /// Create a source whose payload is sampled once when a press is accepted.
    ///
    /// The factory runs after the pointer, button, disabled state and idle
    /// session checks succeed, before entering the pending or active state.
    /// Its result is the frozen payload of every update in that gesture;
    /// movement and release never call the factory again. A later accepted
    /// press takes a fresh snapshot. With slop, the snapshot is taken at the
    /// press, before the movement threshold is exceeded.
    ///
    /// Use [`source`](Self::source) for fixed setup data. All capture,
    /// cancellation and disabled behavior is the same. Setup only.
    #[track_caller]
    pub fn source_with(self, payload: impl Fn() -> T + 'static) -> Part<DragState> {
        let id = self.sources.update(Generation::next);
        let active_source = self.active_source;
        let input = part(move |input| DragState {
            input,
            active: active_source.get() == Some(id),
        });
        let part_state = input.state();
        self.observed.set(Some(part_state));
        Part::new(part_state, move |mut host| {
            let caller_disabled = host.accessible_disabled();
            host.part(input)
                .on_pointer(move |event: PointerEvent| -> kivact_core::Result<Handled> {
                    match self.observe(
                        id,
                        &payload,
                        event,
                        caller_disabled.is_some_and(|value| value.get_untracked()),
                    ) {
                        None => Ok(Handled::No),
                        Some(Step::Pending) => Ok(Handled::Yes),
                        Some(Step::Started { start, moved }) => {
                            self.report(start)?;
                            self.report(moved)?;
                            Ok(Handled::Yes)
                        }
                        Some(Step::Update(update)) => {
                            self.report(update)?;
                            Ok(Handled::Yes)
                        }
                        Some(Step::Stop {
                            stop,
                            owns_edit,
                            update,
                        }) => {
                            let handled = update.is_some();
                            self.stop(stop, owns_edit, update)?;
                            Ok(if handled { Handled::Yes } else { Handled::No })
                        }
                    }
                })
                .on_key(move |event: KeyEvent| -> kivact_core::Result<Handled> {
                    if event.key() != Key::Escape {
                        return Ok(Handled::No);
                    }
                    if self.clear_pending() {
                        return Ok(Handled::Yes);
                    }
                    let Some(active) = self.take_active() else {
                        return Ok(Handled::No);
                    };
                    let cancel = active.update(active.last, event.modifiers(), DragPhase::Cancel);
                    self.stop(Stop::Cancel, active.edit == EditOwnership::Owned, cancel)?;
                    Ok(Handled::Yes)
                })
        })
    }

    /// Pass the Start or Move update to its callback; a Start begins the edit
    /// first.
    fn report(self, update: DragUpdate<T>) -> kivact_core::Result<()> {
        let callback = if update.phase == DragPhase::Start {
            self.begin_on_press();
            self.settings.get().start
        } else {
            self.settings.get().update
        };
        match callback {
            Some(callback) => callback.try_call(update),
            None => Ok(()),
        }
    }

    fn begin_pointer(
        self,
        id: Generation,
        payload: &dyn Fn() -> T,
        event: PointerEvent,
        button: MouseButton,
        pos: Point,
        source_disabled: bool,
    ) -> Option<Step<T>> {
        let buttons = self.settings.get().buttons;
        if buttons & bit(button) == 0
            || !pos.x.is_finite()
            || !pos.y.is_finite()
            || self.is_disabled()
            || source_disabled
            || !self
                .session
                .state()
                .with_untracked(|state| matches!(state, DragSession::Idle))
        {
            return None;
        }
        let start = DragStart {
            pointer: event.id(),
            button,
            pos,
            payload: payload(),
        };
        self.session.send(machine::Event::Begin(
            Active {
                start: Rc::new(start),
                source: id,
                last: pos,
                edit: EditOwnership::Borrowed,
                axis: super::data::AxisLock::Free,
                velocity: super::data::Velocity::Disabled,
            },
            clock_now(),
        ));
        self.session.state().with_untracked(|state| match state {
            DragSession::Active(active) => active
                .update(pos, event.modifiers(), DragPhase::Start)
                .map(Step::Update),
            DragSession::Pending { .. } => Some(Step::Pending),
            _ => None,
        })
    }

    fn move_pointer(self, event: PointerEvent, pos: Point, valid: bool) -> Option<Step<T>> {
        let phase = DragPhase::Move;
        if !valid {
            // An invalid move must not corrupt the drag's positions.
            return None;
        }
        let pending = self
            .session
            .state()
            .with_untracked(|state| matches!(state, DragSession::Pending { .. }));
        self.session
            .send(machine::Event::Position(pos, clock_now()));
        self.session.state().with_untracked(|state| match state {
            DragSession::Active(active) => {
                let moved = active.update(pos, event.modifiers(), phase)?;
                if pending {
                    Some(Step::Started {
                        start: active.update(pos, event.modifiers(), DragPhase::Start)?,
                        moved,
                    })
                } else {
                    Some(Step::Update(moved))
                }
            }
            DragSession::Pending { .. } => Some(Step::Pending),
            _ => None,
        })
    }

    /// Advance the session by one pointer event of source `id`.
    fn observe(
        self,
        id: Generation,
        payload: &dyn Fn() -> T,
        event: PointerEvent,
        source_disabled: bool,
    ) -> Option<Step<T>> {
        let pos = match self.coordinates {
            DragCoordinates::Local => event.pos(),
            DragCoordinates::Window => event.window_pos(),
        };
        let valid = pos.x.is_finite() && pos.y.is_finite();
        if let PointerPhase::Down(press) = event.phase() {
            return self.begin_pointer(id, payload, event, press.button, pos, source_disabled);
        }
        if !matches!(
            event.phase(),
            PointerPhase::Move | PointerPhase::Up(_) | PointerPhase::Cancel
        ) {
            return None;
        }
        let (owned, releases) = self
            .session
            .state()
            .with_untracked(|session| match session {
                DragSession::Active(active) | DragSession::Pending { active, .. }
                    if active.source == id && active.start.pointer == event.id() =>
                {
                    let releases = matches!(
                        event.phase(),
                        PointerPhase::Up(press) if press.button == active.start.button
                    );
                    (true, releases)
                }
                _ => (false, false),
            });
        if !owned {
            return None;
        }
        let phase = match event.phase() {
            PointerPhase::Move => {
                if self.is_disabled() {
                    if let Some(active) = self.take_active() {
                        let _ =
                            self.stop(Stop::Disabled, active.edit == EditOwnership::Owned, None);
                    }
                    return None;
                }
                DragPhase::Move
            }
            PointerPhase::Up(_) if releases => DragPhase::End,
            PointerPhase::Cancel => DragPhase::Cancel,
            _ => return None,
        };
        if phase == DragPhase::Move {
            return self.move_pointer(event, pos, valid);
        }
        if self.clear_pending() {
            return Some(Step::Pending);
        }
        let active = self.take_active()?;
        let active = machine::position(
            &active,
            if valid { pos } else { active.last },
            clock_now(),
            self.settings.get().axis_lock,
        );
        // A terminal event with invalid coordinates still ends the drag,
        // where the pointer last was.
        let pos = if valid { pos } else { active.last };
        let stop = if phase == DragPhase::End {
            Stop::Release
        } else if source_disabled {
            Stop::CancelDisabled
        } else {
            Stop::Cancel
        };
        Some(Step::Stop {
            stop,
            owns_edit: active.edit == EditOwnership::Owned,
            update: active.update(pos, event.modifiers(), phase),
        })
    }
}

fn update<T>(
    start: DragStart<T>,
    pos: Point,
    modifiers: Modifiers,
    phase: DragPhase,
) -> Option<DragUpdate<T>> {
    let delta = Point::new(pos.x - start.pos.x, pos.y - start.pos.y);
    if !delta.x.is_finite() || !delta.y.is_finite() {
        return None;
    }
    Some(DragUpdate {
        phase,
        pointer: start.pointer,
        button: start.button,
        start_pos: start.pos,
        pos,
        delta,
        velocity: Point::new(0.0, 0.0),
        modifiers,
        payload: start.payload,
    })
}
