//! Role: Windows-specific input handling.
//!
//! The Windows subclass of a shell window (`SetWindowSubclass`): answers
//! the IME's `WM_IME_REQUEST` queries from the focused text input's state,
//! notes the client position of button and wheel messages, notes the kind,
//! cancellation and pen tilt of every touch and pen pointer of a pointer
//! frame (`WM_POINTER*`, `GetPointerFrameInfo`) and drops the mouse messages Windows synthesizes from
//! them (see the parent module).
//!
//! Requests answered (others fall to the next procedure):
//! - `IMR_RECONVERTSTRING`: the text with the selection as the range to
//!   reconvert (an empty selection lets the IME choose around the caret);
//!   nothing while composing.
//! - `IMR_CONFIRMRECONVERTSTRING`: the range the IME will reconvert,
//!   noted for the runner (it becomes the selection, so the composition
//!   that follows replaces it).
//! - `IMR_DOCUMENTFEED`: the text with the composition (or the caret) as
//!   the target, for the IME's predictions.
//! - `IMR_QUERYCHARPOSITION`: the screen position and line height of a
//!   character of the composition (or the caret), and the text area.
//!
//! Invariants:
//! - Each block holds one unsafe
//!   operation and says why it is sound. Buffers the IME passes are read
//!   through a slice of the size the IME wrote into them, after checking
//!   that it covers what is written.
//! - The subclass holds one strong count of the window's `NativeInput`
//!   (its reference data) from installing to removing; it is removed once,
//!   by whichever comes first: the window's destruction (`WM_NCDESTROY`)
//!   or the drop of the [`Subclass`] guard. The count is given back only
//!   when the removal succeeds; otherwise it is leaked, since the
//!   procedure may still run with it.
//! - Nothing here panics (a panic cannot unwind out of a window procedure).
//!
//! `WM_SETTINGCHANGE` reads the system's settings again: the double-click
//! time and distance (`GetDoubleClickTime`, `SM_CXDOUBLECLK` /
//! `SM_CYDOUBLECLK`), the lines and characters a wheel notch scrolls
//! (`SPI_GETWHEELSCROLLLINES`, `SPI_GETWHEELSCROLLCHARS`), the animation
//! effects setting (`SPI_GETCLIENTAREAANIMATION`, reduced motion) and the
//! locale; when one changed, it notes the change and wakes the loop.
//!
//! Debug builds only: the registered message `ArvaneDebugDpiChange` (any
//! process may post it; `wparam` is a DPI) makes the window send itself the
//! `WM_DPICHANGED` a move to a monitor of that DPI sends, so the GUI
//! harness drives the real scale-change path on machines whose monitors
//! all have one DPI (Windows refuses a `WM_DPICHANGED` from another
//! process, and changing the display's scale is a system setting).
#![allow(
    unsafe_code,
    reason = "a window subclass and the IME's request buffers are Win32 FFI; \
              winit 0.30 answers neither"
)]

use std::rc::Rc;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{ClientToScreen, ScreenToClient};
use windows_sys::Win32::UI::Input::Ime::{
    IMECHARPOSITION, IMR_CONFIRMRECONVERTSTRING, IMR_DOCUMENTFEED, IMR_QUERYCHARPOSITION,
    IMR_RECONVERTSTRING, RECONVERTSTRING,
};
use windows_sys::Win32::UI::Input::Pointer::{
    GetPointerFrameInfo, GetPointerInfo, GetPointerPenInfo, POINTER_FLAG_CANCELED, POINTER_INFO,
    POINTER_PEN_INFO,
};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetMessageExtraInfo, GetSystemMetrics, GetWindowRect, KillTimer, PEN_MASK_TILT_X,
    PEN_MASK_TILT_Y, POINTER_MESSAGE_FLAG_CANCELED, PT_PEN, PT_TOUCH, RegisterWindowMessageW,
    SM_CXDOUBLECLK, SM_CYDOUBLECLK, SPI_GETCLIENTAREAANIMATION, SPI_GETWHEELSCROLLCHARS,
    SPI_GETWHEELSCROLLLINES, SendMessageW, SetTimer, SystemParametersInfoW, WM_DPICHANGED,
    WM_IME_REQUEST, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL,
    WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCDESTROY, WM_POINTERDOWN, WM_POINTERUP, WM_POINTERUPDATE,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETTINGCHANGE, WM_TIMER, WM_XBUTTONDOWN, WM_XBUTTONUP,
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;

