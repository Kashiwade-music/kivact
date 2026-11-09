//! Role: Resolve paint and text styles for current interaction states.
//!
//! State styles (`StateStyles`): the paint a host draws a node with, its
//! base paint with the deltas of the states it is in. The core resolves
//! `disabled` and `selected`; the host owns focus, hover and pressed, the
//! group states and enter and exit (`presence`). The engine evaluates them
//! all from its own state (the hover chains, the focus): no pointer
//! `Enter`/`Leave` and no signal is read for them.
//!
//! Invariants:
//! - Deltas stack focus < focus visible < group focus < group hover <
//!   hover < pressed < enter < exit: a later one overrides the fields it
//!   sets. The focus deltas apply while the node itself (not one of its
//!   scene regions) has the focus, focus visible only while the focus
//!   indicator shows.
//! - A node is group hovered while the nearest ancestor that is a group
//!   is hovered, and group focused while the focus is on that group or on
//!   a node below it (CSS `:focus-within`).
//! - A node is hovered while it is in the hover chain (the hit and its
//!   ancestors, as CSS `:hover`, so the hover of a child is its parent's
//!   too), and shows its pressed state only while it is pressed and
//!   hovered.
//! - When a group's hover changes, the nodes below it with a group hover
//!   delta are repainted, those below a nested group excepted (they follow
//!   that group); when the focus enters or leaves groups, so are the nodes
//!   below them with a group focus delta (`Model::group_dependents`),
//!   also when a removal takes the focus (its groups are read before the
//!   removal). A node made a group, or made none, restyles the nodes below
//!   it with a group delta, whose nearest group it changes
//!   ([`Model::set_states`]).
//! - A state change captures the displayed paint first, so a transition
//!   starts from what is shown (`state_motion`); a state delta that
//!   transforms or reorders (`tree::states_place`) asks for a layout pass
//!   to place the node again.
//! - A text color delta colors the text below the node; the nearest
//!   ancestor-or-self with an active one wins.

use kivact_types::{ComputedTextStyle, NodeId, PaintDelta, PaintProps, Rgba, StateStyles};

use crate::model::Model;
use crate::mutation::Mutation;

fn merge(p: &mut PaintProps, d: &PaintDelta) {
    if let Some(b) = &d.background {
        p.background = Some(b.clone());
    }
    if d.border.is_some() {
        p.border = d.border;
    }
    if d.ring.is_some() {
        p.ring = d.ring;
    }
    if let Some(r) = d.radius {
        p.radius = r;
    }
    if let Some(e) = &d.effects {
        p.effects = e.clone();
    }
    if let Some(o) = d.opacity {
        p.opacity = o;
    }
    if let Some(s) = &d.shadows {
        p.shadows = s.clone();
    }
    if let Some(s) = &d.scrollbar {
        let scrollbar = p.scrollbar.get_or_insert_with(Default::default);
        if let Some(width) = s.width {
            scrollbar.width = width;
        }
        if let Some((thumb, track)) = s.colors {
            scrollbar.thumb = thumb;
            scrollbar.track = track;
        }
    }
    if let Some(c) = d.cursor {
        p.cursor = c;
    }
    if let Some(t) = d.transform {
        p.transform = t;
    }
}

impl Model {
    /// The state deltas of `id` that apply now, lowest first.
    pub(crate) fn active_deltas(&self, id: NodeId) -> Vec<&PaintDelta> {
        self.deltas(id, false)
    }

