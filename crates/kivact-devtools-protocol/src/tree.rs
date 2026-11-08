//! Role: The element tree: node values, tree requests and tree changes.

use serde::{Deserialize, Serialize};

use crate::common::{NodeId, NodeKind, Role, SourceLocation, WindowId};

/// One element of a window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ElementInfo {
    /// The id of the node.
    pub id: NodeId,
    /// The window of the node.
    pub window: WindowId,
    /// The parent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<NodeId>,
    /// The children.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<NodeId>,
    /// The kind.
    pub kind: NodeKind,
    /// The accessible role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// The accessible name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The text of a text node (the first 80 characters).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The test id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_id: Option<String>,
    /// Where the node was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceLocation>,
    /// Whether the node is hidden.
    pub hidden: bool,
    /// Whether the node is leaving.
    pub leaving: bool,
    /// How many times a prop changed since recording started.
    pub update_count: u32,
}

/// The parameters of `get_tree`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetTreeParams {
    /// The window.
    pub window: WindowId,
    /// The root to read from (the window's root when absent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<NodeId>,
    /// The maximum depth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
}

/// A snapshot of the element tree of one window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreeSnapshot {
    /// The window.
    pub window: WindowId,
    /// The root.
    pub root: NodeId,
    /// The revision of the tree.
    pub revision: u64,
    /// The nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<ElementInfo>,
}

/// The nodes that changed in one window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreeChanged {
    /// The window.
    pub window: WindowId,
    /// The revision of the tree.
    pub revision: u64,
    /// The added or changed nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upserted: Vec<ElementInfo>,
    /// The removed nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed: Vec<NodeId>,
}
