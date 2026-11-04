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
    pub(crate) part: bool,
    pub(crate) input: SemanticInput,
    pub(crate) layer: usize,
    pub(crate) at: &'static Location<'static>,
    pub(crate) reactive: bool,
    pub(crate) text: bool,
    pub(crate) apply: Rc<dyn Fn(&mut Semantics<FocusTarget>)>,
    pub(crate) relation: Option<(Relation, NodeRef)>,
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

/// Deferred transformations, interleaved with caller modifier segments.
pub(crate) enum Modifier {
    Caller(Box<super::common::Common>),
    Part(Box<dyn FnOnce(super::node::Node) -> super::node::Node>),
}