use super::state::{ContactNote, MouseSettings, NativeInput};
use arvane_shell::ShellImeText;
use kivact_types::PointerKind;

/// The subclass id (one subclass per window).
const ID: usize = 0x4172_7661;
/// The size of `RECONVERTSTRING` (its string follows it).
const HEADER: usize = std::mem::size_of::<RECONVERTSTRING>();

/// The installed subclass; dropping it removes the subclass if the window
/// still has it.
#[derive(Debug)]
pub(crate) struct Subclass {
    hwnd: HWND,
    input: Rc<NativeInput>,
}

impl Subclass {
    /// Subclass `window` (created on this thread) to feed `input`; `None`
    /// when it has no Win32 handle or Windows refuses.
    pub(crate) fn install(window: &Window, input: &Rc<NativeInput>) -> Option<Self> {
        let RawWindowHandle::Win32(handle) = window.window_handle().ok()?.as_raw() else {
            return None;
        };
        let hwnd = handle.hwnd.get();
        let data = Rc::into_raw(Rc::clone(input));
        // SAFETY: `hwnd` is a live window created on this thread (winit
        // creates windows on the event loop's thread, which runs this);
        // `subclass` has the SUBCLASSPROC signature; `data` stays valid
        // until the subclass is removed (`release`).
        let installed = unsafe { SetWindowSubclass(hwnd, Some(subclass), ID, data as usize) };
        if installed == 0 {
            // SAFETY: `data` came from `Rc::into_raw` above and the failed
            // call kept no copy of it.
            drop(unsafe { Rc::from_raw(data) });
            return None;
        }
        input.subclassed.set(true);
        Some(Subclass {
            hwnd,
            input: Rc::clone(input),
        })
    }
}

impl Drop for Subclass {
    fn drop(&mut self) {
        if self.input.subclassed.replace(false) {
            release(self.hwnd, Rc::as_ptr(&self.input));
        }
    }
}

/// Remove the subclass of `hwnd` and give back its strong count of `data`.
/// If Windows refuses to remove it, the procedure may still run with
/// `data`, so its count is leaked instead (a few words per window).
fn release(hwnd: HWND, data: *const NativeInput) {
    // SAFETY: removes the subclass `install` put on `hwnd`, on the
    // window's thread (a destroyed window's subclass is gone already).
    if unsafe { RemoveWindowSubclass(hwnd, Some(subclass), ID) } == 0 {
        return;
    }
    // SAFETY: `data` points to the allocation whose count `install` gave
    // the subclass with `Rc::into_raw`; the subclass is removed, so this
    // is its last use.
    drop(unsafe { Rc::from_raw(data) });
}

