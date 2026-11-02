//! Role: Translates X11 keys using the server's xkbcommon keymap and state.
#![allow(
    unsafe_code,
    reason = "xkbcommon exposes its owned state through C pointers"
)]

use arvane_shell::{ShellError, ShellEvent};
use kivact_types::{Key, KeyCode, Modifiers};
use std::collections::HashSet;
use std::ptr::{self, NonNull};
use x11rb::protocol::xkb::{ConnectionExt as _, EventType, ID, MapPart, SelectEventsAux};
use x11rb::xcb_ffi::XCBConnection;
use xkbcommon_dl::{self as xkb, x11};

/// An owned xkbcommon context.
struct XkbContext {
    api: &'static xkb::XkbCommon,
    raw: NonNull<xkb::xkb_context>,
}
impl Drop for XkbContext {
    fn drop(&mut self) {
        // SAFETY: raw is the owned reference returned by xkb_context_new; keymaps are dropped first.
        unsafe { (self.api.xkb_context_unref)(self.raw.as_ptr()) };
    }
}
/// An owned xkbcommon keymap.
struct XkbKeymap {
    api: &'static xkb::XkbCommon,
    raw: NonNull<xkb::xkb_keymap>,
}
impl Drop for XkbKeymap {
    fn drop(&mut self) {
        // SAFETY: raw is an owned reference and the state referencing it was dropped first.
        unsafe { (self.api.xkb_keymap_unref)(self.raw.as_ptr()) };
    }
}
/// An owned xkbcommon state.
struct XkbState {
    api: &'static xkb::XkbCommon,
    raw: NonNull<xkb::xkb_state>,
}
impl Drop for XkbState {
    fn drop(&mut self) {
        // SAFETY: raw is an owned reference returned by xkb_x11_state_new_from_device.
        unsafe { (self.api.xkb_state_unref)(self.raw.as_ptr()) };
    }
}
/// A compiled keymap and its state; `state` drops before `keymap` (field order).
struct Map {
    state: XkbState,
    keymap: XkbKeymap,
}

pub(super) struct Keyboard {
    api: &'static xkb::XkbCommon,
    x11: &'static x11::XkbCommonX11,
    // Drops before `context` (field order).
    map: Map,
    context: XkbContext,
    pressed: HashSet<u8>,
    modifiers: Modifiers,
}

