//! Role: The DevTools highlight and pick mode.

use kivact_types::{NodeId, WindowId};

/// The node the DevTools highlights, and whether it draws the layout overlay.
#[derive(Clone, Copy)]
pub(crate) struct Highlight {
    pub(crate) node: NodeId,
    pub(crate) layout: bool,
}

/// The DevTools highlight and pick mode: the one node the user inspects,
/// either chosen in a panel or hovered while picking.
#[derive(Clone, Copy)]
pub(crate) enum Inspector {
    Idle {
        highlight: Option<Highlight>,
    },
    Picking {
        window: WindowId,
        hover: Option<Highlight>,
    },
}

impl Inspector {
    /// The node the DevTools draws as highlighted.
    pub(crate) fn highlight(&self) -> Option<Highlight> {
        match self {
            Self::Idle { highlight } => *highlight,
            Self::Picking { hover, .. } => *hover,
        }
    }
}
