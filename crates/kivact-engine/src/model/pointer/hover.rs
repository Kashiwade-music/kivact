//! Role: Hover and press queries: hovered nodes, groups and the cursor.

use super::state::CursorKind;
use crate::model::Model;
use kivact_types::{NodeId, PointerId};

impl Model {
    /// Whether `id` is under a primary pointer (in its hover chain).
    pub(crate) fn is_hovered(&self, id: NodeId) -> bool {
        self.pointer
            .state
            .tracks
            .values()
            .any(|t| t.chain().contains(&id))
    }

    /// The nearest ancestor of `id` that is a group.
    pub(crate) fn nearest_group(&self, id: NodeId) -> Option<NodeId> {
        self.tree.chain(Some(id)).into_iter().skip(1).find(|x| {
            self.tree
                .get(*x)
                .and_then(|n| n.states())
                .is_some_and(|s| s.group)
        })
    }

    /// Whether the nearest ancestor of `id` that is a group is hovered.
    pub(crate) fn is_group_hovered(&self, id: NodeId) -> bool {
        self.nearest_group(id).is_some_and(|g| self.is_hovered(g))
    }

    /// The nodes below the groups among `nodes` whose nearest group is one
    /// of them and that have a group style `has` (a nested group itself,
    /// not below it).
    pub(crate) fn group_dependents(
        &self,
        nodes: &[NodeId],
        has: impl Fn(&kivact_types::StateStyles) -> bool,
    ) -> Vec<NodeId> {
        nodes
            .iter()
            .filter(|g| self.is_group(**g))
            .flat_map(|g| self.group_members(*g, &has))
            .collect()
    }

    /// The nodes below `node` that have a group style `has` and no group
    /// between them and `node` (a nested group itself, not below it):
    /// what takes its group states from `node` while it is a group.
    pub(crate) fn group_members(
        &self,
        node: NodeId,
        has: impl Fn(&kivact_types::StateStyles) -> bool,
    ) -> Vec<NodeId> {
        let mut out = Vec::new();
        let Some(n) = self.tree.get(node) else {
            return out;
        };
        let mut stack: Vec<NodeId> = self
            .tree
            .flat_children(self.tree.nodes.id_at(n.index))
            .collect();
        while let Some(x) = stack.pop() {
            let Some(c) = self.tree.get(x) else { continue };
            if c.states().is_some_and(&has) {
                out.push(x);
            }
            if !self.is_group(x) {
                stack.extend(self.tree.flat_children(self.tree.nodes.id_at(c.index)));
            }
        }
        out
    }

    /// Whether `id` is a group (`Style::group`).
    fn is_group(&self, id: NodeId) -> bool {
        self.tree
            .get(id)
            .and_then(|n| n.states())
            .is_some_and(|s| s.group)
    }

    /// Whether `id` shows its pressed state (pressed by a primary pointer
    /// that still hovers it).
    pub(crate) fn is_pressed(&self, id: NodeId) -> bool {
        self.pointer
            .state
            .tracks
            .values()
            .any(|t| t.pressed().contains(&id) && t.chain().contains(&id))
    }

    /// The node under the pointer: the head of the first non-empty hover
    /// chain (the mouse's first).
    pub(crate) fn hovered(&self) -> Option<NodeId> {
        self.pointer
            .state
            .tracks
            .values()
            .find_map(|t| t.chain().first().copied())
    }

    /// The cursor the mouse shows, decided by the first of the mouse's
    /// capture and then its hover chain (nearest the pointer first) that
    /// has one: the cursor its paint names (state styles and transitions
    /// included), else the text cursor for an enabled text input, the
    /// pointing hand for an activatable node; with none, the arrow.
    pub(crate) fn cursor(&self) -> CursorKind {
        let mouse = self.pointer.state.track(PointerId::MOUSE);
        for id in mouse
            .capture()
            .into_iter()
            .chain(mouse.chain().iter().copied())
        {
            if let Some(n) = self.tree.get(id) {
                let specified = self.effective_paint(id).cursor;
                if specified != CursorKind::Auto {
                    return specified;
                }
                if self.editors[n.index.0]
                    .as_deref()
                    .map(|editor| editor.editor.state())
                    .is_some()
                    && !n.disabled()
                {
                    return CursorKind::Text;
                }
                if n.activatable() {
                    return CursorKind::Pointer;
                }
            }
        }
        CursorKind::Default
    }

    pub(super) fn scoped(&self, hit: Option<NodeId>) -> Option<NodeId> {
        hit.filter(|h| self.tree.contains(*h) && self.in_scope(*h))
    }
}
