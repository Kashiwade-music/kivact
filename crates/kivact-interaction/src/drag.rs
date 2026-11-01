//! Role: Owner-scoped pointer drag sessions attached to sources as parts.
//!
//! [`use_drag`] keeps only the pointer identity and the start of one session
//! in its owner; the caller takes any snapshot of its own value in
//! `on_start`, and an external binding write does not take it again. A drag
//! is one edit of an [`EditSession`] through [`Drag::edit`] (begun at the
//! press) or [`Drag::edit_on_change`] (begun at the first [`Drag::change`],
//! so a press that only selects reports nothing). The splitter and the
//! slider's track use `edit`; Creative's domain editors use `edit_on_change`
//! with a source on the canvas and on each region, and take a middle-button
//! pan in the same drag with [`Drag::buttons`]. Zoom and pan does not use a
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
//! - The disabled effect, `cancel` and owner disposal drop the session
//!   without a callback; the engine Cancel that disabling the node sends is
//!   observed by `on_end`. A drag stopped by the disabled value commits its
//!   edit.

use crate::edit::EditSession;
use kivact_core::draw::Point;
use kivact_core::primitive::{Part, PartState, part};
use kivact_core::view::HandlerResult;
use kivact_core::view::{
    Handled, IntoValue, Key, KeyEvent, Modifiers, MouseButton, PointerEvent, PointerId,
    PointerPhase, callback,
};
use kivact_core::{Callback, Signal, Value, Var, effect, signal, var};

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

/// What a drag does to the edit session given to [`Drag::edit`] or
/// [`Drag::edit_on_change`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditStep {
    /// The press started the drag.
    Press,
    /// The drag is about to write a change ([`Drag::change`]).
    Change,
    Commit,
    Cancel,
}

#[derive(Clone)]
struct Active<T> {
    start: DragStart<T>,
    source: u64,
    /// The last valid position, where a drag that Escape stops ends.
    last: Point,
}

/// The bit of `button` in [`Drag`]'s set of starting buttons.
fn bit(button: MouseButton) -> u8 {
    match button {
        MouseButton::Primary => 1,
        MouseButton::Secondary => 2,
        MouseButton::Middle => 4,
        MouseButton::Back => 8,
        MouseButton::Forward => 16,
    }
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
/// without calling a callback.
pub struct Drag<T: 'static> {
    state: Var<Option<Active<T>>>,
    /// The release's [`on_end`](Drag::on_end) is running: a change there
    /// still belongs to the drag.
    releasing: Var<bool>,
    active: Signal<Option<u64>>,
    sources: Var<u64>,
    coordinates: DragCoordinates,
    buttons: u8,
    disabled: Option<Value<bool>>,
    start: Option<Callback<DragUpdate<T>>>,
    update: Option<Callback<DragUpdate<T>>>,
    end: Option<Callback<DragUpdate<T>>>,
    edit: Option<Callback<EditStep>>,
}

impl<T: 'static> Clone for Drag<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Drag<T> {}

/// Create a drag session under the current setup owner, reporting positions in
/// `coordinates`. Setup only.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::prelude;
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{DragCoordinates, DragUpdate, use_drag};
/// use kivact::prelude::*;
/// let offset = signal(0.0_f32);
/// let started = signal(0.0_f32);
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
pub fn use_drag<T: Clone + 'static>(coordinates: DragCoordinates) -> Drag<T> {
    Drag {
        state: var(None),
        releasing: var(false),
        active: signal(None),
        sources: var(0),
        coordinates,
        buttons: bit(MouseButton::Primary),
        disabled: None,
        start: None,
        update: None,
        end: None,
        edit: None,
    }
}

