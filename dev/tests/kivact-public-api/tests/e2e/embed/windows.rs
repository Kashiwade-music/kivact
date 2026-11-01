//! Role: Win32 embedding input, clipping and UIA regressions.
#![allow(
    unsafe_code,
    reason = "queries and input target only owned Win32 test windows"
)]
use super::support::{SERIAL, wait};
use kivact::{prelude::*, style::Color, test::TestParent, view::MouseButton};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::*,
    UI::{Input::KeyboardAndMouse::VK_F4, WindowsAndMessaging::*},
};
/// The system STATIC subclass must answer Win32 native hit testing.
#[test]
fn embedded_surface_receives_native_mouse_hit_tests() {
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
/// Win32 child creation requires the parent to belong to the calling GUI thread.
#[test]
fn a_parent_owned_by_another_thread_is_refused() {
    let _serial = SERIAL.lock().unwrap();
    let (hwnd_tx, hwnd_rx) = std::sync::mpsc::channel();
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    let host = std::thread::spawn(move || {
        let parent = TestParent::open(200, 120).unwrap();
        hwnd_tx.send(parent.win32_hwnd().unwrap() as usize).unwrap();
        while done_rx.try_recv().is_err() {
            parent.pump(Duration::from_millis(5));
        }
    });
    let hwnd = hwnd_rx.recv().unwrap();
    let refused = App::new("Foreign parent")
        .window_size(160, 100)
        .embed(
            kivact::embed::Parent::win32(hwnd as *mut std::ffi::c_void).scale(1.0),
            || text("Foreign parent"),
        )
        .err()
        .expect("a parent from another thread must be refused");
    assert!(
        refused.to_string().contains("calling GUI thread"),
        "unexpected error: {refused}"
    );
    done_tx.send(()).unwrap();
    host.join().unwrap();
}
mod uia {
    //! Role: Embedded Windows UIA provider contract.
    #![allow(
        unsafe_code,
        reason = "UI Automation's COM client API is unsafe; confined to this real-window test"
    )]
    use super::*;
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::{
        Foundation::HWND,
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
            CoUninitialize,
        },
        UI::Accessibility::{
            CUIAutomation, IUIAutomation, TreeScope_Descendants, UIA_ButtonControlTypeId,
            UIA_NamePropertyId,
        },
    };
    fn query(hwnd: isize) -> windows::core::Result<bool> {
        // SAFETY: this dedicated client thread initializes its own COM apartment.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.ok()?;
        let result = (|| {
            // SAFETY: COM is initialized and the requested interface matches the class.
            let automation: IUIAutomation =
                unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }?;
            // SAFETY: the test keeps the native parent alive until the client finishes.
            let root = unsafe { automation.ElementFromHandle(HWND(hwnd as *mut _)) }?;
            let name = VARIANT::from("Embedded action");
            // SAFETY: the property and its string variant have matching types.
            let condition =
                unsafe { automation.CreatePropertyCondition(UIA_NamePropertyId, &name) }?;
            // SAFETY: both interfaces belong to this client's apartment and are alive.
            let button = unsafe { root.FindFirst(TreeScope_Descendants, &condition) }?;
            // SAFETY: button is a live COM interface returned by UI Automation.
            let named = unsafe { button.CurrentName() }? == "Embedded action";
            // SAFETY: the queried element remains alive on this COM apartment.
            Ok(named && unsafe { button.CurrentControlType() }? == UIA_ButtonControlTypeId)
        })();
        // SAFETY: every COM interface above has been dropped on this apartment.
        unsafe { CoUninitialize() };
        result
    }
    /// A real Windows COM client must find the embedded native provider.
    #[test]
    fn native_uia_client_reads_the_embedded_button() {
        let _serial = SERIAL.lock().unwrap();
        let parent = TestParent::open(320, 240).unwrap();
        let app = App::new("Embedded accessibility")
            .window_size(240, 160)
            .embed(parent.parent(), || {
                node()
                    .role(Role::Button)
                    .name("Embedded action")
                    .focusable(true)
                    .style(style().size(160, 80))
                    .child(text("Action"))
            })
            .unwrap();
        wait(&parent, || parent.capture().is_ok());
        let hwnd = parent.win32_hwnd().unwrap() as isize;
        let (send, receive) = std::sync::mpsc::channel();
        let client = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(8);
            loop {
                let result = query(hwnd).map_err(|error| error.to_string());
                if matches!(result, Ok(true)) || Instant::now() >= deadline {
                    let _ = send.send(result);
                    break;
                }
                // The provider builds its first semantics tree after UIA activation.
                std::thread::sleep(Duration::from_millis(25));
            }
        });
        let mut result = None;
        wait(&parent, || match receive.try_recv() {
            Ok(value) => {
                result = Some(value);
                true
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => false,
            Err(error) => panic!("UIA client disconnected: {error}"),
        });
        client.join().unwrap();
        assert!(result.unwrap().unwrap());
        assert!(app.error().is_none());
    }
}

