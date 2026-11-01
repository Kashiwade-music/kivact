//! Role: Canvas hit regions: their semantics, declaration and transformed geometry.

use super::recorder::Draw;
use crate::effect::{Shape, basic};
use crate::geometry::{Point, Rect};
use crate::path::Path;
use kivact_wire as p;
use kivact_wire::{A11yAction, RegionId, Role};

/// What a node's setup declared about a canvas region (`Node::region`),
/// resolved by node mounting and handed to the painter with
/// [`Draw::declare_regions`] before the drawing callbacks run, so that
/// `region`, `region_shape` and `region_path` can publish the region's
/// accessibility leaf.
#[derive(Clone, Debug, PartialEq)]
pub struct RegionSemantics {
    /// Role of the accessibility leaf.
    pub role: Role,
    /// Accessible name.
    pub name: String,
    /// Accessible description, if any.
    pub description: Option<String>,
    /// Accessible value text, if any.
    pub value: Option<String>,
    /// Actions the leaf accepts.
    pub actions: Vec<A11yAction>,
    /// Typed semantic properties (checked, selected, expanded, ...), if any.
    pub properties: Option<Box<p::SemanticsProperties>>,
    /// The region is disabled.
    pub disabled: bool,
    /// The region is hidden from accessibility.
    pub hidden: bool,
    /// The region can take the focus.
    pub focusable: bool,
    /// Position in the focus order among the node's regions.
    pub focus_order: u32,
    /// Where a press moves the focus, if not onto the region itself.
    pub focus_on_press: Option<p::FocusTarget>,
}

impl Draw {
    /// Supply the node's declared regions before the drawing callbacks run;
    /// a region drawn without a declaration is reported and dropped.
    pub fn declare_regions(&mut self, declarations: Vec<(RegionId, RegionSemantics)>) {
        self.declarations = declarations.into_iter().collect();
    }

    /// Paint the hit geometry of a region declared by the canvas node's setup.
    ///
    /// Semantics and input come from the attached part. Regions follow the
    /// current transform and remain independent of draw clips. An undeclared
    /// identifier or repeated geometry in one drawing layer is reported and
    /// ignored. A declaration without geometry is not a native focus target.
    pub fn region(&mut self, id: RegionId, rect: Rect) {
        self.push_region(id, rect);
    }

    pub(super) fn push_region(&mut self, id: RegionId, rect: Rect) -> bool {
        let Some(semantics) = self.declarations.get(&id) else {
            self.region_errors
                .push(format!("canvas region {} has no setup declaration", id.0));
            return false;
        };
        if !self.drawn_regions.insert(id) {
            self.region_errors.push(format!(
                "canvas region {} has duplicate geometry in one layer",
                id.0
            ));
            return false;
        }
        if ![rect.x, rect.y, rect.width, rect.height]
            .iter()
            .all(|v| v.is_finite())
        {
            self.region_errors
                .push(format!("canvas region {} has invalid geometry", id.0));
            return false;
        }
        // Initial layout may supply a zero-sized canvas; it has no hit geometry.
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return false;
        }
        let semantics = semantics.clone();
        let pts = [
            self.transform.apply(Point::new(rect.x, rect.y)),
            self.transform
                .apply(Point::new(rect.x + rect.width, rect.y + rect.height)),
            self.transform
                .apply(Point::new(rect.x + rect.width, rect.y)),
            self.transform
                .apply(Point::new(rect.x, rect.y + rect.height)),
        ];
        if !pts
            .iter()
            .all(|point| point.x.is_finite() && point.y.is_finite())
        {
            self.region_errors
                .push(format!("canvas region {} has an invalid transform", id.0));
            return false;
        }
        let x0 = pts.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let y0 = pts.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let x1 = pts.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let y1 = pts.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        self.regions.push(p::Region {
            shape: None,
            transform: self.transform.m,
            local_rect: rect,
            id,
            rect: p::Rect {
                x: x0,
                y: y0,
                width: x1 - x0,
                height: y1 - y0,
            },
            properties: semantics.properties,
            description: semantics.description,
            disabled: semantics.disabled,
            hidden: semantics.hidden,
            focusable: semantics.focusable,
            focus_order: semantics.focus_order,
            focus_on_press: semantics.focus_on_press,
            role: semantics.role,
            name: semantics.name,
            value: semantics.value,
            actions: semantics.actions,
        });
        true
    }

    /// A shaped region resolved against `rect`, following the current transform.
    /// Draw clips do not restrict the region's hit area.
    pub fn region_shape(&mut self, id: RegionId, rect: Rect, shape: &Shape) {
        if self.push_region(id, rect)
            && let Some(region) = self.regions.last_mut()
        {
            region.shape = Some(basic(rect, shape));
        }
    }

    /// A path region; `rect` supplies its accessibility bounds.
    /// Draw clips do not restrict the region's hit area.
    pub fn region_path(&mut self, id: RegionId, rect: Rect, path: &Path) {
        if self.push_region(id, rect)
            && let Some(region) = self.regions.last_mut()
        {
            region.shape = Some(path.wire());
        }
    }
}
