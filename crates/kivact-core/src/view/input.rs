//! Role: Public input values and handler events: keyboard, pointer, wheel and a11y.
//!
//! Public input values and handler events: keyboard, pointer, wheel and
//! accessibility device actions, delivered to handlers with their regions.
//! Shared input values come from
//! the shared input vocabulary; handler events retain their routing facts.

use kivact_types as p;
pub use kivact_types::{
    ButtonPress, FilePhase, InputPurpose, Key, KeyCode, Modifiers, MouseButton, PointerButtons,
    PointerId, PointerPhase,
};

/// What kind of device a pointer is ([`PointerEvent::kind`]).
///
/// A touch or pen contact presses like the primary button from its
/// `Down` to its `Up` (or `Cancel`) and hovers only while it lasts:
/// `Enter` comes before its `Down`, `Leave` after its `Up` or
/// `Cancel`. The mouse's own `Leave` comes before the contact's
/// `Enter`, and a mouse resting in the window enters again after the
/// contact's `Leave`.
pub use kivact_types::PointerKind;

use kivact_style::draw::{Point, RegionId};

/// The input that requested dismissal. Unmounting an overlay does not
/// produce a dismissal event; applications close it explicitly themselves.
#[doc(inline)]
pub use kivact_types::DismissReason;

/// An assistive-technology request delivered to a node or one of its drawn
/// regions (and the dismissals delivered to an overlay's Dismiss handler).
/// Regions preserve the identity declared by `Draw::region`, so externally
/// authored editors can operate the requested clip, key or item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct A11yEvent {
    /// What is requested, with a `SetValue`'s text or a `Dismiss`'s reason.
    pub request: crate::view::A11yRequest,
    /// `None` for the root node, `Some` for a drawn region.
    pub region: Option<RegionId>,
}

pub use kivact_runtime::Handled;

/// A keyboard event with its original host route.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    pub(crate) routed: kivact_engine::contract::Routed<p::KeyInput>,
    pub(crate) target_role: Option<p::Role>,
}
impl KeyEvent {
    pub fn key(&self) -> Key {
        self.routed.input.key
    }
    pub fn code(&self) -> KeyCode {
        self.routed.input.code
    }
    pub fn modifiers(&self) -> Modifiers {
        self.routed.input.modifiers
    }
    pub fn repeat(&self) -> bool {
        matches!(
            self.routed.input.phase,
            p::KeyPhase::Down { repeat: true, .. }
        )
    }
    pub fn text(&self) -> Option<&str> {
        match &self.routed.input.phase {
            p::KeyPhase::Down { text, .. } => text.as_deref(),
            p::KeyPhase::Up => None,
        }
    }
    pub fn target_role(&self) -> Option<p::Role> {
        self.target_role
    }
    pub fn pointer(&self) -> Option<Point> {
        self.routed.input.pointer
    }
    pub fn region(&self) -> Option<RegionId> {
        self.routed.region
    }
    pub(crate) fn origin(&self) -> Option<p::FocusTarget> {
        self.routed.target.map(|node| p::FocusTarget {
            node,
            region: self.routed.region,
        })
    }
}

/// Pointer sample and node-local route delivered to a handler.
#[derive(Clone, Copy, Debug)]
pub struct PointerEvent {
    pub(crate) routed: kivact_engine::contract::Routed<p::PointerSample>,
    pub(crate) capture_action: fn(&PointerEvent, bool),
}
impl PartialEq for PointerEvent {
    fn eq(&self, other: &Self) -> bool {
        self.routed == other.routed
    }
}
impl PointerEvent {
    /// Route this pointer's subsequent events to this node. Call from its handler.
    /// Capture ends on the final button release, cancellation, or removal of the node.
    /// Capturing a hovering pointer also works without a button held.
    pub fn capture(&self) {
        (self.capture_action)(self, true);
    }

    /// Release explicit or implicit capture so subsequent events follow hit testing.
    /// Call from the capturing node's pointer handler.
    pub fn release_capture(&self) {
        (self.capture_action)(self, false);
    }

    pub fn id(&self) -> PointerId {
        self.routed.input.id
    }
    pub fn phase(&self) -> PointerPhase {
        self.routed.input.phase
    }
    pub fn pos(&self) -> Point {
        self.routed.local
    }
    pub fn window_pos(&self) -> Point {
        self.routed.input.position
    }
    pub fn buttons(&self) -> PointerButtons {
        self.routed.input.buttons
    }
    pub fn modifiers(&self) -> Modifiers {
        self.routed.input.modifiers
    }
    pub fn kind(&self) -> PointerKind {
        self.routed.input.kind
    }
    pub fn pressure(&self) -> Option<f64> {
        self.routed.input.pressure
    }
    pub fn tilt(&self) -> Option<(f64, f64)> {
        self.routed.input.tilt
    }
    pub fn is_primary(&self) -> bool {
        self.routed.input.is_primary
    }
    pub fn region(&self) -> Option<RegionId> {
        self.routed.region
    }
    pub(crate) fn origin(&self) -> p::FocusTarget {
        p::FocusTarget {
            node: self.routed.target.expect("pointer handler has a target"),
            region: self.routed.region,
        }
    }
}

/// Wheel movement and node-local route delivered to a handler.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelEvent {
    pub(crate) routed: kivact_engine::contract::Routed<p::WheelInput>,
}
impl WheelEvent {
    pub fn delta(&self) -> Point {
        self.routed.input.delta
    }
    pub fn pos(&self) -> Point {
        self.routed.local
    }
    pub fn window_pos(&self) -> Point {
        self.routed.input.position
    }
    pub fn modifiers(&self) -> Modifiers {
        self.routed.input.modifiers
    }
    pub fn region(&self) -> Option<RegionId> {
        self.routed.region
    }
    pub(crate) fn origin(&self) -> p::FocusTarget {
        p::FocusTarget {
            node: self.routed.target.expect("wheel handler has a target"),
            region: self.routed.region,
        }
    }
}

/// OS file drag in node-local and window logical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct FileEvent {
    /// Drag phase.
    pub phase: FilePhase,
    /// Files reported by the operating system.
    pub files: Vec<std::path::PathBuf>,
    /// Node-local logical position.
    pub pos: Point,
    /// Window logical position.
    pub window_pos: Point,
}

/// Whether a text editor has an active IME composition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Composition {
    /// No composition is active.
    #[default]
    Idle,
    /// The IME is composing text.
    Composing,
}
