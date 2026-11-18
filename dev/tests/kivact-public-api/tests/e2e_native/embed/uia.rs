//! Role: Embedded Windows UIA provider contract.
#![allow(
    unsafe_code,
    reason = "UI Automation's COM client API is unsafe; confined to this real-window test"
)]
use super::support::{SERIAL, wait};
use kivact::prelude::*;
use kivact::test::TestParent;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, TreeScope_Descendants, UIA_ButtonControlTypeId,
    UIA_NamePropertyId,
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
        let condition = unsafe { automation.CreatePropertyCondition(UIA_NamePropertyId, &name) }?;
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
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }

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
    assert_eq!(app.status(), kivact::embed::EmbedStatus::Running);
}
