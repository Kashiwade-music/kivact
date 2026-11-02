//! Role: Mounting resolves the parts and canvas regions attached to a node.

use std::rc::Rc;

use kivact_wire::NodeId;
use std::panic::Location;

use super::{part_input, parts, regions};
use crate::view::common::{Common, Constraints, Registry};
use crate::view::descriptor::Mcx;
use crate::view::part_data::PartBinding;

/// What the parts and regions attached to one node resolved to. A node
/// without parts or regions has none of them.
pub(super) struct Attachments {
    constraints: Option<Constraints>,
    bindings: Option<Rc<Vec<PartBinding>>>,
    regions: Option<Rc<Registry>>,
    /// The accessibility actions the node offers, its own and its parts'.
    pub(super) actions: Vec<crate::view::A11yAction>,
}

/// Stage 2: bind the parts' constraints and the canvas regions, collect the
/// actions the node offers, and route the node's input handlers through the
/// parts' defaults.
pub(super) fn bind(
    c: &mut Common,
    cx: &Mcx<'_>,
    id: NodeId,
    at: &'static Location<'static>,
) -> Attachments {
    let has_parts = !c.part_bindings.is_empty() || !c.regions.is_empty();
    let constraints = has_parts.then(|| parts::constraints(c, cx, at, None));
    let bindings = constraints.map(|_| Rc::new(std::mem::take(&mut c.part_bindings)));
    let mut actions = c.handlers.actions();
    for action in c.part_actions.actions() {
        if !actions.contains(&action) {
            actions.push(action);
        }
    }
    let regions = constraints.map(|constraints| {
        let root = part_input::root_target(
            &c.handlers,
            std::mem::take(&mut c.part_actions),
            constraints,
            cx,
        );
        regions::prepare(std::mem::take(&mut c.regions), root, cx, id, at)
    });

    if let Some(regions) = &regions {
        c.handlers = part_input::route(id, std::mem::take(&mut c.handlers), regions.clone());
    }
    Attachments {
        constraints,
        bindings,
        regions,
        actions,
    }
}

impl Attachments {
    /// The canvas regions the draw layers declare, if the node has any.
    pub(super) fn regions(&self) -> Option<Rc<Registry>> {
        self.regions.clone()
    }

    /// Last stage: bind the node's references, then run the parts' bind
    /// hooks and bind the regions. Binding installs layout and scroll
    /// observers, so it follows all retained handlers and draw-size
    /// observers; hooks still precede the first commit.
    pub(super) fn finish(
        self,
        cx: &Mcx<'_>,
        id: NodeId,
        c: &mut Common,
        at: &'static Location<'static>,
    ) {
        let owner = cx.owner;
        let refs = std::mem::take(&mut c.node_ref);
        for (index, reference) in refs.iter().enumerate() {
            if !refs[..index].iter().any(|previous| previous == reference) {
                reference.bind(cx.rt, id, at);
            }
        }
        if let (Some(constraints), Some(bindings), Some(regions)) =
            (self.constraints, self.bindings, self.regions)
        {
            owner.with_setup(|| {
                parts::bind_hooks(owner, &bindings, constraints);
                regions.bind();
            });
        }
    }
}
