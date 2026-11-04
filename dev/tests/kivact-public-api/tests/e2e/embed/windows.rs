//! Role: Win32 embedding input, clipping and UIA regressions.
#![allow(
    unsafe_code,
    reason = "queries and input target only owned Win32 test windows"
)]
use super::support::{SERIAL, wait};
use kivact::prelude::*;
use kivact::test::TestParent;
use kivact::view::MouseButton;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_F4;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, GW_CHILD, GetWindow, HTCLIENT, PostMessageW, SendMessageW, WM_NCHITTEST,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};
/// The system STATIC subclass must answer Win32 native hit testing.
#[test]
fn embedded_surface_receives_native_mouse_hit_tests() {
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }

    let _serial = SERIAL.lock().unwrap();
    let parent = TestParent::open(200, 120).unwrap();
    let app = App::new("Native hit target")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), || text("Native hit target"))
        .unwrap();
    let parent_hwnd = parent.win32_hwnd().unwrap() as isize;
    // SAFETY: this parent owns exactly one embedded child during this test.
    let child = unsafe { GetWindow(parent_hwnd, GW_CHILD) };
    assert_ne!(child, 0);
    let mut point = windows_sys::Win32::Foundation::POINT { x: 10, y: 10 };
    // SAFETY: the child is live and point is valid writable storage.
    let converted = unsafe { windows_sys::Win32::Graphics::Gdi::ClientToScreen(child, &mut point) };
    assert_ne!(converted, 0);
    let at = ((point.y as u16 as u32) << 16 | point.x as u16 as u32) as isize;
    // SAFETY: this synchronous query targets our live child on its UI thread.
    let hit = unsafe { SendMessageW(child, WM_NCHITTEST, 0, at) };
    assert_eq!(
        hit, HTCLIENT as isize,
        "the embedded surface must receive native mouse input"
    );
    drop(app);
}

/// Capture and system-key fallthrough are Win32 message contracts.
#[test]
fn chord_keeps_drag_and_alt_f4_reaches_host() {
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }

    let _serial = SERIAL.lock().expect("Windows screen regression lock");
    let parent = TestParent::open(320, 240).unwrap();
    let phases = Arc::new(Mutex::new(Vec::new()));
    let output = phases.clone();
    let app = App::new("Chord capture")
        .window_size(320, 100)
        .embed(parent.parent().scale(1.0), move || {
            let value = signal(0.5);
            slider("Gain", value)
                .range(0.0..=1.0)
                .on_edit(move |phase| output.lock().unwrap().push(phase))
                .style(style().width(300).height(40))
        })
        .unwrap();
    wait(&parent, || parent.capture().is_ok());
    parent.press(150.0, 20.0).unwrap();
    parent.move_to(200.0, 20.0).unwrap();
    parent
        .press_button(MouseButton::Secondary, 200.0, 20.0)
        .unwrap();
    parent.pump(Duration::from_millis(50));
    assert_eq!(
        *phases.lock().unwrap(),
        [kivact::interaction::EditPhase::Begin]
    );
    parent
        .release_button(MouseButton::Secondary, 200.0, 20.0)
        .unwrap();
    parent.move_to(250.0, 20.0).unwrap();
    parent.release(250.0, 20.0).unwrap();
    wait(&parent, || {
        phases.lock().unwrap().last() == Some(&kivact::interaction::EditPhase::Commit)
    });
    assert!(
        !phases
            .lock()
            .unwrap()
            .contains(&kivact::interaction::EditPhase::Cancel)
    );
    let host = parent.win32_hwnd().unwrap() as HWND;
    // SAFETY: search only the direct children of this test-owned host.
    let child = unsafe { FindWindowExW(host, 0, std::ptr::null(), std::ptr::null()) };
    assert_ne!(child, 0);
    let before = parent.close_requests();
    assert_ne!(
        // SAFETY: queue Alt+F4's system-key message only to this owned editor child.
        unsafe { PostMessageW(child, WM_SYSKEYDOWN, VK_F4 as usize, (1 << 29) | 1) },
        0
    );
    assert_ne!(
        // SAFETY: queue the matching system-key release to the same owned child.
        unsafe {
            PostMessageW(
                child,
                WM_SYSKEYUP,
                VK_F4 as usize,
                (1 << 31) | (1 << 30) | (1 << 29) | 1,
            )
        },
        0
    );
    wait(&parent, || parent.close_requests() == before + 1);
    assert!(!parent.is_closed());
    assert!(app.error().is_none());
}
