//! Role: Apply layout motion steps to shared transforms.
//!
//! Layout runs move boxes in window space (`window_layout_rect`). A node's
//! transform cancels the motion of its nearest moving ancestor, so each
//! node lands on its own box.

use super::state::LayoutAnimation;
use crate::contract::NodeType;
use crate::model::Model;
use kivact_types::motion::{PlayDirection, Repeat, Transition};
use kivact_types::{Animatable, CornerRadius, Corners, Len, NodeId, Rect, Transform};

/// The scale from the box `goal` to the box `current`, 1 along an empty side.
pub(crate) fn box_scale(goal: Rect, current: Rect) -> (f64, f64) {
    let ratio = |c: f64, g: f64| if g == 0.0 { 1.0 } else { c / g };
    (
        ratio(current.width, goal.width),
        ratio(current.height, goal.height),
    )
}

/// The window map that carries the box `a` onto the box `b`.
fn box_map(a: Rect, b: Rect) -> Transform {
    let (sx, sy) = box_scale(a, b);
    Transform::translate(-a.x, -a.y)
        .then(Transform::scale_xy(sx, sy))
        .then(Transform::translate(b.x, b.y))
}

/// A layout run plays once, forward: its repeat and direction do not apply.
fn once(timing: Transition) -> Transition {
    Transition {
        repeat: Repeat::Times(1),
        direction: PlayDirection::Normal,
        ..timing
    }
}

/// The window map of `run`'s box: from its goal to its current box.
fn run_map(run: &LayoutAnimation) -> Transform {
    box_map(
        Rect::from_components(&run.to),
        Rect::from_components(&run.current),
    )
}

/// Divide one px radius by a scale (kept when the scale is not positive).
fn unscale(px: f64, scale: f64) -> f64 {
    if scale > 0.0 { px / scale } else { px }
}

/// The corner radii of a box drawn scaled by (`sx`, `sy`) in a layout run:
/// their px parts are divided by the scale, so the corners keep their
/// radius on screen.
pub(crate) fn unscaled_corners(radius: Corners, sx: f64, sy: f64) -> Corners {
    let corner = |r: CornerRadius| CornerRadius {
        x: Len {
            px: unscale(r.x.px, sx),
            ..r.x
        },
        y: Len {
            px: unscale(r.y.px, sy),
            ..r.y
        },
    };
    Corners {
        top_left: corner(radius.top_left),
        top_right: corner(radius.top_right),
        bottom_right: corner(radius.bottom_right),
        bottom_left: corner(radius.bottom_left),
    }
}

impl Model {
    /// The window layout boxes of the nodes with a layout transition, before
    /// a layout pass.
    pub(crate) fn capture_layout_motion(&self) -> Vec<(NodeId, Rect)> {
        self.sources
            .layout_transitions()
            .filter_map(|id| Some((id, self.window_layout_rect(id)?)))
            .collect()
    }

    /// Retarget the runs after a layout pass: each node whose window box
    /// moved since `previous` starts a run from its old box, and so does each
    /// node that takes over a departed shared layout. The mailbox is emptied.
    pub(crate) fn retarget_layout_motion(&mut self, previous: Vec<(NodeId, Rect)>) {
        self.report_duplicate_layout_ids();
        let arrivals = self.take_arrivals(&previous);
        for (id, old) in previous.into_iter().chain(arrivals) {
            self.retarget_run(id, old);
        }
        self.layout_motion.shared.departed.clear();
    }