// A separate process selects the renderer before any App or test workers exist.
fn isolated(name: &str, gpu: bool) -> bool {
    if std::env::var("KIVACT_WINDOWS_REWORK_CASE").as_deref() == Ok(name) {
        return false;
    }
    let mut command = std::process::Command::new(
        std::env::current_exe().expect("locate the regression test executable"),
    );
    command
        .args(["--exact", &format!("embed::windows::{name}"), "--nocapture"])
        .env("KIVACT_WINDOWS_REWORK_CASE", name);
    if gpu {
        command.env_remove("ARVANE_RENDERER");
    } else {
        command.env("ARVANE_RENDERER", "cpu");
    }
    let log =
        std::env::temp_dir().join(format!("kivact-windows-{name}-{}.log", std::process::id()));
    let _ = std::fs::remove_file(&log);
    let status = command
        .env("KIVACT_FRAME_LOG", &log)
        .status()
        .expect("run the isolated renderer regression");
    assert!(
        status.success(),
        "Windows regression child failed; frame evidence: {}",
        log.display()
    );
    true
}
fn gpu_evidence(log: &str) -> (usize, bool) {
    let mut gpu = false;
    let mut frames = 0;
    for line in log.lines() {
        if line.contains("renderer=gpu adapter=") {
            gpu = true;
        }
        if line.starts_with("# renderer=cpu") {
            assert!(
                !gpu,
                "GPU clipping fell back to CPU after activation: {log}"
            );
        }
        if gpu && line.starts_with("F ") {
            frames += 1;
        }
    }
    (frames, gpu)
}
fn clipping(gpu: bool) {
    let parent = TestParent::open(320, 240).expect("open the clipping test host");
    let app = App::new("Sibling clipping")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), || {
            node().style(style().size(180, 100).background(Color::WHITE))
        })
        .expect("open the embedded clipping editor");
    let log = std::env::var_os("KIVACT_FRAME_LOG").expect("isolated clipping test has a frame log");
    let notes = || std::fs::read_to_string(&log).unwrap_or_default();
    wait(&parent, || {
        let renderer_ready = if gpu {
            let (frames, active) = gpu_evidence(&notes());
            active && frames > 0
        } else {
            true
        };
        parent
            .capture()
            .is_ok_and(|shot| shot.pixel(20, 20) == Some(0xffffff))
            && renderer_ready
    });
    parent
        .cover(40, 20, 60, 40, [23, 97, 181])
        .expect("create the higher sibling cover");
    parent.pump(Duration::from_millis(50));
    let before = if gpu {
        let (frames, active) = gpu_evidence(&notes());
        assert!(active, "GPU renderer disappeared before resize");
        frames
    } else {
        0
    };
    // Force a new present after placing the higher sibling, including GPU resize.
    app.set_size(161, 100);
    wait(&parent, || {
        let renderer_ready = if gpu {
            let (frames, active) = gpu_evidence(&notes());
            active && frames > before
        } else {
            true
        };
        app.size() == (161, 100) && renderer_ready
    });
    parent.pump(Duration::from_millis(100));
    let shot = parent
        .capture_host()
        .expect("capture the host with its overlapping sibling");
    if gpu {
        let (frames, active) = gpu_evidence(&notes());
        assert!(
            active && frames > before,
            "capture lacks a new actual GPU frame"
        );
    }
    assert_eq!(shot.pixel(20, 20), Some(0xffffff));
    assert_eq!(
        shot.pixel(60, 40),
        Some(0x1761b5),
        "present painted over the higher sibling"
    );
    assert!(app.error().is_none());
}
/// STATIC/GDI parent-DC clipping is specific to Windows.
#[test]
fn cpu_present_preserves_a_higher_sibling() {
    let _serial = SERIAL.lock().expect("Windows screen regression lock");
    if isolated("cpu_present_preserves_a_higher_sibling", false) {
        return;
    }
    clipping(false);
}
/// DXGI must actually present; CPU fallback cannot satisfy this regression.
#[cfg(feature = "gpu")]
#[test]
fn gpu_present_preserves_a_higher_sibling() {
    let _serial = SERIAL.lock().expect("Windows screen regression lock");
    if isolated("gpu_present_preserves_a_higher_sibling", true) {
        return;
    }
    clipping(true);
}
/// Capture and system-key fallthrough are Win32 message contracts.
#[test]
fn chord_keeps_drag_and_alt_f4_reaches_host() {
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
