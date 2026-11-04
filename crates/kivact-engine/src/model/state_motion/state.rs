//! Role: The state motion group's state: the transitions of moving state styles.

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

/// Where a state animation is in its life.
#[derive(Debug)]
pub(super) enum AnimPhase {
    /// The shown value was captured before a state change; the next retarget
    /// starts it from `current` or drops it. Never stepped.
    Captured,
    /// Moving from `from` to `to` since `start` (host ns).
    Running {
        from: Vec<f32>,
        to: Vec<f32>,
        start: u64,
    },
}

#[derive(Debug)]
pub(crate) struct StateAnimation {
    pub(super) transition: std::sync::Arc<Transition>,
    pub(super) target: PaintProps,
    pub(super) current: Vec<f32>,
    /// A spring's velocity, step time and iteration; it survives a
    /// re-capture, so an interrupted spring goes on with its velocity.
    pub(super) spring: SpringRun,
    pub(super) colors_text: bool,
    /// Interpolates a base color that descendants may inherit.
    pub(super) colors_base_text: bool,
    pub(super) phase: AnimPhase,
}

impl StateAnimation {
    /// Whether it moves a state color or an unshadowed base color.
    pub(crate) fn colors_text(&self, inherits_base: impl FnOnce() -> bool) -> bool {
        self.colors_text || (self.colors_base_text && inherits_base())
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
