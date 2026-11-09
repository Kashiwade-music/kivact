//! Role: Style overrides, forced states and theme tokens.

use serde::{Deserialize, Serialize};

use crate::common::{InteractiveState, NodeId, SourceLocation};

/// What a style override applies to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OverrideScope {
    /// One node.
    Node {
        /// The node.
        node: NodeId,
    },
    /// One call site.
    CallSite {
        /// The call site.
        location: SourceLocation,
    },
    /// One theme token.
    Token {
        /// The name of the token.
        token: String,
    },
}

/// The parameters of `set_style_override`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SetStyleOverrideParams {
    /// What the override applies to.
    pub scope: OverrideScope,
    /// The property.
    pub property: String,
    /// The value.
    pub value: String,
}

/// One style override.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleOverride {
    /// The id of the override.
    pub id: u64,
    /// What the override applies to.
    pub scope: OverrideScope,
    /// The property.
    pub property: String,
    /// The value.
    pub value: String,
}

/// Every style override that is set.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleOverrideList {
    /// The overrides.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<StyleOverride>,
}

/// The parameters of `clear_style_overrides`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClearStyleOverridesParams {
    /// One override, or every override when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
}

/// The parameters of `force_state`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForceStateParams {
    /// The node.
    pub node: NodeId,
    /// The states to force (an empty list releases them).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<InteractiveState>,
}

/// The kind of a theme token.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    /// A color.
    Color,
    /// A length.
    Length,
    /// A number.
    Number,
    /// Text.
    Text,
}

/// One theme token.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeToken {
    /// The name of the token.
    pub name: String,
    /// The kind of the token.
    pub kind: TokenKind,
    /// The value.
    pub value: String,
}

/// The theme tokens of the main window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeTokenList {
    /// The tokens.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<ThemeToken>,
}