    /// Start, or retarget, the run of `id` from the window box `old` to its
    /// current window box, unless the box did not move.
    fn retarget_run(&mut self, id: NodeId, old: Rect) {
        let Some(timing) = self
            .tree
            .get(id)
            .and_then(|n| n.layout())
            .and_then(|l| l.layout_transition.clone())
        else {
            return;
        };
        let timing = once(timing);
        let Some(new) = self.window_layout_rect(id) else {
            return;
        };
        if old == new {
            return;
        }
        self.repaint_corrected_below(id);
        let mut from = Vec::new();
        old.to_components(&mut from);
        let mut to = Vec::new();
        new.to_components(&mut to);
        let mut spring = self
            .layout_motion
            .runs
            .get(&id)
            .map(|a| a.spring.clone())
            .unwrap_or_default();
        let timing = if let Some(active) = self.layout_motion.runs.get(&id) {
            from = active.current.clone();
            timing.retargeted(&active.from, &active.to, &from, &to)
        } else {
            timing
        };
        if self.frame.reduced_motion() && timing.decorative {
            self.layout_motion.runs.remove(&id);
            self.stale_fact(id);
            self.mark_corrected_below(id);
            return;
        }
        let start = self.frame.request_motion_frame();
        spring.start(start, &timing, &from, &to);
        self.layout_motion.runs.insert(
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
        self.stale_fact(id);
        // A descendant with a layout transition cancels this new run (it is a
        // layer while the run moves), so its facts are derived again.
        self.mark_corrected_below(id);
    }

    /// Mark the nodes below `id` in its layer that have a layout transition:
    /// their own `layout_motion_transform` cancels the motion of `id`, so they
    /// are layers while it runs and stop being layers when it ends.
    fn mark_corrected_below(&mut self, id: NodeId) {
        let mut stack: Vec<NodeId> = self.tree.flat_children(id).collect();
        while let Some(child) = stack.pop() {
            let Some(n) = self.tree.get(child) else {
                continue;
            };
            if n.is_layer() {
                continue;
            }
            if self.has_layout_transition(child) {
                self.stale_fact(child);
            }
            stack.extend(self.tree.flat_children(child));
        }
    }

    /// Step every run to `now`; a finished run is dropped at its goal.
    pub(crate) fn step_layout_motion(&mut self, now: u64, reduced: bool) -> bool {
        if self.layout_motion.runs.is_empty() {
            return false;
        }
        let moving: Vec<_> = self.layout_motion.runs.keys().copied().collect();
        for id in moving {
            self.repaint(id);
            self.a11y.subtree(id);
            self.repaint_corrected_below(id);
        }
        let before: Vec<NodeId> = self.layout_motion.runs.keys().copied().collect();
        let tree = &self.tree;
        self.layout_motion.runs.retain(|id, a| {
            tree.contains(*id)
                && !(reduced && a.timing.decorative)
                && !a
                    .timing
                    .step(&a.from, &a.to, a.start, now, &mut a.current, &mut a.spring)
        });
        for id in before {
            if !self.layout_motion.runs.contains_key(&id) {
                // A run that ended may stop making its node a layer (and the
                // nodes that cancelled it stop being layers too).
                self.stale_fact(id);
                self.mark_corrected_below(id);
            }
        }
        self.changes.restack = true;
        self.changes.paint = true;
        !self.layout_motion.runs.is_empty()
    }

    /// Move the runs' times past a stop that ended at `now` (the stop began
    /// at `since`): they go on from where they stood.
    pub(crate) fn shift_layout_times(&mut self, since: u64, now: u64) {
        for a in self.layout_motion.runs.values_mut() {
            a.start = if a.start <= since {
                a.start.saturating_add(now.saturating_sub(since))
            } else {
                now
            };
            a.spring.last_ns = now;
        }
    }

    /// The window layout box of `id`: its layout position, moved by each
    /// ancestor's position less its scroll, with no transform applied.
    /// `None` when it has no layout yet.
    pub(crate) fn window_layout_rect(&self, id: NodeId) -> Option<Rect> {
        let size = self.computed_layout(id)?.size;
        let (mut x, mut y) = (0.0, 0.0);
        let mut cur = id;
        loop {
            let node = self.tree.get(cur)?;
            let at = if node.is_layer() && node.kind() == NodeType::Overlay {
                self.overlay_origin(cur)
            } else {
                self.location(node)
            };
            x += at.x;
            y += at.y;
            if node.is_layer() {
                break;
            }
            let Some(parent) = self.tree.flat_parent(node) else {
                break;
            };
            let scroll = self.tree.get(parent)?.scroll;
            x -= scroll.x;
            y -= scroll.y;
            cur = parent;
        }
        Some(Rect::new(x, y, size.width, size.height))
    }

    /// The current over the goal scale of the run of `id` (`None`: no run).
    pub(crate) fn layout_motion_scale(&self, id: NodeId) -> Option<(f64, f64)> {
        let run = self.layout_motion.runs.get(&id)?;
        Some(box_scale(
            Rect::from_components(&run.to),
            Rect::from_components(&run.current),
        ))
    }

    /// The window map of the nearest ancestor of `id` in its layer that has
    /// a run (`None`: none moves above it).
    fn moving_ancestor_map(&self, id: NodeId) -> Option<Transform> {
        let mut cur = self.tree.get(id)?;
        while !cur.is_layer() {
            let parent = self.tree.flat_parent(cur)?;
            if let Some(run) = self.layout_motion.runs.get(&parent) {
                return Some(run_map(run));
            }
            cur = self.tree.get(parent)?;
        }
        None
    }

    /// The layout motion of `id` as a transform of its box (about its
    /// top-left corner): the window map of its run, with the motion of the
    /// nearest moving ancestor cancelled. A node with a layout transition
    /// but no run of its own still cancels that ancestor's motion, so its
    /// window box is neither moved nor scaled by it. `None` when there is
    /// nothing to cancel.
    pub(crate) fn layout_motion_transform(&self, id: NodeId) -> Option<Transform> {
        let (goal, own, outer) = match self.layout_motion.runs.get(&id) {
            Some(run) => (
                Rect::from_components(&run.to),
                run_map(run),
                self.moving_ancestor_map(id).unwrap_or(Transform::IDENTITY),
            ),
            None if self.has_layout_transition(id) => (
                self.window_layout_rect(id)?,
                Transform::IDENTITY,
                self.moving_ancestor_map(id)?,
            ),
            None => return None,
        };
        let own = own.then(outer.invert()?);
        Some(
            Transform::translate(goal.x, goal.y)
                .then(own)
                .then(Transform::translate(-goal.x, -goal.y)),
        )
    }

    /// Redraw the nodes below `id` in its layer that have a layout
    /// transition: their corrections cancel `id`'s motion, so their inks
    /// and layers change with it.
    fn repaint_corrected_below(&mut self, id: NodeId) {
        let mut stack: Vec<NodeId> = self.tree.flat_children(id).collect();
        while let Some(child) = stack.pop() {
            let Some(n) = self.tree.get(child) else {
                continue;
            };
            if n.is_layer() {
                continue;
            }
            if self.has_layout_transition(child) {
                self.repaint(child);
            }
            stack.extend(self.tree.flat_children(child));
        }
    }

    /// Whether `id` has a layout transition (its layout motion may run).
    fn has_layout_transition(&self, id: NodeId) -> bool {
        self.tree
            .get(id)
            .and_then(|n| n.layout())
            .is_some_and(|l| l.layout_transition.is_some())
    }
}
