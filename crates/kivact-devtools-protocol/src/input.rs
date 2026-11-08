//! Role: Injected input, input logs and the IME state.

use serde::{Deserialize, Serialize};

use crate::common::{A11yAction, MouseButton, NodeId, Rect, WindowId};

/// One step of an injected input.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InputStep {
    /// Click the center of a node.
    Click {
        /// The node.
        node: NodeId,
    },
    /// Move the mouse.
    MouseMove {
        /// The horizontal position.
        x: f64,
        /// The vertical position.
        y: f64,
    },
    /// Press a mouse button.
    MouseDown {
        /// The button.
        button: MouseButton,
    },
    /// Release a mouse button.
    MouseUp {
        /// The button.
        button: MouseButton,
    },
    /// Scroll the wheel.
    Wheel {
        /// The horizontal lines (one line is 20 px).
        lines_x: f64,
        /// The vertical lines (a positive value scrolls down).
        lines_y: f64,
    },
    /// Press and release a key chord.
    Key {
        /// The chord (`"Enter"`, `"Ctrl+Enter"`, ...).
        key: String,
    },
    /// Type text, one key per character.
    Text {
        /// The text.
        text: String,
    },
    /// Perform an accessibility action.
    A11y {
        /// The node.
        node: NodeId,
        /// The action.
        action: A11yAction,
        /// The value for `SetValue`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
}

/// The parameters of `inject_input`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InjectInputParams {
    /// The window.
    pub window: WindowId,
    /// The steps, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<InputStep>,
}

/// Where an input event came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    /// The platform.
    Platform,
    /// The DevTools.
    Injected,
}

/// The outcome of one handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandlerOutcome {
    /// The handler handled the event.
    Handled,
    /// The handler did not handle the event.
    NotHandled,
    /// The handler prevented the default.
    PreventDefault,
    /// The handler failed.
    Failed,
}

/// One handler that ran for an input event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HandlerCall {
    /// The node of the handler.
    pub node: NodeId,
    /// The event.
    pub event: String,
    /// The outcome.
    pub outcome: HandlerOutcome,
}

/// One recorded input event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputLogEntry {
    /// The sequence number (1 from the start of the log).
    pub seq: u64,
    /// When the event arrived (the app's clock, nanoseconds).
    pub at_ns: u64,
    /// The window.
    pub window: WindowId,
    /// The kind of the event.
    pub kind: String,
    /// The detail of the event.
    pub detail: String,
    /// Where the event came from.
    pub source: InputSource,
    /// The node the event was sent to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<NodeId>,
    /// The route of the event, from the target to the root.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub route: Vec<NodeId>,
    /// The handlers that ran.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handlers: Vec<HandlerCall>,
}

/// The recorded input events.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputLog {
    /// The entries, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<InputLogEntry>,
}

/// The parameters of `list_inputs`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListInputsParams {
    /// Only entries after this sequence number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    /// The newest number of entries to return.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// The text input state of a window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImeState {
    /// The window.
    pub window: WindowId,
    /// The node that receives text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeId>,
    /// Whether an IME composition is in progress.
    pub composing: bool,
    /// The text being composed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preedit: Option<String>,
    /// The area of the cursor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_area: Option<Rect>,
    /// The purpose of the text input.
    pub purpose: String,
}
