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

use crate::edit::EditSession;
use kivact_core::custom::{Part, PartState, part};
use kivact_core::geometry::Point;
use kivact_core::state::{Callback, Signal, Value, Var, effect, memo, signal, untrack, var};
use kivact_core::state::{IntoValue, callback};
use kivact_core::view::HandlerResult;
use kivact_core::view::{
    Handled, Key, KeyEvent, Modifiers, MouseButton, PointerEvent, PointerId, PointerPhase,
};
use std::rc::Rc;

/// Coordinates used consistently for a drag's start, position and delta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragCoordinates {
    /// Node-local logical coordinates. Suitable for mapping within a fixed box.
    Local,
    /// Window logical coordinates. Suitable for moving or resizing a box.
    Window,
}

/// The operation a [`DragUpdate`] reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragPhase {
    /// A primary pointer began the session.
    Start,
    /// The active pointer moved.
    Move,
    /// The active pointer released the primary button.
    End,
    /// The engine canceled the active pointer.
    Cancel,
}

/// Identity, position and payload captured once at the start of a drag.
#[derive(Clone, Debug)]
pub struct DragStart<T> {
    /// Identity of the pointer.
    pub pointer: PointerId,
    /// The button that started the drag; a touch or pen contact is
    /// [`MouseButton::Primary`].
    pub button: MouseButton,
    /// Initial position in the chosen coordinate space.
    pub pos: Point,
    /// The payload of the source the drag started on.
    pub payload: T,
}

/// A drag update in the session's chosen coordinate space.
#[derive(Clone, Debug)]
pub struct DragUpdate<T> {
    /// Accepted drag operation.
    pub phase: DragPhase,
    /// Identity captured on Start.
    pub pointer: PointerId,
    /// The button captured on Start.
    pub button: MouseButton,
    /// Initial position in the chosen coordinate space.
    pub start_pos: Point,
    /// Current position in the same coordinate space.
    pub pos: Point,
    /// Current position minus initial position.
    pub delta: Point,
    /// Modifier keys held during this pointer event.
    pub modifiers: Modifiers,
    /// The payload of the source the drag started on, cloned for each update.
    pub payload: T,
}

/// Reactive drawing state of an attached drag source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DragState {
    pub input: PartState,
    /// A drag that started on this source is in progress.
    pub active: bool,
}

/// One drag of a [`DragApi`]: nothing, a pointer moving, or the release's
/// `on_end` running.
enum DragSession<T> {
    Idle,
    Active(Active<T>),
    /// The release's `on_end` runs: a `change` there still belongs to the drag.
    Releasing {
        owns_edit: bool,
    },
}

#[derive(Clone)]
struct Active<T> {
    start: DragStart<T>,
    source: u64,
    /// The last valid position, where a drag that Escape stops ends.
    last: Point,
    /// This drag began the edit given to [`DragApi::edit`] or
    /// [`DragApi::edit_on_change`]; only then does it commit or cancel it.
    owns_edit: bool,
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
    /// The disabled value became true: no callback, the drag's own edit
    /// commits where it is.
    Disabled,
    /// [`DragApi::cancel`]: no callback, the drag's own edit cancels.
    Dropped,
}

/// When a drag begins its edit.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BeginAt {
    Press,
    Change,
}

/// The edit session a drag drives, with its value type erased.
trait DragEdit {
    fn is_active(&self) -> bool;
    fn begin(&self);
    fn commit(&self);
    fn cancel(&self);
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

/// What the setters of a [`DragApi`] store; read at the event, so the setters
/// work in any order and after the sources are made.
struct DragSettings<T: 'static> {
    buttons: u8,
    start: Option<Callback<DragUpdate<T>>>,
    update: Option<Callback<DragUpdate<T>>>,
    end: Option<Callback<DragUpdate<T>>>,
    edit: Option<(BeginAt, Rc<dyn DragEdit>)>,
}

impl<T: 'static> Default for DragSettings<T> {
    fn default() -> Self {
        Self {
            buttons: bit(MouseButton::Primary),
            start: None,
            update: None,
            end: None,
            edit: None,
        }
    }
}

