//! Role: Shared drag values, options and gesture snapshots.
use crate::DragVelocity;
use kivact_core::custom::PartState;
use kivact_core::geometry::Point;
use kivact_core::state::Callback;
use kivact_core::view::{Modifiers, MouseButton, PointerId};
use kivact_types::cell::Generation;
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
    /// Recent pointer velocity in logical pixels per millisecond; zero unless tracking is enabled.
    pub velocity: Point,
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

pub(super) struct Active<T> {
    pub(super) start: Rc<DragStart<T>>,
    pub(super) source: Generation,
    pub(super) last: Point,
    pub(super) edit: EditOwnership,
    pub(super) axis: AxisLock,
    pub(super) velocity: Velocity,
}
impl<T> Clone for Active<T> {
    fn clone(&self) -> Self {
        Self {
            start: self.start.clone(),
            source: self.source,
            last: self.last,
            edit: self.edit,
            axis: self.axis,
            velocity: self.velocity.clone(),
        }
    }
}

/// When a drag begins its edit.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BeginAt {
    Press,
    Change,
}

/// The edit session a drag drives, with its value type erased.
pub(super) trait DragEdit {
    fn is_active(&self) -> bool;
    fn begin(&self);
    fn commit(&self);
    fn cancel(&self);
}

/// What the setters of a [`crate::DragApi`] store; read at the event, so the setters
/// work in any order and after the sources are made.
pub(super) struct DragOptions<T: 'static> {
    pub(super) buttons: u8,
    pub(super) slop: f64,
    pub(super) axis_lock: bool,
    pub(super) track_velocity: bool,
    pub(super) start: Option<Callback<DragUpdate<T>>>,
    pub(super) update: Option<Callback<DragUpdate<T>>>,
    pub(super) end: Option<Callback<DragUpdate<T>>>,
    pub(super) edit: Option<(BeginAt, Rc<dyn DragEdit>)>,
}

impl<T: 'static> Clone for DragOptions<T> {
    fn clone(&self) -> Self {
        Self {
            buttons: self.buttons,
            slop: self.slop,
            axis_lock: self.axis_lock,
            track_velocity: self.track_velocity,
            start: self.start,
            update: self.update,
            end: self.end,
            edit: self.edit.clone(),
        }
    }
}

impl<T: 'static> Default for DragOptions<T> {
    fn default() -> Self {
        Self {
            buttons: bit(MouseButton::Primary),
            slop: 0.0,
            axis_lock: false,
            track_velocity: false,
            start: None,
            update: None,
            end: None,
            edit: None,
        }
    }
}

/// The bit of `button` in [`crate::DragApi`]'s set of starting buttons.
pub(super) fn bit(button: MouseButton) -> u8 {
    match button {
        MouseButton::Primary => 1,
        MouseButton::Secondary => 2,
        MouseButton::Middle => 4,
        MouseButton::Back => 8,
        MouseButton::Forward => 16,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EditOwnership {
    Owned,
    Borrowed,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AxisLock {
    Free,
    Horizontal,
    Vertical,
}
impl AxisLock {
    pub(super) fn constrain(self, point: Point) -> Point {
        match self {
            Self::Free => point,
            Self::Horizontal => Point::new(point.x, 0.0),
            Self::Vertical => Point::new(0.0, point.y),
        }
    }
}
#[derive(Clone)]
pub(super) enum Velocity {
    Disabled,
    Tracking(Rc<DragVelocity>),
}
impl Velocity {
    pub(super) fn at(&self, now: u64) -> Point {
        match self {
            Self::Disabled => Point::new(0.0, 0.0),
            Self::Tracking(samples) => samples.pixels_per_ms(now),
        }
    }
}
