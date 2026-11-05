//! Role: Map platform keys and normalize repeats and key text.
use super::raw::{PlatformKey, RawKey};
use crate::ShellEvent;
use kivact_types::{Key, KeyCode, KeyInput, KeyPhase, Modifiers, Point};
use std::collections::HashSet;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PhysicalKey {
    Vk(u16),
    X11(u8),
    Code(KeyCode),
}
#[derive(Default)]
pub struct KeyPipeline {
    pressed: HashSet<PhysicalKey>,
}
impl KeyPipeline {
    pub fn key(&mut self, raw: RawKey, modifiers: Modifiers, pointer: Option<Point>) -> ShellEvent {
        let (key, code, physical) = match raw.key {
            PlatformKey::Vk { vk, chr, code } => (key_from_vk(vk, chr), code, PhysicalKey::Vk(vk)),
            PlatformKey::Keysym {
                keysym,
                chr,
                keycode,
            } => (
                key_from_keysym(keysym, chr),
                code_from_x11_keycode(keycode),
                PhysicalKey::X11(keycode),
            ),
            PlatformKey::Code { key, code } => (key, code, PhysicalKey::Code(code)),
        };
        let phase = if raw.pressed {
            let repeated = !self.pressed.insert(physical);
            KeyPhase::Down {
                repeat: raw.repeat.unwrap_or(repeated),
                text: raw
                    .text
                    .filter(|text| !text.is_empty() && text.chars().all(|c| !c.is_control())),
            }
        } else {
            self.pressed.remove(&physical);
            KeyPhase::Up
        };
        ShellEvent::Key(KeyInput {
            key,
            code,
            phase,
            modifiers,
            pointer,
        })
    }
    pub fn focus_lost(&mut self) {
        self.pressed.clear();
    }
}
pub fn key_from_vk(vk: u16, chr: Option<char>) -> Key {
    match vk {
        13 => Key::Enter,
        9 => Key::Tab,
        32 => Key::Space,
        8 => Key::Backspace,
        46 => Key::Delete,
        27 => Key::Escape,
        38 => Key::ArrowUp,
        40 => Key::ArrowDown,
        37 => Key::ArrowLeft,
        39 => Key::ArrowRight,
        36 => Key::Home,
        35 => Key::End,
        33 => Key::PageUp,
        34 => Key::PageDown,
        93 => Key::ContextMenu,
        v if (112..=135).contains(&v) => Key::F((v - 112 + 1) as u8),
        _ => chr.map_or(Key::Unidentified, Key::Char),
    }
}
pub fn key_from_keysym(sym: u32, chr: Option<char>) -> Key {
    match sym {
        0xff0d | 0xff8d => Key::Enter,
        0xff09 | 0xfe20 => Key::Tab,
        0x20 | 0xff80 => Key::Space,
        0xff08 => Key::Backspace,
        0xffff => Key::Delete,
        0xff1b => Key::Escape,
        0xff51 => Key::ArrowLeft,
        0xff52 => Key::ArrowUp,
        0xff53 => Key::ArrowRight,
        0xff54 => Key::ArrowDown,
        0xff50 => Key::Home,
        0xff57 => Key::End,
        0xff55 => Key::PageUp,
        0xff56 => Key::PageDown,
        0xff67 => Key::ContextMenu,
        0xffbe..=0xffd5 => Key::F((sym - 0xffbe + 1) as u8),
        _ => chr
            .filter(|c| !c.is_control())
            .map_or(Key::Unidentified, Key::Char),
    }
}

