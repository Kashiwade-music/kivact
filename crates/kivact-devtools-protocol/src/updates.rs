//! Role: Recorded turns and frames: cause graphs, stages and frame timings.

use serde::{Deserialize, Serialize};

use crate::common::{NodeId, SourceLocation, WindowId};

/// What is being recorded.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordingSettings {
    /// Record turns and their cause graphs.
    pub updates: bool,
    /// Record frames.
    pub frames: bool,
    /// Record input events.
    pub input: bool,
}

/// A stage of a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnStage {
    /// The turn begins.
    Begin,
    /// Reports are collected.
    Reports,
    /// Changes are dispatched.
    Dispatch,
    /// Async work runs.
    Async,
    /// The changes are committed.
    Commit,
    /// Nodes are mounted.
    Mounts,
    /// User effects run.
    UserEffects,
    /// Output is written.
    Output,
}

/// How long one stage of a turn took.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StageTime {
    /// The stage.
    pub stage: TurnStage,
    /// When the stage started (the app's clock, nanoseconds).
    pub start_ns: u64,
    /// How long the stage took (nanoseconds).
    pub duration_ns: u64,
}

/// The kind of a cause.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseKind {
    /// Input.
    Input,
    /// A timer.
    Timer,
    /// An async task.
    Task,
    /// A mount.
    Mount,
    /// A DevTools call.
    Devtools,
    /// A write to a signal.
    SignalWrite,
    /// A memo re-ran.
    MemoRun,
    /// An effect ran.
    EffectRun,
    /// A prop changed.
    PropChange,
    /// Layout.
    Layout,
    /// Paint.
    Paint,
    /// A frame was presented.
    Frame,
}

/// One node of a cause graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CauseNode {
    /// The id of the node in the graph.
    pub id: u32,
    /// The kind of the cause.
    pub kind: CauseKind,
    /// A label.
    pub label: String,
    /// The type of the value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    /// A hint at the name of the value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_hint: Option<String>,
    /// Where the value was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceLocation>,
    /// The node the cause changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeId>,
    /// The window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowId>,
    /// How many events the node stands for.
    pub count: u32,
    /// Whether the work changed nothing.
    pub wasted: bool,
    /// For a merged node (`count` > 1): the element nodes of the merged members, in first-seen order without duplicates; empty otherwise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<NodeId>,
}

/// An edge of a cause graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CauseEdge {
    /// The source node.
    pub source: u32,
    /// The target node.
    pub target: u32,
}

/// Why layout ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutReason {
    /// A normal pass.
    Pass,
    /// Styles changed.
    Style,
    /// The viewport changed.
    Viewport,
}

/// One layout pass.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutPass {
    /// The window.
    pub window: WindowId,
    /// The reasons layout ran.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<LayoutReason>,
    /// How many nodes were measured.
    pub measured: u32,
    /// How many nodes were laid out.
    pub laid_out: u32,
}

/// The renderer of a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Renderer {
    /// The CPU rasterizer.
    Cpu,
    /// The GPU rasterizer.
    Gpu,
}

/// One presented frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameInfo {
    /// The window.
    pub window: WindowId,
    /// When the frame ended (the app's clock, nanoseconds).
    pub end_ns: u64,
    /// How long the work took (nanoseconds).
    pub work_ns: u64,
    /// How long the scene took (nanoseconds).
    pub scene_ns: u64,
    /// How long rasterizing took (nanoseconds).
    pub raster_ns: u64,
    /// How long presenting took (nanoseconds).
    pub present_ns: u64,
    /// How many chunks were rastered.
    pub chunks_rastered: u32,
    /// How many pixels were rastered.
    pub pixels_rastered: u64,
    /// How many pixels were damaged.
    pub damage_pixels: u64,
    /// The renderer.
    pub renderer: Renderer,
    /// The length of the frame period (nanoseconds).
    pub frame_period_ns: u64,
    /// Whether the frame took longer than the frame period.
    pub over_budget: bool,
}

/// Work that changed nothing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WastedWork {
    /// Writes of an equal value.
    pub equal_writes: u32,
    /// Memos that returned an equal value.
    pub equal_memos: u32,
    /// Writes of an unchanged prop.
    pub unchanged_props: u32,
}

/// One recorded turn and its cause graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TurnGraph {
    /// The turn number.
    pub turn: u64,
    /// When the turn started (the app's clock, nanoseconds).
    pub started_ns: u64,
    /// How long the turn took (nanoseconds).
    pub duration_ns: u64,
    /// The stages.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<StageTime>,
    /// The nodes of the cause graph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<CauseNode>,
    /// The edges of the cause graph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<CauseEdge>,
    /// The nodes that changed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_nodes: Vec<NodeId>,
    /// The layout passes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout: Vec<LayoutPass>,
    /// The frames.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<FrameInfo>,
    /// The work that changed nothing.
    pub wasted: WastedWork,
    /// Whether the rest of the graph was dropped.
    pub truncated: bool,
}

/// The parameters of `list_turns`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListTurnsParams {
    /// Only turns after this number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    /// The newest number of turns to return.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// How long to wait for a new turn (milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_ms: Option<u32>,
}

/// Recorded turns.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TurnList {
    /// The turns, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub turns: Vec<TurnGraph>,
}

/// The parameters of `list_frames`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListFramesParams {
    /// One window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowId>,
    /// Only frames that ended after this time (nanoseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    /// The newest number of frames to return.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Recorded frames.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameList {
    /// The frames, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<FrameInfo>,
}
