//! Role: Map the pointer along a slider's track to the value of a thumb.

use crate::axis::Axis;
use crate::drag::{Drag, DragCoordinates, DragUpdate, use_drag};
use crate::edit::EditSession;
use crate::focus_return::{FocusReturn, FocusReturnTarget, PopupCloseReason, ReasonSet};
use crate::policy::scalar::ScalarRange;
use kivact_core::draw::{Point, Rect};
use kivact_core::view::{NodeRef, ReadingDirection, current_direction};
use kivact_core::{Binding, Signal, Value, Var, memo, signal, var};

/// One thumb as the pointer moves it: its writer, range and element.
#[derive(Clone, Copy)]
pub(crate) struct Target {
    pub(crate) binding: Binding<f64>,
    pub(crate) range: Value<ScalarRange>,
    pub(crate) thumb: NodeRef,
    pub(crate) rect: Value<Option<Rect>>,
    pub(crate) bounds: Option<(Value<f64>, Value<f64>)>,
}

impl Target {
    /// Normalize by the current range and invoke the writer, as
    /// [`Scalar::set`](super::Scalar::set) does.
    pub(crate) fn set(self, value: f64) {
        if let Some(value) = self.range.get_untracked().normalize(value) {
            self.binding.set(value);
        }
    }

    pub(crate) fn limits(self) -> (f64, f64) {
        if let Some((min, max)) = self.bounds {
            let min = min.get_untracked();
            (min, max.get_untracked().max(min))
        } else {
            let range = self.range.get_untracked();
            (range.min(), range.max())
        }
    }

    pub(crate) fn length(self, geometry: Geometry) -> f64 {
        self.rect.get().map_or(0.0, |rect| geometry.length(rect))
    }
}

/// Where a press of a slider's drag landed: the drag's payload.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Grip {
    /// The track: the thumb nearest to the press moves.
    Track,
    /// The thumb with this index.
    Thumb(usize),
    /// A range's body ([`RangeScalar::body`]): the span between the two
    /// thumbs, which moves both.
    ///
    /// [`RangeScalar::body`]: crate::RangeScalar::body
    Body,
}

/// What the active drag holds, from its start to its end.
#[derive(Clone, Copy)]
enum Grab {
    /// Thumb `index`; `offset` is the pointer's distance from its center
    /// along the axis (0 from the track).
    Thumb { index: usize, offset: f64 },
    /// A range's body: the pointer along the axis at the press, and both
    /// thumbs' values then.
    Body { from: f64, ends: [f64; 2] },
}

/// Where `value` lies in `range`, clamped to 0..=1.
pub(crate) fn fraction(value: f64, range: ScalarRange) -> f64 {
    let span = range.max() - range.min();
    if !value.is_finite() || span <= 0.0 {
        return 0.0;
    }
    ((value - range.min()) / span).clamp(0.0, 1.0)
}

/// Where a track is and which way its value grows.
#[derive(Clone, Copy)]
pub(crate) struct Geometry {
    pub(crate) rect: Value<Option<Rect>>,
    pub(crate) axis: Axis,
    /// The value grows from the right edge (a horizontal track in a
    /// right-to-left subtree); the thumbs' arrows mirror.
    pub(crate) rtl: bool,
}

impl Geometry {
    pub(crate) fn along(self, p: Point) -> f64 {
        if self.axis.is_horizontal() { p.x } else { p.y }
    }

    pub(crate) fn length(self, r: Rect) -> f64 {
        if self.axis.is_horizontal() {
            r.width
        } else {
            r.height
        }
    }

    /// Where `pos` (along the axis, in window coordinates) lies on the
    /// track of a thumb `thumb` long, from 0 at the minimum to 1 at the
    /// maximum.
    pub(crate) fn fraction_at(self, pos: f64, thumb: f64) -> Option<f64> {
        let rect = self.rect.get()?;
        let start = if self.axis.is_horizontal() {
            rect.x
        } else {
            rect.y
        };
        let usable = self.length(rect) - thumb;
        let offset = pos - start - thumb / 2.0;
        let f = if usable > 0.0 {
            (offset / usable).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Some(if self.rtl || !self.axis.is_horizontal() {
            1.0 - f
        } else {
            f
        })
    }
}

/// The pointer state the thumbs and the track of one slider share.
#[derive(Clone, Copy)]
pub(crate) struct Pointer {
    pub(crate) geometry: Var<Option<Geometry>>,
    pub(crate) thumbs: Var<Vec<Target>>,
    /// A track is attached; without one the thumbs ignore the pointer.
    attached: Signal<bool>,
    /// What the active drag holds; `None` outside a drag.
    grab: Var<Option<Grab>>,
}

impl Pointer {
    pub(crate) fn attach_track(self, rect: Value<Option<Rect>>, axis: Axis) {
        let rtl = axis.is_horizontal() && current_direction() == ReadingDirection::Rtl;
        self.geometry.set(Some(Geometry { rect, axis, rtl }));
        self.attached.set(true);
    }

