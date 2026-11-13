//! Role: The embedded child's surface presence, derived from its visibility.

use arvane_shell::ShellEvent;
use kivact_types::embed::Visibility;

/// Whether the child can draw, and the surface generation that says so.
///
/// A hidden window is a window without a drawable surface: hiding delivers
/// `SurfaceDestroyed` and showing creates the next generation with
/// `SurfaceCreated`, so the existing "cannot draw" path is the only one.
#[derive(Clone, Copy)]
pub(in crate::port) enum ChildSurface {
    /// The window is shown with this surface generation.
    Shown {
        /// The current generation.
        generation: u64,
    },
    /// The window is hidden; the last generation is kept for the next show.
    Hidden {
        /// The generation the next show advances from.
        last: u64,
    },
}

impl ChildSurface {
    /// The surface of a window opened with `visibility`.
    pub(in crate::port) fn new(visibility: Visibility) -> Self {
        match visibility {
            Visibility::Shown => Self::Shown { generation: 1 },
            Visibility::Hidden => Self::Hidden { last: 0 },
        }
    }

    /// The current surface generation, or `None` while hidden.
    pub(in crate::port) fn generation(&self) -> Option<u64> {
        match self {
            Self::Shown { generation } => Some(*generation),
            Self::Hidden { .. } => None,
        }
    }

    /// Apply `visibility`; the boundary event, or `None` when it did not change.
    pub(in crate::port) fn apply(&mut self, visibility: Visibility) -> Option<ShellEvent> {
        match (visibility, &*self) {
            (Visibility::Shown, Self::Hidden { last }) => {
                let generation = last
                    .checked_add(1)
                    .expect("invariant: surface generations fit u64");
                *self = Self::Shown { generation };
                Some(ShellEvent::SurfaceCreated)
            }
            (Visibility::Hidden, Self::Shown { generation }) => {
                let last = *generation;
                *self = Self::Hidden { last };
                Some(ShellEvent::SurfaceDestroyed)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the surface presence derived from visibility.
    use super::ChildSurface;
    use arvane_shell::ShellEvent;
    use kivact_types::embed::Visibility;

    #[test]
    fn hidden_has_no_surface_until_shown() {
        let mut surface = ChildSurface::new(Visibility::Hidden);
        assert_eq!(surface.generation(), None);
        assert_eq!(surface.apply(Visibility::Hidden), None);
        assert_eq!(
            surface.apply(Visibility::Shown),
            Some(ShellEvent::SurfaceCreated)
        );
        assert_eq!(surface.generation(), Some(1));
        assert_eq!(
            surface.apply(Visibility::Hidden),
            Some(ShellEvent::SurfaceDestroyed)
        );
        assert_eq!(surface.generation(), None);
        assert_eq!(
            surface.apply(Visibility::Shown),
            Some(ShellEvent::SurfaceCreated)
        );
        assert_eq!(surface.generation(), Some(2));
    }
}
