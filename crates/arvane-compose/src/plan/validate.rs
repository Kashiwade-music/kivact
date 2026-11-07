//! Role: Checks that a composition is a tree the plan can draw.

use kivact_types::geometry3d::Matrix4;
use kivact_types::render::{Composition, LayerEffects, LayerItem};

use super::PlanError;

/// Check the composition before planning: a finite scale, an identity root
/// transform and no effects on the root, every child referenced once from
/// its parent (never the root), and every export naming a layer.
pub(super) fn check(comp: &Composition, scale: f64) -> Result<(), PlanError> {
    if !(scale.is_finite() && scale > 0.0) {
        return Err(PlanError::new(
            "the device scale must be finite and positive",
        ));
    }
    let root = comp
        .layers
        .first()
        .ok_or_else(|| PlanError::new("the composition has no root layer"))?;
    if root.transform != Matrix4::IDENTITY {
        return Err(PlanError::new(
            "the root layer must have the identity transform",
        ));
    }
    if root.effects != LayerEffects::default() {
        return Err(PlanError::new("the root layer cannot carry effects"));
    }
    let count = comp.layers.len();
    let mut referenced = vec![false; count];
    referenced[0] = true;
    for layer in &comp.layers {
        if !layer.transform.is_finite() {
            return Err(PlanError::new("a layer transform is not finite"));
        }
        for item in &layer.items {
            if let LayerItem::Layer { index, .. } = item {
                let i = *index as usize;
                if i >= count {
                    return Err(PlanError::new(format!(
                        "child layer {index} is out of range"
                    )));
                }
                if referenced[i] {
                    return Err(PlanError::new(format!(
                        "layer {index} is referenced by more than one parent, or is the root"
                    )));
                }
                referenced[i] = true;
            }
        }
    }
    for export in &comp.exports {
        if export.layer as usize >= count {
            return Err(PlanError::new(format!(
                "export layer {} is out of range",
                export.layer
            )));
        }
        if !(export.scale.is_finite() && export.scale > 0.0) {
            return Err(PlanError::new(
                "an export scale must be finite and positive",
            ));
        }
    }
    Ok(())
}
