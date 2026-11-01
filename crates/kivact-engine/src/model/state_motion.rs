//! Role: Transitions of state styles; only transitioning nodes allocate them.
//!
//! A node's paint transition (`PaintProps::transition`, kivact's
//! `TransitionProp::Paint`) moves its paint from what is shown to its
//! target paint, the same way whatever changed the target: a paint or
//! state style write (a signal, a style change), the host's hover,
//! pressed or focus, a group state, an entrance or an exit. It moves the
//! components of the paint (`wire::paint_vec`) and the text color: the
//! canonical transform's parts and origin component by component (so
//! `rotate(0)` to `rotate(360)` turns once); what it does not interpolate
//! switches to the target at once.
//!
//! Invariants:
//! - A state change captures the displayed value first
//!   (`Model::prepare_state_motion`); sync retargets the captures once,
//!   frames step them.
//! - Under reduced motion a decorative transition ends at once; a
//!   transition whose target is what is shown ends without a frame.
//! - A moving transform asks for a layout pass every frame (the node is
//!   placed, hit and reported where it is drawn).
//! - While nothing can be drawn the transitions keep their place in time
//!   (`Model::shift_state_times`): they go on from where they paused.

use crate::model::Model;
use kivact_wire::motion::Kind;
use kivact_wire::{NodeId, PaintProps, PaintVec};
use state::StateAnimation;

pub(crate) mod state;

impl Model {
    /// Capture the displayed value before the host changes a state.
    pub(crate) fn prepare_state_motion(&mut self, id: NodeId) {
        if let Some(a) = self.motion.0.as_mut().and_then(|m| m.get_mut(&id)) {
            a.pending = true;
            return;
        }
        let Some(transition) = self
            .tree
            .get(id)
            .and_then(|n| n.paint())
            .and_then(|p| p.transition.clone())
        else {
            return;
        };
        let colors_text = self
            .tree
            .get(id)
            .and_then(|n| n.states())
            .is_some_and(|s| s.deltas().any(|d| d.text_color.is_some()));
        let paint = self.target_paint(id);
        let c = self.text_color(id);
        let mut current = PaintVec::of(&paint).components().to_vec();
        current.extend_from_slice(&[c.r, c.g, c.b, c.a]);
        let len = current.len();
        let a = StateAnimation {
            transition,
            target: paint,
            from: current.clone(),
            to: current.clone(),
            current,
            velocity: vec![0.0; len],
            start: 0,
            last: 0,
            pending: true,
            colors_text,
        };
        self.motion
            .0
            .get_or_insert_with(Default::default)
            .insert(id, a);
    }

    /// Retarget pending captures after input or a prop commit, not every frame.
    pub(crate) fn retarget_states(&mut self, now: u64, reduced: bool) -> bool {
        let Some(mut animations) = self.motion.0.take() else {
            return false;
        };
        animations.retain(|id, a| {
            if self.tree.get(*id).is_none() {
                return false;
            }
            if !a.pending {
                return true;
            }
            a.pending = false;
            let target = self.target_paint(*id);
            let Some(timing) = target.transition.clone() else {
                return false;
            };
            a.transition = timing;
            a.colors_text |= self
                .tree
                .get(*id)
                .and_then(|n| n.states())
                .is_some_and(|s| s.deltas().any(|d| d.text_color.is_some()));
            let c = self.text_color(*id);
            let mut to = PaintVec::of(&target).components().to_vec();
            to.extend_from_slice(&[c.r, c.g, c.b, c.a]);
            if to == a.current || (reduced && a.transition.decorative) {
                return false;
            }
            a.from = a.current.clone();
            a.to = to;
            a.target = target;
            a.start = now;
            a.last = now;
            if let Some(n) = self.tree.get_mut(*id) {
                n.place.transformed |=
                    !a.current_paint().transform.is_none() || !a.target.transform.is_none();
            }
            true
        });
        let active = !animations.is_empty();
        if active {
            self.motion.0 = Some(animations);
        }
        active
    }

    pub(crate) fn step_states(&mut self, now: u64, reduced: bool) -> bool {
        let Some(mut animations) = self.motion.0.take() else {
            return false;
        };
        animations.retain(|id, a| {
            if self.tree.get(*id).is_none() {
                return false;
            }
            let elapsed = now.saturating_sub(a.start) as f32 / 1e6;
            let done = if reduced && a.transition.decorative {
                true
            } else {
                match a.transition.kind {
                    Kind::Spring(s) => {
                        if elapsed < a.transition.delay_ms {
                            false
                        } else {
                            let dt = now.saturating_sub(a.last) as f32 / 1e9;
                            kivact_wire::motion::spring_step(
                                s,
                                &mut a.current,
                                &mut a.velocity,
                                &a.to,
                                dt,
                            )
                        }
                    }
                    _ => a
                        .transition
                        .sample_timed(&a.from, &a.to, elapsed, &mut a.current),
                }
            };
            a.last = now;
            self.repaint(*id);
            if a.target.transform != PaintProps::default().transform
                || self.tree.get(*id).is_some_and(|n| n.place.transformed)
            {
                self.changes.a11y_subtrees.insert(*id);
                self.changes.needs_layout = true;
                self.changes.moved = true;
            }
            if done && let Some(n) = self.tree.get_mut(*id) {
                n.place.transformed = !a.target.transform.is_none();
            }
            !done
        });
        let active = !animations.is_empty();
        if active {
            self.motion.0 = Some(animations);
        }
        active
    }

    pub(crate) fn shift_state_times(&mut self, paused: u64, now: u64) {
        if let Some(animations) = &mut self.motion.0 {
            for a in animations.values_mut() {
                a.start = if a.start <= paused {
                    a.start.saturating_add(now.saturating_sub(paused))
                } else {
                    now
                };
                a.last = now;
            }
        }
    }
}
