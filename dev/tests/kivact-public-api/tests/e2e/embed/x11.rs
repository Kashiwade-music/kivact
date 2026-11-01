//! Role: X11-specific embedding services and native protocol checks.
use super::support::{SERIAL, wait};
use kivact::{platform, prelude::*, test::TestParent};
use std::sync::{Arc, Mutex};
/// XIM is optional on X11; native keyboard input must remain usable.
// This does not force either case: it runs with whatever XMODIFIERS the environment has.
#[test]
fn x11_keyboard_input_works_whether_or_not_an_xim_server_runs() {
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    let parent = TestParent::open(320, 240).unwrap();
    let keys = Arc::new(Mutex::new(0));
    let output = keys.clone();
    let app = App::new("X11 optional XIM")
        .window_size(240, 160)
        .embed(parent.parent().scale(1.0), move || {
            assert!(platform::capabilities().native_accessibility);
            assert!(!platform::capabilities().ime_reconversion);
            node()
                .focusable(true)
                .style(style().size(200, 120))
                .on_key(move |_| *output.lock().unwrap() += 1)
                .child(text("Keyboard without XIM"))
        })
        .unwrap();
    wait(&parent, || parent.capture().is_ok());
    parent.click(20.0, 20.0).unwrap();
    parent.text("abc").unwrap();
    parent.key(kivact::view::KeyCode::Tab).unwrap();
    wait(&parent, || *keys.lock().unwrap() >= 3);
    assert!(app.error().is_none());
}
/// XDND is the X11 file-transfer protocol used by the embedded receiver.
#[test]
fn x11_xdnd_delivers_the_file_path() {
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    let parent = TestParent::open(320, 240).unwrap();
    let received = Arc::new(Mutex::new(Vec::new()));
    let output = received.clone();
    let app = App::new("XDND receiver")
        .window_size(240, 160)
        .embed(parent.parent(), move || {
            node()
                .style(style().size(240, 160))
                .on_file(move |event: kivact::view::FileEvent| {
                    if event.phase == kivact::view::FilePhase::Drop {
                        *output.lock().unwrap() = event.files;
                    }
                })
        })
        .unwrap();
    wait(&parent, || parent.capture().is_ok());
    let files = vec![std::path::PathBuf::from("/tmp/embedded 日本語 #%.txt")];
    parent.drop_files(&files, 20.0, 20.0).unwrap();
    wait(&parent, || !received.lock().unwrap().is_empty());
    assert_eq!(*received.lock().unwrap(), files);
    assert!(app.error().is_none());
}
/// A real AT-SPI bus and client are needed to query the Linux provider.
#[test]
#[ignore = "Requires running AT-SPI bus and python3-pyatspi; see manual-verification.md Linux embedding"]
fn x11_atspi_client_reads_the_embedded_button() {
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    let parent = TestParent::open(320, 240).unwrap();
    let app = App::new("Embedded AT-SPI")
        .window_size(240, 160)
        .embed(parent.parent(), || {
            node()
                .role(Role::Button)
                .name("Embedded AT-SPI action")
                .focusable(true)
                .style(style().size(160, 80))
                .child(text("Action"))
        })
        .unwrap();
    wait(&parent, || parent.capture().is_ok());
    let mut client = std::process::Command::new("python3")
        .arg("-c")
        .arg(
            r#"
import pyatspi, time
end = time.monotonic() + 8
def find(node):
    try:
        if node.name == 'Embedded AT-SPI action' and node.getRole() == pyatspi.ROLE_PUSH_BUTTON:
            return True
        return any(find(child) for child in node)
    except Exception:
        return False
while time.monotonic() < end:
    if find(pyatspi.Registry.getDesktop(0)):
        raise SystemExit(0)
    time.sleep(.05)
raise SystemExit('named embedded button absent from AT-SPI')
"#,
        )
        .spawn()
        .unwrap();
    let mut status = None;
    wait(&parent, || {
        status = client.try_wait().unwrap();
        status.is_some()
    });
    assert!(status.unwrap().success());
    assert!(app.error().is_none());
}
