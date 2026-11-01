//! Role: Drop relations the published tree cannot carry.

use crate::model::Model;
use kivact_wire as wire;

impl Model {
    /// Validate relationships after resolving node and region platform identities.
    pub(super) fn validate_relations(
        &self,
        source: wire::FocusTarget,
        props: &mut kivact_types::SemanticsProperties,
    ) {
        // A relation names nodes the platform tree has: one that is
        // gone, not attached or not exposed (no role) is left out. A
        // hidden target stays, as `aria-describedby` allows.
        for ids in [
            &mut props.described_by,
            &mut props.controls,
            &mut props.labelled_by,
        ] {
            ids.retain(|target| {
                let target = self.semantic_owner(*target);
                target.is_some_and(|target| self.attached(target) && self.exposed(target).is_some())
            });
        }
        // The active descendant is shown, has a role and lies inside the
        // node or inside a node it controls (the ARIA combobox pattern: a
        // text field whose list is a sibling). The controls were resolved
        // just above, so a target of a dropped relation does not count.
        props.active_descendant = props.active_descendant.filter(|native| {
            let Some(target) = self.semantic_identity(*native) else {
                return false;
            };
            let Some(node) = self.node(target.node) else {
                return false;
            };
            if !self.tree().shown(target.node) || node.role().is_none() {
                return false;
            }
            if let Some(region) = target.region
                && (node.disabled() || node.region(region).is_none_or(|r| r.hidden || r.disabled))
            {
                return false;
            }
            let inside = |ancestor: wire::FocusTarget| match ancestor.region {
                Some(_) => target == ancestor,
                None => self.tree().is_in_subtree(target.node, ancestor.node),
            };
            inside(source)
                || props
                    .controls
                    .iter()
                    .filter_map(|c| self.semantic_identity(*c))
                    .any(inside)
        });
    }
}
