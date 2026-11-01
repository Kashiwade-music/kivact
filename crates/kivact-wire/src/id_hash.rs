//! Role: Provides cheaply hashed maps keyed by node identifiers.
//!
//! Maps keyed by node id, hashed cheaply: the core allocates node ids
//! (monotonically, never reused), so no input can flood a map, and a
//! multiplicative mix spreads sequential ids over the buckets. The tree and
//! layout lookups of every layout pass, hit test and frame go through
//! these maps (SipHash cost more than the lookup's memory access).

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

use crate::NodeId;

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
