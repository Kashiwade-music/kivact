//! Role: `Stagger`: the delay of each item of a list, measured from an origin
//! (design §12.7).

use super::Easing;

/// Which item a stagger counts from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaggerFrom {
    /// The first item.
    Start,
    /// The last item.
    End,
    /// The middle item (the average of the first and last for an even count).
    Center,
    /// The two ends, moving inwards.
    Edges,
    /// The item at this index.
    Index(usize),
}

/// The delay between list items: each item starts a step later than the one
/// before it, counted from `from`, and the step follows `easing`.
#[derive(Clone, Debug, PartialEq)]
pub struct Stagger {
    each_ms: f64,
    from: StaggerFrom,
    easing: Easing,
}

impl Stagger {
    /// A stagger of `each_ms` per step, from the start and linear.
    pub fn new(each_ms: impl Into<f64>) -> Stagger {
        Stagger {
            each_ms: each_ms.into(),
            from: StaggerFrom::Start,
            easing: Easing::Linear,
        }
    }

    /// This stagger counted from `from`.
    pub fn from(self, from: StaggerFrom) -> Stagger {
        Stagger { from, ..self }
    }

    /// This stagger with `easing` applied to the distance.
    pub fn easing(self, easing: Easing) -> Stagger {
        Stagger { easing, ..self }
    }

    /// The delay (ms) of item `index` of `count` items.
    pub fn delay(&self, index: usize, count: usize) -> f64 {
        if count == 0 || index >= count {
            return 0.0;
        }
        let last = count - 1;
        let (distance, max) = match self.from {
            StaggerFrom::Edges => (index.min(last - index) as f64, (last / 2) as f64),
            StaggerFrom::Start => distance_from(index, 0.0, last),
            StaggerFrom::End => distance_from(index, last as f64, last),
            StaggerFrom::Center => distance_from(index, last as f64 / 2.0, last),
            StaggerFrom::Index(origin) => distance_from(index, origin as f64, last),
        };
        if max == 0.0 {
            return 0.0;
        }
        self.easing.apply(distance / max) * max * self.each_ms
    }
}

/// The distance of `index` from `origin` and the largest distance any item
/// of `0..=last` has from it (the normalizer of the stagger).
fn distance_from(index: usize, origin: f64, last: usize) -> (f64, f64) {
    let distance = (index as f64 - origin).abs();
    let max = origin.abs().max((last as f64 - origin).abs());
    (distance, max)
}
