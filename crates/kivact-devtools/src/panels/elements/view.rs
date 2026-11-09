//! Role: The element tree and details of the Elements panel.

use kivact::interaction::TreeNode;
use kivact::prelude::*;
use kivact::ui::split;
use kivact_devtools_protocol as p;

use crate::panels::elements::{details, locator, owners};
use crate::state::{Inspected, Session};
use crate::widgets::source_link::source_link;
use crate::widgets::virtual_tree::virtual_tree;

/// The accessible name of the element tree.
const TREE_LABEL: &str = "Element tree";
/// The accessible name of the owner tree.
const OWNERS_LABEL: &str = "Owners";

/// The element tree, the owner tree, the details and their actions.
pub(crate) fn view(session: Session) -> impl IntoView {
    let sizes = signal(vec![0.38, 0.62]);
    // xtask-allow(state-fields): independent: the owner tree replaces the element tree; the layout overlay is shown for either
    let owners_on = signal(false);
    // xtask-allow(state-fields): independent: the layout overlay is shown for whichever tree is displayed
    let layout = signal(false);
    let layout_toggle = binding(
        move || layout.get(),
        move |on: bool| {
            layout.set(on);
            if let Some(node) = session.inspected.with_untracked(Inspected::selected) {
                session.highlight(Some(node), on);
            }
        },
    );
    let tree = node()
        .style(style().size(pct(100), pct(100)).min_height(240))
        .child(
            show(move || owners_on.get(), move || owners::owners(session))
                .fallback(move || element_tree(session, layout_toggle)),
        );
    let details = details::tabs(session, layout_toggle);
    column((
        actions(session, owners_on),
        split(sizes, (tree, details))
            .grip(true)
            .style(style().size(pct(100), pct(100))),
    ))
    .style(style().size(pct(100), pct(100)))
    .on_key(move |event: KeyEvent| pick_shortcut(session, &event))
}

/// The pick and copy buttons and the owner toggle.
fn actions(session: Session, owners_on: Signal<bool>) -> impl IntoView {
    kivact::ui::toolbar("Element actions")
        .start((
            button("Pick element").on_press(move || start_pick(session)),
            checkbox(OWNERS_LABEL, owners_on),
            button("Copy locator")
                .disabled(move || session.detail.latest().flatten().is_none())
                .on_press(move || copy_locator(session)),
        ))
        .style(style().padding_x(8).padding_y(4))
}

/// The virtualized element tree, or a spinner while the tree loads.
fn element_tree(session: Session, layout: Binding<bool>) -> impl IntoView {
    let nodes = memo(move || session.inspected.with(tree_nodes));
    show(
        move || {
            session
                .inspected
                .with(|inspected| matches!(inspected, Inspected::Loaded { .. }))
        },
        move || {
            let expanded = signal(root_keys(session));
            virtual_tree(
                TREE_LABEL,
                nodes,
                expanded,
                selection(session, layout),
                move |id| element_row(session, *id),
            )
        },
    )
    .fallback(|| kivact::ui::spinner().style(style().padding(8)))
}

/// The selection binding of the element tree: inspect, then highlight.
fn selection(session: Session, layout: Binding<bool>) -> Binding<Option<p::NodeId>> {
    binding(
        move || session.inspected.with(Inspected::selected),
        move |id: Option<p::NodeId>| {
            session.inspected.update(|inspected| inspected.select(id));
            session.highlight(id, layout.get_untracked());
        },
    )
}

/// One element row: role and name, creation location and update count.
fn element_row(session: Session, id: p::NodeId) -> View {
    let node = memo(move || find_node(session, id));
    row((
        text(move || node.with(|node| node.as_ref().map_or_else(String::new, node_label))),
        show_some(
            memo(move || node.with(|node| node.as_ref().and_then(|n| n.source.clone()))),
            move |location| source_link(location.get(), session),
        ),
        show(
            move || node.with(|node| node.as_ref().is_some_and(|node| node.update_count > 0)),
            move || {
                kivact::ui::badge(move || {
                    node.with(|node| {
                        node.as_ref()
                            .map_or(0, |node| node.update_count)
                            .to_string()
                    })
                })
            },
        ),
    ))
    .style(move || {
        let dim = node.with(|node| {
            node.as_ref()
                .is_some_and(|node| node.leaving || node.kind == p::NodeKind::Contents)
        });
        style()
            .gap(6)
            .align_items(Align::Center)
            .font_size(12)
            .opacity(if dim { 0.5 } else { 1.0 })
    })
    .into_view()
}

/// The nodes of the loaded tree as flat tree nodes.
fn tree_nodes(inspected: &Inspected) -> Vec<TreeNode<p::NodeId>> {
    let Inspected::Loaded { tree, .. } = inspected else {
        return Vec::new();
    };
    tree.nodes
        .iter()
        .map(|node| TreeNode::new(node.id, node.parent, node_label(node)))
        .collect()
}

/// The tree row label of an element: its role and name, else its text or kind.
fn node_label(node: &p::ElementInfo) -> String {
    if let Some(role) = node.role {
        return match &node.name {
            Some(name) => format!("{role:?} \"{name}\""),
            None => format!("{role:?}"),
        };
    }
    if let Some(text) = &node.text {
        return format!("\"{text}\"");
    }
    if let Some(name) = &node.name {
        return name.clone();
    }
    format!("{:?}", node.kind)
}

/// The elements of the loaded tree.
fn find_node(session: Session, id: p::NodeId) -> Option<p::ElementInfo> {
    session.inspected.with(|inspected| match inspected {
        Inspected::Loaded { tree, .. } => tree.nodes.iter().find(|node| node.id == id).cloned(),
        Inspected::Loading { .. } => None,
    })
}

/// The root the element tree starts expanded.
fn root_keys(session: Session) -> Vec<p::NodeId> {
    session.inspected.with(|inspected| match inspected {
        Inspected::Loaded { tree, .. } => vec![tree.root],
        Inspected::Loading { .. } => Vec::new(),
    })
}

/// Start picking an element of the inspected window.
fn start_pick(session: Session) {
    session.send::<p::StartPick>(
        p::StartPickParams {
            window: session.window(),
        },
        |_| (),
    );
}

/// Copy the selected node's locator expression.
fn copy_locator(session: Session) {
    let Some(Some(detail)) = session.detail.latest() else {
        return;
    };
    if let Some(text) = locator::locator_of(session, &detail) {
        spawn(kivact::platform::copy_text(text));
    }
}

/// Start pick mode on Ctrl+Shift+C.
fn pick_shortcut(session: Session, event: &KeyEvent) -> Handled {
    let modifiers = event.modifiers();
    if matches!(event.key(), Key::Char('c' | 'C'))
        && modifiers.ctrl
        && modifiers.shift
        && !event.repeat()
    {
        start_pick(session);
        Handled::Yes
    } else {
        Handled::No
    }
}
