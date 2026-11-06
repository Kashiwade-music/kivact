//! Role: Apply layout motion steps to shared transforms.

use super::state::LayoutAnimation;
use crate::model::Model;
use kivact_types::{Animatable, NodeId, Rect, Transform};

impl Model {
    pub(crate) fn capture_layout_motion(&self) -> Vec<(NodeId, Rect)> {
        self.tree
            .iter()
            .filter_map(|(id, n)| {
                n.layout()?.layout_transition.as_ref()?;
                let l = self.computed_layout(id)?;
                Some((
                    id,
                    Rect::new(l.location.x, l.location.y, l.size.width, l.size.height),
                ))
            })
            .collect()
    }

    pub(crate) fn retarget_layout_motion(&mut self, previous: Vec<(NodeId, Rect)>) {
        for (id, old) in previous {
            let Some(timing) = self
                .tree
                .get(id)
                .and_then(|n| n.layout())
                .and_then(|l| l.layout_transition.clone())
            else {
                continue;
            };
            let Some(l) = self.computed_layout(id) else {
                continue;
            };
            let new = Rect::new(l.location.x, l.location.y, l.size.width, l.size.height);
            if old == new {
                continue;
            }
            let mut from = Vec::new();
            old.to_components(&mut from);
            let mut to = Vec::new();
            new.to_components(&mut to);
            let mut spring = self
                .layout_motion
                .0
                .get(&id)
                .map(|a| a.spring.clone())
                .unwrap_or_default();
            let timing = if let Some(active) = self.layout_motion.0.get(&id) {
                from = active.current.clone();
                timing.retargeted(&active.from, &active.to, &from, &to)
            } else {
                timing
            };
            if self.frame.reduced_motion() && timing.decorative {
                self.layout_motion.0.remove(&id);
                continue;
            }
            let start = self.frame.request_motion_frame();
            spring.restart(start);
            self.layout_motion.0.insert(
                id,
                LayoutAnimation {
                    current: from.clone(),
                    from,
                    to,
                    timing,
                    start,
                    spring,
                },
            );
            self.changes.restack = true;
        }
    }

    pub(crate) fn step_layout_motion(&mut self, now: u64, reduced: bool) -> bool {
        if self.layout_motion.0.is_empty() {
            return false;
        }
        let moving: Vec<_> = self.layout_motion.0.keys().copied().collect();
        for id in moving {
            self.repaint(id);
            self.a11y.subtree(id);
        }
        let tree = &self.tree;
        self.layout_motion.0.retain(|id, a| {
            tree.contains(*id)
                && !(reduced && a.timing.decorative)
                && !a
                    .timing
                    .step(&a.from, &a.to, a.start, now, &mut a.current, &mut a.spring)
        });
        self.changes.restack = true;
        self.changes.paint = true;
        !self.layout_motion.0.is_empty()
    }

    pub(crate) fn shift_layout_times(&mut self, since: u64, now: u64) {
        for a in self.layout_motion.0.values_mut() {
            a.start = if a.start <= since {
                a.start.saturating_add(now.saturating_sub(since))
            } else {
                now
            };
            a.spring.last_ns = now;
        }
    }

    pub(crate) fn layout_motion_transform(&self, id: NodeId) -> Option<Transform> {
        let a = self.layout_motion.0.get(&id)?;
        let current = Rect::from_components(&a.current);
        let goal = Rect::from_components(&a.to);
        let sx = if goal.width == 0.0 {
            1.0
        } else {
            current.width / goal.width
        };
        let sy = if goal.height == 0.0 {
            1.0
        } else {
            current.height / goal.height
        };
        Some(
            Transform::scale_xy(sx, sy)
                .then(Transform::translate(current.x - goal.x, current.y - goal.y)),
        )
    }
}
