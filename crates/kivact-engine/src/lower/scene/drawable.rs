//! Role: Lowering of a canvas drawable command: placed where its host draws it.
//!
//! Invariants:
//! - Only the drawing that places a drawable (its host's first drawing, see
//!   `lower::drawables`) adopts its layers; the others draw nothing.
//! - A drawing in a node that is not the drawable's host draws nothing and is
//!   recorded for one report (§10.2).

use super::state::State;
use crate::lower::context::Context;
use kivact_types::{DrawableId, Point};

impl Context<'_> {
    /// Lower the drawable `id` drawn at `at` by command `index` of the scene
    /// being lowered (see the module invariants).
    pub(super) fn paint_drawable(
        &self,
        id: DrawableId,
        at: Point,
        index: u32,
        state: &mut State<'_>,
    ) {
        let Some(site) = self.drawables else {
            return;
        };
        let mut plan = site.plan.borrow_mut();
        if plan.hosts.get(&id) != Some(&site.host) {
            plan.misplaced.push((site.host, id));
            return;
        }
        if plan.winners.get(&id) != Some(&(site.slot, index)) {
            return;
        }
        let Some(layers) = plan.layers.remove(&id) else {
            return;
        };
        drop(plan);
        let at = Point {
            x: state.origin.x + at.x,
            y: state.origin.y + at.y,
        };
        state.out.drawable(layers, at);
    }
}
