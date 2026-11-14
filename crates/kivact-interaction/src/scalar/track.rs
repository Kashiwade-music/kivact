//! Role: Map the pointer along a slider's track to the value of a thumb.

use crate::axis::Axis;
use crate::drag::{DragApi, DragCoordinates, DragUpdate, use_drag};
use crate::edit::EditSession;
use crate::focus_return::{FocusReturn, FocusReturnTarget, PopupCloseReason, ReasonSet};
use kivact_core::geometry::{Point, Rect};
use kivact_core::state::{Value, Var, memo, var};
use kivact_core::view::{ReadingDirection, use_direction};

use super::track_data::Grab;
pub(crate) use super::track_data::{Geometry, Grip, Target, nearest};
use super::track_machine::{Event, Machine, State};
use kivact_core::state::{MachineHandle, machine};

/// The pointer state the thumbs and the track of one slider share.
#[derive(Clone, Copy)]
pub(crate) struct Pointer {
    phase: MachineHandle<Machine>,
    pub(crate) thumbs: Var<Vec<Target>>,
    attached: Value<bool>,
}

impl Pointer {
    pub(crate) fn attach_track(self, rect: Value<Option<Rect>>, axis: Axis) {
        let rtl = axis.is_horizontal() && use_direction().get() == ReadingDirection::Rtl;
        self.phase.send(Event::Attach(Geometry { rect, axis, rtl }));
    }

    /// Setup only.
    #[track_caller]
    pub(crate) fn new() -> Self {
        let phase = machine(State::Detached, || Machine, |_, effect| match effect {});
        let attached = memo(move || matches!(phase.get(), State::Attached { .. }));
        Pointer {
            phase,
            thumbs: var(Vec::new()),
            attached,
        }
    }

    pub(crate) fn geometry(self) -> Option<Geometry> {
        match self.phase.get() {
            State::Detached => None,
            State::Attached { geometry, .. } => Some(geometry),
        }
    }

    /// Move both thumbs from where they were at the press by the pointer's
    /// travel from `from` to `pos`, keeping their distance: the travel is
    /// quantized by the range's step and stops where an end meets a bound.
    ///
    /// The linear mapping keeps the value distance exactly; any other mapping
    /// keeps the position distance instead, so the ends no longer move by the
    /// same amount of value.
    fn move_body(self, from: f64, ends: [f64; 2], pos: f64) {
        let (Some(geometry), thumbs) = (self.geometry(), self.thumbs.get()) else {
            return;
        };
        let (Some(&low), Some(&high)) = (thumbs.first(), thumbs.get(1)) else {
            return;
        };
        let Some(rect) = geometry.rect.get() else {
            return;
        };
        let usable = geometry.length(rect) - low.length(geometry);
        if usable <= 0.0 {
            return;
        }
        if low.mapping.get().is_linear() {
            self.move_body_linear(low, high, ends, (from, pos), usable, geometry);
        } else {
            self.move_body_mapped(low, high, ends, (from, pos), usable, geometry);
        }
    }

    /// The value-distance body drag of the linear mapping.
    fn move_body_linear(
        self,
        low: Target,
        high: Target,
        ends: [f64; 2],
        positions: (f64, f64),
        usable: f64,
        geometry: Geometry,
    ) {
        let (from, pos) = positions;
        let range = low.range.get();
        let mut delta = (pos - from) / usable * (range.max() - range.min());
        if geometry.rtl || !geometry.axis.is_horizontal() {
            delta = -delta;
        }
        if let Some(step) = range.step() {
            delta = (delta / step).round() * step;
        }
        let [a, b] = ends;
        let delta = delta.clamp(range.min() - a.min(b), range.max() - a.max(b));
        // The end moving away from the other goes first, so neither stops
        // at the other's old value.
        if delta > 0.0 {
            high.set(b + delta);
            low.set(a + delta);
        } else {
            low.set(a + delta);
            high.set(b + delta);
        }
    }