/// The window subclass procedure (`SUBCLASSPROC`) installed by `install`.
///
/// # Safety
///
/// Only Windows may call this, as the subclass procedure that `install`
/// registered, on the thread that owns `hwnd`, with the arguments of the
/// message it is delivering. `data` must be the `NativeInput` pointer that
/// `install` gave the subclass, which holds a strong count of it until the
/// subclass is removed.
unsafe extern "system" fn subclass(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    data: usize,
) -> LRESULT {
    // SAFETY: `data` is the `NativeInput` the subclass holds a strong count
    // of until it is removed (below, or by the guard on this thread), and
    // the window procedure runs on the thread that owns it.
    let input = unsafe { &*(data as *const NativeInput) };
    match message {
        WM_IME_REQUEST => {
            if let Some(answer) = ime_request(hwnd, input, wparam, lparam) {
                return answer;
            }
        }
        // A touch or pen contact, which winit reports as a touch: note what
        // winit leaves out (its kind, a cancellation, a pen's tilt) for
        // every pointer of the message's frame. winit reports all of them
        // from this one message and skips the frame's other messages, so a
        // note of the message's pointer alone would miss the others' (a
        // canceled two-finger frame would lift the second finger).
        WM_POINTERDOWN | WM_POINTERUPDATE | WM_POINTERUP => {
            let id = (wparam & 0xffff) as u32;
            // The message's own flags (`IS_POINTER_CANCELED_WPARAM`) say a
            // cancellation of its pointer too.
            let canceled = (wparam >> 16) as u32 & POINTER_MESSAGE_FLAG_CANCELED != 0;
            let mut notes = frame(id);
            for note in &mut notes {
                note.canceled |= canceled && note.id == u64::from(id);
            }
            if let Ok(mut contacts) = input.contacts.try_borrow_mut() {
                *contacts = notes;
            }
        }
        // Mouse messages Windows made from a touch or pen contact, which
        // arrives as a contact: dropped, so it is not the mouse as well.
        WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP
        | WM_MBUTTONDOWN | WM_MBUTTONUP
            if from_contact() =>
        {
            return 0;
        }
        WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN
        | WM_MBUTTONUP | WM_XBUTTONDOWN | WM_XBUTTONUP => {
            let p = point(lparam);
            input.pointer.set(Some((f64::from(p.x), f64::from(p.y))));
        }
        WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
            // A wheel message carries a screen position.
            let mut p = point(lparam);
            // SAFETY: `hwnd` is this procedure's live window; `p` is a
            // valid POINT for the call.
            let mapped = unsafe { ScreenToClient(hwnd, &mut p) } != 0;
            input
                .pointer
                .set(mapped.then(|| (f64::from(p.x), f64::from(p.y))));
        }
        WM_TIMER if wparam == FILE_TIMER => {
            let point = cursor_at(hwnd, input.scale.get());
            if input.file_pointer.replace(point) != point
                && let Some(waker) = input.waker.borrow().as_ref()
            {
                waker.wake();
            }
        }
        WM_SETTINGCHANGE => {
            let mouse = mouse_settings();
            let mut changed = input.mouse.replace(mouse) != mouse;
            if let Some(reduce) = reduced_motion() {
                changed |= input.reduced_motion.replace(reduce) != reduce;
            }
            let locale = super::locale::system_locale();
            changed |= input.locale.replace(locale.clone()) != locale;
            if changed {
                input.env_changed.set(true);
                if let Ok(waker) = input.waker.try_borrow()
                    && let Some(waker) = waker.as_ref()
                {
                    waker.wake();
                }
            }
        }
        _ if cfg!(debug_assertions) && message == debug_dpi_message() => {
            debug_dpi_change(hwnd, wparam);
            return 0;
        }
        WM_NCDESTROY if input.subclassed.replace(false) => {
            // SAFETY: forwards the message down the chain, as every
            // subclass procedure must.
            let result = unsafe { DefSubclassProc(hwnd, message, wparam, lparam) };
            // `input` is not used after this.
            release(hwnd, data as *const NativeInput);
            return result;
        }
        _ => {}
    }
    // SAFETY: forwards the message down the chain, as every subclass
    // procedure must.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

/// The extra info of a mouse message Windows synthesized from a touch or
/// pen contact (`MI_WP_SIGNATURE`, under the mask `0xFFFFFF00`). Compared
/// as the low 32 bits: as an `isize` it would overflow on 32-bit targets.
const FROM_CONTACT: u32 = 0xff51_5700;

/// Whether the mouse message being handled was synthesized from a touch
/// or pen contact.
fn from_contact() -> bool {
    // SAFETY: a plain query of the current message's extra info, on the
    // thread handling it.
    let extra = unsafe { GetMessageExtraInfo() };
    extra as u32 & 0xffff_ff00 == FROM_CONTACT
}

/// What the platform says about every touch or pen pointer of the frame
/// of the pointer `id`'s message that winit does not report (only `id`'s
/// when the frame cannot be read); other kinds of pointer are left out.
fn frame(id: u32) -> Vec<ContactNote> {
    // SAFETY: `POINTER_INFO` is plain data; all zeros is a valid value.
    let empty: POINTER_INFO = unsafe { std::mem::zeroed() };
    let mut count = 0u32;
    // SAFETY: with a count of zero the call writes no buffer (it may be
    // null) and only reports how many pointers the frame has.
    let counted = unsafe { GetPointerFrameInfo(id, &mut count, std::ptr::null_mut()) } != 0;
    if !counted || count == 0 {
        return contact(id).into_iter().collect();
    }
    let mut infos = vec![empty; count as usize];
    // SAFETY: `infos` holds `count` valid `POINTER_INFO`s, the number the
    // call is told it may fill.
    if unsafe { GetPointerFrameInfo(id, &mut count, infos.as_mut_ptr()) } == 0 {
        return contact(id).into_iter().collect();
    }
    // The call reports how many it filled (never more than it was given).
    infos.truncate(count as usize);
    infos.iter().filter_map(note).collect()
}

/// What the platform says about the touch or pen pointer `id` that winit
/// does not report; `None` for another kind of pointer or when unknown.
fn contact(id: u32) -> Option<ContactNote> {
    // SAFETY: `POINTER_INFO` is plain data; all zeros is a valid value.
    let mut info: POINTER_INFO = unsafe { std::mem::zeroed() };
    // SAFETY: `info` is a valid `POINTER_INFO` for the call to fill.
    if unsafe { GetPointerInfo(id, &mut info) } == 0 {
        return None;
    }
    note(&info)
}

