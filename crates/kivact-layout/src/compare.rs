//! Role: Normal recorded-measurement revalidation and independent result reconciliation.

use crate::borrowed_tree::{LayoutTree, LeafMeasurer};
use crate::columns::NodeLayoutColumns;
use crate::pass_state::LayoutState;
use kivact_types::NodeId;

/// Complete neutral geometry, opaque so cache and Taffy representations stay private.
/// Equality includes rounded/unrounded location, size, overflow rectangle,
/// scrollbars, border, padding and margin, and excludes traversal order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    rounded: Option<taffy::Layout>,
    unrounded: taffy::Layout,
}

/// One node-qualified difference between independently derived layout results.
#[derive(Clone, Debug)]
pub struct LayoutDifference {
    /// The logical node whose derived result differs.
    pub id: kivact_types::NodeId,
    /// Geometry, constraint or measurement evidence, without cache handles.
    pub detail: String,
}

fn geometry(mut layout: taffy::Layout) -> taffy::Layout {
    layout.order = 0;
    layout
}

impl NodeLayoutColumns {
    /// Read a complete geometry value without exposing caches or traversal order.
    /// `slot` is zero-based. Outside allocated capacity this returns None.
    /// A value within capacity does not prove that the slot contains a live node;
    /// callers needing identity must resolve it in the paired authoritative tree.
    pub fn geometry(&self, slot: usize) -> Option<Geometry> {
        let values = &self.values;
        let at = slot.checked_add(1)?;
        Some(Geometry {
            rounded: values.final_layout.get(at)?.map(geometry),
            unrounded: geometry(*values.unrounded.get(at)?),
        })
    }
    /// Revalidate every recorded request only when the record is complete.
    /// No result is copied back.
    /// `slot` must be live and allocated in this instance's domain, and `id` must
    /// identify that same node. The measurer must describe its current content.
    pub fn remeasures_same(&self, slot: usize, id: NodeId, measure: &mut dyn LeafMeasurer) -> bool {
        let record = &self.states[slot + 1].measured;
        !record.incomplete
            && record
                .sizes
                .iter()
                .all(|(request, retained)| measure.measure(id, *request) == *retained)
    }
    /// Compare retained requests with current content, including bounded incomplete records.
    /// This diagnostic operation never updates retained measurements or repairs layout.
    /// `tree` must have the same live identity-to-slot mapping and allocated slot
    /// domain as these columns. The measurer may describe an independent source
    /// snapshot with that mapping; this does not support unrelated trees.
    pub fn compare_measurements(
        &self,
        tree: &dyn LayoutTree,
        measure: &mut dyn LeafMeasurer,
    ) -> Vec<LayoutDifference> {
        let mut errors = Vec::new();
        for slot in 0..self.states.len().saturating_sub(1) {
            let Some(id) = tree.id_at(slot) else {
                continue;
            };
            for (request, retained) in &self.states[slot + 1].measured.sizes {
                let rebuilt = measure.measure(id, *request);
                if *retained != rebuilt {
                    errors.push(LayoutDifference {
                        id,
                        detail: format!(
                            "measurement {request:?}: retained {retained:?}, rebuilt {rebuilt:?}"
                        ),
                    });
                }
            }
        }
        errors
    }
    /// Discard incremental shortcuts and rebuild derived styles from raw live inputs.
    /// Existing optional results are retained until visited; this does not synthesize
    /// results for detached nodes or replace any authoritative source state.
    /// Tree, state and columns must be paired within the same slot allocation
    /// domain, including vacant slots; this never migrates or remaps an allocation.
    pub fn rebuild_all(&mut self, tree: &dyn LayoutTree, state: &mut LayoutState) {
        for slot in 0..self.states.len().saturating_sub(1) {
            if let Some(id) = tree.id_at(slot) {
                self.rebuild_style(tree, state, id);
            }
        }
        for cursor in &self.cursors {
            cursor.forget();
        }
        state.relayout.clear();
        for record in &mut self.states {
            record.measured = Default::default();
        }
        for cache in &mut self.values.caches {
            cache.clear();
        }
        self.values.performed.fill(None);
    }
    /// Compare full geometry and available parent-visible outputs for identical inputs.
    /// None/None is the sole geometry skip. This is strict equality, distinct from
    /// the normalized-overflow-reach optimization used by absolute-parent relayout.
    /// Both columns instances and `tree` must have the same capacity, live identity
    /// set and identity-to-slot mapping (for example, snapshots of one allocation).
    /// This compares derived results, not arbitrary unrelated or reordered trees.
    pub fn compare(&self, other: &Self, tree: &dyn LayoutTree) -> Vec<LayoutDifference> {
        let mut errors = Vec::new();
        let before = &self.values;
        let after = &other.values;
        for slot in 0..self.states.len().saturating_sub(1) {
            let Some(id) = tree.id_at(slot) else {
                continue;
            };
            let at = slot + 1;
            if before.final_layout[at].is_none() && after.final_layout[at].is_none() {
                continue;
            }
            if let Some(input) = before.performed[at] {
                let a = before.caches[at].clone().get(&input);
                let b = after.caches[at].clone().get(&input);
                if let (Some(a), Some(b)) = (a, b)
                    && a != b
                {
                    errors.push(LayoutDifference {
                        id,
                        detail: format!(
                            "output/baselines for {input:?}: retained {a:?}, rebuilt {b:?}"
                        ),
                    });
                }
            }
            let a = self.geometry(slot);
            let b = other.geometry(slot);
            if a != b {
                errors.push(LayoutDifference {
                    id,
                    detail: format!(
                        "rounded/unrounded geometry/overflow: retained {a:?}, rebuilt {b:?}"
                    ),
                });
            }
        }
        errors
    }
}
