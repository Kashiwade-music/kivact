//! Role: The accepted semantics tree, and incremental updates applied to it.
//!
//! Invariants:
//! - `parents` names the one parent of every node but the root, and agrees
//!   with the children lists; every node is reachable from the root.
//! - [`AcceptedTree::apply`] checks everything before changing anything: on an
//!   error the tree is as it was.
//! - A node that loses its parent and gets no other one in the same update
//!   leaves the tree with its subtree.
//! - An update checks the text runs it changes, not the texts' others
//!   (`runs`).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use kivact_types::render::{SemanticsNode, SemanticsTree, SemanticsUpdate};
use kivact_types::{SemanticsId, SemanticsTextRun};

use super::error::AccessibilityError;
use super::runs::Runs;
use super::validation::check_values;

/// A validated semantics hierarchy accepted by a window's platform tree.
/// Read access is borrowed inside a [`super::shared::PlatformTree`] conversion;
/// node references always describe the same accepted revision.
pub struct AcceptedTree {
    root: SemanticsId,
    nodes: HashMap<SemanticsId, SemanticsNode>,
    parents: HashMap<SemanticsId, SemanticsId>,
    runs: Runs,
}

/// What an update changed: the nodes, and the text runs they added.
pub(crate) struct Applied {
    pub(crate) nodes: Vec<SemanticsId>,
    pub(crate) runs: Vec<Arc<SemanticsTextRun>>,
}

impl AcceptedTree {
    /// The state of a validated complete tree.
    pub(crate) fn new(tree: &SemanticsTree) -> Self {
        let mut parents = HashMap::new();
        for node in &tree.nodes {
            for child in &node.children {
                parents.insert(*child, node.id);
            }
        }
        AcceptedTree {
            root: tree.root,
            nodes: tree.nodes.iter().map(|n| (n.id, n.clone())).collect(),
            parents,
            runs: Runs::new(tree.nodes.iter()),
        }
    }

    /// The identity of the one root.
    pub fn root(&self) -> SemanticsId {
        self.root
    }

    /// The accepted node with this identity, if present.
    pub fn node(&self, id: SemanticsId) -> Option<&SemanticsNode> {
        self.nodes.get(&id)
    }

    /// The parent of `id` (`None` for the root and absent nodes).
    pub fn parent(&self, id: SemanticsId) -> Option<SemanticsId> {
        self.parents.get(&id).copied()
    }

    /// The nodes in parent-before-child order.
    pub fn nodes(&self) -> Vec<&SemanticsNode> {
        let mut nodes = Vec::with_capacity(self.nodes.len());
        let mut pending = vec![self.root];
        while let Some(id) = pending.pop() {
            if let Some(node) = self.nodes.get(&id) {
                nodes.push(node);
                pending.extend(node.children.iter().rev().copied());
            }
        }
        nodes
    }

    /// The tree in parent-before-child order.
    pub(crate) fn tree(&self, focus: Option<SemanticsId>) -> SemanticsTree {
        SemanticsTree {
            root: self.root,
            focus,
            nodes: self.nodes().into_iter().cloned().collect(),
        }
    }

