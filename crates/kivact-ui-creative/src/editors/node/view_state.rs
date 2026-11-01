//! Role: Application-owned bindings for this component view state.
use crate::editors::node::model::NodeLink;
use kivact::interaction::ViewTransform;
use kivact::{Binding, signal};
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
    #[allow(
        clippy::new_without_default,
        reason = "creation makes signals under the current setup owner; a `Default` impl would hide that"
    )]
    pub fn new() -> Self {
        Self {
            selection: signal(Vec::new()).into(),
            link: signal(None).into(),
            transform: signal(ViewTransform::default()).into(),
        }
    }
}
