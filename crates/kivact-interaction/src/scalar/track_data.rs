//! Role: Passive thumb targets, track geometry, grips and active grabs.
use crate::axis::Axis;
use crate::options_cell::OptionsCell;
use crate::policy::scalar::ScalarRange;
use crate::policy::scalar_mapping::ScalarMapping;
use kivact_core::geometry::{Point, Rect};
use kivact_core::state::{Binding, Value};
use kivact_core::view::NodeRef;

/// One thumb as the pointer moves it: its writer, range, mapping and element.
#[derive(Clone, Copy)]
pub(crate) struct Target {
    pub(crate) binding: Binding<f64>,
    pub(crate) range: Value<ScalarRange>,
    pub(crate) mapping: OptionsCell<ScalarMapping>,
    pub(crate) thumb: NodeRef,
    pub(crate) rect: Value<Option<Rect>>,
    // xtask-allow(state-fields): option: caller constraints on the thumb, absent when it is unbounded
    pub(crate) bounds: Option<(Value<f64>, Value<f64>)>,
}

impl Target {
    /// Normalize by the current range and invoke the writer, as
    /// [`ScalarApi::set`](super::ScalarApi::set) does.
    pub(crate) fn set(self, value: f64) {
        if let Some(value) = self.range.get_untracked().normalize(value) {
            self.binding.set(value);
        }
    }

    /// Where the current value lies along the control, through the mapping.
    pub(crate) fn position(self, value: f64) -> f64 {
        self.mapping
            .get()
            .position(value, self.range.get_untracked())
    }

    /// Write the value at a position along the control, through the mapping.
    pub(crate) fn set_position(self, position: f64) {
        let range = self.range.get_untracked();
        if let Some(value) = self.mapping.get().value(position, range) {
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

    /// The positions of [`limits`](Self::limits), in the same order.
    pub(crate) fn position_limits(self) -> (f64, f64) {
        let (min, max) = self.limits();
        (self.position(min), self.position(max))
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
    /// A range's body ([`RangeScalarApi::body`]): the span between the two
    /// thumbs, which moves both.
    ///
    /// [`RangeScalarApi::body`]: crate::RangeScalarApi::body
    Body,
}

/// What the active drag holds, from its start to its end.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Grab {
    /// Thumb `index`; `offset` is the pointer's distance from its center
    /// along the axis (0 from the track).
    Thumb { index: usize, offset: f64 },
    /// A range's body: the pointer along the axis at the press, and both
    /// thumbs' values then.
    Body { from: f64, ends: [f64; 2] },
}

/// The thumb whose value lies nearest to `pointer` (a fraction of the
/// range); between two thumbs at the same value, the one on the pointer's
/// side.
pub(crate) fn nearest(thumbs: &[Target], pointer: Option<f64>) -> usize {
    let Some(pointer) = pointer else { return 0 };
    let mut best = (0, f64::INFINITY);
    for (index, thumb) in thumbs.iter().enumerate() {
        let at = thumb.position(thumb.binding.get());
        let distance = (pointer - at).abs();
        if distance < best.1 || (distance == best.1 && pointer > at) {
            best = (index, distance);
        }
    }
    best.0
}

/// Where a track is and which way its value grows.
#[derive(Clone, Copy, PartialEq)]
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
