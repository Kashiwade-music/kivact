//! Role: Transition animations with easing and spring.
//!
//! Transitions: tween, spring and keyframes, with delay, repeat and the
//! decorative flag used by reduced motion.

use super::values::Animatable;

pub(crate) use kivact_types::motion::Kind;
use kivact_types::motion::{Easing, PlayDirection, Repeat, Spring};

/// How a value moves to a new target: the engine's transition, built
/// through these methods only.
#[derive(Clone, Debug, PartialEq)]
pub struct Transition(kivact_types::motion::Transition);

impl Transition {
    fn new(kind: Kind) -> Self {
        Transition(kivact_types::motion::Transition {
            kind,
            delay_ms: 0.0,
            repeat: Repeat::Times(1),
            direction: PlayDirection::Normal,
            decorative: true,
        })
    }

    /// A tween over `ms` milliseconds along `easing`.
    pub fn tween(ms: impl Into<f64>, easing: Easing) -> Self {
        Self::new(Kind::Tween {
            ms: (ms.into() as f32).max(0.0),
            easing,
        })
    }

    /// A spring with the given stiffness, damping and mass.
    pub fn spring(s: Spring) -> Self {
        Self::new(Kind::Spring(s))
    }

    /// Play through absolute keyframes `(offset, value)` over `ms`; the
    /// animation ends at the target.
    pub fn keyframes<T: Animatable>(
        frames: impl IntoIterator<Item = (f32, T)>,
        ms: impl Into<f64>,
    ) -> Self {
        let mut v: Vec<(f32, Vec<f32>)> = frames
            .into_iter()
            .map(|(o, t)| {
                let mut c = Vec::with_capacity(T::COMPONENTS);
                t.to_components(&mut c);
                (o.clamp(0.0, 1.0), c)
            })
            .collect();
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self::new(Kind::Keyframes {
            frames: v,
            ms: (ms.into() as f32).max(0.0),
        })
    }

    /// Wait `ms` milliseconds before the first iteration starts.
    pub fn delay(mut self, ms: impl Into<f64>) -> Self {
        self.0.delay_ms = (ms.into() as f32).max(0.0);
        self
    }

    /// How many times it plays (`Repeat::Times(1)` by default;
    /// `Repeat::Forever` until the target changes; `Repeat::Times(0)` ends at
    /// once on the target). A spring's iteration ends when it settles.
    pub fn repeat(mut self, repeat: Repeat) -> Self {
        self.0.repeat = repeat;
        self
    }

    /// Play every other iteration backwards (a spring heads back to where it
    /// started).
    pub fn alternate(mut self) -> Self {
        self.0.direction = PlayDirection::Alternate;
        self
    }

    /// Decorative transitions (default) complete instantly under reduced motion.
    ///
    /// Pass `false` for essential motion, which keeps moving under reduced
    /// motion: motion that carries information, such as a spinner that says
    /// work is going on (as the platforms' own spinners keep turning).
    pub fn decorative(mut self, on: bool) -> Self {
        self.0.decorative = on;
        self
    }
}

impl Transition {
    /// The protocol form the engine plays.
    pub fn as_wire(&self) -> &kivact_types::motion::Transition {
        &self.0
    }
}