    /// The position-distance body drag of every other mapping.
    fn move_body_mapped(
        self,
        low: Target,
        high: Target,
        ends: [f64; 2],
        positions: (f64, f64),
        usable: f64,
        geometry: Geometry,
    ) {
        let (from, pos) = positions;
        let mut delta = (pos - from) / usable;
        if geometry.rtl || !geometry.axis.is_horizontal() {
            delta = -delta;
        }
        let (pa, pb) = (low.position(ends[0]), low.position(ends[1]));
        let delta = delta.clamp(-pa.min(pb), 1.0 - pa.max(pb));
        if delta > 0.0 {
            high.set_position(pb + delta);
            low.set_position(pa + delta);
        } else {
            low.set_position(pa + delta);
            high.set_position(pb + delta);
        }
    }

    pub(crate) fn write(self, index: usize, pos: f64) {
        let (Some(geometry), Some(thumb)) =
            (self.geometry(), self.thumbs.get().get(index).copied())
        else {
            return;
        };
        if let Some(f) = geometry.fraction_at(pos, thumb.length(geometry)) {
            thumb.set_position(f);
        }
    }

    /// A press on a thumb, the track or the body starts moving a thumb.
    fn start(self, update: &DragUpdate<Grip>) {
        let (Some(geometry), thumbs) = (self.geometry(), self.thumbs.get()) else {
            return;
        };
        let Some(&first) = thumbs.first() else { return };
        let pos = geometry.along(update.pos);
        if update.payload == Grip::Body {
            let value = |index: usize| thumbs.get(index).map_or(0.0, |t| t.binding.get_untracked());
            self.phase.send(Event::Grab(Grab::Body {
                from: pos,
                ends: [value(0), value(1)],
            }));
            return;
        }
        let index = match update.payload {
            Grip::Thumb(index) => index,
            Grip::Track | Grip::Body => {
                nearest(&thumbs, geometry.fraction_at(pos, first.length(geometry)))
            }
        };
        let target = thumbs[index];
        let offset = match (update.payload, target.rect.get()) {
            (Grip::Thumb(_), Some(rect)) => {
                pos - (geometry.along(Point::new(rect.x, rect.y)) + geometry.length(rect) / 2.0)
            }
            _ => 0.0,
        };
        self.phase.send(Event::Grab(Grab::Thumb { index, offset }));
        // The first thumb takes the press's focus (`focus_on_press`); another
        // one moved from the track takes it too, unless it cannot (a thumb
        // made `focusable(false)` under a track that keeps the focus).
        if update.payload == Grip::Track && index > 0 {
            FocusReturn {
                target: FocusReturnTarget::Anchor(target.thumb),
                on: ReasonSet::PROGRAMMATIC,
            }
            .request(PopupCloseReason::Programmatic, None);
        }
        self.write(index, pos - offset);
    }

    fn follow(self, update: &DragUpdate<Grip>) {
        let Some(geometry) = self.geometry() else {
            return;
        };
        match self.phase.get() {
            State::Attached {
                gesture: super::track_machine::TrackGesture::Grabbing(Grab::Thumb { index, offset }),
                ..
            } => {
                self.write(index, geometry.along(update.pos) - offset);
            }
            State::Attached {
                gesture: super::track_machine::TrackGesture::Grabbing(Grab::Body { from, ends }),
                ..
            } => {
                self.move_body(from, ends, geometry.along(update.pos));
            }
            _ => {}
        }
    }
}

/// The drag of one slider's thumbs and track: its payload is where the press
/// landed. Disabled while `disabled` holds a true value or no track is
/// attached. Setup only.
#[track_caller]
pub(crate) fn drag(
    session: EditSession<[f64; 2]>,
    snapshot: Value<[f64; 2]>,
    pointer: Pointer,
    disabled: Value<bool>,
) -> DragApi<Grip> {
    let off = memo(move || !pointer.attached.get() || disabled.get());
    use_drag::<Grip>(DragCoordinates::Window)
        .disabled(off)
        .edit(session, move || snapshot.get_untracked())
        .on_start(move |update: DragUpdate<Grip>| pointer.start(&update))
        .on_update(move |update: DragUpdate<Grip>| pointer.follow(&update))
        .on_end(move |_: DragUpdate<Grip>| pointer.phase.send(Event::Stop))
}
