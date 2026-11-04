//! Role: Win32 mouse normalization, keyboard text and synchronous cursor answers.
use super::state::State;
use crate::key_code;
use arvane_shell::ShellEvent;
use kivact_types::{ButtonPress, Cursor, Key, Modifiers, MouseButton, Point, PointerPhase};
use std::rc::Rc;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
use windows_sys::Win32::System::SystemServices::{
    MK_LBUTTON, MK_MBUTTON, MK_RBUTTON, MK_XBUTTON1, MK_XBUTTON2,
};
use windows_sys::Win32::UI::Controls::WM_MOUSELEAVE;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetCapture, GetKeyState, GetKeyboardLayout, GetKeyboardState, ReleaseCapture, SetCapture,
    SetFocus, TME_LEAVE, TRACKMOUSEEVENT, ToUnicodeEx, TrackMouseEvent, VK_APPS, VK_BACK,
    VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_F24, VK_HOME, VK_LCONTROL,
    VK_LEFT, VK_LMENU, VK_LWIN, VK_MENU, VK_NEXT, VK_PRIOR, VK_RCONTROL, VK_RETURN, VK_RIGHT,
    VK_RMENU, VK_RWIN, VK_SHIFT, VK_SPACE, VK_TAB, VK_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_IBEAM, IDC_NO, IDC_SIZEALL, IDC_SIZENS, IDC_SIZEWE,
    LoadCursorW, PM_REMOVE, PeekMessageW, SetCursor, WM_CAPTURECHANGED, WM_CHAR, WM_DEADCHAR,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP, WM_XBUTTONDOWN, WM_XBUTTONUP, XBUTTON1,
};

