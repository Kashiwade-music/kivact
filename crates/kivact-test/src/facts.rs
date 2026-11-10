//! Role: Ephemeral test queries over engine facts and core creation locations.
use kivact_core::launch::Inspect;
use kivact_engine::api::Reader;
use kivact_engine::contract::{NodeKind, NodeProperties, NodeType, SceneLayer};
use kivact_types::NodeId;
use std::collections::HashMap;
use std::panic::Location;
pub(super) struct Node {
    // xtask-allow(state-fields): independent: a per-query engine fact; any combination is valid
    pub(super) kind: Option<NodeType>,
    pub(super) children: Vec<NodeId>,
    // xtask-allow(state-fields): independent: a per-query engine fact; any combination is valid
    pub(super) parent: Option<NodeId>,
    // xtask-allow(state-fields): independent: a per-query creation location; any combination is valid
    pub(super) at: Option<&'static Location<'static>>,
}
pub(super) struct Facts {
    reader: Reader,
    pub(super) root: NodeId,
    nodes: HashMap<NodeId, Node>,
}
impl Facts {
    pub(super) fn new(inspect: Inspect<'_>, window: kivact_types::WindowId) -> Self {
        let root = inspect
            .roots()
            .into_iter()
            .find(|(id, _)| *id == window)
            .map(|(_, root)| root)
            .expect("invariant: the window of a Screen is open");
        let reader = inspect.reader();
        let mut nodes = HashMap::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if !inspect.contains(id) {
                continue;
            }
            let kind = reader.with_node(id, |n| n.map(|n| n.kind.node_type()));
            let children = reader.children(id);
            stack.extend(children.iter().copied());
            nodes.insert(
                id,
                Node {
                    kind,
                    children,
                    parent: reader.parent(id),
                    at: inspect.located_at(id),
                },
            );
        }
        Self {
            reader,
            root,
            nodes,
        }
    }
    pub(super) fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }
    pub(super) fn is_transparent(&self, id: NodeId) -> bool {
        self.with_node(id, |node| {
            node.is_some_and(|node| {
                node.layout
                    .as_ref()
                    .is_some_and(|layout| layout.display == kivact_types::Display::Contents)
            })
        })
    }
    /// Project retained children through structural Contents boxes in tree order.
    pub(super) fn children(&self, id: NodeId) -> Vec<NodeId> {
        let mut children = Vec::new();
        let mut stack: Vec<NodeId> = self
            .get(id)
            .into_iter()
            .flat_map(|node| node.children.iter().rev().copied())
            .collect();
        while let Some(child) = stack.pop() {
            if self.is_transparent(child) {
                if let Some(node) = self.get(child) {
                    stack.extend(node.children.iter().rev().copied());
                }
            } else if self.get(child).is_some() {
                children.push(child);
            }
        }
        children
    }
    pub(super) fn reader(&self) -> Reader {
        self.reader.clone()
    }
    pub(super) fn with_node<R>(
        &self,
        id: NodeId,
        f: impl FnOnce(Option<&NodeProperties>) -> R,
    ) -> R {
        self.reader.with_node(id, f)
    }
    pub(super) fn scene(
        &self,
        id: NodeId,
        layer: SceneLayer,
    ) -> Option<std::sync::Arc<kivact_types::DrawList>> {
        self.reader.scene(id, layer)
    }
    pub(super) fn content(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| match &node.kind {
                NodeKind::Text(data) => data.content.clone(),
                _ => None,
            })
        })
    }
    pub(super) fn input_value(&self, id: NodeId) -> Option<String> {
        self.reader().edit_state(id).map(|state| state.into_value())
    }
    pub(super) fn placeholder(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| {
                node.editing
                    .as_ref()
                    .and_then(|data| data.placeholder.clone())
            })
        })
    }
    pub(super) fn name(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_deref())
                .and_then(|semantics| semantics.name.clone())
        })
    }
    pub(super) fn role(&self, id: NodeId) -> Option<kivact_types::Role> {
        self.with_node(id, |node| node.and_then(NodeProperties::role))
    }
    pub(super) fn value(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_deref())
                .and_then(|semantics| semantics.value_text.clone())
        })
    }
    pub(super) fn selected(&self, id: NodeId) -> Option<bool> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_deref())
                .and_then(|semantics| semantics.selected)
        })
    }
    pub(super) fn checked(&self, id: NodeId) -> Option<kivact_types::CheckedState> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_deref())
                .and_then(|semantics| semantics.checked)
        })
    }
    pub(super) fn expanded(&self, id: NodeId) -> Option<bool> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_deref())
                .and_then(|semantics| semantics.expanded)
        })
    }
    pub(super) fn hidden(&self, id: NodeId) -> Option<bool> {
        self.with_node(id, |node| node.and_then(|node| node.input.hidden))
    }
    pub(super) fn disabled(&self, id: NodeId) -> Option<bool> {
        self.with_node(id, |node| node.and_then(|node| node.input.disabled))
    }
    pub(super) fn test_id(&self, id: NodeId) -> Option<String> {
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_deref())
                .and_then(|semantics| semantics.test_id.clone())
        })
    }
}
