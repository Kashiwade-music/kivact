//! Role: The state motion group's state: the transitions of moving state styles.

use std::collections::BTreeMap;

use kivact_wire::motion::Transition;
use kivact_wire::{NodeId, PaintProps, PaintVec, Rgba};

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
    /// Moving from `from` to `to` since `start` (host ns); last stepped at `last`.
    Running {
        from: Vec<f32>,
        to: Vec<f32>,
        start: u64,
        last: u64,
    },
}

#[derive(Debug)]
pub(crate) struct StateAnimation {
    pub(super) transition: std::sync::Arc<Transition>,
    pub(super) target: PaintProps,
    pub(super) current: Vec<f32>,
    pub(super) velocity: Vec<f32>,
    pub(super) colors_text: bool,
    pub(super) phase: AnimPhase,
}

impl StateAnimation {
    /// Whether it moves an inherited text color.
    pub(crate) fn colors_text(&self) -> bool {
        self.colors_text
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