    /// Apply and validate the nodes carried by `update` (same root).
    /// Returns those nodes and the text runs they added.
    pub(crate) fn apply(
        &mut self,
        update: &SemanticsUpdate,
    ) -> Result<Applied, AccessibilityError> {
        let mut incoming: HashMap<SemanticsId, &SemanticsNode> = HashMap::new();
        for node in &update.nodes {
            if incoming.insert(node.id, node).is_some() {
                return Err(AccessibilityError::DuplicateNode(node.id));
            }
            check_values(node)?;
        }
        let changed: Vec<&SemanticsNode> = update.nodes.iter().collect();
        let edits = self.reparent(&changed, &incoming)?;
        let parent = |id: SemanticsId| match edits.get(&id) {
            Some(p) => *p,
            None => self.parents.get(&id).copied(),
        };
        // Every changed node hangs from the root, without a cycle.
        let limit = self.nodes.len() + incoming.len() + 1;
        for node in &changed {
            if let Some(active) = node.semantics.active_descendant {
                if !incoming.contains_key(&active) && !self.nodes.contains_key(&active) {
                    return Err(AccessibilityError::MissingNode(active));
                }
                // Inside the node, or inside (or at) a node it controls.
                let controls = &node.semantics.controls[..];
                let mut at = active;
                let mut steps = 0;
                loop {
                    if controls.contains(&at) {
                        break;
                    }
                    at = parent(at).ok_or(AccessibilityError::InvalidProperties(node.id))?;
                    if at == node.id {
                        break;
                    }
                    steps += 1;
                    if steps > limit {
                        return Err(AccessibilityError::InvalidProperties(node.id));
                    }
                }
            }
        }
        self.check_relations(&changed, &incoming, &edits)?;
        for node in &changed {
            let mut at = node.id;
            let mut steps = 0;
            while at != update.root {
                at = parent(at).ok_or(AccessibilityError::UnreachableNode(node.id))?;
                steps += 1;
                if steps > limit {
                    return Err(AccessibilityError::InvalidTree(node.id));
                }
            }
        }
        if let Some(focus) = update.focus {
            let get = |id: SemanticsId| incoming.get(&id).copied().or_else(|| self.nodes.get(&id));
            let node = get(focus).ok_or(AccessibilityError::InvalidFocus(focus))?;
            if node.semantics.disabled || (focus != update.root && !node.semantics.focusable) {
                return Err(AccessibilityError::InvalidFocus(focus));
            }
            // Shown up to the root, through the parents after the update:
            // a focus in a subtree the update removes is invalid.
            let (mut at, mut steps) = (focus, 0);
            loop {
                if get(at).is_none_or(|n| n.semantics.hidden) {
                    return Err(AccessibilityError::InvalidFocus(focus));
                }
                if at == update.root {
                    break;
                }
                at = parent(at).ok_or(AccessibilityError::InvalidFocus(focus))?;
                steps += 1;
                if steps > limit {
                    return Err(AccessibilityError::InvalidFocus(focus));
                }
            }
        }
        let dropped = self.dropped(&incoming, &edits);
        let shown: Vec<&SemanticsNode> = changed
            .iter()
            .copied()
            .filter(|n| !dropped.contains(&n.id))
            .collect();
        let runs = self.runs.check(
            &shown,
            |id| self.nodes.get(&id),
            |id| {
                !dropped.contains(&id)
                    && (incoming.contains_key(&id) || self.nodes.contains_key(&id))
            },
        )?;
        self.runs.commit(&runs);
        // Commit.
        let mut orphans = Vec::new();
        for (child, p) in edits {
            match p {
                Some(p) => {
                    self.parents.insert(child, p);
                }
                None => {
                    self.parents.remove(&child);
                    orphans.push(child);
                }
            }
        }
        let ids: Vec<SemanticsId> = changed.iter().map(|n| n.id).collect();
        for node in changed {
            // A changed node the update also drops was not checked: its
            // accepted runs leave with it.
            if dropped.contains(&node.id)
                && let Some(old) = self.nodes.get(&node.id)
            {
                self.runs.forget(old);
            }
            self.nodes.insert(node.id, node.clone());
        }
        while let Some(id) = orphans.pop() {
            if let Some(node) = self.nodes.remove(&id) {
                self.runs.forget(&node);
                for child in node.children {
                    if self.parents.get(&child) == Some(&id) {
                        self.parents.remove(&child);
                        orphans.push(child);
                    }
                }
            }
        }
        Ok(Applied {
            nodes: ids,
            runs: runs.added.into_iter().map(|(_, run)| run).collect(),
        })
    }

