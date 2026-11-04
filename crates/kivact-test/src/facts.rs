//! Role: Ephemeral test queries over engine facts and core creation locations.
use kivact_core::launch::Inspect;
use kivact_engine::api::Reader;
use kivact_engine::contract::{NodeKind, NodeProperties, NodeType, SceneLayer};
use kivact_types::NodeId;
use std::collections::HashMap;
use std::panic::Location;
pub(super) struct Node {
    // xtask-allow(state-fields): immutable per-query engine/creation metadata, not a lifecycle phase.
    pub(super) kind: Option<NodeType>,
    pub(super) children: Vec<NodeId>,
    // xtask-allow(state-fields): immutable per-query engine/creation metadata, not a lifecycle phase.
    pub(super) parent: Option<NodeId>,
    // xtask-allow(state-fields): immutable per-query engine/creation metadata, not a lifecycle phase.
    pub(super) at: Option<&'static Location<'static>>,
}
impl Node {
    pub(super) fn is_host(&self) -> bool {
        true
    }
}
pub(super) struct Facts {
    reader: Reader,
    pub(super) root: NodeId,
    nodes: HashMap<NodeId, Node>,
}
impl Facts {
    pub(super) fn new(inspect: Inspect<'_>) -> Self {
        let root = inspect.root();
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
        self.with_node(id, |node| {
            node.and_then(|node| node.semantics.as_deref())
                .and_then(|semantics| semantics.role)
        })
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
