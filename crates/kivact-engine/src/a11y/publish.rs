//! Role: Build the frame's dirty accessibility payload and submit it to the window.

use super::hash;
use crate::model::Model;

impl Model {
    /// Construct pending dirty payloads and submit them to the shell.
    ///
    /// Initial, reactivated, new-root and rejected publications send a complete
    /// replacement. Other updates send changed fingerprints in parent-before-child
    /// order; a parent's changed children detach removed descendants in the shell.
    /// Focus alone may send an empty delta. IDs can be allocated before acceptance,
    /// but hashes, clipping/dependency metadata and dirt commit only on acceptance.
    pub(crate) fn accessibility<W: arvane::Window>(
        &mut self,
        window: &mut W,
        active: bool,
        engine: &mut arvane::Engine,
    ) {
        if !active {
            self.a11y.root = None;
            self.a11y.invalidate();
            return;
        }
        if !self.a11y.pending && self.a11y.root.is_some() {
            return;
        }
        if self.tree().root().map(|id| kivact_types::NodeId(id.0)) != self.a11y.root {
            self.a11y.full = true;
        }
        let Some(mut candidate) = self.build_dirty_semantics(engine) else {
            self.a11y.invalidate();
            return;
        };
        let hashes: Vec<_> = candidate
            .nodes
            .iter()
            .map(|node| (node.id.0, hash::fingerprint(node)))
            .collect();
        let replace = self.a11y.full || self.a11y.root != Some(candidate.root);
        if !replace {
            candidate
                .nodes
                .retain(|node| self.a11y.hashes.get(&node.id.0) != Some(&hash::fingerprint(node)));
        }
        #[cfg(feature = "testing")]
        {
            self.a11y.published += candidate.nodes.len() as u64;
        }
        let (root, focus) = (candidate.root, candidate.focus);
        let accepted =
            if !replace && candidate.nodes.is_empty() && self.a11y.focus == candidate.focus {
                Ok(())
            } else if replace {
                window.replace_accessibility(&candidate)
            } else {
                window.update_accessibility(&kivact_types::protocol::SemanticsUpdate {
                    root,
                    focus,
                    nodes: candidate.nodes,
                })
            };
        if accepted.is_ok() {
            self.a11y.hashes.extend(hashes);
            if let Some(metadata) = self.a11y.prepared.take() {
                self.a11y.hashes.retain(|id, _| metadata.live.contains(id));
                self.a11y
                    .region_ids
                    .retain(|key, _| metadata.regions.contains(key));
                self.a11y
                    .native_regions
                    .retain(|_, key| metadata.regions.contains(key));
                self.a11y
                    .authored
                    .retain(|key, _| metadata.authored.contains(key));
                self.a11y.metadata = std::rc::Rc::new(metadata);
            }
            self.a11y.metadata_refresh = false;
            self.a11y.root = Some(root);
            self.a11y.focus = focus;
            self.a11y.pending = false;
            self.a11y.full = false;
            self.a11y.nodes.clear();
            self.a11y.subtrees.clear();
            self.a11y.removed.clear();
        } else {
            self.a11y.invalidate();
        }
    }
}
