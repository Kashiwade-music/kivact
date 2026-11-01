//! Role: Portable keyboard, modifier, button, file drag and IME values.

use std::ops::Range;

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
pub enum ImeEvent {
    /// The composition (preedit) is now `text`; `selection` is the part of
    /// it the IME marks (its caret), if it says.
    Preedit {
        text: String,
        selection: Option<Range<usize>>,
    },
    /// The IME committed `text`; any composition ended with it.
    Commit(String),
    /// The IME took this range of the session's text for reconversion: the
    /// next composition replaces it.
    Selection(Range<usize>),
    /// The IME session ended without a commit: the composition is gone.
    End,
}
