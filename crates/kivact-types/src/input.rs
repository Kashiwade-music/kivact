//! Role: Portable keyboard, modifier, button, file drag and IME values.
use crate::Point;

use std::ops::Range;
use std::path::PathBuf;

/// Logical key independent of platform scan codes: what the key means in the
/// current layout, like the Web's `KeyboardEvent.key`. The physical position
/// (`KeyboardEvent.code`) is [`KeyCode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    /// Printable scalar.
    Char(char),
    /// Return key.
    Enter,
    /// Tab key.
    Tab,
    /// Space key.
    Space,
    /// Backspace key.
    Backspace,
    /// Forward delete key.
    Delete,
    /// Escape key.
    Escape,
    /// Up arrow.
    ArrowUp,
    /// Down arrow.
    ArrowDown,
    /// Left arrow.
    ArrowLeft,
    /// Right arrow.
    ArrowRight,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page up key.
    PageUp,
    /// Page down key.
    PageDown,
    /// Context menu key (Menu / Application key).
    ContextMenu,
    /// Function key number.
    F(u8),
    /// Key without a supported logical meaning.
    Unidentified,
}

/// Keyboard modifier state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    /// Shift is held.
    pub shift: bool,
    /// Control is held.
    pub ctrl: bool,
    /// Alt is held.
    pub alt: bool,
    /// Platform command key is held.
    pub meta: bool,
}

/// Mouse button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Primary button.
    Primary,
    /// Secondary button.
    Secondary,
    /// Middle button.
    Middle,
    /// Back thumb button (button four).
    Back,
    /// Forward thumb button (button five).
    Forward,
}

/// Physical keyboard position, independent of the keyboard layout, like the
/// Web's `KeyboardEvent.code`. The meaning in the layout is [`Key`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// KeyA physical position.
    KeyA,
    /// KeyB physical position.
    KeyB,
    /// KeyC physical position.
    KeyC,
    /// KeyD physical position.
    KeyD,
    /// KeyE physical position.
    KeyE,
    /// KeyF physical position.
    KeyF,
    /// KeyG physical position.
    KeyG,
    /// KeyH physical position.
    KeyH,
    /// KeyI physical position.
    KeyI,
    /// KeyJ physical position.
    KeyJ,
    /// KeyK physical position.
    KeyK,
    /// KeyL physical position.
    KeyL,
    /// KeyM physical position.
    KeyM,
    /// KeyN physical position.
    KeyN,
    /// KeyO physical position.
    KeyO,
    /// KeyP physical position.
    KeyP,
    /// KeyQ physical position.
    KeyQ,
    /// KeyR physical position.
    KeyR,
    /// KeyS physical position.
    KeyS,
    /// KeyT physical position.
    KeyT,
    /// KeyU physical position.
    KeyU,
    /// KeyV physical position.
    KeyV,
    /// KeyW physical position.
    KeyW,
    /// KeyX physical position.
    KeyX,
    /// KeyY physical position.
    KeyY,
    /// KeyZ physical position.
    KeyZ,
    /// Digit0 physical position.
    Digit0,
    /// Digit1 physical position.
    Digit1,
    /// Digit2 physical position.
    Digit2,
    /// Digit3 physical position.
    Digit3,
    /// Digit4 physical position.
    Digit4,
    /// Digit5 physical position.
    Digit5,
    /// Digit6 physical position.
    Digit6,
    /// Digit7 physical position.
    Digit7,
    /// Digit8 physical position.
    Digit8,
    /// Digit9 physical position.
    Digit9,
    /// Numpad0 physical position.
    Numpad0,
    /// Numpad1 physical position.
    Numpad1,
    /// Numpad2 physical position.
    Numpad2,
    /// Numpad3 physical position.
    Numpad3,
    /// Numpad4 physical position.
    Numpad4,
    /// Numpad5 physical position.
    Numpad5,
    /// Numpad6 physical position.
    Numpad6,
    /// Numpad7 physical position.
    Numpad7,
    /// Numpad8 physical position.
    Numpad8,
    /// Numpad9 physical position.
    Numpad9,
    /// Enter physical position.
    Enter,
    /// NumpadEnter physical position.
    NumpadEnter,
    /// NumpadAdd physical position.
    NumpadAdd,
    /// NumpadSubtract physical position.
    NumpadSubtract,
    /// NumpadMultiply physical position.
    NumpadMultiply,
    /// NumpadDivide physical position.
    NumpadDivide,
    /// NumpadDecimal physical position.
    NumpadDecimal,
    /// Tab physical position.
    Tab,
    /// Space physical position.
    Space,
    /// Backspace physical position.
    Backspace,
    /// Delete physical position.
    Delete,
    /// Escape physical position.
    Escape,
    /// ArrowUp physical position.
    ArrowUp,
    /// ArrowDown physical position.
    ArrowDown,
    /// ArrowLeft physical position.
    ArrowLeft,
    /// ArrowRight physical position.
    ArrowRight,
    /// Home physical position.
    Home,
    /// End physical position.
    End,
    /// PageUp physical position.
    PageUp,
    /// PageDown physical position.
    PageDown,
    /// ShiftLeft physical position.
    ShiftLeft,
    /// ShiftRight physical position.
    ShiftRight,
    /// ControlLeft physical position.
    ControlLeft,
    /// ControlRight physical position.
    ControlRight,
    /// AltLeft physical position.
    AltLeft,
    /// AltRight physical position.
    AltRight,
    /// SuperLeft physical position.
    SuperLeft,
    /// SuperRight physical position.
    SuperRight,
    /// Backquote physical position.
    Backquote,
    /// Minus physical position.
    Minus,
    /// Equal physical position.
    Equal,
    /// BracketLeft physical position.
    BracketLeft,
    /// BracketRight physical position.
    BracketRight,
    /// Backslash physical position.
    Backslash,
    /// Semicolon physical position.
    Semicolon,
    /// Quote physical position.
    Quote,
    /// Comma physical position.
    Comma,
    /// Period physical position.
    Period,
    /// Slash physical position.
    Slash,
    /// CapsLock physical position.
    CapsLock,
    /// NumLock physical position.
    NumLock,
    /// ScrollLock physical position.
    ScrollLock,
    /// Insert physical position.
    Insert,
    /// Pause physical position.
    Pause,
    /// PrintScreen physical position.
    PrintScreen,
    /// F1 physical position.
    F1,
    /// F2 physical position.
    F2,
    /// F3 physical position.
    F3,
    /// F4 physical position.
    F4,
    /// F5 physical position.
    F5,
    /// F6 physical position.
    F6,
    /// F7 physical position.
    F7,
    /// F8 physical position.
    F8,
    /// F9 physical position.
    F9,
    /// F10 physical position.
    F10,
    /// F11 physical position.
    F11,
    /// F12 physical position.
    F12,
    /// F13 physical position.
    F13,
    /// F14 physical position.
    F14,
    /// F15 physical position.
    F15,
    /// F16 physical position.
    F16,
    /// F17 physical position.
    F17,
    /// F18 physical position.
    F18,
    /// F19 physical position.
    F19,
    /// F20 physical position.
    F20,
    /// F21 physical position.
    F21,
    /// F22 physical position.
    F22,
    /// F23 physical position.
    F23,
    /// F24 physical position.
    F24,
    #[default]
    /// Unidentified physical position.
    Unidentified,
}

