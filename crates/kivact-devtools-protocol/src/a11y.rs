//! Role: The accessibility tree, audits, focus and Tab order.

use serde::{Deserialize, Serialize};

use crate::common::{NodeId, Rect, WindowId};
use crate::node::SemanticsInfo;

/// One node of an accessibility tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct A11yNode {
    /// The node.
    pub node: NodeId,
    /// The parent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<NodeId>,
    /// The children.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<NodeId>,
    /// The semantics.
    pub semantics: SemanticsInfo,
    /// The bounds.
    pub bounds: Rect,
}

/// The accessibility tree of a window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct A11yTree {
    /// The window.
    pub window: WindowId,
    /// The nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<A11yNode>,
}

/// A rule of the accessibility audit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditRule {
    /// An interactive element has no name.
    MissingName,
    /// Text has too little contrast.
    LowContrast,
    /// A target is too small.
    SmallTarget,
    /// A focusable element has no role.
    FocusableWithoutRole,
    /// A test id is used twice in one window.
    DuplicateTestId,
    /// An image has no name.
    UnlabelledImage,
}

/// One finding of the accessibility audit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditFinding {
    /// The rule.
    pub rule: AuditRule,
    /// The node.
    pub node: NodeId,
    /// The message.
    pub message: String,
    /// The measured value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// The limit of the rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<f64>,
}

/// The findings of the accessibility audit of a window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditReport {
    /// The window.
    pub window: WindowId,
    /// The findings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<AuditFinding>,
}

/// Why focus moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusReason {
    /// A pointer.
    Pointer,
    /// The keyboard.
    Keyboard,
    /// The app.
    Programmatic,
}

/// One move of focus.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FocusMove {
    /// When focus moved (the app's clock, nanoseconds).
    pub at_ns: u64,
    /// The node that was focused before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<NodeId>,
    /// The node that is focused after.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<NodeId>,
    /// Why focus moved.
    pub reason: FocusReason,
}

/// The focus of a window and its history.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FocusInfo {
    /// The window.
    pub window: WindowId,
    /// The focused node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeId>,
    /// Whether the window is focused.
    pub visible: bool,
    /// The recent moves, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<FocusMove>,
}

/// The Tab order of a window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TabOrder {
    /// The window.
    pub window: WindowId,
    /// The nodes, in Tab order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<NodeId>,
}

/// The parameters that name one window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowParams {
    /// The window.
    pub window: WindowId,
}
