//! Role: These values carry mounted part constraints, references and hooks.

use std::rc::Rc;

use kivact_runtime::Prop as InputProp;
use kivact_runtime::UiError;

use crate::state::Value;
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
    pub(crate) hooks: Vec<Rc<dyn Fn(PartMount) -> Result<(), UiError>>>,
}

/// Whether this exact target has keyboard focus, and whether the host asks
/// for a visible focus indication.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Focus {
    /// The target does not have keyboard focus.
    #[default]
    None,
    /// The target has keyboard focus.
    Focused,
    /// The target has keyboard focus and the host requests a visible
    /// focus indication.
    FocusedVisible,
}

/// Actual input and availability state of an attached node or canvas region.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartState {
    // xtask-allow(state-fields): public observation snapshot - A primary pointer or focused activation key is held down
    /// A primary pointer or focused activation key is held down. Always
    /// `false` while the target is disabled or hidden.
    pub pressed: bool,
    // xtask-allow(state-fields): public observation snapshot - The pointer is over this exact target
    /// The pointer is over this exact target. Always `false` while the
    /// target is disabled or hidden.
    pub hovered: bool,
    /// This exact target's keyboard focus; [`Focus::FocusedVisible`] also
    /// means the host requests a visible focus indication.
    pub focus: Focus,
    // xtask-allow(state-fields): public observation snapshot - A caller or attached behavior disables this target
    /// A caller or attached behavior disables this target.
    pub disabled: bool,
    // xtask-allow(state-fields): public observation snapshot - A caller or attached behavior hides this target
    /// A caller or attached behavior hides this target.
    pub hidden: bool,
}

impl PartState {
    /// This exact target has keyboard focus.
    pub fn is_focused(&self) -> bool {
        self.focus != Focus::None
    }

    /// The host requests a visible focus indication for this exact target.
    pub fn is_focus_visible(&self) -> bool {
        self.focus == Focus::FocusedVisible
    }
}
