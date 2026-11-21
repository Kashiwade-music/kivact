//! Role: Fold every authored modifier segment of a node, in order, into one snapshot.

use crate::view::{Node, NodeRef, Variants};
use kivact_runtime::{IntoValue, Value};

/// What the authors of a node wrote on it so far, folded over its modifier
/// segments in order (the segments a part or `absorb` closed, then the
/// open one): later values win, variants merge by type. A `#[component]`
/// body receives it as its `#[caller]` argument; the recipe of the root and
/// of every part read the same value.
#[derive(Clone, Default)]
pub struct Authored {
    variants: Variants,
    disabled: Option<Value<bool>>,
    expanded: Option<Value<bool>>,
    invalid: Option<Value<bool>>,
    name: Option<Value<String>>,
    description: Option<Value<String>>,
    labelled_by: Vec<NodeRef>,
    has_press: bool,
}

impl Node {
    /// The authored modifiers of this node so far (see [`Authored`]).
    #[doc(hidden)]
    #[track_caller]
    pub fn authored(&self) -> Authored {
        let mut variants = Variants::default();
        let mut has_press = false;
        for common in self.authored_segments() {
            if let Some(v) = common.variants.as_deref() {
                variants.merge(v);
            }
            has_press |= common.handlers.activate.is_some();
        }
        Authored {
            variants,
            disabled: self.disabled_source().map(IntoValue::into_value),
            expanded: self.expanded_source().map(IntoValue::into_value),
            invalid: self.invalid_source().map(IntoValue::into_value),
            name: self.name_source().map(IntoValue::into_value),
            description: self.description_source().map(IntoValue::into_value),
            labelled_by: self.labelled_by_sources(),
            has_press,
        }
    }

    /// The closed caller segments in order, then the open one.
    fn authored_segments(&self) -> impl Iterator<Item = &crate::view::element_data::Common> {
        self.modifiers
            .iter()
            .filter_map(|m| match m {
                crate::view::element_data::Modifier::Caller(c) => Some(c.as_ref()),
                crate::view::element_data::Modifier::Part(_) => None,
            })
            .chain(std::iter::once(&self.common))
    }
}

impl Authored {
    /// The variants the authors gave, merged by type.
    pub fn variants(&self) -> Variants {
        self.variants.clone()
    }
    /// The disabled state the authors gave, if any.
    pub fn disabled(&self) -> Option<Value<bool>> {
        self.disabled
    }
    /// The expanded state the authors gave, if any.
    pub fn expanded(&self) -> Option<Value<bool>> {
        self.expanded
    }
    /// The invalid state the authors gave, if any.
    pub fn invalid(&self) -> Option<Value<bool>> {
        self.invalid
    }
    /// The accessible name the authors gave, if any.
    pub fn name(&self) -> Option<Value<String>> {
        self.name
    }
    /// The accessible description the authors gave, if any.
    pub fn description(&self) -> Option<Value<String>> {
        self.description
    }
    /// The nodes the authors named as this node's labels.
    pub fn labelled_by(&self) -> &[NodeRef] {
        &self.labelled_by
    }
    /// Whether an author gave the node a press handler (no handler is copied).
    pub fn has_press(&self) -> bool {
        self.has_press
    }
}