    /// Setup only.
    #[track_caller]
    pub(crate) fn new() -> Self {
        Pointer {
            geometry: var(None),
            thumbs: var(Vec::new()),
            attached: signal(false),
            grab: var(None),
        }
    }

    /// Move both thumbs from where they were at the press by the pointer's
    /// travel from `from` to `pos`, keeping their distance: the travel is
    /// quantized by the range's step and stops where an end meets a bound.
    fn move_body(self, from: f64, ends: [f64; 2], pos: f64) {
        let (Some(geometry), thumbs) = (self.geometry.get(), self.thumbs.get()) else {
            return;
        };
        let (Some(&low), Some(&high)) = (thumbs.first(), thumbs.get(1)) else {
            return;
        };
        let Some(rect) = geometry.rect.get() else {
            return;
        };
        let range = low.range.get();
        let usable = geometry.length(rect) - low.length(geometry);
        if usable <= 0.0 {
            return;
        }
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

    pub(crate) fn write(self, index: usize, pos: f64) {
        let (Some(geometry), Some(thumb)) =
            (self.geometry.get(), self.thumbs.get().get(index).copied())
        else {
            return;
        };
        let range = thumb.range.get();
        if let Some(f) = geometry.fraction_at(pos, thumb.length(geometry)) {
            thumb.set(range.min() + f * (range.max() - range.min()));
        }
    }

    /// A press on a thumb, the track or the body starts moving a thumb.
    fn start(self, update: &DragUpdate<Grip>) {
        let (Some(geometry), thumbs) = (self.geometry.get(), self.thumbs.get()) else {
            return;
        };
        let Some(&first) = thumbs.first() else { return };
        let pos = geometry.along(update.pos);
        if update.payload == Grip::Body {
            let value = |index: usize| thumbs.get(index).map_or(0.0, |t| t.binding.get_untracked());
            self.grab.set(Some(Grab::Body {
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
        self.grab.set(Some(Grab::Thumb { index, offset }));
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
        let Some(geometry) = self.geometry.get() else {
            return;
        };
        match self.grab.get() {
            Some(Grab::Thumb { index, offset }) => {
                self.write(index, geometry.along(update.pos) - offset);
            }
            Some(Grab::Body { from, ends }) => {
                self.move_body(from, ends, geometry.along(update.pos));
            }
            None => {}
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
    disabled: Signal<Option<Value<bool>>>,
) -> Drag<Grip> {
    let off = memo(move || !pointer.attached.get() || disabled.get().is_some_and(|d| d.get()));
    use_drag::<Grip>(DragCoordinates::Window)
        .disabled(off)
        .edit(session, move || snapshot.get_untracked())
        .on_start(move |update: DragUpdate<Grip>| pointer.start(&update))
        .on_update(move |update: DragUpdate<Grip>| pointer.follow(&update))
        .on_end(move |_: DragUpdate<Grip>| pointer.grab.set(None))
}

/// The thumb whose value lies nearest to `pointer` (a fraction of the
/// range); between two thumbs at the same value, the one on the pointer's
/// side.
pub(crate) fn nearest(thumbs: &[Target], pointer: Option<f64>) -> usize {
    let Some(pointer) = pointer else { return 0 };
    let mut best = (0, f64::INFINITY);
    for (index, thumb) in thumbs.iter().enumerate() {
        let at = fraction(thumb.binding.get(), thumb.range.get());
        let distance = (pointer - at).abs();
        if distance < best.1 || (distance == best.1 && pointer > at) {
            best = (index, distance);
        }
    }
    best.0
}
