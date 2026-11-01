//! Role: Identifiers shared by the core and the engine: nodes, requests and revisions.

/// Identity of a retained node. Allocated by the core, monotonically
/// increasing within one App and never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

/// Identity of a request that expects a result event (focus, platform effect).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RequestId(pub u64);

/// Revision of the engine's tree. Every bridge `Writer::commit` ends the
/// next revision; revision 0 means "nothing committed yet".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Revision(pub u64);

/// Identity of a hit/semantics region inside one node's scene. Unique within
/// that node only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegionId(pub u64);

/// A native keyboard focus target: a retained node or one of its regions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FocusTarget {
    /// Retained node that owns the target.
    pub node: NodeId,
    /// Region within the node, or the node itself when absent.
    pub region: Option<RegionId>,
}
