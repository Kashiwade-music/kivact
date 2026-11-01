//! Role: Tree structure and hierarchy inspection.
//!
//! Structure check of a `Screen` after each settle: the host nodes of the
//! core's retained tree, with its fragments flattened, are the engine's
//! tree (the same nodes, kinds and child order), and the core keeps no
//! record that is not reachable. Properties are not compared: the engine's
//! tree is their only store.

use kivact_wire::NodeId;

use super::model::Screen;
use crate::view::state::ViewState;
use crate::view::tree::Tree;

/// Host children of a core host node, flattening fragments.
fn core_host_children(tree: &Tree, id: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    let Some(n) = tree.get(id) else { return out };
    let mut stack: Vec<NodeId> = n.children.iter().rev().copied().collect();
    while let Some(c) = stack.pop() {
        match tree.get(c) {
            Some(cn) if cn.is_host() => out.push(c),
            Some(cn) => stack.extend(cn.children.iter().rev().copied()),
            None => {}
        }
    }
    out
}

impl Screen {
    /// Structure check: differences between the engine's tree and the
    /// host nodes of the core retained tree (reachable host nodes, kinds,
    /// child order).
    pub(crate) fn structure_mismatch(&self) -> Vec<String> {
        use std::collections::HashSet;
        let mut errs = Vec::new();
        let root = match ViewState::tree(&self.rt).root {
            Some(r) => r,
            None => return errs,
        };
        // every core record is reachable from the root (no detached leftovers)
        let core_hosts = {
            let tree = ViewState::tree(&self.rt);
            let mut reach = 0usize;
            let mut hosts = Vec::new();
            let mut stack = vec![root];
            while let Some(id) = stack.pop() {
                let Some(n) = tree.get(id) else {
                    errs.push(format!("core child {} has no record", id.0));
                    continue;
                };
                reach += 1;
                if n.is_host() {
                    hosts.push(id);
                    if !n.live {
                        errs.push(format!("core host node {} reachable but not live", id.0));
                    }
                }
                stack.extend(n.children.iter().copied());
            }
            if reach != tree.len() {
                errs.push(format!(
                    "core tree holds {} records, {} reachable from the root",
                    tree.len(),
                    reach
                ));
            }
            hosts
        };
        {
            let host = self.host.borrow();
            let pre = host.preorder();
            if pre.len() != host.node_count() {
                errs.push(format!(
                    "the engine holds {} nodes, {} reachable",
                    host.node_count(),
                    pre.len()
                ));
            }
            let a: HashSet<NodeId> = core_hosts.iter().copied().collect();
            let b: HashSet<NodeId> = pre.iter().copied().collect();
            for x in a.difference(&b) {
                errs.push(format!("node {} in core, not on host", x.0));
            }
            for x in b.difference(&a) {
                errs.push(format!("node {} on host, not in core", x.0));
            }
        }
        let host = self.host.borrow();
        let tree = ViewState::tree(&self.rt);
        for id in core_hosts {
            let kids = core_host_children(&tree, id);
            let (Some(cn), Some(kind)) = (tree.get(id), host.kind(id)) else {
                continue;
            };
            if cn.kind != Some(kind) {
                errs.push(format!("node {}: kind {:?} vs {:?}", id.0, cn.kind, kind));
            }
            let children = host.children(id).unwrap_or_default();
            if kids != children {
                errs.push(format!(
                    "node {}: children core {:?} host {:?}",
                    id.0,
                    kids.iter().map(|k| k.0).collect::<Vec<_>>(),
                    children.iter().map(|k| k.0).collect::<Vec<_>>()
                ));
            }
        }
        errs
    }
}