    /// Every node a relation (described by, controls, labelled by) names
    /// is in the tree after the update: the relations of the changed nodes
    /// and, when the update drops nodes, of the others.
    fn check_relations(
        &self,
        changed: &[&SemanticsNode],
        incoming: &HashMap<SemanticsId, &SemanticsNode>,
        edits: &HashMap<SemanticsId, Option<SemanticsId>>,
    ) -> Result<(), AccessibilityError> {
        let dropped = self.dropped(incoming, edits);
        let present = |id: &SemanticsId| {
            !dropped.contains(id) && (incoming.contains_key(id) || self.nodes.contains_key(id))
        };
        let check = |node: &SemanticsNode| -> Result<(), AccessibilityError> {
            let props = &node.semantics;
            let targets = [&props.described_by, &props.controls, &props.labelled_by];
            match targets.into_iter().flatten().find(|id| !present(id)) {
                Some(missing) => Err(AccessibilityError::MissingNode(*missing)),
                None => Ok(()),
            }
        };
        for node in changed {
            check(node)?;
        }
        if !dropped.is_empty() {
            for node in self.nodes.values() {
                if !dropped.contains(&node.id) && !incoming.contains_key(&node.id) {
                    check(node)?;
                }
            }
        }
        Ok(())
    }

    /// The nodes the update removes from the tree: the children their
    /// parents drop and no one takes, with their subtrees.
    fn dropped(
        &self,
        incoming: &HashMap<SemanticsId, &SemanticsNode>,
        edits: &HashMap<SemanticsId, Option<SemanticsId>>,
    ) -> HashSet<SemanticsId> {
        let parent = |id: SemanticsId| match edits.get(&id) {
            Some(p) => *p,
            None => self.parents.get(&id).copied(),
        };
        let mut dropped = HashSet::new();
        let mut pending: Vec<SemanticsId> = edits
            .iter()
            .filter_map(|(child, p)| p.is_none().then_some(*child))
            .collect();
        while let Some(id) = pending.pop() {
            if !dropped.insert(id) {
                continue;
            }
            let node = incoming.get(&id).copied().or_else(|| self.nodes.get(&id));
            for child in node.into_iter().flat_map(|n| n.children.iter()) {
                if parent(*child) == Some(id) {
                    pending.push(*child);
                }
            }
        }
        dropped
    }

    /// The parent edits of `changed`: each child a changed node drops
    /// (`None` unless another changed node takes it) and each child it
    /// takes. A child taken while another node still lists it, listed
    /// twice, or missing is an error.
    fn reparent(
        &self,
        changed: &[&SemanticsNode],
        incoming: &HashMap<SemanticsId, &SemanticsNode>,
    ) -> Result<HashMap<SemanticsId, Option<SemanticsId>>, AccessibilityError> {
        let mut edits: HashMap<SemanticsId, Option<SemanticsId>> = HashMap::new();
        let changed_ids: HashSet<SemanticsId> = changed.iter().map(|n| n.id).collect();
        for node in changed {
            if let Some(old) = self.nodes.get(&node.id) {
                for child in &old.children {
                    if self.parents.get(child) == Some(&node.id) {
                        edits.insert(*child, None);
                    }
                }
            }
        }
        for node in changed {
            for child in &node.children {
                if !incoming.contains_key(child) && !self.nodes.contains_key(child) {
                    return Err(AccessibilityError::MissingNode(*child));
                }
                if *child == node.id {
                    return Err(AccessibilityError::InvalidTree(*child));
                }
                let current = match edits.get(child) {
                    Some(p) => *p,
                    // Kept by an unchanged parent, which still lists it.
                    None => self
                        .parents
                        .get(child)
                        .copied()
                        .filter(|p| !changed_ids.contains(p)),
                };
                if current.is_some() {
                    return Err(AccessibilityError::InvalidTree(*child));
                }
                edits.insert(*child, Some(node.id));
            }
        }
        // The root never has a parent.
        if let Some(Some(_)) = edits.get(&self.root) {
            return Err(AccessibilityError::InvalidTree(self.root));
        }
        edits.remove(&self.root);
        Ok(edits)
    }
}
