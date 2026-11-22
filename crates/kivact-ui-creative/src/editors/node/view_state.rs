//! Role: Application-owned bindings for this component view state.
use crate::editors::node::model::NodeLink;
use kivact::interaction::ZoomPan;
use kivact::state::Signal;
use kivact::state::signal;
/// The view state of a [`node_editor`](crate::editors::node).
#[derive(Clone, Copy)]
pub struct NodeState {
    /// The selected node IDs.
    pub selection: Signal<Vec<u64>>,
    /// The selected link, if any.
    pub link: Signal<Option<NodeLink>>,
    /// The canvas's pan and zoom.
    pub transform: Signal<ZoomPan>,
}

impl NodeState {
    /// Nothing selected and the identity transform. Setup only.
    #[track_caller]
    pub fn new() -> Self {
        Self {
            selection: signal(Vec::new()),
            link: signal(None),
            transform: signal(ZoomPan::default()),
        }
    }
}

impl Default for NodeState {
    /// Same as [`NodeState::new`]. Setup only.
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}