    /// The state deltas of `id` that apply now, lowest first; with `entering`,
    /// its enter paint too.
    fn deltas(&self, id: NodeId, entering: bool) -> Vec<&PaintDelta> {
        let Some(node) = self.tree.get(id) else {
            return Vec::new();
        };
        let Some(s) = node.states() else {
            return Vec::new();
        };
        [
            s.focus.as_deref().filter(|_| {
                self.focused()
                    == Some(kivact_types::FocusTarget {
                        node: id,
                        region: None,
                    })
            }),
            s.focus_visible.as_deref().filter(|_| {
                self.focus.visible()
                    && self.focused()
                        == Some(kivact_types::FocusTarget {
                            node: id,
                            region: None,
                        })
            }),
            s.group_focus
                .as_deref()
                .filter(|_| self.is_group_focused(id)),
            s.group_hover
                .as_deref()
                .filter(|_| self.is_group_hovered(id)),
            s.hover.as_deref().filter(|_| self.is_hovered(id)),
            s.pressed.as_deref().filter(|_| self.is_pressed(id)),
            s.enter.as_deref().filter(|_| entering),
            s.exit.as_deref().filter(|_| self.tree.leaving(id)),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    /// Replace the state styles of `id` when they differ. A node made a
    /// group or made none changes the nearest group of the nodes below it
    /// with a group style: they move from what they show now.
    pub(crate) fn set_states(
        &mut self,
        id: NodeId,
        states: StateStyles,
        mutation: Mutation,
    ) -> bool {
        let regrouped = self
            .tree
            .get(id)
            .is_some_and(|n| n.states().is_some_and(|s| s.group) != states.group);
        let members = if regrouped {
            self.group_members(id, |s| s.group_hover.is_some() || s.group_focus.is_some())
        } else {
            Vec::new()
        };
        for x in &members {
            self.prepare_state_motion(*x);
        }
        let changed = self.set_boxed_prop(
            id,
            (states != StateStyles::default()).then_some(states),
            |p| Some(&mut p.states),
            mutation,
        );
        for x in members {
            self.restyled(x);
        }
        changed
    }

    /// The state styles that apply to `id` changed: it is drawn again, and
    /// placed again when a state moves or reorders it.
    pub(crate) fn restyled(&mut self, id: NodeId) {
        self.repaint(id);
        if self
            .tree
            .get(id)
            .and_then(|n| n.states())
            .is_some_and(crate::tree::states_place)
        {
            self.a11y.subtree(id);
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
            self.changes.moved();
        }
    }

    /// The paint of `id` with its active state deltas applied.
    pub(crate) fn target_paint(&self, id: NodeId) -> PaintProps {
        self.paint_with(id, false)
    }

    /// The paint `id` enters from: its target paint with its enter paint over it.
    pub(crate) fn enter_paint(&self, id: NodeId) -> PaintProps {
        self.paint_with(id, true)
    }

    fn paint_with(&self, id: NodeId, entering: bool) -> PaintProps {
        let mut p = self
            .tree
            .get(id)
            .and_then(|n| n.paint())
            .cloned()
            .unwrap_or_default();
        for d in self.deltas(id, entering) {
            merge(&mut p, d);
        }
        p
    }

    /// The paint `id` is drawn with now: its state motion's current paint, and
    /// its corner radii kept on screen while its layout run scales the box.
    pub(crate) fn effective_paint(&self, id: NodeId) -> PaintProps {
        let mut paint = self
            .motion
            .get(id)
            .map_or_else(|| self.target_paint(id), |a| a.current_paint());
        if let Some((sx, sy)) = self.layout_motion_scale(id) {
            paint.radius = crate::model::layout_motion::unscaled_corners(paint.radius, sx, sy);
        }
        paint
    }

    /// The text style of `id`: the inherited `Text` prop, colored by the
    /// nearest ancestor-or-self with an active text color delta.
    pub(crate) fn text_style(&self, id: NodeId) -> ComputedTextStyle {
        let mut t = ComputedTextStyle::clone(&self.text_props(id));
        t.color = self.text_color(id);
        t
    }

    /// The text color of `id`: the one of the nearest ancestor-or-self with
    /// an active text color delta, else the inherited `Text` prop's.
    pub(crate) fn text_color(&self, id: NodeId) -> Rgba {
        self.text_color_with(id, false)
    }

    /// The text color `id` enters from: its enter paint's text color over its
    /// target one.
    pub(crate) fn enter_text_color(&self, id: NodeId) -> Rgba {
        self.text_color_with(id, true)
    }

    fn text_color_with(&self, id: NodeId, entering: bool) -> Rgba {
        let chain = self.tree.chain(Some(id));
        for (index, &x) in chain.iter().enumerate() {
            if let Some(a) = self.motion.get(x)
                && a.colors_text(|| {
                    chain[..index].iter().all(|&descendant| {
                        self.tree
                            .get(descendant)
                            .and_then(|node| node.text())
                            .is_none_or(|style| style.color.is_none())
                    })
                })
            {
                return a.current_color();
            }
            if let Some(c) = self
                .deltas(x, entering && x == id)
                .into_iter()
                .filter_map(|d| d.text_color)
                .next_back()
            {
                return c;
            }
        }
        self.text_props(id).color
    }
}
