//! Role: Mounting binds the references, part hooks and canvas regions attached to a node.

use std::rc::Rc;

use kivact_types::NodeId;
use std::panic::Location;

use super::parts;
use crate::view::descriptor::Mcx;
use crate::view::node::{Constraints, Registry};
use crate::view::node_ref::NodeRef;
use crate::view::part_data::PartBinding;

/// What the parts and canvas regions attached to one node resolved to (C18).
pub(super) struct Attached {
    pub(super) constraints: Constraints,
    pub(super) bindings: Vec<PartBinding>,
    pub(super) regions: Rc<Registry>,
}

/// Last stage: bind the node's references (each once, in order), then run the parts'
/// bind hooks and bind the regions under the node's setup owner. Binding installs layout
/// and scroll observers, so it follows all retained handlers and draw-size observers; hooks
/// still precede the first commit.
pub(super) fn finish(
    cx: &Mcx<'_>,
    id: NodeId,
    refs: Vec<NodeRef>,
    attached: Option<Attached>,
    at: &'static Location<'static>,
) {
    for (index, reference) in refs.iter().enumerate() {
        if !refs[..index].contains(reference) {
            reference.bind(cx.rt, id, at);
        }
    }
    if let Some(Attached {
        constraints,
        bindings,
        regions,
    }) = attached
    {
        let owner = cx.owner;
        owner.with_setup(|| {
            parts::bind_hooks(owner, &bindings, constraints);
            regions.bind();
        });
    }
}