/// The note of the pointer `info` describes; `None` for a mouse.
fn note(info: &POINTER_INFO) -> Option<ContactNote> {
    let id = info.pointerId;
    let kind = match info.pointerType {
        PT_TOUCH => PointerKind::Touch,
        PT_PEN => PointerKind::Pen,
        _ => return None,
    };
    let tilt = (kind == PointerKind::Pen).then(|| pen_tilt(id)).flatten();
    Some(ContactNote {
        id: u64::from(id),
        kind,
        canceled: info.pointerFlags & POINTER_FLAG_CANCELED != 0,
        tilt,
    })
}

/// The tilt of the pen `id` (degrees along x and y), when it reports one.
fn pen_tilt(id: u32) -> Option<(f32, f32)> {
    // SAFETY: `POINTER_PEN_INFO` is plain data; all zeros is a valid value.
    let mut pen: POINTER_PEN_INFO = unsafe { std::mem::zeroed() };
    // SAFETY: `pen` is a valid `POINTER_PEN_INFO` for the call to fill.
    if unsafe { GetPointerPenInfo(id, &mut pen) } == 0 {
        return None;
    }
    (pen.penMask & (PEN_MASK_TILT_X | PEN_MASK_TILT_Y) != 0)
        .then_some((pen.tiltX as f32, pen.tiltY as f32))
}

/// The signed client or screen position packed into a mouse message.
fn point(lparam: LPARAM) -> POINT {
    POINT {
        x: i32::from((lparam & 0xffff) as u16 as i16),
        y: i32::from(((lparam >> 16) & 0xffff) as u16 as i16),
    }
}

/// The answer to an IME request, or `None` to leave it to the next
/// procedure (no focused text input, or a request not handled here).
pub(crate) fn ime_request(
    hwnd: HWND,
    input: &NativeInput,
    request: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    // Not borrowed elsewhere while the window procedure runs; if it were,
    // the request falls to the next procedure rather than panic.
    let text = input.text.try_borrow().ok()?;
    let text = text.as_ref()?;
    let request = u32::try_from(request).ok()?;
    match request {
        IMR_RECONVERTSTRING => Some(if text.composition.is_some() {
            0
        } else {
            reconvert_string(
                lparam,
                &text.text.encode_utf16().collect::<Vec<u16>>(),
                text.selection,
            )
        }),
        IMR_DOCUMENTFEED => {
            let caret = (text.caret, text.caret);
            Some(reconvert_string(
                lparam,
                &text.text.encode_utf16().collect::<Vec<u16>>(),
                text.composition.unwrap_or(caret),
            ))
        }
        IMR_CONFIRMRECONVERTSTRING => {
            let range = confirmed(lparam, text.text.encode_utf16().collect::<Vec<u16>>().len());
            input.reconvert.set(range.or(input.reconvert.get()));
            Some(LRESULT::from(range.is_some()))
        }
        IMR_QUERYCHARPOSITION => Some(LRESULT::from(char_position(
            hwnd,
            text,
            input.scale.get(),
            lparam,
        ))),
        _ => None,
    }
}

/// The IME's buffer at `lparam` as bytes: as long as the size it wrote
/// into its first field, which must cover `needed` bytes.
fn buffer<'a>(lparam: LPARAM, needed: usize) -> Option<&'a mut [u8]> {
    if lparam == 0 {
        return None;
    }
    let ptr = lparam as *mut u8;
    // SAFETY: a non-null `lparam` of these requests points to a structure
    // whose first field is its size in bytes (`dwSize`), written by the
    // IME; it may be unaligned.
    let size = unsafe { ptr.cast::<u32>().read_unaligned() } as usize;
    if size < needed {
        return None;
    }
    // SAFETY: the IME owns `size` bytes at `ptr` for the duration of the
    // request and nothing else touches them meanwhile.
    Some(unsafe { std::slice::from_raw_parts_mut(ptr, size) })
}

fn put(buffer: &mut [u8], at: usize, value: u32) {
    if let Some(field) = buffer.get_mut(at..at + 4) {
        field.copy_from_slice(&value.to_le_bytes());
    }
}

fn get(buffer: &[u8], at: usize) -> u32 {
    buffer
        .get(at..at + 4)
        .and_then(|field| field.try_into().ok())
        .map_or(0, u32::from_le_bytes)
}

