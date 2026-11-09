//! Role: The review: items, comments, style changes and the exported document.

use serde::{Deserialize, Serialize};

use crate::app::AppInfo;
use crate::common::{InteractiveState, NodeId, Rect, Role, SourceLocation, WindowId};
use crate::emulation::Emulation;
use crate::node::CallFrame;
use crate::style::OverrideScope;

/// What a review item points at.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReviewTarget {
    /// One element, as it was when the comment was added.
    Node {
        /// The node.
        node: NodeId,
        /// The role.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        role: Option<Role>,
        /// The name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        /// The test id.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        test_id: Option<String>,
        /// Where the node was created.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<SourceLocation>,
        /// The call chain.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        call_chain: Vec<CallFrame>,
        /// The bounds.
        bounds: Rect,
        /// The interactive states.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        states: Vec<InteractiveState>,
    },
    /// One window.
    Window {},
}

/// One style override copied into a review item.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleChange {
    /// What the override applies to.
    pub scope: OverrideScope,
    /// The property.
    pub property: String,
    /// The value before the override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// The value after the override.
    pub after: String,
}

/// One comment of a review item.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    /// The id of the comment (`C1.1`, ...).
    pub id: String,
    /// The text.
    pub text: String,
    /// When the comment was added (RFC 3339, UTC).
    pub created: String,
}

/// One item of the review.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviewItem {
    /// The id of the item (`C1`, ...).
    pub id: String,
    /// The title of the window.
    pub window_title: String,
    /// The window.
    pub window: WindowId,
    /// What the item points at.
    pub target: ReviewTarget,
    /// The style overrides of the node.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub style_changes: Vec<StyleChange>,
    /// The path of the screenshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// The comments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<Comment>,
}

/// The whole review of a session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviewDocument {
    /// The schema of the document (`kivact-devtools-review/1`).
    pub schema: String,
    /// The app.
    pub app: AppInfo,
    /// The session.
    pub session: String,
    /// When the review was created (RFC 3339, UTC).
    pub created: String,
    /// When the review was last changed (RFC 3339, UTC).
    pub updated: String,
    /// The emulation when the review was written.
    pub environment: Emulation,
    /// The items.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ReviewItem>,
}

/// The parameters of `add_comment`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddCommentParams {
    /// The window.
    pub window: WindowId,
    /// The node, or `None` for a comment on the window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeId>,
    /// The text.
    pub text: String,
    /// Whether to attach a screenshot.
    pub screenshot: bool,
}

/// The parameters of `edit_comment`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditCommentParams {
    /// The id of the comment.
    pub comment: String,
    /// The new text.
    pub text: String,
}

/// The parameters of `remove_comment`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RemoveCommentParams {
    /// The id of the comment.
    pub comment: String,
}

/// The files `export_review` wrote.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportedReview {
    /// The path of `review.json`.
    pub json: String,
    /// The path of `review.md`.
    pub markdown: String,
}
