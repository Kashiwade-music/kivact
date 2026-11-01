//! Role: The state motion group's state: the transitions of moving state styles.

use std::collections::BTreeMap;

use kivact_wire::motion::Transition;
use kivact_wire::{NodeId, PaintProps, PaintVec, Rgba};

/// The state motion group: the transitions of the nodes whose state
/// styles are moving (allocated only while one moves).
#[derive(Default)]
pub(crate) struct StateMotion(pub(super) Option<BTreeMap<NodeId, StateAnimation>>);

impl StateMotion {
    /// The transition of `id`, while it moves.
    pub(crate) fn get(&self, id: NodeId) -> Option<&StateAnimation> {
        self.0.as_ref()?.get(&id)
    }

    /// Whether a transition moves.
    pub(crate) fn is_active(&self) -> bool {
        self.0.is_some()
    }

    /// Drop the transition of `id`.
    pub(crate) fn forget(&mut self, id: NodeId) {
        if let Some(m) = &mut self.0 {
            m.remove(&id);
            if m.is_empty() {
                self.0 = None;
            }
        }
    }

    /// The nodes whose transitions move.
    pub(crate) fn moving(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.0.iter().flat_map(|m| m.keys().copied())
    }
}

#[derive(Debug)]
pub(crate) struct StateAnimation {
    pub(super) transition: std::sync::Arc<Transition>,
    pub(super) target: PaintProps,
    pub(super) from: Vec<f32>,
    pub(super) to: Vec<f32>,
    pub(super) current: Vec<f32>,
    pub(super) velocity: Vec<f32>,
    pub(super) start: u64,
    pub(super) last: u64,
    pub(super) pending: bool,
    pub(super) colors_text: bool,
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
        Rgba::new(
            self.current[at],
            self.current[at + 1],
            self.current[at + 2],
            self.current[at + 3],
        )
    }
}
