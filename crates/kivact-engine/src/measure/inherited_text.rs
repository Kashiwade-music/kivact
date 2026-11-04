//! Role: A node's computed text style, inherited up the tree and cached per slot.

use crate::measure::state::MeasureColumns;
use crate::tree::NodeTree;
use kivact_types::{ComputedTextStyle, NodeId, TextStyle};
use std::rc::Rc;

/// The inherited text style of `id`, resolved from its ancestors' `Text`
/// props: the resolver behind [`computed_text`], which the paint and
/// measure paths read instead (the checks compare the two).
pub(crate) fn text_props(tree: &crate::tree::NodeTree, id: NodeId) -> ComputedTextStyle {
    let mut ancestors = Vec::new();
    let mut cur = Some(id);
    while let Some(x) = cur {
        let Some(n) = tree.nodes.get(&x) else { break };
        ancestors.push(n);
        cur = tree.parent_id(n);
    }
    ancestors
        .into_iter()
        .rev()
        .fold(ComputedTextStyle::default(), |parent, node| {
            ComputedTextStyle::inherit(&parent, node.text().unwrap_or(&TextStyle::default()))
        })
}

/// The inherited text style of `id` as kept in `columns`: resolved once
/// after the node or an ancestor changed (`Model::forget_text_below`),
/// then shared by every reader. A node the tree does not have reads the
/// defaults.
pub(crate) fn computed_text(
    tree: &crate::tree::NodeTree,
    columns: &MeasureColumns,
    id: NodeId,
) -> Rc<ComputedTextStyle> {
    let Some(n) = tree.get(id) else {
        return Rc::new(ComputedTextStyle::default());
    };
    if let Some(kept) = columns.computed_text(n.index.0) {
        return kept;
    }
    let resolved = Rc::new(text_props(tree, id));
    columns.set_computed_text(n.index.0, Rc::clone(&resolved));
    resolved
}

/// The inherited input text style, resolved and unwrapped, without display-only
/// decoration, clipping, line limits or case conversion. Carets use typed text.
pub(crate) fn input_style(
    tree: &NodeTree,
    columns: &MeasureColumns,
    id: NodeId,
    engine: &mut arvane::Engine,
) -> kivact_types::ComputedTextStyle {
    let mut style = kivact_lower::assets::text_style(engine, &computed_text(tree, columns, id));
    style.wrap = kivact_types::TextWrap::NoWrap;
    style.decoration = kivact_types::TextDecoration::default();
    style.overflow = kivact_types::TextOverflow::Clip;
    style.max_lines = None;
    style.capsize = false;
    style.transform = kivact_types::TextTransform::None;
    style
}
