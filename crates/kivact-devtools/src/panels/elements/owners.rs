//! Role: The owner tree of the Elements panel.

use kivact::interaction::TreeNode;
use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;
use crate::widgets::source_link::source_link;
use crate::widgets::virtual_tree::virtual_tree;

/// The owner tree of the inspected window, with the first node of an owner selected.
pub(crate) fn owners(session: Session) -> impl IntoView {
    let owners = resource(
        move || session.window(),
        move |window| {
            let source = session.source();
            async move {
                source
                    .call::<p::GetOwners>(p::WindowParams { window })
                    .await
            }
        },
    );
    let selected = signal(None::<u64>);
    let selection = binding(
        move || selected.get(),
        move |id: Option<u64>| {
            selected.set(id);
            let Some(id) = id else {
                return;
            };
            let Some(node) = owners.latest().and_then(|tree| first_node(&tree, id)) else {
                return;
            };
            session
                .inspected
                .update(|inspected| inspected.select(Some(node)));
        },
    );
    node()
        .style(style().size(pct(100), pct(100)))
        .child(dynamic(
            move || owners.latest(),
            move |tree| match tree {
                None => text("Loading owners…")
                    .style(style().padding(8))
                    .into_view(),
                Some(tree) => {
                    let expanded = signal(all_ids(&tree));
                    let nodes = rows(&tree).into_value();
                    virtual_tree("Owners", nodes, expanded, selection, move |id| {
                        owner_row(session, owners, *id)
                    })
                    .into_view()
                }
            },
        ))
}

/// One owner row: its label and where it was created.
fn owner_row(session: Session, owners: Resource<p::OwnerTree>, id: u64) -> View {
    let owner = memo(move || owners.latest().and_then(|tree| find(&tree, id).cloned()));
    row((
        text(move || {
            owner.with(|owner| {
                owner
                    .as_ref()
                    .map(|owner| owner.label.clone())
                    .unwrap_or_default()
            })
        }),
        show_some(
            memo(move || owner.with(|owner| owner.as_ref().and_then(|o| o.location.clone()))),
            move |location| source_link(location.get(), session),
        ),
    ))
    .style(style().gap(6).align_items(Align::Center).font_size(12))
    .into_view()
}

/// Every owner id of the tree, parents before children.
fn all_ids(tree: &p::OwnerTree) -> Vec<u64> {
    let mut ids = Vec::new();
    for root in &tree.roots {
        collect_ids(root, &mut ids);
    }
    ids
}

fn collect_ids(owner: &p::OwnerNode, ids: &mut Vec<u64>) {
    ids.push(owner.id);
    for child in &owner.children {
        collect_ids(child, ids);
    }
}

/// The owners of the tree as flat tree nodes, parents before children.
fn rows(tree: &p::OwnerTree) -> Vec<TreeNode<u64>> {
    let mut rows = Vec::new();
    for root in &tree.roots {
        push_row(root, None, &mut rows);
    }
    rows
}

fn push_row(owner: &p::OwnerNode, parent: Option<u64>, rows: &mut Vec<TreeNode<u64>>) {
    rows.push(TreeNode::new(owner.id, parent, owner.label.clone()));
    for child in &owner.children {
        push_row(child, Some(owner.id), rows);
    }
}

/// The first node an owner created.
fn first_node(tree: &p::OwnerTree, id: u64) -> Option<p::NodeId> {
    find(tree, id).and_then(|owner| owner.nodes.first().copied())
}

/// The owner with `id`, at any depth.
fn find(tree: &p::OwnerTree, id: u64) -> Option<&p::OwnerNode> {
    tree.roots.iter().find_map(|root| find_in(root, id))
}

fn find_in(owner: &p::OwnerNode, id: u64) -> Option<&p::OwnerNode> {
    if owner.id == id {
        return Some(owner);
    }
    owner.children.iter().find_map(|child| find_in(child, id))
}
