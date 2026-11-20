//! Role: The generation of a window's platform surface across platform destroys.

use crate::events::ShellEvent;
use std::num::NonZeroU64;

/// A platform surface that the platform may destroy and make again
/// (mobile suspension); each new surface has a new generation.
#[derive(Clone, Copy, Debug)]
pub enum SurfacePresence {
    /// The surface of `generation` (the first is 1).
    Live { generation: NonZeroU64 },
    /// Destroyed by the platform; the last one was of generation `last`.
    Destroyed { last: NonZeroU64 },
}

impl SurfacePresence {
    /// The first surface.
    pub fn new() -> Self {
        Self::Live {
            generation: NonZeroU64::MIN,
        }
    }
    /// The live surface's generation; `None` while destroyed.
    pub fn generation(self) -> Option<NonZeroU64> {
        match self {
            Self::Live { generation } => Some(generation),
            Self::Destroyed { .. } => None,
        }
    }
    /// Apply a platform create (`live`) or destroy; the event to deliver when it changed.
    pub fn apply(&mut self, live: bool) -> Option<ShellEvent> {
        match (*self, live) {
            (Self::Destroyed { last }, true) => {
                *self = Self::Live {
                    generation: last.saturating_add(1),
                };
                Some(ShellEvent::SurfaceCreated)
            }
            (Self::Live { generation }, false) => {
                *self = Self::Destroyed { last: generation };
                Some(ShellEvent::SurfaceDestroyed)
            }
            _ => None,
        }
    }
}

impl Default for SurfacePresence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::SurfacePresence;
    use crate::events::ShellEvent;

    #[test]
    fn a_surface_made_again_has_a_new_generation() {
        let mut surface = SurfacePresence::new();
        assert_eq!(surface.apply(false), Some(ShellEvent::SurfaceDestroyed));
        assert_eq!(surface.generation(), None);
        assert_eq!(surface.apply(true), Some(ShellEvent::SurfaceCreated));
        assert_eq!(surface.generation().map(|g| g.get()), Some(2));
        assert_eq!(surface.apply(true), None);
    }
}