fn u32_of(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// Answer a `RECONVERTSTRING` request: without a buffer the size it
/// needs, else the buffer filled with `units` and `range` (UTF-16) as the
/// composition and target; 0 when the buffer is too small.
fn reconvert_string(lparam: LPARAM, units: &[u16], range: (usize, usize)) -> LRESULT {
    // The string and a terminating NUL follow the header.
    let size = HEADER + 2 * (units.len() + 1);
    if lparam == 0 {
        return LRESULT::try_from(size).unwrap_or(0);
    }
    let Some(buffer) = buffer(lparam, size) else {
        return 0;
    };
    let (start, end) = (range.0.min(units.len()), range.1.min(units.len()));
    let (start, end) = (start.min(end), start.max(end));
    put(buffer, 0, u32_of(size)); // dwSize
    put(buffer, 4, 0); // dwVersion
    put(buffer, 8, u32_of(units.len())); // dwStrLen
    put(buffer, 12, u32_of(HEADER)); // dwStrOffset
    for at in [16, 24] {
        put(buffer, at, u32_of(end - start)); // dwCompStrLen, dwTargetStrLen
        put(buffer, at + 4, u32_of(2 * start)); // byte offsets in the string
    }
    let string = &mut buffer[HEADER..size];
    for (unit, out) in units.iter().chain([&0]).zip(string.as_chunks_mut::<2>().0) {
        *out = unit.to_le_bytes();
    }
    LRESULT::try_from(size).unwrap_or(0)
}

/// The range (UTF-16 in a text of `len` units) of a confirmed
/// `RECONVERTSTRING`, if it lies in the text.
fn confirmed(lparam: LPARAM, len: usize) -> Option<(usize, usize)> {
    let buffer = buffer(lparam, HEADER)?;
    let start = get(buffer, 20) as usize / 2; // dwCompStrOffset (bytes)
    let end = start.checked_add(get(buffer, 16) as usize)?; // dwCompStrLen
    (end <= len).then_some((start, end))
}

/// Answer an `IMECHARPOSITION` request: the screen position and line
/// height of the character `dwCharPos` past the composition's start (the
/// caret without one: the boxes the host gives start there), and the text
/// area. Whether it was answered.
fn char_position(hwnd: HWND, text: &ShellImeText, scale: f64, lparam: LPARAM) -> bool {
    let Some(buffer) = buffer(lparam, std::mem::size_of::<IMECHARPOSITION>()) else {
        return false;
    };
    let base = text.composition.map_or(text.caret, |c| c.0);
    let Some(rect) = (get(buffer, 4) as usize).checked_add(base).and_then(|at| {
        at.checked_sub(text.rects_from)
            .and_then(|i| text.rects.get(i))
            .copied()
    }) else {
        return false;
    };
    let mut origin = POINT { x: 0, y: 0 };
    // SAFETY: `hwnd` is this procedure's live window; `origin` is a valid
    // POINT for the call.
    if unsafe { ClientToScreen(hwnd, &mut origin) } == 0 {
        return false;
    }
    // Saturating: the window procedure must not panic (debug overflow
    // checks), and a casting `as` saturates too.
    let px = |v: f32| (f64::from(v) * scale).round() as i32;
    let at = |x: f32, y: f32| {
        (
            origin.x.saturating_add(px(x)),
            origin.y.saturating_add(px(y)),
        )
    };
    let (x, y) = at(rect.x, rect.y);
    put(buffer, 8, x as u32); // pt.x
    put(buffer, 12, y as u32); // pt.y
    put(buffer, 16, px(rect.height).max(1) as u32); // cLineHeight
    let area = text.area;
    let (left, top) = at(area.x, area.y);
    let (right, bottom) = at(area.x + area.width, area.y + area.height);
    for (offset, value) in [(20, left), (24, top), (28, right), (32, bottom)] {
        put(buffer, offset, value as u32); // rcDocument
    }
    true
}

/// The id of the registered message `ArvaneDebugDpiChange` (`u32::MAX`,
/// which no message has, when it cannot be registered).
fn debug_dpi_message() -> u32 {
    static ID: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *ID.get_or_init(|| {
        let name: Vec<u16> = "ArvaneDebugDpiChange".encode_utf16().chain([0]).collect();
        // SAFETY: `name` is a NUL-terminated UTF-16 string that outlives
        // the call.
        let id = unsafe { RegisterWindowMessageW(name.as_ptr()) };
        if id == 0 { u32::MAX } else { id }
    })
}

/// Send `hwnd` the `WM_DPICHANGED` of a move to a monitor of `dpi`
/// (debug builds; see the module doc). winit sizes the window for the new
/// scale itself; the suggested rectangle is the window's own.
fn debug_dpi_change(hwnd: HWND, dpi: WPARAM) {
    let dpi = (dpi & 0xffff).max(1);
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    // SAFETY: `hwnd` is this procedure's live window; `rect` is a valid
    // RECT for the call.
    if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
        return;
    }
    // SAFETY: a synchronous message to this thread's own window; `rect`
    // outlives the call, which reads it as the suggested rectangle.
    unsafe {
        SendMessageW(
            hwnd,
            WM_DPICHANGED,
            dpi << 16 | dpi,
            &rect as *const RECT as LPARAM,
        )
    };
}