/// Phase of an OS file drag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilePhase {
    /// Entered a target.
    Enter,
    /// Moved over a target.
    Over,
    /// Left or cancelled.
    Leave,
    /// Files were dropped.
    Drop,
}

/// What the platform's input method (IME) does with a node that has an IME
/// session (`NodeProperties::ime`). Offsets are UTF-8 byte offsets.
#[derive(Clone, Debug, PartialEq)]
pub enum ImeInput {
    /// The composition (preedit) is now `text`; `selection` is the part of
    /// it the IME marks (its caret), if it says.
    Preedit {
        text: String,
        selection: Option<Range<usize>>,
    },
    /// The IME committed `text`; any composition ended with it.
    Commit(String),
    /// The IME took this UTF-8 byte range in the last surrounding-text
    /// snapshot for reconversion: the next composition replaces it.
    /// An editor that supplied an excerpt adds that excerpt's byte origin
    /// when applying the range to its complete document.
    Selection(Range<usize>),
    /// The IME session ended without a commit: the composition is gone.
    End,
}

/// A normalized pointer sample, shared from the window to the listener.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerSample {
    /// The pointer's identity for the lifetime of its contact.
    pub id: PointerId,
    /// The device that produced the sample.
    pub kind: PointerKind,
    /// The transition and its press data, when present.
    pub phase: PointerPhase,
    /// Logical window coordinates.
    pub position: Point,
    /// Buttons held after the transition.
    pub buttons: PointerButtons,
    /// The button changed by this sample, if any.
    pub button: Option<MouseButton>,
    /// Click count and button for a press or release, if any.
    pub press: Option<ButtonPress>,
    /// Pressure from zero to one, when reported by the device.
    pub pressure: Option<f64>,
    /// Pen tilt from vertical in degrees, when reported.
    pub tilt: Option<(f64, f64)>,
    /// The sample's monotonic timestamp in nanoseconds.
    pub time_ns: u64,
    /// Modifier keys held for this sample.
    pub modifiers: Modifiers,
    /// Whether this is the primary active pointer of its kind.
    pub is_primary: bool,
}

