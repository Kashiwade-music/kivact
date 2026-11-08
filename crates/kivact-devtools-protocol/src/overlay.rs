//! Role: Highlight, overlays, pick mode and screenshots.

use serde::{Deserialize, Serialize};

use crate::common::{NodeId, WindowId};

/// The parameters of `highlight`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HighlightParams {
    /// The node to highlight, or `None` to clear it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeId>,
    /// Whether to show the layout boxes of the children too.
    pub layout: bool,
}

/// The debugging overlays drawn over every window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OverlaySettings {
    /// Flash the boxes of the nodes that were redrawn.
    pub paint_flashing: bool,
    /// Outline layout problems.
    pub layout_lint: bool,
    /// Color the nodes by how often they are redrawn.
    pub update_heatmap: bool,
    /// Show the Tab order.
    pub tab_order: bool,
}

/// The parameters of `start_pick`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StartPickParams {
    /// The window to pick in.
    pub window: WindowId,
}

/// The phase of a pick event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickPhase {
    /// The pointer is over a node.
    Hover,
    /// A node was selected.
    Select,
    /// Pick mode was cancelled.
    Cancel,
}

/// A pick mode event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PickEvent {
    /// The window.
    pub window: WindowId,
    /// The node under the pointer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeId>,
    /// The phase.
    pub phase: PickPhase,
}

/// The parameters of `screenshot`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScreenshotParams {
    /// The window.
    pub window: WindowId,
    /// One element of the window, or the whole window when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeId>,
}

/// A PNG image.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Image {
    /// The PNG bytes, base64 encoded.
    pub png_base64: String,
    /// The width of the image.
    pub width: u32,
    /// The height of the image.
    pub height: u32,
}
