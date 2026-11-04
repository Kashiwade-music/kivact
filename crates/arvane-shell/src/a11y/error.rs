//! Role: Defines accessibility validation errors.

use kivact_types::SemanticsId;

/// Malformed semantics input, safe to send between threads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessibilityError {
    /// Identifiers must be unique.
    DuplicateNode(SemanticsId),
    /// The root, a referenced child, or a node a relation (described by,
    /// controls, labelled by) names is missing.
    MissingNode(SemanticsId),
    /// A repeated tree position or cycle violates tree ownership.
    InvalidTree(SemanticsId),
    /// A node is not reachable from the root.
    UnreachableNode(SemanticsId),
    /// Focus refers to an absent, hidden, disabled or non-focusable node, or
    /// one the update leaves out of the tree.
    InvalidFocus(SemanticsId),
    /// Bounds or scroll offsets are not finite, or bounds have a negative
    /// size.
    InvalidBounds(SemanticsId),
    /// Numeric properties are non-finite or form an inverted range.
    InvalidProperties(SemanticsId),
}

impl std::fmt::Display for AccessibilityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "accessibility error: {self:?}")
    }
}

impl std::error::Error for AccessibilityError {}
