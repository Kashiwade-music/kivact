//! Role: Application-owned expandable file-tree entries and views.
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::style::{Align, ColorToken, pct, style};
use kivact::view::{IntoValue, Role, for_each};
use kivact::{Binding, binding, memo, signal};
use std::collections::HashMap;

use kivact::interaction::ListSelection;
use kivact::ui::TreeNode;

/// A reusable file-tree entry, independent of the application's file transport.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeEntry {
    pub id: String,
    pub label: String,
    pub children: Vec<TreeEntry>,
    /// Explicit folder identity, including folders with no children.
    pub folder: bool,
    pub marker: String,
    pub initially_expanded: bool,
}
impl TreeEntry {
    /// A selectable leaf.
    pub fn file(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
            folder: false,
            marker: String::new(),
            initially_expanded: false,
        }
    }
    /// A folder that initially opens its children.
    pub fn folder(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<TreeEntry>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children,
            folder: true,
            marker: String::new(),
            initially_expanded: true,
        }
    }
    /// Start a folder with its contents collapsed.
    pub fn collapsed(mut self) -> Self {
        self.initially_expanded = false;
        self
    }
    /// Add a small revision marker to the end of a row.
    pub fn marker(mut self, marker: impl Into<String>) -> Self {
        self.marker = marker.into();
        self
    }
}

/// What the rows draw of an entry.
#[derive(Clone, PartialEq)]
struct Facts {
    label: String,
    marker: String,
    folder: bool,
}

/// Flatten the nested entries into nodes, noting the folders to open first.
fn flatten(
    entries: Vec<TreeEntry>,
    parent: Option<&str>,
    nodes: &mut Vec<TreeNode<String>>,
    facts: &mut HashMap<String, Facts>,
    open: &mut Vec<String>,
) {
    for entry in entries {
        let folder = entry.folder;
        nodes.push(TreeNode::new(
            entry.id.clone(),
            parent.map(str::to_owned),
            entry.label.clone(),
        ));
        if folder && entry.initially_expanded {
            open.push(entry.id.clone());
        }
        facts.insert(
            entry.id.clone(),
            Facts {
                label: entry.label,
                marker: entry.marker,
                folder,
            },
        );
        flatten(entry.children, Some(&entry.id), nodes, facts, open);
    }
}

/// Expandable tree with application-owned selected leaf ID.
///
/// The tree is one tab stop and follows the ARIA tree view (see
/// [`Tree`](kivact::interaction::Tree)): Up and Down move over the visible
/// rows, Right and Left open, close and move between a folder and its
/// children. The chevron, Enter and a double click open and close a folder.
/// The focus can rest on a folder, but only a leaf is written to `selection`.
#[track_caller]
pub fn tree(
    entries: impl IntoValue<Vec<TreeEntry>>,
    selection: impl Into<Binding<String>>,
) -> Node {
    let entries = entries.into_value();
    let selection = selection.into();
    let (mut nodes, mut facts, mut initially_open) = (Vec::new(), HashMap::new(), Vec::new());
    flatten(
        entries.get_untracked(),
        None,
        &mut nodes,
        &mut facts,
        &mut initially_open,
    );
    let expanded = signal(initially_open);
    let source = selection;
    let data = memo(move || {
        let (mut nodes, mut facts, mut open) = (Vec::new(), HashMap::new(), Vec::new());
        flatten(entries.get(), None, &mut nodes, &mut facts, &mut open);
        (nodes, facts)
    });
    let nodes = memo(move || data.with(|data| data.0.clone()));
    let facts = memo(move || data.with(|data| data.1.clone()));
    let selected = source.into_value();
    let keys = binding(
        memo(move || {
            let id = selected.get();
            if id.is_empty() { None } else { Some(id) }
        }),
        {
            move |key: Option<String>| {
                if let Some(id) = key
                    && facts.with(|facts| facts.get(&id).is_some_and(|f| !f.folder))
                {
                    source.set(id);
                }
            }
        },
    );
    let tree = kivact::interaction::use_tree(nodes, expanded, ListSelection::Single(keys));
    let tree = tree.on_activate({
        move |id: String| {
            if facts.with(|facts| facts.get(&id).is_some_and(|f| f.folder)) {
                tree.toggle(id);
            }
        }
    });
    let rows = for_each(
        tree.rows(),
        move |row| {
            let presentation = facts.with(|facts| {
                facts
                    .get(&row.key)
                    .map(|f| (f.label.clone(), f.marker.clone(), f.folder))
            });
            (row.key.clone(), row.branch.has_children(), presentation)
        },
        move |_, item| {
            let entry = item.get_untracked();
            let Some(facts) = facts.with(|facts| facts.get(&entry.key).cloned()) else {
                return node();
            };
            let folder = facts.folder;
            let id = entry.key.clone();
            let chevron = node()
                .child(
                    text(move || {
                        (if folder {
                            if expanded.with(|open| open.contains(&id)) {
                                "⌄"
                            } else {
                                "›"
                            }
                        } else {
                            ""
                        })
                        .to_owned()
                    })
                    .style(style().font_size(11).color(ColorToken::TextMuted)),
                )
                .style(style().width(10));
            let chevron = if folder {
                let id = entry.key.clone();
                // A pointer-only expander: the tree's rows take the arrow keys.
                chevron
                    .on_press(move || tree.toggle(id.clone()))
                    .role(Role::Generic)
            } else {
                chevron
            };
            let body = row((
                chevron,
                kivact::control::icon(if folder { "folder" } else { "file-text" }).style(
                    style().size(14, 14).color(if folder {
                        ColorToken::Warning
                    } else {
                        ColorToken::TextMuted
                    }),
                ),
                text(facts.label.clone()).style(style().grow(1).font_size(12.5)),
                crate::mono(facts.marker.clone()),
            ))
            .style(
                style()
                    .height(26)
                    .padding_left(10 + (entry.level as i32 - 1) * 16)
                    .padding_right(10)
                    .gap(6)
                    .align_items(Align::Center)
                    .hover(style().background(ColorToken::SurfaceAlt))
                    .selected(
                        style()
                            .background(ColorToken::Accent)
                            .color(ColorToken::AccentText),
                    ),
            );
            body.part(tree.item(entry.key))
        },
    );
    column(rows)
        .part(tree.list())
        .style(style().padding_y(4).width(pct(100)))
}