impl<T: Clone + 'static> Drag<T> {
    /// Cancel when this source becomes disabled, and refuse new starts while disabled.
    ///
    /// The source's element stays enabled. Setup only. Configure once, before
    /// attaching sources. This is an owner-scoped effect, not an additional
    /// input/capture manager.
    #[track_caller]
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        let disabled = disabled.into_value();
        self.disabled = Some(disabled);
        effect(move || {
            // The edit of a drag that stops this way is kept where it is.
            if disabled.get() && self.discard().is_some() {
                self.step(EditStep::Commit);
            }
        });
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
    /// instead. Setup only. Configure once, before attaching sources.
    ///
    /// ```no_run
    /// # mod kivact {
    /// #     //! Role: Supply facade paths for this documentation example.
    /// #     pub use kivact_core::prelude;
    /// #     pub extern crate kivact_interaction as interaction;
    /// # }
    /// use kivact::interaction::{DragCoordinates, DragUpdate, EditPhase, EditSession, use_drag};
    /// use kivact::prelude::*;
    /// let x = signal(0.0_f32);
    /// let origin = signal(0.0_f32);
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
        mut self,
        session: EditSession<V>,
        start: impl Fn() -> V + 'static,
    ) -> Self {
        self.edit = Some(callback(move |step| match step {
            EditStep::Press => session.begin(start()),
            EditStep::Change => {}
            EditStep::Commit => session.commit(),
            EditStep::Cancel => {
                session.cancel();
            }
        }));
        self
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
    /// selection) still runs. Setup only. Configure once, before attaching
    /// sources.
    ///
    /// ```no_run
    /// # mod kivact {
    /// #     //! Role: Supply facade paths for this documentation example.
    /// #     pub use kivact_core::prelude;
    /// #     pub extern crate kivact_interaction as interaction;
    /// # }
    /// use kivact::interaction::{DragCoordinates, DragUpdate, EditSession, use_drag};
    /// use kivact::prelude::*;
    /// let x = signal(0.0_f32);
    /// let origin = signal(0.0_f32);
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
        mut self,
        session: EditSession<V>,
        start: impl Fn() -> V + 'static,
    ) -> Self {
        self.edit = Some(callback(move |step| match step {
            EditStep::Press => {}
            // `start` reads the value before the drag's first change only:
            // later changes are inside the begun edit.
            EditStep::Change => {
                if !session.is_active() {
                    session.begin(start());
                }
            }
            EditStep::Commit => session.commit(),
            EditStep::Cancel => {
                session.cancel();
            }
        }));
        self
    }

    /// Run `write`, one change of the value the active drag edits. The first
    /// change of a drag given to [`edit_on_change`](Self::edit_on_change)
    /// begins its edit before `write` runs; the edit of [`edit`](Self::edit)
    /// began at the press. A change in the [`on_end`](Self::on_end) of a
    /// release (the last position) belongs to the drag too. Outside a drag
    /// `write` just runs.
    pub fn change<R>(self, write: impl FnOnce() -> R) -> R {
        if self.is_active() || self.releasing.get() {
            self.step(EditStep::Change);
        }
        write()
    }

    /// Also start a drag with these buttons, besides the primary one, such as
    /// a middle-button pan on a canvas whose primary drag edits. The update's
    /// [`button`](DragUpdate::button) says which button started the drag, and
    /// only that button's release ends it. Setup only. Configure once, before
    /// attaching sources.
    pub fn buttons(mut self, buttons: impl IntoIterator<Item = MouseButton>) -> Self {
        self.buttons = buttons
            .into_iter()
            .fold(bit(MouseButton::Primary), |set, button| set | bit(button));
        self
    }

    fn step(self, step: EditStep) {
        if let Some(edit) = self.edit {
            edit.call(step);
        }
    }

    fn is_disabled(self) -> bool {
        self.disabled.is_some_and(|value| value.get_untracked())
    }

    /// Call `f` with the Start update when a drag begins. Setup only.
    pub fn on_start<R: HandlerResult>(mut self, f: impl Fn(DragUpdate<T>) -> R + 'static) -> Self {
        self.start = Some(callback(f));
        self
    }

    /// Call `f` with each Move update of an active drag. Setup only.
    pub fn on_update<R: HandlerResult>(mut self, f: impl Fn(DragUpdate<T>) -> R + 'static) -> Self {
        self.update = Some(callback(f));
        self
    }

    /// Call `f` with the End or Cancel update that finishes a drag; the
    /// update's [`phase`](DragUpdate::phase) says which. Escape on a source
    /// reports Cancel at the last position. Setup only.
    pub fn on_end<R: HandlerResult>(mut self, f: impl Fn(DragUpdate<T>) -> R + 'static) -> Self {
        self.end = Some(callback(f));
        self
    }

    /// Whether a primary drag is active.
    ///
    /// This is an untracked query; calling it in a style or memo does not
    /// subscribe that reader to drag updates. A source's
    /// [`DragState::active`] is the reactive form.
    #[track_caller]
    pub fn is_active(self) -> bool {
        self.state.with(Option::is_some)
    }

    /// Drop the session explicitly without a callback, returning what was
    /// captured at its start. The edit given to [`edit`](Self::edit) is
    /// canceled.
    #[track_caller]
    pub fn cancel(self) -> Option<DragStart<T>> {
        let active = self.discard()?;
        self.step(EditStep::Cancel);
        Some(active.start)
    }

    fn discard(self) -> Option<Active<T>> {
        let active = self.state.update(Option::take);
        if self.active.get_untracked().is_some() {
            self.active.set(None);
        }
        active
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
        let active = self.active;
        part(move |input| DragState {
            input,
            active: active.get() == Some(id),
        })
        .on_pointer(move |event: PointerEvent| -> kivact_core::Result<Handled> {
            let Some(update) = self.observe(id, &payload, event) else {
                return Ok(Handled::No);
            };
            self.report(update)?;
            Ok(Handled::Yes)
        })
        .on_key(move |event: KeyEvent| -> kivact_core::Result<Handled> {
            if event.key != Key::Escape {
                return Ok(Handled::No);
            }
            let Some(active) = self.discard() else {
                return Ok(Handled::No);
            };
            match update(
                active.start,
                active.last,
                event.modifiers,
                DragPhase::Cancel,
            ) {
                Some(update) => self.report(update)?,
                None => self.step(EditStep::Cancel),
            }
            Ok(Handled::Yes)
        })
    }

    /// Pass one accepted update to its callback and advance the edit.
    fn report(self, update: DragUpdate<T>) -> kivact_core::Result<()> {
        let phase = update.phase;
        let callback = match phase {
            DragPhase::Start => {
                self.step(EditStep::Press);
                self.start
            }
            DragPhase::Move => self.update,
            DragPhase::End | DragPhase::Cancel => self.end,
        };
        let releasing = phase == DragPhase::End;
        if releasing {
            self.releasing.set(true);
        }
        let result = match callback {
            Some(callback) => callback.try_call(update),
            None => Ok(()),
        };
        if releasing {
            self.releasing.set(false);
        }
        match phase {
            DragPhase::End => self.step(EditStep::Commit),
            DragPhase::Cancel => self.close_edit(true),
            _ => {}
        }
        result
    }

    /// Advance the session by one pointer event of source `id`.
    fn observe(self, id: u64, payload: &T, event: PointerEvent) -> Option<DragUpdate<T>> {
        let pos = match self.coordinates {
            DragCoordinates::Local => event.pos,
            DragCoordinates::Window => event.window_pos,
        };
        let owns = |active: &Option<Active<T>>| {
            active
                .as_ref()
                .is_some_and(|a| a.source == id && a.start.pointer == event.id)
        };
        let valid = pos.x.is_finite() && pos.y.is_finite();
        if event.phase == PointerPhase::Down {
            let button = event.button?;
            if self.buttons & bit(button) == 0 || !valid || self.is_disabled() || self.is_active() {
                return None;
            }
            let start = DragStart {
                pointer: event.id,
                button,
                pos,
                payload: payload.clone(),
            };
            self.state.set(Some(Active {
                start: start.clone(),
                source: id,
                last: pos,
            }));
            self.active.set(Some(id));
            return update(start, pos, event.modifiers, DragPhase::Start);
        }
        if !matches!(
            event.phase,
            PointerPhase::Move | PointerPhase::Up | PointerPhase::Cancel
        ) {
            return None;
        }
        if !self.state.with(owns) {
            return None;
        }
        let releases = self.state.with(|active| {
            active
                .as_ref()
                .is_some_and(|a| event.button == Some(a.start.button))
        });
        let phase = match event.phase {
            PointerPhase::Move => {
                if self.is_disabled() {
                    self.discard();
                    self.step(EditStep::Commit);
                    return None;
                }
                DragPhase::Move
            }
            PointerPhase::Up if releases => DragPhase::End,
            PointerPhase::Cancel => DragPhase::Cancel,
            _ => return None,
        };
        let terminal = matches!(phase, DragPhase::End | DragPhase::Cancel);
        if !terminal {
            if !valid {
                // An invalid move must not corrupt the drag's positions.
                return None;
            }
            let start = self.state.update(|active| {
                let active = active.as_mut()?;
                active.last = pos;
                Some(active.start.clone())
            })?;
            return update(start, pos, event.modifiers, phase);
        }
        let active = self.discard()?;
        // A terminal event with invalid coordinates still ends the drag,
        // where the pointer last was.
        let pos = if valid { pos } else { active.last };
        let update = update(active.start, pos, event.modifiers, phase);
        if update.is_none() {
            // No update reaches the source's handler, which would have closed
            // the edit: close it here, so it never stays open.
            self.close_edit(phase == DragPhase::Cancel);
        }
        update
    }

    /// End the edit of a drag that stopped: a cancel cancels it, unless the
    /// disabled value stopped the drag, which ends where it is; a release
    /// commits it.
    fn close_edit(self, canceled: bool) {
        self.step(if canceled && !self.is_disabled() {
            EditStep::Cancel
        } else {
            EditStep::Commit
        });
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
