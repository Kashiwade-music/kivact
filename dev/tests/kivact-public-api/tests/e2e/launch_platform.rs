//! Role: Platform services testing.
//!
//! A platform request the platform has no service for (opening a path on
//! macOS): the window's unsupported error reaches the App as
//! `ErrorKind::Unsupported`, not as a failure. The test window makes that
//! answer only through the driver (`ArvaneTestHost`), so this needs the
//! crate-internal `Screen::with_host`; the rest of the platform services
//! are checked through the public API (`platform_paths.rs`).

use std::cell::RefCell;
use std::rc::Rc;

use kivact::{ErrorKind, UiError};

use super::launch_support::button_box;
use kivact::platform;
use kivact::prelude::Role;
use kivact::task::spawn;
use kivact::test::render;

#[test]
fn an_unsupported_service_is_an_unsupported_error() {
    let result: Rc<RefCell<Option<Result<(), UiError>>>> = Rc::default();
    let got = result.clone();
    let screen = render(move || {
        button_box("open").on_press(move || {
            let got = got.clone();
            spawn(async move {
                *got.borrow_mut() = Some(platform::open_path("C:/media").await);
            });
        })
    });
    screen
        .script()
        .fail_next_effect_unsupported("cannot open C:/media: unsupported");
    screen.get_by_role(Role::Button, "open").click();
    let error = result
        .borrow_mut()
        .take()
        .expect("answered")
        .expect_err("unsupported");
    assert_eq!(*error.kind(), ErrorKind::Unsupported);
    assert!(screen.opened_paths().is_empty());
}
