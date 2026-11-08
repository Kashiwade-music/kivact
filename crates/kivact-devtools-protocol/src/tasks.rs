//! Role: Tasks, resources, actions and timers.

use serde::{Deserialize, Serialize};

use crate::common::{OwnerId, SourceLocation};

/// The kind of a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    /// A task.
    Task,
    /// A resource.
    Resource,
    /// An action.
    Action,
    /// A timer.
    Timer,
}

/// The phase of async work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsyncPhase {
    /// Idle.
    Idle,
    /// Pending.
    Pending,
    /// Ready.
    Ready,
    /// Failed.
    Failed,
}

/// One task, resource, action or timer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskInfo {
    /// The id of the task.
    pub id: u64,
    /// The kind.
    pub kind: TaskKind,
    /// Where the task was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceLocation>,
    /// The owner of the task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<OwnerId>,
    /// The phase.
    pub phase: AsyncPhase,
    /// When a timer is due (the app's clock, nanoseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline_ns: Option<u64>,
}

/// The tasks, resources, actions and timers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskList {
    /// The tasks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<TaskInfo>,
    /// How much async work is still blocking.
    pub outstanding_blocking: u32,
}
