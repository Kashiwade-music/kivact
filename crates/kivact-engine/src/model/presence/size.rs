//! Role: Step presence size with the state transition frame clock.

use super::state::SizeMotion;
use crate::model::Model;
use crate::model::motion_run::MotionRun;
use kivact_types::motion::SpringRun;
use kivact_types::{LayoutProps, NodeId, Orientation};

/// The axis `l` animates its size along when it enters or exits: none
/// for an unspecified one, which names no axis to scale.
fn size_axis(l: &LayoutProps, entering: bool) -> Option<Orientation> {
    (if entering { l.enter_size } else { l.exit_size })
        .filter(|axis| *axis != Orientation::Unspecified)
}

impl Model {
    pub(crate) fn size_shown(&self, id: NodeId, entering: bool) -> bool {
        self.tree.get(id).and_then(|n| n.layout()).is_some_and(|l| {
            size_axis(l, entering).is_some()
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
        let Some(axis) = size_axis(l, entering) else {
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
            .map_or(if entering { 0.0 } else { 1.0 }, |m| m.run.current[0]);
        self.layout
            .set_presence_scale(&self.tree, n.index.0, Some((axis, from)));
        let to = if entering { 1.0 } else { 0.0 };
        let start = self.frame.request_motion_frame();
        self.presence.sizes.insert(
            id,
            SizeMotion {
                axis,
                run: MotionRun::start(
                    vec![from],
                    vec![to],
                    transition,
                    SpringRun::default(),
                    start,
                ),
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
            let done = m.run.step(now, reduced);
            self.layout.set_presence_scale(
                &self.tree,
                n.index.0,
                if done && m.run.to[0] == 1.0 {
                    None
                } else {
                    Some((
                        m.axis,
                        if done {
                            m.run.to[0]
                        } else {
                            m.run.current[0].clamp(0.0, 1.0)
                        },
                    ))
                },
            );
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
            self.changes.restack = true;
            self.stale_fact(*id);
            self.repaint(*id);
            !done
        });
        let active = !sizes.is_empty();
        self.presence.sizes = sizes;
        active
    }
}
