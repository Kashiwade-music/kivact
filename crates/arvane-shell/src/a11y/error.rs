//! Role: Defines accessibility validation errors.

use kivact_types::NodeId;

/// Malformed semantics input, safe to send between threads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessibilityError {
    /// Identifiers must be unique.
    DuplicateNode(NodeId),
    /// The root, a referenced child, or a node a relation (described by,
    /// controls, labelled by) names is missing.
    MissingNode(NodeId),
    /// A repeated tree position or cycle violates tree ownership.
    InvalidTree(NodeId),
    /// A node is not reachable from the root.
    UnreachableNode(NodeId),
    /// Focus refers to an absent, hidden, disabled or non-focusable node, or
    /// one the update leaves out of the tree.
    InvalidFocus(NodeId),
    /// Bounds or scroll offsets are not finite, or bounds have a negative
    /// size.
    InvalidBounds(NodeId),
    /// Numeric properties are non-finite or form an inverted range.
    InvalidProperties(NodeId),
}

impl std::fmt::Display for AccessibilityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "accessibility error: {self:?}")
    }
}

impl std::error::Error for AccessibilityError {}