// Xorg's evdev keycodes are Linux input keycodes plus eight; these identify
// physical positions, while all layout-dependent meaning comes from XKB.
pub fn code_from_x11_keycode(code: u8) -> KeyCode {
    match code {
        9 => KeyCode::Escape,
        10 => KeyCode::Digit1,
        11 => KeyCode::Digit2,
        12 => KeyCode::Digit3,
        13 => KeyCode::Digit4,
        14 => KeyCode::Digit5,
        15 => KeyCode::Digit6,
        16 => KeyCode::Digit7,
        17 => KeyCode::Digit8,
        18 => KeyCode::Digit9,
        19 => KeyCode::Digit0,
        20 => KeyCode::Minus,
        21 => KeyCode::Equal,
        22 => KeyCode::Backspace,
        23 => KeyCode::Tab,
        24 => KeyCode::KeyQ,
        25 => KeyCode::KeyW,
        26 => KeyCode::KeyE,
        27 => KeyCode::KeyR,
        28 => KeyCode::KeyT,
        29 => KeyCode::KeyY,
        30 => KeyCode::KeyU,
        31 => KeyCode::KeyI,
        32 => KeyCode::KeyO,
        33 => KeyCode::KeyP,
        34 => KeyCode::BracketLeft,
        35 => KeyCode::BracketRight,
        36 => KeyCode::Enter,
        37 => KeyCode::ControlLeft,
        38 => KeyCode::KeyA,
        39 => KeyCode::KeyS,
        40 => KeyCode::KeyD,
        41 => KeyCode::KeyF,
        42 => KeyCode::KeyG,
        43 => KeyCode::KeyH,
        44 => KeyCode::KeyJ,
        45 => KeyCode::KeyK,
        46 => KeyCode::KeyL,
        47 => KeyCode::Semicolon,
        48 => KeyCode::Quote,
        49 => KeyCode::Backquote,
        50 => KeyCode::ShiftLeft,
        51 => KeyCode::Backslash,
        52 => KeyCode::KeyZ,
        53 => KeyCode::KeyX,
        54 => KeyCode::KeyC,
        55 => KeyCode::KeyV,
        56 => KeyCode::KeyB,
        57 => KeyCode::KeyN,
        58 => KeyCode::KeyM,
        59 => KeyCode::Comma,
        60 => KeyCode::Period,
        61 => KeyCode::Slash,
        62 => KeyCode::ShiftRight,
        63 => KeyCode::NumpadMultiply,
        64 => KeyCode::AltLeft,
        65 => KeyCode::Space,
        66 => KeyCode::CapsLock,
        _ => extended_x11_keycode(code),
    }
}

/// Function, keypad, and navigation positions outside the typing block.
fn extended_x11_keycode(code: u8) -> KeyCode {
    match code {
        67 => KeyCode::F1,
        68 => KeyCode::F2,
        69 => KeyCode::F3,
        70 => KeyCode::F4,
        71 => KeyCode::F5,
        72 => KeyCode::F6,
        73 => KeyCode::F7,
        74 => KeyCode::F8,
        75 => KeyCode::F9,
        76 => KeyCode::F10,
        77 => KeyCode::NumLock,
        78 => KeyCode::ScrollLock,
        79 => KeyCode::Numpad7,
        80 => KeyCode::Numpad8,
        81 => KeyCode::Numpad9,
        82 => KeyCode::NumpadSubtract,
        83 => KeyCode::Numpad4,
        84 => KeyCode::Numpad5,
        85 => KeyCode::Numpad6,
        86 => KeyCode::NumpadAdd,
        87 => KeyCode::Numpad1,
        88 => KeyCode::Numpad2,
        89 => KeyCode::Numpad3,
        90 => KeyCode::Numpad0,
        91 => KeyCode::NumpadDecimal,
        95 => KeyCode::F11,
        96 => KeyCode::F12,
        104 => KeyCode::NumpadEnter,
        105 => KeyCode::ControlRight,
        106 => KeyCode::NumpadDivide,
        107 => KeyCode::PrintScreen,
        108 => KeyCode::AltRight,
        110 => KeyCode::Home,
        111 => KeyCode::ArrowUp,
        112 => KeyCode::PageUp,
        113 => KeyCode::ArrowLeft,
        114 => KeyCode::ArrowRight,
        115 => KeyCode::End,
        116 => KeyCode::ArrowDown,
        117 => KeyCode::PageDown,
        118 => KeyCode::Insert,
        119 => KeyCode::Delete,
        127 => KeyCode::Pause,
        133 => KeyCode::SuperLeft,
        134 => KeyCode::SuperRight,
        _ => KeyCode::Unidentified,
    }
}
