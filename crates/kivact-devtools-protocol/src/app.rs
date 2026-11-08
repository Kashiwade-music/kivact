//! Role: The protocol version, app, window, client and session values.

use serde::{Deserialize, Serialize};

use crate::common::{BuildKind, Rect, WindowId, WindowMode};

/// The wire version; see design 4.2.1 for when to bump it.
pub const PROTOCOL_VERSION: u32 = 1;

/// The app behind the agent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppInfo {
    /// The name of the app.
    pub name: String,
    /// The process id.
    pub pid: u64,
    /// The version of Kivact.
    pub kivact_version: String,
    /// The build kind.
    pub build: BuildKind,
    /// The path of the executable.
    pub exe: String,
    /// The working directory.
    pub cwd: String,
    /// The root of the workspace.
    pub workspace_root: String,
}

/// One open window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowInfo {
    /// The id of the window.
    pub id: WindowId,
    /// The title.
    pub title: String,
    /// The bounds (the position is always 0).
    pub bounds: Rect,
    /// The scale factor.
    pub scale: f64,
    /// Whether the window has focus.
    pub focused: bool,
    /// The mode.
    pub mode: WindowMode,
    /// The parent window, for a tool or modal window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<WindowId>,
}

/// Every open window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowList {
    /// The windows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub windows: Vec<WindowInfo>,
}

/// The kind of a client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    /// The DevTools app.
    Devtools,
    /// An MCP server.
    Mcp,
    /// The Python client.
    Python,
    /// Another client.
    Other,
}

/// The client that opens a connection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientInfo {
    /// The name of the client.
    pub name: String,
    /// The version of the client.
    pub version: String,
    /// The kind of the client.
    pub kind: ClientKind,
}

/// The first request of a connection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InitializeParams {
    /// The protocol version of the client.
    pub protocol: u32,
    /// The token of the discovery file.
    pub token: String,
    /// The client.
    pub client: ClientInfo,
}

/// The answer of `initialize`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InitializeResult {
    /// The protocol version of the target.
    pub protocol: u32,
    /// The app.
    pub app: AppInfo,
}

/// One discovery file of an agent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiscoveryFile {
    /// The protocol version of the target.
    pub protocol: u32,
    /// The process id.
    pub pid: u64,
    /// The name of the app.
    pub app_name: String,
    /// The version of Kivact.
    pub kivact_version: String,
    /// The port of the agent.
    pub port: u32,
    /// The token a client must send.
    pub token: String,
    /// When the agent started (Unix milliseconds).
    pub started_unix_ms: u64,
    /// The root of the workspace.
    pub workspace_root: String,
}

/// A notification topic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Topic {
    /// Element tree changes.
    Tree,
    /// Recorded turns.
    Turns,
    /// Recorded frames.
    Frames,
    /// Pick mode events.
    Pick,
    /// Reported errors.
    Errors,
    /// Requests for the DevTools.
    Devtools,
    /// Script output.
    Scripts,
    /// Window changes.
    Windows,
}

/// The topics a connection subscribes to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubscribeParams {
    /// The topics (replaces the earlier choice).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<Topic>,
}
