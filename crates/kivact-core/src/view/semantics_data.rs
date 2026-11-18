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
    pub(super) reactive: bool,
    pub(crate) apply: Rc<dyn Fn(&mut Semantics<FocusTarget>)>,
}

/// What a patch keeps beside `apply` for later readers: the preserved input
/// of a state or name modifier, a relation, an orientation, or the
/// `semantics_text` mark. At most one per patch.
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
    /// `described_by`, `controls` or `labelled_by`: the relation and its target.
    Relation(Relation, NodeRef),
    /// `.orientation(..)`: its source, which `scroll` compares with its axes.
    Orientation(Prop<Orientation>),
    /// `semantics_text`: the node's semantics are published after its draw layers.
    Text,
}
