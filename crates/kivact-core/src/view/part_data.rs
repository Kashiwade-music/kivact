//! Role: These values carry mounted part constraints, references and hooks.

use std::rc::Rc;

use kivact_runtime::Prop as InputProp;
use kivact_runtime::UiError;

use crate::state::{Signal, Value};
use crate::view::NodeRef;
use crate::view::part::HeldInput;

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

/// Whether a target takes input, and what input holds it when it does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    /// The target takes input.
    Enabled {
        /// A primary pointer or focused activation key is held down.
        pressed: bool,
        /// The pointer is over this exact target.
        hovered: bool,
    },
    /// A caller or attached behavior disables the target.
    Disabled,
    /// A caller or attached behavior hides the target (wins over Disabled:
    /// a hidden target is not drawn).
    Hidden,
}

impl Default for Availability {
    fn default() -> Self {
        Self::Enabled {
            pressed: false,
            hovered: false,
        }
    }
}

/// Actual input and availability facts of an attached node or canvas region
/// at one moment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartFacts {
    /// This exact target's keyboard focus; [`Focus::FocusedVisible`] also
    /// means the host requests a visible focus indication.
    pub focus: Focus,
    /// Whether this target takes input; pressed and hovered exist only while it does.
    pub availability: Availability,
}

impl PartFacts {
    /// This exact target has keyboard focus.
    pub fn is_focused(&self) -> bool {
        self.focus != Focus::None
    }

    /// The host requests a visible focus indication for this exact target.
    pub fn is_focus_visible(&self) -> bool {
        self.focus == Focus::FocusedVisible
    }

    /// A primary pointer or focused activation key is held down.
    pub fn is_pressed(&self) -> bool {
        matches!(
            self.availability,
            Availability::Enabled { pressed: true, .. }
        )
    }

    /// The pointer is over this exact target.
    pub fn is_hovered(&self) -> bool {
        matches!(
            self.availability,
            Availability::Enabled { hovered: true, .. }
        )
    }

    /// A caller or attached behavior disables this target.
    pub fn is_disabled(&self) -> bool {
        self.availability == Availability::Disabled
    }

    /// A caller or attached behavior hides this target.
    pub fn is_hidden(&self) -> bool {
        self.availability == Availability::Hidden
    }
}

/// The input state of a part, read when asked. A style or a part state that
/// never asks does not run again when the pointer hovers or presses the part;
/// hover and press looks belong in state styles (`style().hover(..)`,
/// `.active(..)`).
///
/// A live state reads the part's facts when it is asked, so a reactive reader
/// tracks only the facts it asks for. A fixed state holds given facts (a test,
/// a preview, a forced state).
#[derive(Clone, Copy)]
pub struct PartState {
    repr: PartStateRepr,
}

#[derive(Clone, Copy)]
enum PartStateRepr {
    Live(LivePart),
    Fixed(PartFacts),
}

/// The handles a live part reads its facts from: the ones its attachment owns.
#[derive(Clone, Copy)]
pub(crate) struct LivePart {
    pub(crate) focus: Value<Focus>,
    pub(crate) availability: Signal<Option<(Value<bool>, Value<bool>)>>,
    pub(crate) input: Signal<HeldInput>,
}

impl LivePart {
    /// The same part: it reads the same handles.
    pub(crate) fn same(self, other: Self) -> bool {
        let input = Value::<HeldInput>::from(self.input) == Value::from(other.input);
        let availability = Value::<Option<(Value<bool>, Value<bool>)>>::from(self.availability)
            == Value::from(other.availability);
        self.focus == other.focus && input && availability
    }
}

impl PartState {
    /// A live state of the part whose handles are `live`.
    pub(crate) fn live(live: LivePart) -> Self {
        Self {
            repr: PartStateRepr::Live(live),
        }
    }

    /// A fixed state with the given facts: a test, a preview or a forced state.
    pub fn fixed(facts: PartFacts) -> Self {
        Self {
            repr: PartStateRepr::Fixed(facts),
        }
    }

    /// The facts of this exact target now. A reactive reader that calls this
    /// tracks the facts it reads.
    pub fn get(&self) -> PartFacts {
        match self.repr {
            PartStateRepr::Live(live) => live.facts(),
            PartStateRepr::Fixed(facts) => facts,
        }
    }

    /// This exact target has keyboard focus.
    pub fn is_focused(&self) -> bool {
        self.get().is_focused()
    }

    /// The host requests a visible focus indication for this exact target.
    pub fn is_focus_visible(&self) -> bool {
        self.get().is_focus_visible()
    }

    /// A primary pointer or focused activation key is held down.
    pub fn is_pressed(&self) -> bool {
        self.get().is_pressed()
    }

    /// The pointer is over this exact target.
    pub fn is_hovered(&self) -> bool {
        self.get().is_hovered()
    }

    /// A caller or attached behavior disables this target.
    pub fn is_disabled(&self) -> bool {
        self.get().is_disabled()
    }

    /// A caller or attached behavior hides this target.
    pub fn is_hidden(&self) -> bool {
        self.get().is_hidden()
    }
}

impl Default for PartState {
    fn default() -> Self {
        Self::fixed(PartFacts::default())
    }
}

impl PartialEq for PartState {
    /// Two live states are equal when they read the same part; two fixed
    /// states are equal when their facts are; a live and a fixed state never are.
    fn eq(&self, other: &Self) -> bool {
        match (self.repr, other.repr) {
            (PartStateRepr::Live(a), PartStateRepr::Live(b)) => a.same(b),
            (PartStateRepr::Fixed(a), PartStateRepr::Fixed(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for PartState {}

impl std::fmt::Debug for PartState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PartState").field(&self.get()).finish()
    }
}