/// The bit of `button` in [`DragApi`]'s set of starting buttons.
fn bit(button: MouseButton) -> u8 {
    match button {
        MouseButton::Primary => 1,
        MouseButton::Secondary => 2,
        MouseButton::Middle => 4,
        MouseButton::Back => 8,
        MouseButton::Forward => 16,
    }
}

/// What one pointer event did to the session.
enum Step<T> {
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
    session: Signal<DragSession<T>>,
    /// The source of the active drag, derived from `session`.
    active_source: Value<Option<u64>>,
    sources: Var<u64>,
    coordinates: DragCoordinates,
    settings: Var<DragSettings<T>>,
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
    let session = signal(DragSession::<T>::Idle);
    let active_source = memo(move || {
        session.with(|session| match session {
            DragSession::Active(active) => Some(active.source),
            DragSession::Idle | DragSession::Releasing { .. } => None,
        })
    });
    let drag = DragApi {
        observed: signal(None),
        session,
        active_source,
        sources: var(0),
        coordinates,
        settings: var(DragSettings::default()),
        disabled: signal(None),
    };
    effect(move || {
        let off = drag
            .disabled
            .with(|disabled| disabled.is_some_and(|value| value.get()));
        if off {
            untrack(|| {
                if let Some(active) = drag.take_active() {
                    let _ = drag.stop(Stop::Disabled, active.owns_edit, None);
                }
            });
        }
    });
    drag
}

