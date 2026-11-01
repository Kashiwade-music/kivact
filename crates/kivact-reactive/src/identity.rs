//! Role: Opaque App, owner and graph node identities.
//!
//! The numeric identity, the arena slot and its generation remain internal.
//! Equality, ordering, hashing and diagnostic formatting preserve them, so
//! an identity is never confused with a later one that reuses its slot.

use std::fmt;

use crate::arena::Key;

/// Identity of one registered App, without retaining or accessing its runtime.
///
/// App identities are issued by runtime registration and are never reused.
/// Comparing an identity does not validate liveness. Handles continue to
/// validate both App identity and resource lifetime when used.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppId(pub(crate) u32);

impl AppId {
    /// The existing execution sentinel: no App frame is running.
    pub(crate) const NONE: Self = Self(0);
}

impl fmt::Debug for AppId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for AppId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

/// Identity of one resource owner within its App, including the generation
/// of its slot. An identity alone neither keeps the owner alive nor says
/// whether it still is; [`crate::Owner`] pairs it with its App.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnerId(pub(crate) Key);

/// Identity of one reactive graph node (source, memo or effect) within its
/// App, including the generation of its slot. It names a node for
/// diagnostics ([`crate::Runtime::node_info`]); no handle can be made from it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub(crate) Key);

impl fmt::Debug for OwnerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OwnerId({}:{})", self.0.slot, self.0.generation)
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({}:{})", self.0.slot, self.0.generation)
    }
}
