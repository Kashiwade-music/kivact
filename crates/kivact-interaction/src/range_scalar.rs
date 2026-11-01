//! Role: Two thumbs editing the ends of one range without crossing.

use crate::drag::Drag;
use crate::edit::EditPhase;
use crate::policy::scalar::{ScalarRange, ScalarRangeError};
use crate::scalar::{BODY, Edit, IntoScalarRange, Scalar, ScalarState, Shared, pulled_inside};
use kivact_core::primitive::{Part, PartState, part};
use kivact_core::view::{HandlerResult, IntoValue, Orientation};
use kivact_core::{Binding, Value, binding, effect_on, memo};

/// Which end of a [`RangeScalar`] a thumb edits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RangeThumb {
    /// The lower end; it never passes the upper one.
    Low,
    /// The upper end; it never passes the lower one.
    High,
}

/// Copy handle for a slider with two thumbs, the behavior of react-aria's
/// `useSlider` with two `useSliderThumb`s (Ark UI's range slider).
///
/// Each end is a [`Scalar`] of the same range whose writes stop at the other
/// end, so the thumbs never cross: a write past the other thumb lands on it.
/// The published `value_min` of the high thumb is the low value, and the
/// `value_max` of the low thumb is the high value. The [`track`](Self::track)
/// moves the thumb nearest to the press (between two thumbs on the same
/// value, the one on the press's side), and the [`body`](Self::body), the
/// span between the thumbs, moves both. Both ends share one edit
/// ([`on_edit`](Self::on_edit)): a drag, a key press and an assistive
/// technology action are each one edit, and a canceled drag writes both ends
/// back.
#[derive(Clone, Copy)]
pub struct RangeScalar {
    low: Scalar,
    high: Scalar,
}

/// Use the two bindings as the sole writers of the ends of a range of values.
///
/// `range` bounds and quantizes both, as for [`use_scalar`](crate::use_scalar).
/// The thumbs keep `low <= high` for their own writes; direct writes to the
/// bindings are not validated. When the range changes, one effect moves both
/// ends that lie outside the new range to its nearest bound together, so they
/// stay ordered whatever order the thumbs were made in. Setup only.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::{control, prelude};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{RangeThumb, ScalarRange, use_range_scalar};
/// use kivact::prelude::*;
/// let (from, to) = (signal(20.0), signal(80.0));
/// let price = use_range_scalar(from, to, ScalarRange::new(0.0, 100.0).unwrap());
/// let view = node()
///     .part(price.track(Orientation::Horizontal))
///     .style(style().size(200, 24))
///     .child((
///         node().part(price.thumb(RangeThumb::Low)).name("Minimum price"),
///         node().part(price.thumb(RangeThumb::High)).name("Maximum price"),
///     ));
/// ```
#[track_caller]
pub fn use_range_scalar(
    low: impl Into<Binding<f64>>,
    high: impl Into<Binding<f64>>,
    range: impl IntoScalarRange,
) -> RangeScalar {
    let (low, high) = (low.into(), high.into());
    let range = range.into_range();
    let snapshot = memo(move || [low.get(), high.get()]);
    let shared = Shared::new(snapshot, move |[from, to]| {
        low.set(from);
        high.set(to);
    });
    let (low_value, high_value) = (low.into_value(), high.into_value());
    let end = |index: usize, write: Binding<f64>, bounds: (Value<f64>, Value<f64>)| Scalar {
        binding: write,
        range,
        key_step: None,
        disabled: None,
        edit: Some(Edit::new(shared, index, Some(bounds))),
    };
    let min = memo(move || range.get().min());
    let max = memo(move || range.get().max());
    let ends = RangeScalar {
        low: end(
            0,
            binding(low_value, move |v: f64| low.set(v.min(high.get()))),
            (min, high_value),
        ),
        high: end(
            1,
            binding(high_value, move |v: f64| high.set(v.max(low.get()))),
            (low_value, max),
        ),
    };
    // One effect for both ends: moved one at a time, the first would pass the
    // other, which still lies outside the new range.
    effect_on(range, move |range| {
        let from = pulled_inside(low.get_untracked(), *range);
        let to = pulled_inside(high.get_untracked(), *range);
        if let Some(from) = from {
            low.set(from);
        }
        if let Some(to) = to {
            high.set(to);
        }
    });
    shared
        .pointer
        .thumbs
        .set(vec![ends.low.target(), ends.high.target()]);
    ends
}

impl RangeScalar {
    /// Override the positive keyboard and accessibility increment of both
    /// thumbs, as [`Scalar::key_step`] does.
    pub fn key_step(mut self, step: f64) -> Result<Self, ScalarRangeError> {
        self.low = self.low.key_step(step)?;
        self.high = self.high.key_step(step)?;
        Ok(self)
    }

    /// Ignore input and assistive technology on both thumbs and the track
    /// while true. Setup only.
    #[track_caller]
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        let disabled = disabled.into_value();
        self.low = self.low.disabled(disabled);
        self.high = self.high.disabled(disabled);
        self
    }

    /// Report the start and end of each edit of either end as an
    /// [`EditPhase`], with the rules of [`Scalar::on_edit`]. Setup only.
    pub fn on_edit<R: HandlerResult>(self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.low.on_edit(f);
        self
    }

    /// Run `write` as one edit of the range, as [`Scalar::once`] does; a
    /// write of both ends (moving the whole interval) is one edit too.
    /// Call it from a handler.
    pub fn once(self, write: impl FnOnce() -> bool) -> bool {
        self.low.once(write)
    }

    /// Make each drag of `drag` one edit of the range, as [`Scalar::scrub`]
    /// does: a cancel writes both ends from before the press back. Setup only.
    pub fn scrub<T: Clone + 'static>(self, drag: Drag<T>) -> Drag<T> {
        self.low.scrub(drag)
    }

    /// The scalar operations of one end; its writes stop at the other end.
    pub fn end(self, thumb: RangeThumb) -> Scalar {
        match thumb {
            RangeThumb::Low => self.low,
            RangeThumb::High => self.high,
        }
    }

    /// The bounds and optional quantization, following a reactive range.
    pub fn range(self) -> Value<ScalarRange> {
        self.low.range()
    }

    /// The thumb of one end: [`Scalar::thumb`] with the other end as its
    /// published bound. Setup only.
    #[track_caller]
    pub fn thumb(self, thumb: RangeThumb) -> Part<ScalarState> {
        self.end(thumb).thumb()
    }

    /// The track both thumbs move along, as [`Scalar::track`]; a press moves
    /// the thumb nearest to it, which takes the focus. A component that keeps
    /// one focus for both ends makes the thumbs `focusable(false)` and the
    /// track focusable: the press then leaves the focus on the track. Setup
    /// only.
    #[track_caller]
    pub fn track(self, axis: Orientation) -> Part<PartState> {
        self.low.track(axis)
    }

    /// The span between the thumbs (the filled range), inside the
    /// [`track`](Self::track): a press on it drags both ends together,
    /// keeping their distance. The travel follows the pointer along the
    /// track, snaps to the range's step and stops where an end meets a
    /// bound. The drag is one edit of both ends, and Escape or a canceled
    /// pointer writes both back. A press focuses the low thumb. Setup only.
    #[track_caller]
    pub fn body(self) -> Part<PartState> {
        let edit = self
            .low
            .edit
            .expect("invariant: use_range_scalar gives every end an edit");
        part(|input| input)
            .focus_on_press(edit.thumb)
            .with(edit.shared.drag.source(Some(BODY)))
    }
}
