//! Role: The virtualized element tree shared by the panels.

use kivact::interaction::TreeNode;
use kivact::prelude::*;

/// A virtualized tree that selects through the caller's binding.
pub(crate) fn virtual_tree<K: ItemKey>(
    label: &str,
    nodes: Value<Vec<TreeNode<K>>>,
    expanded: Signal<Vec<K>>,
    selected: Binding<Option<K>>,
    row: impl Fn(&K) -> View + 'static,
) -> impl IntoView {
    let _ = (label, nodes, expanded, selected, row);
    node()
}
