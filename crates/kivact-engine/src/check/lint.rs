//! Role: Debug-build layout warnings for texts whose box or content changed.
//!
//! The layout warnings of debug builds: after a layout pass the texts
//! whose boxes (or content) changed are checked again, from the lines'
//! advances and metrics only (no glyph outlines are read), and each defect
//! found is queued as `HostEvent::LayoutWarning` (the node, the rule and
//! how far), under the rules and defaults of `LayoutCheck::default`. It runs
//! from `Model::relayout`, between the layout pass and the sync after it.
//!
//! Invariants:
//! - A text is checked when its box key (its size and content box, and
//!   where it lies in its visual parent, with that parent's size and
//!   content box) differs from the one it was last checked with, or when
//!   its content was measured again (a write, an inherited style). A
//!   scroll or a transform changes no key.
//! - A defect is reported when its text is checked, so once per (node,
//!   rule) until the text's box or content changes; a text that is checked
//!   again and is still wrong is reported again.
//! - A node that is gone or not shown is forgotten.

use crate::check::{text_findings, visual_parent_of};
use crate::model::Model;
use kivact_wire::{HostEvent, LayoutCheck, NodeId, NodeType, Point};
use state::BoxKey;

pub(crate) mod state;

/// The box key of the text `id`.
fn key(host: &Model, id: NodeId, check: &LayoutCheck) -> Option<BoxKey> {
    let tree = host;
    let own = tree.computed_layout(id)?;
    let parent = visual_parent_of(host, id, check).and_then(|v| {
        let l = tree.computed_layout(v)?;
        // Where the text is in the visual parent's border box: the sum of
        // the locations from the text up to it.
        let (mut x, mut y) = (0.0, 0.0);
        let mut cur = id;
        while cur != v {
            let loc = tree.computed_layout(cur)?.location;
            x += loc.x;
            y += loc.y;
            cur = host.tree().parent_id(host.node(cur)?)?;
        }
        Some((v, l.size, l.content_box, Point { x, y }))
    });
    Some(BoxKey {
        size: own.size,
        content: own.content_box,
        parent,
    })
}

impl Model {
    /// Check the texts after a layout pass: all of them when boxes may
    /// have moved (`laid_out`), else those whose content was measured again
    /// (`remeasured`), and queue a `LayoutWarning` for each defect of a
    /// text whose box key or content changed.
    pub(crate) fn lint_layout(
        &mut self,
        engine: &mut arvane::Engine,
        laid_out: bool,
        remeasured: &[NodeId],
    ) {
        let mut lint = std::mem::take(&mut self.lint);
        let host = &*self;
        lint.keys.retain(|id, _| host.tree().contains(*id));
        let is_text = |id: &NodeId| {
            host.node(*id).is_some_and(|n| {
                n.kind() == NodeType::Text
                    || host.editors[n.index.0]
                        .as_deref()
                        .map(|editor| editor.editor.state())
                        .is_some()
            })
        };
        let mut ids: Vec<NodeId> = if laid_out {
            host.tree()
                .iter()
                .filter(|(id, _)| is_text(id))
                .map(|(id, _)| id)
                .collect()
        } else {
            remeasured.iter().copied().filter(is_text).collect()
        };
        ids.sort_unstable();
        ids.dedup();
        let mut warnings = Vec::new();
        for id in ids {
            if !host.tree().shown(id) {
                lint.keys.remove(&id);
                continue;
            }
            let Some(now) = key(host, id, &lint.check) else {
                continue;
            };
            if lint.keys.get(&id) == Some(&now) && !remeasured.contains(&id) {
                continue;
            }
            lint.keys.insert(id, now);
            let mut findings = Vec::new();
            text_findings(host, engine, &lint.check, id, false, &mut findings);
            warnings.extend(findings.into_iter().map(|f| HostEvent::LayoutWarning {
                node: id,
                rule: f.rule,
                amount: f.amount,
            }));
        }
        self.lint = lint;
        for w in warnings {
            self.push(w);
        }
    }
}