/// A normalized key press or release, independent of its routing target.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyInput {
    /// Logical meaning in the current keyboard layout.
    pub key: Key,
    /// Physical keyboard position.
    pub code: KeyCode,
    /// Press or release, including repeat and produced text.
    pub phase: KeyPhase,
    /// Modifier keys held for this event.
    pub modifiers: Modifiers,
    /// The last mouse position in logical window coordinates, if inside.
    pub pointer: Option<Point>,
}

/// A wheel movement before node-local routing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelInput {
    /// Movement in logical pixels.
    pub delta: Point,
    /// Mouse position in logical window coordinates.
    pub position: Point,
    /// Modifier keys held for the movement.
    pub modifiers: Modifiers,
}

/// An operating-system file drag before node-local routing.
#[derive(Clone, Debug, PartialEq)]
pub struct FileDrop {
    /// Transition of the file drag.
    pub phase: FilePhase,
    /// Files offered or dropped, in platform order.
    pub files: Vec<PathBuf>,
    /// Position in logical window coordinates.
    pub position: Point,
}

/// A key's press or release (`HostEvent::Key`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyPhase {
    /// The key went down. `repeat`: the system repeats a held key. `text`:
    /// the text the press produces (a character or a space), `None` for other
    /// keys.
    Down { repeat: bool, text: Option<String> },
    /// The key was released.
    Up,
}

/// Identifies one pointer: the mouse ([`PointerId::MOUSE`]), or one touch
/// or pen contact for as long as it lasts (from its `Down` to its `Up` or
/// `Cancel`). A new contact gets a new id; a window never reuses one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointerId(pub u64);

impl PointerId {
    /// The mouse (and anything the platform reports as one, such as a
    /// touchpad).
    pub const MOUSE: PointerId = PointerId(1);
}

/// What kind of device a pointer is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PointerKind {
    /// A mouse or touchpad: it hovers, has buttons and a wheel.
    #[default]
    Mouse,
    /// A finger on a touch screen: it exists only while in contact.
    Touch,
    /// A pen (stylus) on a digitizer, while in contact.
    Pen,
}

/// The button a `Down` pressed or an `Up` released, and which click of a
/// multi-click the press is. A touch or pen contact presses
/// [`MouseButton::Primary`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ButtonPress {
    /// The button.
    pub button: MouseButton,
    /// Which click of a multi-click the press is: 1 a single click, 2 a
    /// double click, ..; never 0. An `Up` has the count of the press it
    /// ends. The window counts it from the system's double-click time and
    /// distance (per axis) from the last press of the same button; a
    /// contact's taps count the same way from the last contact's `Down`.
    pub clicks: u8,
}

/// The phase of a pointer event (Pointer Events' `pointerdown` ..
/// `pointerleave`), shared by the window's input and the events a node's
/// listener receives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerPhase {
    /// A button was pressed, or a touch or pen contact began.
    Down(ButtonPress),
    /// The pointer moved, with or without buttons held.
    Move,
    /// A button was released, or a contact ended normally (lifted).
    Up(ButtonPress),
    /// The gesture ended without a release the application should act on
    /// (the window lost focus with buttons held, the platform took a
    /// contact over for a gesture or rejected a palm).
    Cancel,
    /// The pointer entered a listener. A window does not report it; the
    /// host synthesizes it when the pointer moves onto a listener.
    Enter,
    /// The pointer left a listener, or the mouse left the window. Held
    /// buttons keep their capture.
    Leave,
}

impl PointerPhase {
    /// The press of a `Down` or the release of an `Up`; `None` for the other
    /// phases.
    pub fn press(self) -> Option<ButtonPress> {
        match self {
            Self::Down(press) | Self::Up(press) => Some(press),
            Self::Move | Self::Cancel | Self::Enter | Self::Leave => None,
        }
    }
}

/// The set of mouse buttons held (Pointer Events' `buttons`). A touch or
/// pen contact holds [`MouseButton::Primary`]
/// while in contact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PointerButtons(u8);

impl PointerButtons {
    /// No button is held.
    pub const NONE: Self = Self(0);

    /// Whether `button` is held.
    pub const fn contains(self, button: MouseButton) -> bool {
        self.0 & Self::bit(button) != 0
    }

    /// This set with `button` held (`held`) or released.
    #[must_use]
    pub const fn with(self, button: MouseButton, held: bool) -> Self {
        if held {
            Self(self.0 | Self::bit(button))
        } else {
            Self(self.0 & !Self::bit(button))
        }
    }

    /// Whether no button is held.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    const fn bit(button: MouseButton) -> u8 {
        match button {
            MouseButton::Primary => 1,
            MouseButton::Secondary => 2,
            MouseButton::Middle => 4,
            MouseButton::Back => 8,
            MouseButton::Forward => 16,
        }
    }
}

/// Intended content and software keyboard hint for a text editor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InputPurpose {
    #[default]
    Text,
    Number,
    Decimal,
    Email,
    Url,
    Phone,
    Password,
    Search,
}
