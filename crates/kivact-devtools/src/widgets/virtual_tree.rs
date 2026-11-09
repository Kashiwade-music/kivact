//! Role: The virtualized element tree shared by the panels.

use kivact::interaction::{TreeNode, TreeRow, use_tree};
use kivact::prelude::*;

/// The height of one row.
const ROW_HEIGHT: f64 = 22.0;
/// The rows a Page Up or Page Down moves.
const PAGE_ROWS: usize = 20;
/// The indentation of one level.
const INDENT: f64 = 12.0;

/// A virtualized tree that selects through the caller's binding.
pub(crate) fn virtual_tree<K: ItemKey>(
    label: &str,
    nodes: Value<Vec<TreeNode<K>>>,
    expanded: Signal<Vec<K>>,
    selected: Binding<Option<K>>,
    row: impl Fn(&K) -> View + 'static,
) -> impl IntoView {
    // xtask-allow(state-fields): independent: one pending scroll request
    let scroll_to = signal(None::<(K, bool)>);
    let tree = use_tree(nodes, expanded, selected).page_size(PAGE_ROWS);
    let rows = tree.rows();
    let tree = tree.scroll_with(move |key| {
        let branch = rows.with_untracked(|rows| {
            rows.iter()
                .any(|row| row.key == key && row.branch.has_children())
        });
        scroll_to.set(Some((key, branch)));
    });
    let list = virtual_list(
        rows,
        |entry: &TreeRow<K>| (entry.key.clone(), entry.branch.has_children()),
        move |entry: Value<TreeRow<K>>| {
            let key = entry.with_untracked(|entry| entry.key.clone());
            let part = tree.item(key.clone());
            let state = part.state();
            node()
                .part(part)
                .style(move || {
                    let level = state.get().level.saturating_sub(1);
                    style()
                        .height(ROW_HEIGHT)
                        .width(pct(100))
                        .padding_left(6.0 + f64::from(level) * INDENT)
                        .align_items(Align::Center)
                })
                .child(row(&key))
        },
    )
    .row_height(RowHeight::Fixed(ROW_HEIGHT))
    .scroll_to(scroll_to)
    .name(label)
    .style(style().size(pct(100), pct(100)));
    node()
        .part(tree.list())
        .name(label)
        .style(style().size(pct(100), pct(100)))
        .child(list)
}
