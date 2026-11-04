//! Role: Shared node, region, request and text identities and identity maps.
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// The stable key of a text run; usable on any thread.
/// Authors keep the key across updates; the engine assigns its platform identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextRunId(pub u64);

/// An odd constant (2^64 / the golden ratio): multiplying by it maps
/// sequential ids to distinct low bits and mixes the high bits.
const MIX: u64 = 0x9e37_79b9_7f4a_7c15;

/// Hashes node identifiers for [`IdMap`].
#[derive(Clone, Copy, Debug, Default)]
pub struct IdHasher(u64);

impl Hasher for IdHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_u64(u64::from(*byte));
        }
    }

    #[inline]
    fn write_u64(&mut self, n: u64) {
        self.0 = (self.0.rotate_left(5) ^ n).wrapping_mul(MIX);
    }
}

/// Maps node identifiers to values using [`IdHasher`].
pub type IdMap<V> = HashMap<NodeId, V, BuildHasherDefault<IdHasher>>;

/// Identity of a retained node. Allocated by the core, monotonically
/// increasing within one App and never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

/// Identity of a request that expects a result event (focus, platform effect).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RequestId(pub u64);

/// Revision of the engine's tree: revision `n` is the tree after the `n`-th
/// bridge `Writer::commit`; revision 0 is the empty tree before the first
/// commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Revision(pub u64);

/// A native keyboard focus target: a retained node or one of its regions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FocusTarget {
    /// Retained node that owns the target.
    pub node: NodeId,
    /// Region within the node, or the node itself when absent.
    pub region: Option<RegionId>,
}

/// Stable host-assigned accessibility node identity; usable on any thread.
/// Arvane treats it as an opaque key with no knowledge of the host's tree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticsId(pub u64);

/// Identity of a hit/semantics region inside one node's scene. Unique within
/// that node only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegionId(pub u64);