/// Whether Windows' "animation effects" setting is off (the
/// reduce-motion preference); `None` when it cannot be read.
pub(crate) fn reduced_motion() -> Option<bool> {
    let mut animate: i32 = 1;
    // SAFETY: `SPI_GETCLIENTAREAANIMATION` writes a BOOL to `pvparam`,
    // which points to `animate` for the duration of the call.
    let read = unsafe {
        SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, (&raw mut animate).cast(), 0)
    };
    (read != 0).then_some(animate == 0)
}

/// A `u32` system parameter (`SystemParametersInfoW`), if readable.
fn parameter(action: u32) -> Option<u32> {
    let mut value: u32 = 0;
    // SAFETY: the `SPI_GET*` actions read here write a `u32` to `pvparam`,
    // which points to `value` for the duration of the call.
    let read = unsafe { SystemParametersInfoW(action, 0, (&raw mut value).cast(), 0) };
    (read != 0).then_some(value)
}

/// Windows' mouse settings (see `MouseSettings`; a value that cannot be
/// read keeps its default).
pub(crate) fn mouse_settings() -> MouseSettings {
    let defaults = MouseSettings::default();
    // SAFETY: a plain query without arguments.
    let double_click_ms = unsafe { GetDoubleClickTime() };
    let metric = |index| {
        // SAFETY: a plain query of a system metric.
        let value = unsafe { GetSystemMetrics(index) };
        u32::try_from(value).ok().filter(|v| *v > 0)
    };
    MouseSettings {
        double_click_ms: if double_click_ms > 0 {
            double_click_ms
        } else {
            defaults.double_click_ms
        },
        double_click_px: (
            metric(SM_CXDOUBLECLK).unwrap_or(defaults.double_click_px.0),
            metric(SM_CYDOUBLECLK).unwrap_or(defaults.double_click_px.1),
        ),
        wheel_lines: parameter(SPI_GETWHEELSCROLLLINES).unwrap_or(defaults.wheel_lines),
        wheel_chars: parameter(SPI_GETWHEELSCROLLCHARS).unwrap_or(defaults.wheel_chars),
    }
}

const FILE_TIMER: usize = 0x41525646;
/// Query the cursor in logical client pixels, on the window thread.
pub(crate) fn file_position(window: &Window) -> Option<kivact_types::Point> {
    let RawWindowHandle::Win32(handle) = window.window_handle().ok()?.as_raw() else {
        return None;
    };
    cursor_at(handle.hwnd.get(), window.scale_factor())
}
fn cursor_at(hwnd: HWND, scale: f64) -> Option<kivact_types::Point> {
    let mut point = POINT { x: 0, y: 0 };
    // SAFETY: point is a writable POINT for this synchronous system query.
    if unsafe { GetCursorPos(&mut point) } == 0 {
        return None;
    }
    // SAFETY: hwnd is the live window on this thread and point stays writable.
    if unsafe { ScreenToClient(hwnd, &mut point) } == 0 {
        return None;
    }
    Some(kivact_types::Point {
        x: (f64::from(point.x) / scale) as f32,
        y: (f64::from(point.y) / scale) as f32,
    })
}
/// A window timer keeps tracking through Windows' modal OLE drag loop.
pub(crate) fn track_files(window: &Window, tracking: bool) {
    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let hwnd = handle.hwnd.get();
    if tracking {
        // SAFETY: hwnd is live on this thread; no callback or pointer is retained.
        unsafe { SetTimer(hwnd, FILE_TIMER, 16, None) };
    } else {
        // SAFETY: removes only this module's timer from the live window.
        unsafe { KillTimer(hwnd, FILE_TIMER) };
    }
}
