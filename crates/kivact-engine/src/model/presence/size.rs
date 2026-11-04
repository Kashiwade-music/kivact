//! Role: Step presence size with the state transition frame clock.

use super::state::SizeMotion;
use crate::model::Model;
use kivact_types::NodeId;

impl Model {
    pub(crate) fn size_shown(&self, id: NodeId, entering: bool) -> bool {
        self.tree.get(id).and_then(|n| n.layout()).is_some_and(|l| {
            (if entering { l.enter_size } else { l.exit_size }).is_some()
                && l.size_transition
                    .as_ref()
                    .is_some_and(|t| !(self.frame.reduced_motion() && t.decorative))
        })
    }

    pub(crate) fn start_size(&mut self, id: NodeId, entering: bool) {
        let Some(n) = self.tree.get(id) else {
            return;
        };
        let Some(l) = n.layout() else {
            return;
        };
        let Some(axis) = (if entering { l.enter_size } else { l.exit_size }) else {
            return;
        };
        let Some(transition) = l.size_transition.clone() else {
            return;
        };
        if self.frame.reduced_motion() && transition.decorative {
            return;
        }
        let from = self
            .presence
            .sizes
            .get(&id)
            .map_or(if entering { 0.0 } else { 1.0 }, |m| m.current[0]);
        self.layout
            .set_presence_scale(n.index.0, Some((axis, from)));
        self.presence.sizes.insert(
            id,
            SizeMotion {
                axis,
                transition,
                start: self.frame.request_motion_frame(),
                from,
                to: if entering { 1.0 } else { 0.0 },
                current: vec![from],
                spring: Default::default(),
            },
        );
        self.changes
            .layout
            .insert(crate::model::changes::LayoutReasons::PASS);
    }

    pub(crate) fn step_sizes(&mut self, now: u64, reduced: bool) -> bool {
        let mut sizes = std::mem::take(&mut self.presence.sizes);
        sizes.retain(|id, m| {
            let Some(n) = self.tree.get(*id) else {
                return false;
            };
            let done = (reduced && m.transition.decorative)
                || m.transition.step(
                    &[m.from],
                    &[m.to],
                    m.start,
                    now,
                    &mut m.current,
                    &mut m.spring,
                );
            self.layout.set_presence_scale(
                n.index.0,
                if done && m.to == 1.0 {
                    None
                } else {
                    Some((
                        m.axis,
                        if done {
                            m.to
                        } else {
                            m.current[0].clamp(0.0, 1.0)
                        },
                    ))
                },
            );
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
            self.changes.restack = true;
            self.repaint(*id);
            !done
        });
        let active = !sizes.is_empty();
        self.presence.sizes = sizes;
        active
    }

    pub(crate) fn shift_size_times(&mut self, paused: u64, now: u64) {
        for m in self.presence.sizes.values_mut() {
            m.start = if m.start <= paused {
                m.start.saturating_add(now.saturating_sub(paused))
            } else {
                now
            };
            m.spring.last_ns = now;
        }
    }
}
