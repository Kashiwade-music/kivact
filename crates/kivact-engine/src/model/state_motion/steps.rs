//! Role: Prepare, retarget, step and shift the transitions of state styles.

use super::state::{AnimPhase, StateAnimation};
use crate::model::Model;
use kivact_wire::motion::Kind;
use kivact_wire::{NodeId, PaintProps, PaintVec};

impl Model {
    /// Capture the displayed value before the host changes a state.
    pub(crate) fn prepare_state_motion(&mut self, id: NodeId) {
        self.capture_state_motion(id, false);
    }

    /// Capture the paint `id` enters from: its target paint with its enter paint over it.
    pub(crate) fn prepare_entrance(&mut self, id: NodeId) {
        self.capture_state_motion(id, true);
    }

    fn capture_state_motion(&mut self, id: NodeId, entering: bool) {
        if let Some(a) = self.motion.0.get_mut(&id) {
            a.phase = AnimPhase::Captured;
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
        let paint = if entering {
            self.enter_paint(id)
        } else {
            self.target_paint(id)
        };
        let c = if entering {
            self.enter_text_color(id)
        } else {
            self.text_color(id)
        };
        let mut current = PaintVec::of(&paint).components().to_vec();
        current.extend_from_slice(&[c.r, c.g, c.b, c.a]);
        let len = current.len();
        let a = StateAnimation {
            transition,
            target: paint,
            current,
            velocity: vec![0.0; len],
            colors_text,
            phase: AnimPhase::Captured,
        };
        self.motion.0.insert(id, a);
    }

    /// Retarget pending captures after input or a prop commit, not every frame.
    pub(crate) fn retarget_states(&mut self, now: u64, reduced: bool) -> bool {
        if self.motion.0.is_empty() {
            return false;
        }
        let mut animations = std::mem::take(&mut self.motion.0);
        animations.retain(|id, a| {
            if self.tree.get(*id).is_none() {
                return false;
            }
            if !matches!(a.phase, AnimPhase::Captured) {
                return true;
            }
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
            a.target = target;
            if let Some(n) = self.tree.get_mut(*id) {
                n.place.transformed |=
                    !a.current_paint().transform.is_none() || !a.target.transform.is_none();
            }
            a.phase = AnimPhase::Running {
                from: a.current.clone(),
                to,
                start: now,
                last: now,
            };
            true
        });
        let active = !animations.is_empty();
        self.motion.0 = animations;
        active
    }

    pub(crate) fn step_states(&mut self, now: u64, reduced: bool) -> bool {
        if self.motion.0.is_empty() {
            return false;
        }
        let mut animations = std::mem::take(&mut self.motion.0);
        animations.retain(|id, a| {
            if self.tree.get(*id).is_none() {
                return false;
            }
            let StateAnimation {
                transition,
                target,
                current,
                velocity,
                phase,
                ..
            } = a;
            let AnimPhase::Running {
                from,
                to,
                start,
                last,
            } = phase
            else {
                return true;
            };
            let elapsed = now.saturating_sub(*start) as f32 / 1e6;
            let done = if reduced && transition.decorative {
                true
            } else {
                match transition.kind {
                    Kind::Spring(s) => {
                        if elapsed < transition.delay_ms {
                            false
                        } else {
                            let dt = now.saturating_sub(*last) as f32 / 1e9;
                            kivact_wire::motion::spring_step(s, current, velocity, to, dt)
                        }
                    }
                    _ => transition.sample_timed(from, to, elapsed, current),
                }
            };
            *last = now;
            self.repaint(*id);
            if target.transform != PaintProps::default().transform
                || self.tree.get(*id).is_some_and(|n| n.place.transformed)
            {
                self.changes.a11y_subtrees.insert(*id);
                self.changes.needs_layout = true;
                self.changes.moved();
            }
            if done && let Some(n) = self.tree.get_mut(*id) {
                n.place.transformed = !target.transform.is_none();
            }
            !done
        });
        let active = animations
            .values()
            .any(|a| matches!(a.phase, AnimPhase::Running { .. }));
        self.motion.0 = animations;
        active
    }

    pub(crate) fn shift_state_times(&mut self, paused: u64, now: u64) {
        for a in self.motion.0.values_mut() {
            if let AnimPhase::Running { start, last, .. } = &mut a.phase {
                *start = if *start <= paused {
                    start.saturating_add(now.saturating_sub(paused))
                } else {
                    now
                };
                *last = now;
            }
        }
    }
}
