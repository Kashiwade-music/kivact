//! Role: Public input values and handler events: keyboard, pointer, wheel and a11y.
//!
//! Public input values and handler events: keyboard, pointer, wheel and
//! accessibility device actions, delivered to handlers with their regions.
//! Shared input values come from
//! the protocol; pointer and wheel events add access to drawing regions.

use kivact_wire as p;
pub use kivact_wire::{
    FilePhase, Key, KeyCode, Modifiers, MouseButton, PointerButtons, PointerId, PointerPhase,
};

/// What kind of device a pointer is ([`PointerEvent::kind`]).
///
/// A touch or pen contact presses like the primary button from its
/// `Down` to its `Up` (or `Cancel`) and hovers only while it lasts:
/// `Enter` comes before its `Down`, `Leave` after its `Up` or
/// `Cancel`. The mouse's own `Leave` comes before the contact's
/// `Enter`, and a mouse resting in the window enters again after the
/// contact's `Leave`.
pub use kivact_wire::PointerKind;

use kivact_reactive::Result;
use kivact_style::draw::{Point, RegionId};

/// The input that requested dismissal. Unmounting an overlay does not
/// produce a dismissal event; applications close it explicitly themselves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DismissReason {
    /// A pointer button was pressed outside the overlay and its anchor.
    OutsidePress,
    /// An unmodified, unhandled Escape reached this overlay.
    Escape,
    /// Assistive technology requested the offered Dismiss action.
    Accessibility,
}

/// An assistive-technology action delivered to a node or one of its drawn
/// regions. Regions preserve the identity declared by `Draw::region`, so
/// externally authored editors can operate the requested clip, key or item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct A11yEvent {
    /// The offered action requested by assistive technology.
    pub action: crate::view::A11yAction,
    /// `None` for the root node, `Some` for a drawn region.
    pub region: Option<RegionId>,
    /// Requested text or numeric value for `SetValue`, when supplied.
    pub value: Option<String>,
    /// Why a Dismiss was requested; `None` for other actions.
    pub dismiss_reason: Option<DismissReason>,
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

mod sealed {
    //! Role: Sealed supertraits that keep SealedKey from being implemented outside the crate.
    //!
    /// Seal of [`KeyHandlerResult`](super::KeyHandlerResult): implemented
    /// exactly for `()`, `Handled` and `Result<Handled, UiError>`.
    pub trait SealedKey {}
}

impl sealed::SealedKey for () {}
impl sealed::SealedKey for Handled {}
impl sealed::SealedKey for Result<Handled> {}

/// Return type of `on_key`/`on_pointer`/`on_wheel` handlers: `()` (not
/// handled), [`Handled`] or `Result<Handled, UiError>`.
#[diagnostic::on_unimplemented(
    message = "a key handler must return `()`, `Handled` or `Result<Handled, UiError>`"
)]
pub trait KeyHandlerResult: sealed::SealedKey {
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

/// A keyboard event delivered to a key press or release handler.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    /// Original host identity, retained independently of focus changes during bubbling.
    pub(crate) origin: Option<kivact_wire::FocusTarget>,
    /// The original keyboard target region, preserved while bubbling.
    pub region: Option<RegionId>,
    pub key: Key,
    /// Physical keyboard position.
    pub code: KeyCode,
    pub modifiers: Modifiers,
    pub repeat: bool,
    /// Text produced by the key, if any.
    pub text: Option<String>,
    /// The semantic role of the original keyboard target, after modal
    /// retargeting. This remains the target's role as the event bubbles;
    /// it is not the role of each handler's node. `None` means no explicit role.
    /// Global shortcuts can use it to leave form controls to their own handlers.
    pub target_role: Option<kivact_wire::Role>,
    /// Where the mouse was in the window, in logical pixels, when the key
    /// arrived: the point a shortcut opens a menu or picker at
    /// (`overlay_at`), whichever node has the keyboard focus.
    ///
    /// It is the last position the window reported, also while a button
    /// is held or another node captures the pointer, and whatever pointer
    /// listener is under it. It is `None` while the mouse is outside the
    /// window (also after it left with a button held) and before it first
    /// entered; a shortcut then picks its own origin, such as the centre of
    /// the focused node's `NodeRef::rect`. A touch or pen contact does not
    /// move it.
    pub pointer: Option<Point>,
}

