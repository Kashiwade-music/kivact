//! Role: Everything known about one element: boxes, styles, semantics and causes.

use serde::{Deserialize, Serialize};

use crate::common::{A11yAction, InteractiveState, NodeId, Rect, Role, SourceLocation};
use crate::tree::ElementInfo;

/// The parameters of `get_node`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetNodeParams {
    /// The node.
    pub node: NodeId,
}

/// The four boxes of a node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoxModel {
    /// The margin box.
    pub margin: Rect,
    /// The border box.
    pub border: Rect,
    /// The padding box.
    pub padding: Rect,
    /// The content box.
    pub content: Rect,
}

/// One frame of a call chain.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CallFrame {
    /// Where the call was made.
    pub location: SourceLocation,
    /// Whether the frame is app code (not Kivact or Arvane).
    pub app_code: bool,
}

/// The origin of a style layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleOrigin {
    /// The DevTools.
    Devtools,
    /// A forced state.
    State,
    /// Selection.
    Selected,
    /// Disabled.
    Disabled,
    /// The caller.
    Caller,
    /// A recipe.
    Recipe,
    /// The base style.
    Base,
    /// The theme default.
    ThemeDefault,
    /// Inherited from an ancestor.
    Inherited,
}

/// One style property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleProperty {
    /// The name of the property.
    pub name: String,
    /// The value.
    pub value: String,
    /// Whether the value is overridden.
    pub overridden: bool,
}

/// One layer of the style stack.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleLayerInfo {
    /// The origin of the layer.
    pub origin: StyleOrigin,
    /// The name of the layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Where the layer was declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceLocation>,
    /// The properties of the layer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<StyleProperty>,
}

/// The accessible semantics of a node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticsInfo {
    /// The role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// The name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// The states.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<String>,
    /// The actions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<A11yAction>,
    /// The nodes that label this node.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labelled_by: Vec<NodeId>,
    /// The nodes that describe this node.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub described_by: Vec<NodeId>,
    /// The nodes this node controls.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<NodeId>,
    /// Whether the node is focusable.
    pub focusable: bool,
}

/// One event handler of a node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HandlerInfo {
    /// The event name.
    pub event: String,
}

/// One read of a value that a prop or effect depends on.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WhyDep {
    /// The kind of the dependency.
    pub kind: String,
    /// The type of the dependency.
    pub ty: String,
    /// Where the dependency was created.
    pub created_at: SourceLocation,
    /// A hint at the name of the dependency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_hint: Option<String>,
    /// Where the dependency was last written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_write: Option<SourceLocation>,
    /// The turn of the last write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_write_turn: Option<u64>,
    /// The reads of this dependency.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reads: Vec<WhyDep>,
}

/// One prop and the values its effect reads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WhyProp {
    /// The prop.
    pub prop: String,
    /// Where the effect was created.
    pub effect_at: SourceLocation,
    /// The values the effect reads.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reads: Vec<WhyDep>,
}

/// Everything known about one element.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeDetail {
    /// The node.
    pub node: ElementInfo,
    /// The boxes.
    pub bounds: BoxModel,
    /// The call chain, from the node's creation to the root.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub call_chain: Vec<CallFrame>,
    /// The style layers, highest priority first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub styles: Vec<StyleLayerInfo>,
    /// The computed values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub computed: Vec<StyleProperty>,
    /// The semantics.
    pub semantics: SemanticsInfo,
    /// The handlers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handlers: Vec<HandlerInfo>,
    /// The interactive states.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<InteractiveState>,
    /// The props and the values their effects read.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub why: Vec<WhyProp>,
}
