//! Role: Map the pointer along a slider's track to the value of a thumb.

use crate::drag::{Drag, DragCoordinates, DragUpdate, use_drag};
use crate::edit::EditSession;
use crate::focus_return::{FocusReturn, FocusReturnTarget, PopupCloseReason, ReasonSet};
use crate::policy::scalar::ScalarRange;
use kivact_core::draw::{Point, Rect};
use kivact_core::primitive::{Part, PartState, part};
use kivact_core::view::{NodeRef, Orientation, ReadingDirection, current_direction};
use kivact_core::{Binding, Signal, Value, Var, memo, signal, var};

/// One thumb as the pointer moves it: its writer, range and element.
#[derive(Clone, Copy)]
pub(crate) struct Target {
    pub(crate) binding: Binding<f64>,
    pub(crate) range: Value<ScalarRange>,
    pub(crate) thumb: NodeRef,
    pub(crate) rect: Value<Option<Rect>>,
}

impl Target {
    /// Normalize by the current range and invoke the writer, as
    /// [`Scalar::set`](super::Scalar::set) does.
    fn set(self, value: f64) {
        if let Some(value) = self.range.get_untracked().normalize(value) {
            self.binding.set(value);
        }
    }

    fn length(self, geometry: Geometry) -> f32 {
        self.rect.get().map_or(0.0, |rect| geometry.length(rect))
    }
}

/// The drag payload of a range's body ([`RangeScalar::body`]): the span
/// between the two thumbs, which moves both.
///
/// [`RangeScalar::body`]: crate::RangeScalar::body
pub(crate) const BODY: usize = usize::MAX;

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
    rect: Value<Option<Rect>>,
    horizontal: bool,
    /// The value grows from the right edge (a horizontal track in a
    /// right-to-left subtree); the thumbs' arrows mirror.
    pub(crate) rtl: bool,
}

impl Geometry {
    fn along(self, p: Point) -> f32 {
        if self.horizontal { p.x } else { p.y }
    }

    fn length(self, r: Rect) -> f32 {
        if self.horizontal { r.width } else { r.height }
    }