impl Keyboard {
    pub(super) fn new(connection: &XCBConnection) -> Result<Self, ShellError> {
        let api = xkb::xkbcommon_option()
            .ok_or_else(|| super::error::platform("libxkbcommon unavailable"))?;
        let x11 = x11::xkbcommon_x11_option()
            .ok_or_else(|| super::error::platform("libxkbcommon-x11 unavailable"))?;
        let raw = connection.get_raw_xcb_connection();
        // SAFETY: raw is our live independent XCB connection; output pointers may be null.
        let enabled = unsafe {
            (x11.xkb_x11_setup_xkb_extension)(
                raw,
                1,
                0,
                x11::xkb_x11_setup_xkb_extension_flags::XKB_X11_SETUP_XKB_EXTENSION_NO_FLAGS,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        if enabled == 0 {
            return Err(super::error::platform("XKB extension unavailable"));
        }
        select_events(connection)?;
        // SAFETY: this creates an owned context without changing process settings.
        let context =
            unsafe { (api.xkb_context_new)(xkb::xkb_context_flags::XKB_CONTEXT_NO_FLAGS) };
        let context = NonNull::new(context)
            .map(|raw| XkbContext { api, raw })
            .ok_or_else(|| super::error::platform("XKB context creation failed"))?;
        let map = compile(api, x11, &context, connection)?;
        Ok(Self {
            api,
            x11,
            map,
            context,
            pressed: HashSet::new(),
            modifiers: Modifiers::default(),
        })
    }

    /// Rebuilds the keymap after the server changed it. A failure keeps the
    /// previous keymap and state so typing continues; held keys stay tracked.
    pub(super) fn reload(&mut self, connection: &XCBConnection) {
        match compile(self.api, self.x11, &self.context, connection) {
            Ok(map) => self.map = map,
            Err(error) => tracing::warn!(
                target: "kivact",
                "X11 embed keymap reload failed; keeping the previous keymap: {error}"
            ),
        }
    }

    /// Applies the modifier state a pointer event carried. Returns the change
    /// when it differs from what was last reported.
    pub(super) fn sync_modifiers(&mut self, mask: u16) -> Option<ShellEvent> {
        self.set_mask(mask);
        let modifiers = self.modifiers();
        (modifiers != self.modifiers).then(|| {
            self.modifiers = modifiers;
            ShellEvent::Modifiers(modifiers)
        })
    }

    fn set_mask(&mut self, mask: u16) {
        // Core events carry the state before this key; include the XKB group bits.
        // SAFETY: state is an owned initialized state and all masks are plain values.
        unsafe {
            (self.api.xkb_state_update_mask)(
                self.map.state.raw.as_ptr(),
                u32::from(mask & 0xff),
                0,
                0,
                0,
                0,
                u32::from((mask >> 13) & 3),
            )
        };
    }

    fn modifiers(&self) -> Modifiers {
        Modifiers {
            shift: self.active(c"Shift"),
            ctrl: self.active(c"Control"),
            alt: self.active(c"Mod1"),
            meta: self.active(c"Mod4"),
        }
    }

    pub(super) fn key(&mut self, detail: u8, mask: u16, down: bool) -> Vec<ShellEvent> {
        self.set_mask(mask);
        let code = physical(detail);
        // SAFETY: state is live; any X11 keycode is accepted by xkbcommon.
        let sym = unsafe { (self.api.xkb_state_key_get_one_sym)(self.map.state.raw.as_ptr(), detail.into()) };
        // SAFETY: keysym conversion accepts unknown keysyms and returns zero for them.
        let scalar = unsafe { (self.api.xkb_keysym_to_utf32)(sym) };
        let key = logical(sym, scalar);
        let repeat = down && !self.pressed.insert(detail);
        if !down {
            self.pressed.remove(&detail);
        }
        let text = if down {
            // SAFETY: a null buffer with size zero asks for the required UTF-8 size.
            let length = unsafe {
                (self.api.xkb_state_key_get_utf8)(self.map.state.raw.as_ptr(), detail.into(), ptr::null_mut(), 0)
            };
            if length > 0 {
                let mut bytes = vec![0_u8; length as usize + 1];
                // SAFETY: the writable buffer includes space for the trailing NUL.
                unsafe {
                    (self.api.xkb_state_key_get_utf8)(
                        self.map.state.raw.as_ptr(),
                        detail.into(),
                        bytes.as_mut_ptr().cast(),
                        bytes.len(),
                    )
                };
                bytes.truncate(length as usize);
                String::from_utf8(bytes)
                    .ok()
                    .filter(|text| text.chars().all(|c| !c.is_control()))
            } else {
                None
            }
        } else {
            None
        };
        let direction = if down {
            xkb::xkb_key_direction::XKB_KEY_DOWN
        } else {
            xkb::xkb_key_direction::XKB_KEY_UP
        };
        // SAFETY: state is live and the direction enum and keycode are valid.
        unsafe { (self.api.xkb_state_update_key)(self.map.state.raw.as_ptr(), detail.into(), direction) };
        let modifiers = self.modifiers();
        self.modifiers = modifiers;
        let event = if down {
            ShellEvent::Key {
                key,
                physical: letter(code),
                code,
                repeat,
                text,
            }
        } else {
            ShellEvent::KeyUp { key, code }
        };
        vec![ShellEvent::Modifiers(modifiers), event]
    }

    fn active(&self, name: &std::ffi::CStr) -> bool {
        // SAFETY: name is NUL-terminated and state is live for this call.
        unsafe {
            (self.api.xkb_state_mod_name_is_active)(
                self.map.state.raw.as_ptr(),
                name.as_ptr(),
                xkb::xkb_state_component::XKB_STATE_MODS_EFFECTIVE,
            ) != 0
        }
    }

    /// Balance a delivered down whose raw release was canceled by XIM.
    /// Unknown and duplicate releases must not produce orphan KeyUp events.
    pub(super) fn release_if_pressed(&mut self, detail: u8, mask: u16) -> Vec<ShellEvent> {
        if self.pressed.contains(&detail) {
            self.key(detail, mask, false)
        } else {
            Vec::new()
        }
    }
    pub(super) fn focus_lost(&mut self) {
        self.pressed.clear();
        self.modifiers = Modifiers::default();
    }
}

/// Compiles the core keyboard's keymap and state.
fn compile(
    api: &'static xkb::XkbCommon,
    x11: &'static x11::XkbCommonX11,
    context: &XkbContext,
    connection: &XCBConnection,
) -> Result<Map, ShellError> {
    let raw = connection.get_raw_xcb_connection();
    // SAFETY: raw is live for the entire keyboard initialization.
    let device = unsafe { (x11.xkb_x11_get_core_keyboard_device_id)(raw) };
    if device < 0 {
        return Err(super::error::platform("XKB core keyboard unavailable"));
    }
    // SAFETY: context and connection are live; this returns an owned keymap.
    let keymap = unsafe {
        (x11.xkb_x11_keymap_new_from_device)(
            context.raw.as_ptr(),
            raw,
            device,
            xkb::xkb_keymap_compile_flags::XKB_KEYMAP_COMPILE_NO_FLAGS,
        )
    };
    let keymap = NonNull::new(keymap)
        .map(|raw| XkbKeymap { api, raw })
        .ok_or_else(|| super::error::platform("XKB keymap creation failed"))?;
    // SAFETY: the keymap and connection remain live; the state gets its own reference.
    let state = unsafe { (x11.xkb_x11_state_new_from_device)(keymap.raw.as_ptr(), raw, device) };
    // A null state drops the keymap owner on the error return.
    let state = NonNull::new(state)
        .map(|raw| XkbState { api, raw })
        .ok_or_else(|| super::error::platform("XKB state creation failed"))?;
    Ok(Map { state, keymap })
}

/// Asks the server to report keymap replacement for the core keyboard.
fn select_events(connection: &XCBConnection) -> Result<(), ShellError> {
    let parts = MapPart::KEY_TYPES
        | MapPart::KEY_SYMS
        | MapPart::MODIFIER_MAP
        | MapPart::EXPLICIT_COMPONENTS
        | MapPart::KEY_ACTIONS
        | MapPart::KEY_BEHAVIORS
        | MapPart::VIRTUAL_MODS
        | MapPart::VIRTUAL_MOD_MAP;
    connection
        .xkb_select_events(
            ID::USE_CORE_KBD.into(),
            EventType::from(0_u16),
            EventType::NEW_KEYBOARD_NOTIFY | EventType::MAP_NOTIFY,
            parts,
            parts,
            &SelectEventsAux::default(),
        )
        .map_err(super::error::platform)?
        .check()
        .map_err(super::error::platform)
}

fn logical(sym: u32, scalar: u32) -> Key {
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
        _ => char::from_u32(scalar)
            .filter(|c| !c.is_control())
            .map_or(Key::Unidentified, Key::Char),
    }
}

fn letter(code: KeyCode) -> Option<char> {
    use KeyCode::*;
    Some(match code {
        KeyA => 'a',
        KeyB => 'b',
        KeyC => 'c',
        KeyD => 'd',
        KeyE => 'e',
        KeyF => 'f',
        KeyG => 'g',
        KeyH => 'h',
        KeyI => 'i',
        KeyJ => 'j',
        KeyK => 'k',
        KeyL => 'l',
        KeyM => 'm',
        KeyN => 'n',
        KeyO => 'o',
        KeyP => 'p',
        KeyQ => 'q',
        KeyR => 'r',
        KeyS => 's',
        KeyT => 't',
        KeyU => 'u',
        KeyV => 'v',
        KeyW => 'w',
        KeyX => 'x',
        KeyY => 'y',
        KeyZ => 'z',
        _ => return None,
    })
}

// Xorg's evdev keycodes are Linux input keycodes plus eight; these identify
// physical positions, while all layout-dependent meaning comes from XKB.
pub(super) fn physical(code: u8) -> KeyCode {
    use KeyCode::*;
    match code {
        9 => Escape,
        10 => Digit1,
        11 => Digit2,
        12 => Digit3,
        13 => Digit4,
        14 => Digit5,
        15 => Digit6,
        16 => Digit7,
        17 => Digit8,
        18 => Digit9,
        19 => Digit0,
        20 => Minus,
        21 => Equal,
        22 => Backspace,
        23 => Tab,
        24 => KeyQ,
        25 => KeyW,
        26 => KeyE,
        27 => KeyR,
        28 => KeyT,
        29 => KeyY,
        30 => KeyU,
        31 => KeyI,
        32 => KeyO,
        33 => KeyP,
        34 => BracketLeft,
        35 => BracketRight,
        36 => Enter,
        37 => ControlLeft,
        38 => KeyA,
        39 => KeyS,
        40 => KeyD,
        41 => KeyF,
        42 => KeyG,
        43 => KeyH,
        44 => KeyJ,
        45 => KeyK,
        46 => KeyL,
        47 => Semicolon,
        48 => Quote,
        49 => Backquote,
        50 => ShiftLeft,
        51 => Backslash,
        52 => KeyZ,
        53 => KeyX,
        54 => KeyC,
        55 => KeyV,
        56 => KeyB,
        57 => KeyN,
        58 => KeyM,
        59 => Comma,
        60 => Period,
        61 => Slash,
        62 => ShiftRight,
        63 => NumpadMultiply,
        64 => AltLeft,
        65 => Space,
        66 => CapsLock,
        67 => F1,
        68 => F2,
        69 => F3,
        70 => F4,
        71 => F5,
        72 => F6,
        73 => F7,
        74 => F8,
        75 => F9,
        76 => F10,
        77 => NumLock,
        78 => ScrollLock,
        79 => Numpad7,
        80 => Numpad8,
        81 => Numpad9,
        82 => NumpadSubtract,
        83 => Numpad4,
        84 => Numpad5,
        85 => Numpad6,
        86 => NumpadAdd,
        87 => Numpad1,
        88 => Numpad2,
        89 => Numpad3,
        90 => Numpad0,
        91 => NumpadDecimal,
        95 => F11,
        96 => F12,
        104 => NumpadEnter,
        105 => ControlRight,
        106 => NumpadDivide,
        107 => PrintScreen,
        108 => AltRight,
        110 => Home,
        111 => ArrowUp,
        112 => PageUp,
        113 => ArrowLeft,
        114 => ArrowRight,
        115 => End,
        116 => ArrowDown,
        117 => PageDown,
        118 => Insert,
        119 => Delete,
        127 => Pause,
        133 => SuperLeft,
        134 => SuperRight,
        _ => Unidentified,
    }
}
