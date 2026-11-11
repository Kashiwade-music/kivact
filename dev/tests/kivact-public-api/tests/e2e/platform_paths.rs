//! Role: Tests for platform path handling and file operations.
//!
//! Choosing a folder, opening a path with the OS default handler and
//! revealing it in the OS file manager: the test host answers the folder
//! dialog from its script and records open and reveal requests in order
//! instead of starting a program; a scripted failure reaches the App
//! (the nearest `on_error`, or the future) without recording the path.

use std::path::PathBuf;

use crate::support::mount;
use kivact::UiError;
use kivact::platform;
use kivact::prelude::*;
use kivact::task::spawn;

fn button(name: &'static str, press: impl Into<Callback<()>>) -> impl Element {
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .style(style().size(60, 24))
        .on_press(press)
}

#[test]
fn open_and_reveal_futures_answer_with_the_result() {
    let (screen, status) = mount(|| {
        let status = signal(String::new());
        let say = move |result: Result<(), UiError>, done: &str| {
            status.set(match result {
                Ok(()) => done.to_string(),
                Err(error) => format!("failed: {}", error.message()),
            });
        };
        let view = column((
            button("open folder", move || {
                spawn(async move { say(platform::open_path("C:/media").await, "opened") });
            }),
            button("reveal folder", move || {
                spawn(async move {
                    say(platform::reveal_path("C:/media").await, "revealed");
                });
            }),
        ));
        (status, view)
    });
    screen.get_by_role(Role::Button, "open folder").click();
    assert_eq!(status.get_untracked(), "opened");
    screen.get_by_role(Role::Button, "reveal folder").click();
    assert_eq!(status.get_untracked(), "revealed");
    assert_eq!(screen.opened_paths(), [PathBuf::from("C:/media")]);
    assert_eq!(screen.revealed_paths(), [PathBuf::from("C:/media")]);

    screen.fail_next_platform_request("no file manager");
    screen.get_by_role(Role::Button, "reveal folder").click();
    assert_eq!(status.get_untracked(), "failed: no file manager");
    assert_eq!(
        screen.revealed_paths().len(),
        1,
        "a failed request is not recorded"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
