//! Role: Queries of the element tree by role, text or test id.

use serde::{Deserialize, Serialize};

use crate::common::{NodeId, Role, WindowId};
use crate::tree::ElementInfo;

/// A query by role and name.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoleQuery {
    /// The window.
    pub window: WindowId,
    /// Search only under this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within: Option<NodeId>,
    /// The role.
    pub role: Role,
    /// The exact name (`None` compares only the role).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// How long `find_*` waits (5000 ms by default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
}

/// A query by text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextQuery {
    /// The window.
    pub window: WindowId,
    /// Search only under this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within: Option<NodeId>,
    /// The exact text.
    pub text: String,
    /// How long `find_*` waits (5000 ms by default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
}

/// A query by test id.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestIdQuery {
    /// The window.
    pub window: WindowId,
    /// Search only under this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within: Option<NodeId>,
    /// The exact test id.
    pub id: String,
    /// How long `find_*` waits (5000 ms by default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
}

/// The one node a query found.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FoundNode {
    /// The node.
    pub node: ElementInfo,
}

/// The node a query found, if any.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaybeNode {
    /// The node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<ElementInfo>,
}

/// Every node a query found.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FoundNodes {
    /// The nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<ElementInfo>,
}
