//! Role: Public input values and handler events: keyboard, pointer, wheel and a11y.
//!
//! Public input values and handler events: keyboard, pointer, wheel and
//! accessibility device actions, delivered to handlers with their regions.
//! Shared input values come from
//! the shared input vocabulary; handler events retain their routing facts.

use kivact_types as p;
pub use kivact_types::{
    ButtonPress, FilePhase, Key, KeyCode, Modifiers, MouseButton, PointerButtons, PointerId,
    PointerPhase,
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

use kivact_reactive::Result;
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

/// What a key/pointer/wheel callback did with its event.
///
/// Like a React (DOM) handler: `Yes` is `stopPropagation()` together with
/// `preventDefault()`, `No` does neither, and `PreventDefault` is
/// `preventDefault()` alone. The default actions are the framework's, run
/// once after the whole chain when no callback prevented them:
///
/// - key press: Enter/Space activates the focused control or region, and
///   an Escape dismisses the topmost overlay offering `Dismiss`. Tab focus
///   moves and native editing happen in the host before the callbacks and
///   are not affected. Key releases have no default action.
/// - wheel: the scroll container around the listener scrolls on.
/// - pointer and file: none. The host has already moved the focus and
///   taken the capture when the callbacks run, and a click's activation is
///   a separate Activate, so `PreventDefault` behaves as `No` there.
///
/// Use `PreventDefault` when a component reacts to a key but an ancestor's
/// shortcut must still see it, for example a pan surface that tracks a
/// held Space without activating the focused control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handled {
    /// Consumed: later callbacks and ancestors do not see the event, and
    /// no default action runs.
    Yes,
    /// Not handled: the event continues to later callbacks and ancestors,
    /// then to its default action.
    No,
    /// The event continues to later callbacks and ancestors exactly as with
    /// `No`, but its default action does not run. A later `Yes` still stops
    /// it; a later `No` does not undo the prevention.
    PreventDefault,
}

impl Handled {
    /// The answer of a chain where `self` and then `next` saw the event:
    /// `Yes` wins, and a prevented default stays prevented.
    pub(crate) fn and(self, next: Handled) -> Handled {
        match (self, next) {
            (Handled::Yes, _) | (_, Handled::Yes) => Handled::Yes,
            (Handled::PreventDefault, _) | (_, Handled::PreventDefault) => Handled::PreventDefault,
            (Handled::No, Handled::No) => Handled::No,
        }
    }
}

impl crate::sealed::KeyHandlerResultSeal for () {}
impl crate::sealed::KeyHandlerResultSeal for Handled {}
impl crate::sealed::KeyHandlerResultSeal for Result<Handled> {}

/// Return type of `on_key`/`on_pointer`/`on_wheel` handlers: `()` (not
/// handled), [`Handled`] or `Result<Handled, UiError>`.
#[diagnostic::on_unimplemented(
    message = "a key handler must return `()`, `Handled` or `Result<Handled, UiError>`"
)]
pub trait KeyHandlerResult: crate::sealed::KeyHandlerResultSeal {
    #[doc(hidden)]
    fn into_handled(self) -> Result<Handled>;
}

impl KeyHandlerResult for () {
    fn into_handled(self) -> Result<Handled> {
        Ok(Handled::No)
    }
}

impl KeyHandlerResult for Handled {
    fn into_handled(self) -> Result<Handled> {
        Ok(self)
    }
}

impl KeyHandlerResult for Result<Handled> {
    fn into_handled(self) -> Result<Handled> {
        self
    }
}

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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    pub(crate) routed: kivact_engine::contract::Routed<p::PointerSample>,
}
impl PointerEvent {
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
