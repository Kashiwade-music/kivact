//! Role: These values carry mounted part constraints, references and hooks.

use std::rc::Rc;

use kivact_runtime::Prop as InputProp;
use kivact_runtime::UiError;

use crate::Value;
use crate::view::NodeRef;

/// Final constraints of the node or canvas region to which a part is attached.
///
/// The hook receiving this value runs under the mounted element's setup owner.
/// Cleanup registered there follows that mounted element's lifetime.
#[derive(Clone, Copy)]
pub struct PartMount {
    /// The part reference, already bound to its actual target.
    pub node: NodeRef,
    /// Whether any caller or attached behavior disables this target.
    pub disabled: Value<bool>,
    /// Whether any caller or attached behavior hides this target.
    pub hidden: Value<bool>,
    /// Whether all caller and attached behavior focus constraints permit focus.
    pub focusable: Value<bool>,
    /// Caller availability, excluding behavior and roving-focus constraints.
    pub eligible: Value<bool>,
}

/// Required behavior constraints retained separately from caller modifiers.
#[derive(Clone, Default)]
pub(crate) struct PartConstraints {
    pub(crate) disabled: Vec<InputProp<bool>>,
    pub(crate) hidden: Vec<InputProp<bool>>,
    pub(crate) focusable: Vec<InputProp<bool>>,
}

/// What a part contributes to the node or region it is attached to besides
/// its handlers: its reference, hover state and mount hooks. Its state
/// observations are `Layer::Observer` entries of its handler lists.
#[derive(Clone)]
pub(crate) struct PartBinding {
    pub(crate) node: NodeRef,
    pub(crate) hovered: Value<bool>,
    pub(crate) hooks: Vec<Rc<dyn Fn(PartMount) -> Result<(), UiError>>>,
}
