//! Role: Store running state style transitions and their color channels.

use std::collections::BTreeMap;

use kivact_types::motion::{SpringRun, Transition};
use kivact_types::{NodeId, PaintProps, PaintVec, Rgba};

/// The state motion group: the transitions of the nodes whose state
/// styles are moving (allocated only while one moves).
#[derive(Default)]
pub(crate) struct StateMotion(pub(super) BTreeMap<NodeId, StateAnimation>);

impl StateMotion {
    /// The transition of `id`, while it moves.
    pub(crate) fn get(&self, id: NodeId) -> Option<&StateAnimation> {
        self.0.get(&id)
    }

    /// Whether a transition moves.
    pub(crate) fn is_active(&self) -> bool {
        !self.0.is_empty()
    }

    /// Drop the transition of `id`.
    pub(crate) fn forget(&mut self, id: NodeId) {
        self.0.remove(&id);
    }

    /// The nodes whose transitions move.
    pub(crate) fn moving(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.0.keys().copied()
    }
}

/// The independently interpolated text color channels.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ColorChannels(u8);

impl ColorChannels {
    pub(super) const TEXT: Self = Self(1);
    pub(super) const BASE_TEXT: Self = Self(2);

    pub(super) fn contains(self, channel: Self) -> bool {
        self.0 & channel.0 != 0
    }

    pub(super) fn set(&mut self, channel: Self, enabled: bool) {
        if enabled {
            self.0 |= channel.0;
        } else {
            self.0 &= !channel.0;
        }
    }
}

/// Where a state animation is in its life.
#[derive(Debug)]
pub(super) enum StateAnimPhase {
    /// The shown value was captured before a state change; the next retarget
    /// starts it from `current` or drops it. Never stepped.
    Captured { from: Vec<f64>, to: Vec<f64> },
    /// Moving from `from` to `to` since `start` (host ns).
    Running {
        from: Vec<f64>,
        to: Vec<f64>,
        start: u64,
    },
}

#[derive(Debug)]
pub(crate) struct StateAnimation {
    pub(super) transition: std::sync::Arc<Transition>,
    pub(super) target: PaintProps,
    pub(super) current: Vec<f64>,
    /// A spring's velocity, step time and iteration; it survives a
    /// re-capture, so an interrupted spring goes on with its velocity.
    pub(super) spring: SpringRun,
    pub(super) colors: ColorChannels,
    pub(super) phase: StateAnimPhase,
}

impl StateAnimation {
    /// Whether it moves a state color or an unshadowed base color.
    pub(crate) fn colors_text(&self, inherits_base: impl FnOnce() -> bool) -> bool {
        self.colors.contains(ColorChannels::TEXT)
            || (self.colors.contains(ColorChannels::BASE_TEXT) && inherits_base())
    }
    pub(crate) fn current_paint(&self) -> PaintProps {
        let mut p = self.target.clone();
        PaintVec::from_components(&self.current).apply(&mut p);
        p
    }
    pub(crate) fn current_color(&self) -> Rgba {
        let at = PaintVec::COMPONENTS;
        Rgba {
            r: self.current[at],
            g: self.current[at + 1],
            b: self.current[at + 2],
            a: self.current[at + 3],
        }
    }
}
