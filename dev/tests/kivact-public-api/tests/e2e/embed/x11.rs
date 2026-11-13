//! Role: X11-specific embedding services and native protocol checks.
use super::support::{SERIAL, wait};
use kivact::platform;
use kivact::prelude::*;
use kivact::test::TestParent;
use std::sync::{Arc, Mutex};
/// XIM is optional on X11; native keyboard input must remain usable.
// This does not force either case: it runs with whatever XMODIFIERS the environment has.
#[test]
fn x11_keyboard_input_works_whether_or_not_an_xim_server_runs() {
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }

    let _serial = SERIAL
        .lock()
        .expect("embed tests run one at a time; an earlier one panicked holding SERIAL");
    let parent = TestParent::open(320, 240).unwrap();
    let keys = Arc::new(Mutex::new(0));
    let output = keys.clone();
    let app = App::new("X11 optional XIM")
        .window_size(240, 160)
        .embed(parent.parent().scale(1.0), move || {
            assert!(platform::capabilities().get().native_accessibility);
            assert!(!platform::capabilities().get().ime_reconversion);
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
    assert_eq!(app.status(), kivact::embed::EmbedStatus::Running);
}