pub(super) fn cursor(cursor: Cursor) {
    let id = match cursor {
        Cursor::None => std::ptr::null(),
        Cursor::Text => IDC_IBEAM,
        Cursor::Pointer => IDC_HAND,
        Cursor::ColResize | Cursor::EwResize => IDC_SIZEWE,
        Cursor::RowResize | Cursor::NsResize => IDC_SIZENS,
        Cursor::Crosshair => IDC_CROSS,
        Cursor::Move => IDC_SIZEALL,
        Cursor::NotAllowed => IDC_NO,
        Cursor::Grab | Cursor::Grabbing => IDC_HAND,
        _ => IDC_ARROW,
    };
    let handle = if id.is_null() {
        0
    } else {
        // SAFETY: stock cursor identifiers are borrowed system resources.
        unsafe { LoadCursorW(0, id) }
    };
    // SAFETY: SetCursor borrows the stock cursor; null intentionally hides it.
    unsafe {
        SetCursor(handle);
    }
}
pub(super) fn cancel(state: &Rc<State>) {
    let mut mouse = state.mouse.get();
    let event = mouse.cancel();
    state.mouse.set(mouse);
    if let Some(event) = event {
        state.event(ShellEvent::Pointer(event));
    }
}
fn position(state: &State, l: LPARAM) -> Point {
    let scale = state.shared.scale();
    Point {
        x: (l as u16 as i16) as f64 / scale,
        y: ((l as u32 >> 16) as u16 as i16) as f64 / scale,
    }
}
fn pointer(state: &Rc<State>, phase: PointerPhase, l: LPARAM) {
    let mut mouse = state.mouse.get();
    let input = mouse.event(phase, Some(position(state, l)));
    state.mouse.set(mouse);
    let Some(input) = input else {
        return;
    };
    let mut event = ShellEvent::Pointer(input);
    let settings = crate::native_input::pointer_settings(state.shared.scale());
    state.clicks.borrow_mut().count(
        &mut event,
        || state.now(),
        || {
            (
                settings.double_click_ns,
                (
                    settings.double_click_distance.0,
                    settings.double_click_distance.1,
                ),
            )
        },
    );
    state.event(event);
}
fn held(key: u16) -> bool {
    // SAFETY: GetKeyState reads this UI thread's keyboard state and retains nothing.
    unsafe { GetKeyState(i32::from(key)) < 0 }
}
pub(super) fn modifiers() -> Modifiers {
    Modifiers {
        shift: held(VK_SHIFT),
        ctrl: held(VK_CONTROL),
        alt: held(VK_MENU),
        meta: held(VK_LWIN) || held(VK_RWIN),
    }
}
pub(super) fn message(state: &Rc<State>, message: u32, w: WPARAM, l: LPARAM) -> Option<LRESULT> {
    let hwnd = state.native.hwnd();
    match message {
        WM_MOUSEMOVE => {
            // SAFETY: TrackMouseEvent receives an initialized, size-tagged request.
            unsafe {
                let mut request = TRACKMOUSEEVENT {
                    cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                TrackMouseEvent(&mut request);
            }
            pointer(state, PointerPhase::Move, l);
            Some(0)
        }
        WM_MOUSELEAVE => {
            let mut mouse = state.mouse.get();
            let event = mouse.event(PointerPhase::Leave, None);
            state.mouse.set(mouse);
            if let Some(event) = event {
                state.event(ShellEvent::Pointer(event));
            }
            Some(0)
        }
        // lParam names the window gaining capture: ours means we re-captured
        // ourselves (a second button went down) and the gesture continues.
        WM_CAPTURECHANGED => {
            if l != hwnd {
                cancel(state);
            }
            Some(0)
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN | WM_LBUTTONUP
        | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP => {
            let down = matches!(
                message,
                WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN
            );
            let button = match message {
                WM_LBUTTONDOWN | WM_LBUTTONUP => MouseButton::Primary,
                WM_RBUTTONDOWN | WM_RBUTTONUP => MouseButton::Secondary,
                WM_MBUTTONDOWN | WM_MBUTTONUP => MouseButton::Middle,
                _ => {
                    if w >> 16 == XBUTTON1 as usize {
                        MouseButton::Back
                    } else {
                        MouseButton::Forward
                    }
                }
            };
            // SAFETY: focus/capture belong to this live HWND on the dispatching thread.
            if down {
                // SAFETY: focus belongs to this live child on its GUI thread.
                unsafe {
                    SetFocus(hwnd);
                }
                // SAFETY: GetCapture reads this thread's capture window only.
                if unsafe { GetCapture() } != hwnd {
                    // SAFETY: capture is requested only on the child dispatch thread.
                    unsafe {
                        SetCapture(hwnd);
                    }
                }
            }
            let press = ButtonPress { button, clicks: 1 };
            pointer(
                state,
                if down {
                    PointerPhase::Down(press)
                } else {
                    PointerPhase::Up(press)
                },
                l,
            );
            if !down
                && w & (MK_LBUTTON | MK_RBUTTON | MK_MBUTTON | MK_XBUTTON1 | MK_XBUTTON2) as usize
                    == 0
            {
                // SAFETY: ReleaseCapture releases only this UI thread's current capture.
                unsafe {
                    ReleaseCapture();
                }
            }
            Some(if matches!(message, WM_XBUTTONDOWN | WM_XBUTTONUP) {
                1
            } else {
                0
            })
        }
        WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
            let mut point = POINT {
                x: l as u16 as i16 as i32,
                y: (l as u32 >> 16) as u16 as i16 as i32,
            };
            // SAFETY: wheel positions are screen coordinates; convert into this child.
            unsafe {
                ScreenToClient(hwnd, &mut point);
            }
            let mut mouse = state.mouse.get();
            let applied = state.shared.applied();
            let scale = applied.scale;
            let previous = mouse.position();
            let event = mouse.event(
                PointerPhase::Move,
                Some(Point {
                    x: point.x as f64 / scale,
                    y: point.y as f64 / scale,
                }),
            );
            state.mouse.set(mouse);
            if let Some(event) = event
                && Some(event.position) != previous
            {
                state.event(ShellEvent::Pointer(event));
            }
            let notch = (w >> 16) as u16 as i16 as f64 / 120.0;
            let (x, y) = if message == WM_MOUSEHWHEEL {
                (-notch, 0.0)
            } else {
                (0.0, notch)
            };
            let settings = crate::native_input::pointer_settings(state.shared.scale());
            let height = applied.height as f64 / scale;
            let (x, y) = crate::wheel::lines(
                x,
                y,
                modifiers().shift,
                height,
                settings.wheel_lines,
                settings.wheel_chars,
            );
            if let Some(position) = state.mouse.get().position() {
                state.event(ShellEvent::Wheel(kivact_types::WheelInput {
                    delta: kivact_types::Point { x, y },
                    position,
                    modifiers: self::modifiers(),
                }));
            }
            Some(0)
        }
        WM_KEYDOWN | WM_KEYUP => {
            keyboard(state, message, w, l);
            Some(0)
        }
        // System keys are reported, then left to the system procedure so the host
        // still gets SC_CLOSE (Alt+F4) and SC_KEYMENU from DefWindowProc.
        WM_SYSKEYDOWN | WM_SYSKEYUP => {
            keyboard(state, message, w, l);
            None
        }
        // Text is consumed together with its key when TranslateMessage supplied it.
        // A separate WM_CHAR (for example test text injection) remains a commit.
        WM_CHAR => {
            let unit = w as u16;
            let high = (0xd800..=0xdbff).contains(&unit);
            let text = match (state.pending_surrogate.take(), high) {
                (None, true) => {
                    state.pending_surrogate.set(Some(unit));
                    None
                }
                // The earlier high surrogate had no low one: it is reported, not lost.
                (Some(_), true) => {
                    state.pending_surrogate.set(Some(unit));
                    Some("\u{fffd}".to_owned())
                }
                (Some(previous), false) => Some(String::from_utf16_lossy(&[previous, unit])),
                (None, false) => Some(String::from_utf16_lossy(&[unit])),
            };
            if let Some(text) = text
                && !text.chars().all(char::is_control)
            {
                state.event(ShellEvent::Ime(kivact_types::ImeInput::Commit(text)));
            }
            Some(0)
        }
        _ => None,
    }
}
fn keyboard(state: &Rc<State>, message: u32, w: WPARAM, l: LPARAM) {
    use winit::keyboard::PhysicalKey;
    use winit::platform::scancode::PhysicalKeyExtScancode;
    let scan = ((l as u32 >> 16) & 0xff) | if l & (1 << 24) != 0 { 0xe000 } else { 0 };
    let code = key_code::physical_code(PhysicalKey::from_scancode(scan));
    let modifiers = modifiers();
    state.event(ShellEvent::Modifiers(modifiers));
    let mut keyboard = [0u8; 256];
    let mut text = [0u16; 16];
    // SAFETY: keyboard is a writable 256-byte keyboard-state output array.
    unsafe {
        GetKeyboardState(keyboard.as_mut_ptr());
    }
    for key in [
        VK_CONTROL,
        VK_LCONTROL,
        VK_RCONTROL,
        VK_MENU,
        VK_LMENU,
        VK_RMENU,
    ] {
        keyboard[key as usize] = 0;
    }
    // SAFETY: the current UI-thread keyboard layout is queried without retaining it.
    let layout = unsafe { GetKeyboardLayout(0) };
    // SAFETY: the state and text arrays have documented sizes. Flag 4 preserves
    // host-owned dead-key state, while cleared Ctrl/Alt gives the logical letter.
    let count = unsafe {
        ToUnicodeEx(
            w as u32,
            scan,
            keyboard.as_ptr(),
            text.as_mut_ptr(),
            text.len() as i32,
            4,
            layout,
        )
    };
    let fallback = if count > 0 {
        String::from_utf16_lossy(&text[..(count as usize).min(text.len())])
    } else {
        String::new()
    };
    let key = match w as u16 {
        VK_RETURN => Key::Enter,
        VK_TAB => Key::Tab,
        VK_SPACE => Key::Space,
        VK_BACK => Key::Backspace,
        VK_DELETE => Key::Delete,
        VK_ESCAPE => Key::Escape,
        VK_UP => Key::ArrowUp,
        VK_DOWN => Key::ArrowDown,
        VK_LEFT => Key::ArrowLeft,
        VK_RIGHT => Key::ArrowRight,
        VK_HOME => Key::Home,
        VK_END => Key::End,
        VK_PRIOR => Key::PageUp,
        VK_NEXT => Key::PageDown,
        VK_APPS => Key::ContextMenu,
        v if (VK_F1..=VK_F24).contains(&v) => Key::F((v - VK_F1 + 1) as u8),
        _ => fallback.chars().next().map_or(Key::Unidentified, Key::Char),
    };
    if matches!(message, WM_KEYUP | WM_SYSKEYUP) {
        state.event(ShellEvent::Key(kivact_types::KeyInput {
            key,
            code,
            phase: kivact_types::KeyPhase::Up,
            modifiers: self::modifiers(),
            pointer: state.mouse.get().window_position(),
        }));
        return;
    }
    let mut units = Vec::new();
    // SAFETY: zeroed MSG is valid writable message output storage.
    let mut msg = unsafe { std::mem::zeroed() };
    loop {
        // SAFETY: the filter removes only this child's character messages, never
        // host window messages. PeekMessageW still dispatches incoming sent messages
        // for this thread (UIA WM_GETOBJECT, cross-thread sends from the host), which
        // may re-enter the window procedure; no borrows are held across this loop.
        let found = unsafe {
            PeekMessageW(
                &mut msg,
                state.native.hwnd(),
                WM_CHAR,
                WM_DEADCHAR,
                PM_REMOVE,
            )
        };
        if found == 0 {
            break;
        }
        if msg.message == WM_CHAR {
            units.push(msg.wParam as u16);
        }
    }
    let text = if !units.is_empty() {
        String::from_utf16_lossy(&units)
    } else if !modifiers.ctrl && !modifiers.alt {
        fallback
    } else {
        String::new()
    };
    state.event(ShellEvent::Key(kivact_types::KeyInput {
        key,
        code,
        phase: kivact_types::KeyPhase::Down {
            repeat: l & (1 << 30) != 0,
            text: (!text.is_empty()).then_some(text),
        },
        modifiers: self::modifiers(),
        pointer: state.mouse.get().window_position(),
    }));
}