    /// Where `pos` (along the axis, in window coordinates) lies on the
    /// track of a thumb `thumb` long, from 0 at the minimum to 1 at the
    /// maximum.
    fn fraction_at(self, pos: f32, thumb: f32) -> Option<f64> {
        let rect = self.rect.get()?;
        let start = if self.horizontal { rect.x } else { rect.y };
        let usable = f64::from(self.length(rect) - thumb);
        let offset = f64::from(pos - start - thumb / 2.0);
        let f = if usable > 0.0 {
            (offset / usable).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Some(if self.rtl || !self.horizontal {
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
    /// The thumb a drag moves and the pointer's distance from its center;
    /// for the body ([`BODY`]), the pointer's position at the press.
    grabbed: Var<(usize, f32)>,
    /// The values of the thumbs when the body was pressed.
    ends: Var<[f64; 2]>,
}

impl Pointer {
    /// Setup only.
    #[track_caller]
    pub(crate) fn new() -> Self {
        Pointer {
            geometry: var(None),
            thumbs: var(Vec::new()),
            attached: signal(false),
            grabbed: var((0, 0.0)),
            ends: var([0.0, 0.0]),
        }
    }

    /// Move both thumbs from where they were at the press by the pointer's
    /// travel from `from` to `pos`, keeping their distance: the travel is
    /// quantized by the range's step and stops where an end meets a bound.
    fn move_body(self, from: f32, pos: f32) {
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
        let usable = f64::from(geometry.length(rect) - low.length(geometry));
        if usable <= 0.0 {
            return;
        }
        let mut delta = f64::from(pos - from) / usable * (range.max() - range.min());
        if geometry.rtl || !geometry.horizontal {
            delta = -delta;
        }
        if let Some(step) = range.step() {
            delta = (delta / step).round() * step;
        }
        let [a, b] = self.ends.get();
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

    fn write(self, index: usize, pos: f32) {
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

    /// A press on the thumb `update.payload` (or on the track: `None`)
    /// starts moving a thumb.
    fn start(self, update: &DragUpdate<Option<usize>>) {
        let (Some(geometry), thumbs) = (self.geometry.get(), self.thumbs.get()) else {
            return;
        };
        let Some(&first) = thumbs.first() else { return };
        let pos = geometry.along(update.pos);
        if update.payload == Some(BODY) {
            let value = |index: usize| thumbs.get(index).map_or(0.0, |t| t.binding.get_untracked());
            self.ends.set([value(0), value(1)]);
            self.grabbed.set((BODY, pos));
            return;
        }
        let index = update
            .payload
            .unwrap_or_else(|| nearest(&thumbs, geometry.fraction_at(pos, first.length(geometry))));
        let target = thumbs[index];
        let offset = match (update.payload, target.rect.get()) {
            (Some(_), Some(rect)) => {
                pos - (geometry.along(Point::new(rect.x, rect.y)) + geometry.length(rect) / 2.0)
            }
            _ => 0.0,
        };
        self.grabbed.set((index, offset));
        // The first thumb takes the press's focus (`focus_on_press`); another
        // one moved from the track takes it too, unless it cannot (a thumb
        // made `focusable(false)` under a track that keeps the focus).
        if update.payload.is_none() && index > 0 {
            FocusReturn {
                target: FocusReturnTarget::Anchor(target.thumb),
                on: ReasonSet::PROGRAMMATIC,
            }
            .request(PopupCloseReason::Programmatic, None);
        }
        self.write(index, pos - offset);
    }

    fn follow(self, update: &DragUpdate<Option<usize>>) {
        let Some(geometry) = self.geometry.get() else {
            return;
        };
        let (index, offset) = self.grabbed.get();
        if index == BODY {
            self.move_body(offset, geometry.along(update.pos));
            return;
        }
        self.write(index, geometry.along(update.pos) - offset);
    }
}

/// The drag of one slider's thumbs and track: its payload is the pressed
/// thumb, `None` on the track. Disabled while `disabled` holds a true value
/// or no track is attached. Setup only.
#[track_caller]
pub(crate) fn drag(
    session: EditSession<[f64; 2]>,
    snapshot: Value<[f64; 2]>,
    pointer: Pointer,
    disabled: Signal<Option<Value<bool>>>,
) -> Drag<Option<usize>> {
    let off = memo(move || !pointer.attached.get() || disabled.get().is_some_and(|d| d.get()));
    use_drag::<Option<usize>>(DragCoordinates::Window)
        .disabled(off)
        .edit(session, move || snapshot.get_untracked())
        .on_start(move |update: DragUpdate<Option<usize>>| pointer.start(&update))
        .on_update(move |update: DragUpdate<Option<usize>>| pointer.follow(&update))
}

/// The track of the thumbs `pointer` and `drag` move; a press focuses
/// `first_thumb`. Setup only.
#[track_caller]
pub(crate) fn track(
    pointer: Pointer,
    drag: Drag<Option<usize>>,
    first_thumb: NodeRef,
    axis: Orientation,
) -> Part<PartState> {
    let horizontal = !matches!(axis, Orientation::Vertical);
    let rtl = horizontal && current_direction() == ReadingDirection::Rtl;
    let surface = part(|input| input).focus_on_press(first_thumb);
    pointer.geometry.set(Some(Geometry {
        rect: surface.node_ref().rect(),
        horizontal,
        rtl,
    }));
    pointer.attached.set(true);
    surface.with(drag.source(None))
}

/// The thumb whose value lies nearest to `pointer` (a fraction of the
/// range); between two thumbs at the same value, the one on the pointer's
/// side.
fn nearest(thumbs: &[Target], pointer: Option<f64>) -> usize {
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
