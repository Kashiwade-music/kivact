//! Role: Counters, snapshots, owner trees and trace export.

use serde::{Deserialize, Serialize};

use crate::common::{NodeId, OwnerId, SourceLocation};

/// Counts of live objects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Counters {
    /// Live reactive values.
    pub live_reactive: u64,
    /// Live owners.
    pub live_owners: u64,
    /// Live objects.
    pub live_objects: u64,
    /// Live constant values.
    pub live_constants: u64,
    /// Nodes in the element trees.
    pub tree_nodes: u64,
    /// Live timers.
    pub timers: u64,
    /// Live frame loops.
    pub frame_loops: u64,
    /// Running animations.
    pub animations: u64,
    /// Turns so far.
    pub turns: u64,
    /// Async work that is still blocking.
    pub outstanding_blocking: u64,
}

/// One saved snapshot of the live objects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapshotInfo {
    /// The id of the snapshot.
    pub id: u64,
    /// When the snapshot was taken (the app's clock, nanoseconds).
    pub taken_ns: u64,
    /// How many owners it holds.
    pub owners: u32,
    /// How many objects it holds.
    pub objects: u32,
}

/// One owner in a snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerEntry {
    /// The owner.
    pub id: OwnerId,
    /// The parent owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<OwnerId>,
    /// Where the owner was created.
    pub source: SourceLocation,
}

/// One object in a snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectEntry {
    /// The object.
    pub id: u64,
    /// The owner of the object.
    pub owner: OwnerId,
    /// The name of the object.
    pub name: String,
    /// Where the object was created.
    pub source: SourceLocation,
}

/// The parameters of `diff_snapshots`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiffSnapshotsParams {
    /// The earlier snapshot.
    pub before: u64,
    /// The later snapshot.
    pub after: u64,
}

/// What changed between two snapshots.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapshotDiff {
    /// Owners that appeared.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added_owners: Vec<OwnerEntry>,
    /// Owners that disappeared.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_owners: Vec<OwnerEntry>,
    /// Objects that appeared.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added_objects: Vec<ObjectEntry>,
    /// Objects that disappeared.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_objects: Vec<ObjectEntry>,
}

/// One owner of an owner tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerNode {
    /// The owner.
    pub id: u64,
    /// The label of the owner.
    pub label: String,
    /// Where the owner was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    /// The nodes the owner created.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<NodeId>,
    /// The child owners.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<OwnerNode>,
}

/// The owner tree of a window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerTree {
    /// The roots.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roots: Vec<OwnerNode>,
}

/// The parameters of `export_trace`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportTraceParams {
    /// The path to write (relative to the workspace root, or absolute).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// The files a call wrote.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportedFiles {
    /// The paths of the files.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
}