/// Pointer event in node-local logical pixels.
///
/// The mouse is `PointerId(1)`; each touch or pen contact has an id of its
/// own while it lasts, holds [`MouseButton::Primary`] in
/// [`buttons`](Self::buttons) while in contact and names it as
/// [`button`](Self::button) at its `Down` and `Up`. Several pointers can be down at once: each one's
/// events go to the node that got its `Down`, so a handler that follows a
/// gesture keeps the [`id`](Self::id) its `Down` had.
///
/// ```ignore
/// node().on_pointer(|e: PointerEvent| {
///     if e.kind == PointerKind::Pen && e.phase == PointerPhase::Move {
///         stroke_to(e.pos, e.pressure.unwrap_or(1.0));
///     }
/// })
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    pub id: PointerId,
    pub phase: PointerPhase,
    /// Where the pointer is, relative to the node's untransformed box
    /// (its own coordinates).
    pub pos: Point,
    /// Where the pointer is in the window, in logical pixels: the point
    /// to anchor a context menu at (`overlay_at`) or to hit test other
    /// nodes with during a drag.
    pub window_pos: Point,
    /// The button this event pressed or released: set on `Down` and `Up`
    /// only, `None` for every other phase.
    pub button: Option<MouseButton>,
    /// The buttons held after this event (Pointer Events' `buttons`): a
    /// `Move` with a button held is a drag, one with none a hover. `Cancel`
    /// and the `Leave` that follows it hold none.
    pub buttons: PointerButtons,
    pub modifiers: Modifiers,
    pub click_count: u8,
    /// Mouse, touch or pen.
    pub kind: PointerKind,
    /// Pressure from 0 to 1 of a touch or pen, when the device reports
    /// it.
    pub pressure: Option<f32>,
    /// The pen's tilt from the vertical (degrees, along x and y), when the
    /// device reports it.
    pub tilt: Option<(f32, f32)>,
    /// Whether this is the primary pointer of its kind (Pointer Events'
    /// `isPrimary`): the mouse always; a touch or pen contact when no other
    /// contact of its kind was down at its `Down`, until it ends. Only the
    /// primary pointers hover, focus, and click (`on_press`); another
    /// contact only reaches `on_pointer`.
    pub is_primary: bool,
    pub(crate) origin: p::FocusTarget,
}

impl PointerEvent {
    /// The `Draw::region` hit by this event, if any.
    pub fn region(&self) -> Option<RegionId> {
        self.origin.region
    }
}

/// The handler's [`PointerEvent`] for a protocol event hitting `region`.
pub(crate) fn pointer_event_from_wire(
    e: p::PointerEvent,
    node: p::NodeId,
    region: Option<p::RegionId>,
) -> PointerEvent {
    PointerEvent {
        id: e.id,
        phase: e.phase,
        pos: e.pos,
        window_pos: e.window_pos,
        button: e.button,
        buttons: e.buttons,
        modifiers: e.modifiers,
        click_count: e.click_count,
        kind: e.kind,
        pressure: e.pressure,
        tilt: e.tilt,
        is_primary: e.is_primary,
        origin: p::FocusTarget { node, region },
    }
}

/// Wheel/trackpad scroll in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelEvent {
    pub delta: Point,
    pub pos: Point,
    pub modifiers: Modifiers,
    pub(crate) origin: p::FocusTarget,
}

impl WheelEvent {
    pub fn region(&self) -> Option<RegionId> {
        self.origin.region
    }
}

/// A wheel event (the region field is private to the public type).
pub(crate) fn wheel_event(
    delta: Point,
    pos: Point,
    modifiers: Modifiers,
    node: p::NodeId,
    region: Option<RegionId>,
) -> WheelEvent {
    WheelEvent {
        delta,
        pos,
        modifiers,
        origin: p::FocusTarget { node, region },
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
