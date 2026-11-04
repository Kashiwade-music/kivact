//! Role: Track virtual presentation surface generations.

use std::num::NonZeroU64;

/// The window's presentation surface, as the test drives the platform.
#[derive(Clone, Copy, Debug)]
pub(crate) enum TestSurface {
    /// The surface of `generation` (the count of surfaces made; the first is 1).
    Live { generation: NonZeroU64 },
    /// Destroyed by the platform; the last one was of generation `last`.
    Destroyed { last: NonZeroU64 },
}

impl TestSurface {
    /// The live surface's generation; `None` while destroyed.
    pub(crate) fn generation(self) -> Option<u64> {
        match self {
            Self::Live { generation } => Some(generation.get()),
            Self::Destroyed { .. } => None,
        }
    }
}
