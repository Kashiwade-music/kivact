//! Role: Build the frame's dirty accessibility payload and submit it to the window.

use super::build::Built;
use super::hash;
use super::state::{A11yPublish, Dirt};
use crate::model::Model;

impl Model {
    /// Construct pending dirty payloads and submit them to the shell.
    ///
    /// Initial, reactivated, new-root and rejected publications send a complete
    /// replacement. Other updates send changed fingerprints in parent-before-child
    /// order; a parent's changed children detach removed descendants in the shell.
    /// Focus alone may send an empty delta. IDs can be allocated before acceptance,
    /// but hashes, clipping/dependency metadata and dirt commit only on acceptance.
    pub(crate) fn accessibility(
        &mut self,
        window: &mut arvane::Window<'_>,
        engine: &mut arvane::Engine,
    ) {
        let root = match self.a11y.publish {
            A11yPublish::Inactive => return,
            A11yPublish::AwaitingFirstTree => None,
            A11yPublish::Published { root } => Some(root),
        };
        if self.a11y.dirt == Dirt::Clean && root.is_some() {
            return;
        }
        let mut dirt = std::mem::replace(&mut self.a11y.dirt, Dirt::Clean);
        if root.is_none()
            || self
                .tree()
                .root()
                .is_some_and(|id| Some(kivact_types::SemanticsId(id.0)) != root)
        {
            dirt = Dirt::Full;
        }
        let Built {
            tree: mut candidate,
            metadata,
        } = self.build_semantics_for(engine, &dirt);
        let hashes: Vec<_> = candidate
            .nodes
            .iter()
            .map(|node| (node.id.0, hash::fingerprint(node)))
            .collect();
        let replace = dirt == Dirt::Full || root != Some(candidate.root);
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
        let accepted = if replace {
            window.replace_accessibility(&candidate)
        } else {
            window.update_accessibility(&kivact_types::render::SemanticsUpdate {
                root,
                focus,
                nodes: candidate.nodes,
            })
        };
        if accepted.is_ok() {
            self.a11y.hashes.extend(hashes);
            if let Some(metadata) = metadata {
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
            self.a11y.publish = A11yPublish::Published { root };
        } else {
            self.a11y.invalidate();
        }
    }
}
