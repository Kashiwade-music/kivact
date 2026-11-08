//! Role: Layout problems found in a window.

use serde::{Deserialize, Serialize};

use crate::common::{NodeId, Rect, WindowId};

/// A rule of the layout lint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutRuleKind {
    /// The contents overflow their own box.
    OverflowOwnBox,
    /// The contents overflow the parent.
    OverflowParent,
    /// Text wrapped.
    Wrapped,
    /// The contents are off center.
    OffCenter,
    /// Text is clipped without an ellipsis.
    ClippedWithoutEllipsis,
}

/// One finding of the layout lint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutFindingInfo {
    /// The node.
    pub node: NodeId,
    /// The rule.
    pub rule: LayoutRuleKind,
    /// How much the rule is exceeded.
    pub amount: f64,
    /// The rectangle of the text.
    pub text_rect: Rect,
    /// The rectangle that limits the text.
    pub limit_rect: Rect,
}

/// The layout findings of a window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutLint {
    /// The window.
    pub window: WindowId,
    /// The findings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<LayoutFindingInfo>,
}
