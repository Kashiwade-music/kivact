//! Role: Application-owned bindings for this component view state.
use crate::editors::node::model::NodeLink;
use kivact::interaction::ViewTransform;
use kivact::{Binding, signal};
/// The view state of a [`node_editor`](crate::editors::node).
/// Built with [`NodeState::new`].
#[derive(Clone, Copy)]
#[non_exhaustive]
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
            selection: signal(Vec::new()).into(),
            link: signal(None).into(),
            transform: signal(ViewTransform::default()).into(),
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
