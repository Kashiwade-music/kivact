//! Role: Passive semantic modifier patches and preserved public input reads.

use super::node_ref::NodeRef;
use kivact_runtime::Prop;
use kivact_types::{FocusTarget, Orientation, Semantics};
use std::panic::Location;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Relation {
    DescribedBy,
    Controls,
    LabelledBy,
}

/// A patch retains its declaration site and its place in the modifier sequence.
#[derive(Clone)]
pub(crate) struct SemanticsPatch {
    // xtask-allow(state-fields): passive patch value - part
    pub(crate) part: bool,
    pub(crate) input: SemanticInput,
    pub(crate) layer: usize,
    pub(crate) at: &'static Location<'static>,
    // xtask-allow(state-fields): passive patch value - reactive
    pub(crate) reactive: bool,
    // xtask-allow(state-fields): passive patch value - text
    pub(crate) text: bool,
    pub(crate) apply: Rc<dyn Fn(&mut Semantics<FocusTarget>)>,
    // xtask-allow(state-fields): passive patch value - relation
    pub(crate) relation: Option<(Relation, NodeRef)>,
    // xtask-allow(state-fields): passive patch value - orientation
    pub(crate) orientation: Option<Prop<Orientation>>,
}

/// Preserved inputs queried by behavior and constraints, on the canonical patch.
#[derive(Clone)]
pub(crate) enum SemanticInput {
    None,
    Role(kivact_types::Role),
    Name(Prop<String>),
    Description(Prop<String>),
    Selected(Prop<bool>),
    Disabled(Prop<bool>),
    Hidden(Prop<bool>),
    Invalid(Prop<bool>),
    Checked(Prop<bool>),
    Indeterminate(Prop<bool>),
    Expanded(Prop<bool>),
    Focusable(Prop<bool>),
}
