//! Role: Build the frame's dirty accessibility payload and submit it to the window.

use super::build::{Built, MetadataUpdate};
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
    /// but hashes, the identity metadata (or a payload build's new clipping) and
    /// the clean dirt commit only on acceptance.
    pub(crate) fn accessibility(
        &mut self,
        window: &mut arvane::Window<'_>,
        engine: &mut arvane::Engine,
    ) {
        let (root, dirt) = match &mut self.a11y.publish {
            A11yPublish::Inactive
            | A11yPublish::Published {
                dirt: Dirt::Clean, ..
            } => return,
            A11yPublish::AwaitingFirstTree => (None, Dirt::Full),
            A11yPublish::Published { root, dirt } => {
                (Some(*root), std::mem::replace(dirt, Dirt::Clean))
            }
        };
        let dirt = if self
            .tree()
            .root()
            .is_some_and(|id| Some(kivact_types::SemanticsId(id.0)) != root)
        {
            Dirt::Full
        } else {
            dirt
        };
        let Built {
            tree: mut candidate,
            metadata,
        } = self.build_semantics_for(engine, &dirt);
        // One fingerprint per built node, used for the comparison and the commit.
        let hashes: Vec<(u64, u64)> = candidate
            .nodes
            .iter()
            .map(|node| (node.id.0, hash::fingerprint(node)))
            .collect();
        let replace = dirt == Dirt::Full || root != Some(candidate.root);
        if !replace {
            let accepted = &self.a11y.hashes;
            let mut at = hashes.iter();
            candidate
                .nodes
                .retain(|_| at.next().is_some_and(|(id, h)| accepted.get(id) != Some(h)));
        }
        #[cfg(feature = "testing")]
        {
            self.a11y.published += candidate.nodes.len() as u64;
        }
        let (new_root, focus) = (candidate.root, candidate.focus);
        let accepted = if replace {
            window.replace_accessibility(&candidate)
        } else {
            window.update_accessibility(&kivact_types::render::SemanticsUpdate {
                root: new_root,
                focus,
                nodes: candidate.nodes,
            })
        };
        if accepted.is_err() {
            // Published: a complete tree next; awaiting the first tree: still.
            self.a11y.invalidate();
            return;
        }
        self.a11y.hashes.extend(hashes);
        match metadata {
            MetadataUpdate::Keep => {}
            // The build used an Rc clone that it dropped: no copy is made.
            MetadataUpdate::Clipping(clipping) => {
                std::rc::Rc::make_mut(&mut self.a11y.metadata).clipping = clipping;
            }
            MetadataUpdate::Replace(metadata) => {
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
        }
        self.a11y.publish = A11yPublish::Published {
            root: new_root,
            dirt: Dirt::Clean,
        };
    }
}
