//! Role: Model's inherited text projection: the computed text style per node.
//!
//! Invariants:
//! - A node's kept text style is what `measure::text_props` resolves for
//!   it now. It is cleared whenever an input of the resolution may have
//!   changed, in the write that changed it (never later, so a read in the
//!   same revision, a state-motion capture included, sees the new
//!   style): a `TextStyle` write clears the node and its whole subtree
//!   (`Model::changed`), an insert or a move clears the moved subtree, and
//!   a removed or reused slot is reset (`MeasureColumns::take_slot`).
//! - Reads fill the entry on demand and hold no borrow of it past the
//!   call, so no user code runs while the column is borrowed.
use crate::model::Model;
use kivact_wire::{ComputedText, NodeId};
use std::rc::Rc;

/// How far a text in the style `text` paints out of its box (its font size
/// and shadows): the culling of the paint list depends on it.
pub(crate) fn text_reach_of(text: &ComputedText) -> f32 {
    kivact_lower::glyphs::text_reach(text.font_size, &text.more().shadows)
}

impl Model {
    /// The inherited text style of `id`, resolved once per change.
    pub(crate) fn text_props(&self, id: NodeId) -> Rc<ComputedText> {
        crate::measure::computed_text(&self.tree, &self.measure, id)
    }

    /// How far the text `id` paints out of its box.
    pub(crate) fn text_reach(&self, id: NodeId) -> f32 {
        text_reach_of(&self.text_props(id))
    }

    /// The input `id`'s inherited text style as it is shaped.
    pub(crate) fn input_style(
        &self,
        id: NodeId,
        engine: &mut arvane::Engine,
    ) -> kivact_types::TextStyle {
        crate::measure::input_style(&self.tree, &self.measure, id, engine)
    }

    /// The kept text styles that differ from a fresh resolution (none when
    /// every invalidation was made). Entries not kept are not filled.
    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) fn text_self_check(&self) -> Vec<String> {
        self.tree
            .nodes
            .iter()
            .filter_map(|(id, node)| {
                let kept = self.measure.computed_text(node.index.0)?;
                let fresh = crate::measure::text_props(&self.tree, *id);
                (*kept != fresh).then(|| {
                    format!(
                        "computed text node {}: kept {kept:?}, resolved {fresh:?}",
                        id.0
                    )
                })
            })
            .collect()
    }

    /// The resolved text style of `id` and its whole subtree is outdated
    /// (the node was written or moved under another parent).
    pub(crate) fn forget_text_below(&mut self, id: NodeId) {
        let mut stack = vec![id];
        while let Some(x) = stack.pop() {
            let Some(n) = self.tree.get(x) else { continue };
            stack.extend(self.tree.children_ids(n));
            self.measure.forget_computed_text(n.index.0);
        }
    }
}
