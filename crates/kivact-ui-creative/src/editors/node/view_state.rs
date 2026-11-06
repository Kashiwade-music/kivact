//! Role: Application-owned bindings for this component view state.
use crate::editors::node::model::NodeLink;
use kivact::interaction::ViewTransform;
use kivact::state::IntoBinding;
use kivact::state::{Binding, signal};
/// The view state of a [`node_editor`](crate::editors::node).
#[derive(Clone, Copy)]
pub struct NodeState {
    /// The selected node IDs.
    pub selection: Binding<Vec<u64>>,
    /// The selected link, if any.
    pub link: Binding<Option<NodeLink>>,
    /// The canvas's pan and zoom.
    pub transform: Binding<ViewTransform>,
}

impl NodeState {
    /// Nothing selected and the identity transform. Setup only.
    #[track_caller]
    pub fn new() -> Self {
        Self {
            selection: signal(Vec::new()).into_binding(),
            link: signal(None).into_binding(),
            transform: signal(ViewTransform::default()).into_binding(),
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
