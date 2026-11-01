//! Role: Tests for platform path handling and file operations.
//!
//! Choosing a folder, opening a path with the OS default handler and
//! revealing it in the OS file manager: the test host answers the folder
//! dialog from its script and records open and reveal requests in order
//! instead of starting a program; a scripted failure reaches the App
//! (the nearest `on_error`, or the future) without recording the path.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::support::mount;
use kivact::UiError;
use kivact::diagnostics::ErrorKind;
use kivact::platform;
use kivact::platform::FileDialog;
use kivact::prelude::*;
use kivact::task::spawn;
use kivact::view::error_boundary;

fn button(name: &'static str, press: impl Fn() + 'static) -> impl IntoView {
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .style(style().size(60, 24))
        .on_press(press)
}

#[test]
fn open_and_reveal_are_recorded_and_their_failures_reach_on_error() {
    let caught: Rc<RefCell<Vec<UiError>>> = Rc::default();
    let seen = caught.clone();
    let (screen, ()) = mount(move || {
        let view = error_boundary(|| {
            column((
                button("open", move || {
                    spawn(platform::open_path("C:/media/clip.wav"));
                }),
                button("reveal", move || {
                    spawn(platform::reveal_path(PathBuf::from("C:/media/clip.wav")));
                }),
            ))
        })
        .on_error(move |e| seen.borrow_mut().push(e));
        ((), view)
    });
    assert!(screen.opened_paths().is_empty());
    assert!(screen.revealed_paths().is_empty());
    screen.get_by_role(Role::Button, "open").click();
    screen.get_by_role(Role::Button, "reveal").click();
    assert_eq!(screen.opened_paths(), [PathBuf::from("C:/media/clip.wav")]);
    assert_eq!(
        screen.revealed_paths(),
        [PathBuf::from("C:/media/clip.wav")]
    );
    assert!(caught.borrow().is_empty(), "{:?}", caught.borrow());

    screen.fail_next_platform_request("no file manager");
    screen.get_by_role(Role::Button, "reveal").click();
    screen.fail_next_platform_request("no handler");
    screen.get_by_role(Role::Button, "open").click();
    let errors = caught.borrow().clone();
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(
        errors.iter().all(|e| *e.kind() == ErrorKind::Platform),
        "{errors:?}"
    );
    assert!(
        errors[0].to_string().contains("no file manager"),
        "{errors:?}"
    );
    assert!(errors[1].to_string().contains("no handler"), "{errors:?}");
    assert_eq!(
        screen.opened_paths().len(),
        1,
        "a failed request is not recorded"
    );
    assert_eq!(
        screen.revealed_paths().len(),
        1,
        "a failed request is not recorded"
    );
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

#[test]
fn the_folder_picker_answers_from_the_scripted_dialogs() {
    let (screen, status) = mount(|| {
        let status = signal(String::new());
        let view = button("choose folder", move || {
            let options = FileDialog {
                title: "Collect files into".to_string(),
                ..FileDialog::default()
            };
            spawn(async move {
                let message = match platform::pick_folder(options).await {
                    Ok(Some(path)) => format!("chosen: {}", path.display()),
                    Ok(None) => "cancelled".to_string(),
                    Err(error) => format!("failed: {}", error.message()),
                };
                status.set(message);
            });
        });
        (status, view)
    });
    let choose = || screen.get_by_role(Role::Button, "choose folder").click();

    choose();
    assert_eq!(status.get_untracked(), "cancelled", "unscripted: cancelled");
    screen.respond_file_dialog(Some(PathBuf::from("C:/projects/collected")));
    choose();
    assert_eq!(status.get_untracked(), "chosen: C:/projects/collected");
    screen.respond_file_dialog(Some(PathBuf::from("C:/projects/kept")));
    screen.fail_next_platform_request("no picker");
    choose();
    assert_eq!(status.get_untracked(), "failed: no picker");
    choose();
    assert_eq!(
        status.get_untracked(),
        "chosen: C:/projects/kept",
        "a failure keeps the scripted answer"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
