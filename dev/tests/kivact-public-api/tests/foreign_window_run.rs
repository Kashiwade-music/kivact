//! Role: `App::run` with a foreign window on a real display: the child is a
//! live native window, a size request resolves, `on_destroy` sees it alive,
//! and `AppHandle::close` ends the App.

#![cfg(any(windows, target_os = "linux"))]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use kivact::App;
use kivact::ErrorKind;
use kivact::platform::NativeWindow;
use kivact::platform::app_handle;
use kivact::prelude::*;
use kivact::view::ForeignChild;
use kivact::view::foreign_window;

#[allow(
    clippy::print_stderr,
    reason = "report an unavailable display or unsupported platform explicitly when skipping the test"
)]
fn foreign_window_run() {
    let child = Rc::new(Cell::new(None::<NativeWindow>));
    let destroyed = Rc::new(Cell::new(0u32));
    let alive_in_destroy = Rc::new(Cell::new(None::<bool>));
    let requested = Rc::new(RefCell::new(None::<String>));
    let supported = Rc::new(Cell::new(false));

    let ready_child = child.clone();
    let ready_requested = requested.clone();
    let destroy_child = child.clone();
    let destroy_alive = alive_in_destroy.clone();
    let destroy_count = destroyed.clone();
    let supported_root = supported.clone();
    let result = App::new("Kivact foreign window")
        .window_size(420.0, 320.0)
        .run(move || {
            let can = kivact::platform::capabilities()
                .get_untracked()
                .foreign_window;
            supported_root.set(can);
            if !can {
                app_handle().close();
                return column((text("no foreign windows"), text("skipped")));
            }
            let open = signal(true);
            column((
                text("host"),
                foreign_window("Plugin editor", open)
                    .size(360.0, 240.0)
                    .mode(WindowMode::Tool)
                    .on_ready(move |foreign: ForeignChild| {
                        ready_child.set(Some(foreign.window()));
                        let requested = ready_requested.clone();
                        spawn(async move {
                            let answer = match foreign.request_size(480, 320).await {
                                Ok(()) => "ok".to_owned(),
                                Err(error) => format!("{:?}", error.kind()),
                            };
                            *requested.borrow_mut() = Some(answer);
                            app_handle().close();
                        });
                    })
                    .on_destroy(move || {
                        let window = destroy_child.get().expect("ready ran first");
                        destroy_alive.set(Some(kivact::test::is_native_window_alive(window)));
                        destroy_count.set(destroy_count.get() + 1);
                    }),
            ))
        });
    match result {
        Ok(()) => {}
        Err(error) if matches!(*error.kind(), ErrorKind::Platform | ErrorKind::Unsupported) => {
            eprintln!("foreign window run skipped: display unavailable: {error}");
            return;
        }
        Err(error) => panic!("the App failed: {error}"),
    }
    let Some(window) = child.get() else {
        if supported.get() {
            panic!("the platform supports foreign windows but none opened");
        }
        eprintln!("foreign window run skipped: the platform has no foreign windows");
        return;
    };
    assert_eq!(requested.borrow().as_deref(), Some("ok"));
    assert_eq!(destroyed.get(), 1, "on_destroy ran before the run ended");
    assert_eq!(alive_in_destroy.get(), Some(true));
    assert!(
        !kivact::test::is_native_window_alive(window),
        "the child is gone"
    );
}

fn main() {
    foreign_window_run();
}