impl<T: Clone + 'static> DragApi<T> {
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
        self.settings.with(|settings| match &settings.edit {
            Some((when, edit)) if *when == at => Some(edit.clone()),
            _ => None,
        })
    }

    /// Mark the active (or releasing) drag as the owner of its edit.
    fn own_edit(self) {
        self.session.update(|session| match session {
            DragSession::Active(active) => active.owns_edit = true,
            DragSession::Releasing { owns_edit } => *owns_edit = true,
            DragSession::Idle => {}
        });
    }

    /// Whether the drag in progress (active or releasing) has no edit of its
    /// own yet; `None` outside a drag.
    fn lacks_edit(self) -> Option<bool> {
        self.session.with_untracked(|session| match session {
            DragSession::Active(active) => Some(!active.owns_edit),
            DragSession::Releasing { owns_edit } => Some(!owns_edit),
            DragSession::Idle => None,
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
            .with_untracked(|session| matches!(session, DragSession::Active(_)))
    }

    /// Drop the session explicitly without a callback, returning what was
    /// captured at its start. The edit the drag began is canceled.
    #[track_caller]
    pub fn cancel(self) -> Option<DragStart<T>> {
        let active = self.take_active()?;
        let _ = self.stop(Stop::Dropped, active.owns_edit, None);
        Some(active.start)
    }

    /// Take the active session out, leaving it idle; a releasing session is
    /// left alone.
    fn take_active(self) -> Option<Active<T>> {
        if !self.is_active() {
            return None;
        }
        let taken = self
            .session
            .update(|session| std::mem::replace(session, DragSession::Idle));
        match taken {
            DragSession::Active(active) => Some(active),
            DragSession::Idle | DragSession::Releasing { .. } => None,
        }
    }

    /// End a drag the caller has already taken: run `on_end` for a release or
    /// an engine Cancel, then commit or cancel the edit the drag began.
    fn stop(
        self,
        stop: Stop,
        mut owns_edit: bool,
        update: Option<DragUpdate<T>>,
    ) -> kivact_core::Result<()> {
        let end = self.settings.with(|settings| settings.end);
        let mut result = Ok(());
        match (stop, update, end) {
            (Stop::Release, Some(update), end) => {
                self.session
                    .update(|session| *session = DragSession::Releasing { owns_edit });
                if let Some(end) = end {
                    result = end.try_call(update);
                }
                // A `change` in `on_end` may have begun the edit.
                owns_edit = self.session.with_untracked(|session| match session {
                    DragSession::Releasing { owns_edit } => *owns_edit,
                    DragSession::Active(_) | DragSession::Idle => owns_edit,
                });
                self.session.update(|session| *session = DragSession::Idle);
            }
            (Stop::Cancel, Some(update), Some(end)) => result = end.try_call(update),
            _ => {}
        }
        if owns_edit && let Some((_, edit)) = self.settings.with(|s| s.edit.clone()) {
            match stop {
                Stop::Release | Stop::Disabled => edit.commit(),
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
        let id = self.sources.update(|count| {
            *count += 1;
            *count
        });
        let active_source = self.active_source;
        let input = part(move |input| DragState {
            input,
            active: active_source.get() == Some(id),
        });
        let part_state = input.state();
        self.observed.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |mut host| {
            let caller_disabled = host.accessible_disabled();
            let own_disabled = self.disabled.get_untracked();
            self.disabled.set(Some(kivact_core::state::memo(move || {
                caller_disabled.is_some_and(|value| value.get())
                    || own_disabled.is_some_and(|value| value.get())
            })));
            host.part(input)
                .node_ref(target)
                .on_pointer(move |event: PointerEvent| -> kivact_core::Result<Handled> {
                    match self.observe(id, &payload, event) {
                        None => Ok(Handled::No),
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
                    let Some(active) = self.take_active() else {
                        return Ok(Handled::No);
                    };
                    let cancel = update(
                        active.start,
                        active.last,
                        event.modifiers(),
                        DragPhase::Cancel,
                    );
                    self.stop(Stop::Cancel, active.owns_edit, cancel)?;
                    Ok(Handled::Yes)
                })
        })
    }

    /// Pass the Start or Move update to its callback; a Start begins the edit
    /// first.
    fn report(self, update: DragUpdate<T>) -> kivact_core::Result<()> {
        let callback = if update.phase == DragPhase::Start {
            self.begin_on_press();
            self.settings.with(|settings| settings.start)
        } else {
            self.settings.with(|settings| settings.update)
        };
        match callback {
            Some(callback) => callback.try_call(update),
            None => Ok(()),
        }
    }

    /// Advance the session by one pointer event of source `id`.
    fn observe(self, id: u64, payload: &T, event: PointerEvent) -> Option<Step<T>> {
        let pos = match self.coordinates {
            DragCoordinates::Local => event.pos(),
            DragCoordinates::Window => event.window_pos(),
        };
        let valid = pos.x.is_finite() && pos.y.is_finite();
        if let PointerPhase::Down(press) = event.phase() {
            let button = press.button;
            let buttons = self.settings.with(|settings| settings.buttons);
            if buttons & bit(button) == 0 || !valid || self.is_disabled() || self.is_active() {
                return None;
            }
            let start = DragStart {
                pointer: event.id(),
                button,
                pos,
                payload: payload.clone(),
            };
            self.session.update(|session| {
                *session = DragSession::Active(Active {
                    start: start.clone(),
                    source: id,
                    last: pos,
                    owns_edit: false,
                });
            });
            return update(start, pos, event.modifiers(), DragPhase::Start).map(Step::Update);
        }
        if !matches!(
            event.phase(),
            PointerPhase::Move | PointerPhase::Up(_) | PointerPhase::Cancel
        ) {
            return None;
        }
        let (owned, releases) = self.session.with_untracked(|session| match session {
            DragSession::Active(active)
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
                        let _ = self.stop(Stop::Disabled, active.owns_edit, None);
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
            if !valid {
                // An invalid move must not corrupt the drag's positions.
                return None;
            }
            let start = self.session.update(|session| match session {
                DragSession::Active(active) => {
                    active.last = pos;
                    Some(active.start.clone())
                }
                DragSession::Idle | DragSession::Releasing { .. } => None,
            })?;
            return update(start, pos, event.modifiers(), phase).map(Step::Update);
        }
        let active = self.take_active()?;
        // A terminal event with invalid coordinates still ends the drag,
        // where the pointer last was.
        let pos = if valid { pos } else { active.last };
        let stop = if phase == DragPhase::End {
            Stop::Release
        } else {
            Stop::Cancel
        };
        Some(Step::Stop {
            stop,
            owns_edit: active.owns_edit,
            update: update(active.start, pos, event.modifiers(), phase),
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
        modifiers,
        payload: start.payload,
    })
}
