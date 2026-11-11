//! Role: Translates X11 keys using the server's xkbcommon keymap and state.
#![allow(
    unsafe_code,
    reason = "xkbcommon exposes its owned state through C pointers"
)]

use arvane_shell::{ModifierState, ShellError, ShellEvent};
use kivact_types::Modifiers;
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
    /// Held only for its drop, which must follow the state's.
    _keymap: XkbKeymap,
}

pub(super) struct Keyboard {
    api: &'static xkb::XkbCommon,
    x11: &'static x11::XkbCommonX11,
    // Drops before `context` (field order).
    map: Map,
    context: XkbContext,
    pressed: HashSet<u8>,
    pipeline: arvane_shell::pipeline::KeyPipeline,
    modifiers: ModifierState,
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
            pipeline: arvane_shell::pipeline::KeyPipeline::default(),
            modifiers: ModifierState::default(),
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
        let modifiers = self.read_modifiers();
        self.modifiers.update(modifiers)
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

    pub(super) fn modifiers(&self) -> Modifiers {
        self.modifiers.current()
    }

    fn read_modifiers(&self) -> Modifiers {
        Modifiers {
            shift: self.active(c"Shift"),
            ctrl: self.active(c"Control"),
            alt: self.active(c"Mod1"),
            meta: self.active(c"Mod4"),
        }
    }

    pub(super) fn key(&mut self, detail: u8, mask: u16, down: bool) -> Vec<ShellEvent> {
        self.set_mask(mask);

        // SAFETY: state is live; any X11 keycode is accepted by xkbcommon.
        let sym = unsafe {
            (self.api.xkb_state_key_get_one_sym)(self.map.state.raw.as_ptr(), detail.into())
        };
        // SAFETY: keysym conversion accepts unknown keysyms and returns zero for them.
        let scalar = unsafe { (self.api.xkb_keysym_to_utf32)(sym) };

        if down {
            self.pressed.insert(detail);
        }
        if !down {
            self.pressed.remove(&detail);
        }
        let text = if down {
            // SAFETY: a null buffer with size zero asks for the required UTF-8 size.
            let length = unsafe {
                (self.api.xkb_state_key_get_utf8)(
                    self.map.state.raw.as_ptr(),
                    detail.into(),
                    ptr::null_mut(),
                    0,
                )
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
        unsafe {
            (self.api.xkb_state_update_key)(self.map.state.raw.as_ptr(), detail.into(), direction)
        };
        let modifiers = self.read_modifiers();
        let changed = self.modifiers.update(modifiers);
        let event = self.pipeline.key(
            arvane_shell::pipeline::RawKey {
                key: arvane_shell::pipeline::PlatformKey::Keysym {
                    keysym: sym,
                    chr: char::from_u32(scalar),
                    keycode: detail,
                },
                phase: if down {
                    arvane_shell::pipeline::RawKeyPhase::Down { repeat: None, text }
                } else {
                    arvane_shell::pipeline::RawKeyPhase::Up
                },
            },
            modifiers,
            None,
        );
        changed.into_iter().chain(std::iter::once(event)).collect()
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
        self.pipeline.focus_lost();
        self.modifiers.update(Modifiers::default());
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
    Ok(Map {
        state,
        _keymap: keymap,
    })
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
