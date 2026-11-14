//! Role: Contracts of a window whose content is a native child window.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::ErrorKind;
use kivact::platform::NativeWindow;
use kivact::prelude::*;
use kivact::test::RenderOptions;
use kivact::view::ForeignChild;
use kivact::view::SurfaceSize;
use kivact::view::foreign_window;

/// What one test's callbacks saw.
#[derive(Default)]
struct Seen {
    ready: u32,
    destroyed: u32,
    sizes: Vec<SurfaceSize>,
    child: Option<ForeignChild>,
    alive_in_destroy: Option<bool>,
}

fn seen() -> Rc<RefCell<Seen>> {
    Rc::new(RefCell::new(Seen::default()))
}

#[test]
fn a_platform_without_the_capability_reports_unsupported_once_and_stays_closed() {
    let (screen, (open, seen)) = mount_with(RenderOptions::from_env(), || {
        let open = signal(true);
        let seen = seen();
        (
            (open, seen.clone()),
            column((
                text("main"),
                foreign_window("Plugin", open).on_ready(move |child| {
                    seen.borrow_mut().ready += 1;
                    seen.borrow_mut().child = Some(child);
                }),
            )),
        )
    });
    screen.run_until_idle();
    assert!(open.get_untracked());
    assert_eq!(seen.borrow().ready, 0);
    let errors = screen.errors();
    assert_eq!(
        errors
            .iter()
            .filter(|e| *e.kind() == ErrorKind::Unsupported)
            .count(),
        1,
        "{errors:?}"
    );
}

#[test]
fn ready_reports_the_handle_and_the_physical_size_at_the_window_scale() {
    let options = RenderOptions::from_env().foreign_windows().scale(1.5);
    let (screen, seen) = mount_with(options, || {
        let open = signal(true);
        let seen = seen();
        (
            seen.clone(),
            column((
                text("main"),
                foreign_window("Plugin", open).on_ready(move |child| {
                    let mut seen = seen.borrow_mut();
                    seen.ready += 1;
                    seen.sizes.push(child.size());
                    seen.child = Some(child);
                }),
            )),
        )
    });
    let window = screen.window("Plugin");
    assert!(window.is_open());
    let seen = seen.borrow();
    assert_eq!(seen.ready, 1);
    assert_eq!(seen.sizes[0].physical_width, 960);
    assert_eq!(seen.sizes[0].physical_height, 720);
    let child = seen.child.expect("ready gave the child");
    assert_eq!(Some(child.window()), window.foreign_child());
    assert!(matches!(child.window(), NativeWindow::X11(_)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_user_resize_and_a_scale_change_each_report_once() {
    let options = RenderOptions::from_env().foreign_windows();
    let (screen, seen) = mount_with(options, || {
        let open = signal(true);
        let seen = seen();
        let resize_seen = seen.clone();
        (
            seen.clone(),
            column((
                text("main"),
                foreign_window("Plugin", open)
                    .on_ready(move |child| seen.borrow_mut().child = Some(child))
                    .on_resize({
                        let seen = resize_seen;
                        move |size| {
                            let mut seen = seen.borrow_mut();
                            seen.sizes.push(size);
                        }
                    }),
            )),
        )
    });
    let window = screen.window("Plugin");
    let before = seen.borrow().sizes.len();
    window.resize(500.0, 400.0);
    assert_eq!(seen.borrow().sizes.len(), before + 1);
    assert_eq!(seen.borrow().sizes.last().unwrap().physical_width, 500);
    window.set_scale(2.0);
    assert_eq!(seen.borrow().sizes.len(), before + 2);
    assert_eq!(seen.borrow().sizes.last().unwrap().scale_factor, 2.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn request_size_sends_the_logical_size_and_reports_success() {
    let options = RenderOptions::from_env().foreign_windows().scale(2.0);
    let (screen, (trigger, result, seen)) = mount_with(options, || {
        let open = signal(true);
        let trigger = signal(0u32);
        let result = signal(None::<String>);
        let seen = seen();
        let request_seen = seen.clone();
        let ready_seen = seen.clone();
        let ready_result = result;
        effect(move || {
            if trigger.get() == 1
                && let Some(child) = request_seen.borrow().child
            {
                let result = result;
                spawn(async move {
                    result.set(Some(match child.request_size(640, 480).await {
                        Ok(()) => "ok".to_owned(),
                        Err(error) => format!("{:?}", error.kind()),
                    }));
                });
            }
        });
        (
            (trigger, result, seen.clone()),
            column((
                text("main"),
                foreign_window("Plugin", open).on_ready(move |child| {
                    ready_seen.borrow_mut().child = Some(child);
                    let result = ready_result;
                    spawn(async move {
                        result.set(Some(match child.request_size(640, 480).await {
                            Ok(()) => "ok".to_owned(),
                            Err(error) => format!("{:?}", error.kind()),
                        }));
                    });
                }),
            )),
        )
    });
    let window = screen.window("Plugin");
    screen.run_until_idle();
    assert_eq!(result.get_untracked().as_deref(), Some("ok"));
    let sizes = window.requested_sizes();
    assert_eq!(sizes.len(), 1);
    assert_eq!(sizes[0].width, 320.0);
    assert_eq!(sizes[0].height, 240.0);
    let _ = trigger;
    assert_eq!(seen.borrow().ready, 0, "ready was not counted here");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn request_size_after_close_resolves_to_closed() {
    let options = RenderOptions::from_env().foreign_windows();
    let (screen, (open, trigger, result)) = mount_with(options, || {
        let open = signal(true);
        let trigger = signal(0u32);
        let result = signal(None::<String>);
        let seen = seen();
        let request_seen = seen.clone();
        effect(move || {
            if trigger.get() == 1
                && let Some(child) = request_seen.borrow().child
            {
                let result = result;
                spawn(async move {
                    result.set(Some(match child.request_size(100, 100).await {
                        Ok(()) => "ok".to_owned(),
                        Err(error) => format!("{:?}", error.kind()),
                    }));
                });
            }
        });
        (
            (open, trigger, result),
            column((
                text("main"),
                foreign_window("Plugin", open).on_ready(move |child| {
                    seen.borrow_mut().child = Some(child);
                }),
            )),
        )
    });
    let window = screen.window("Plugin");
    open.set(false);
    screen.run_until_idle();
    assert!(!window.is_open());
    trigger.set(1);
    screen.run_until_idle();
    screen.run_until_idle();
    assert_eq!(result.get_untracked().as_deref(), Some("Closed"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn resizable_follows_the_binding_and_the_test_backend_records_it() {
    let options = RenderOptions::from_env().foreign_windows();
    let (screen, resizable) = mount_with(options, || {
        let open = signal(true);
        let resizable = signal(true);
        (
            resizable,
            column((
                text("main"),
                foreign_window("Plugin", open).resizable(resizable),
            )),
        )
    });
    let window = screen.window("Plugin");
    assert!(window.is_resizable());
    resizable.set(false);
    screen.run_until_idle();
    assert!(!window.is_resizable());
    resizable.set(true);
    screen.run_until_idle();
    assert!(window.is_resizable());
}

#[test]
fn a_prevented_dismiss_keeps_the_window_and_does_not_destroy_it() {
    let options = RenderOptions::from_env().foreign_windows();
    let (screen, seen) = mount_with(options, || {
        let open = signal(true);
        let seen = seen();
        (
            seen.clone(),
            column((
                text("main"),
                foreign_window("Plugin", open)
                    .on_dismiss(|_| Handled::PreventDefault)
                    .on_destroy({
                        let seen = seen.clone();
                        move || {
                            seen.borrow_mut().destroyed += 1;
                        }
                    }),
            )),
        )
    });
    let window = screen.window("Plugin");
    assert!(!window.request_close());
    assert!(window.is_open());
    assert_eq!(seen.borrow().destroyed, 0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn closing_through_open_runs_on_destroy_while_the_child_is_alive() {
    let options = RenderOptions::from_env().foreign_windows();
    let (screen, (open, seen, ())) = mount_with(options, || {
        let open = signal(true);
        let seen = seen();
        (
            (open, seen.clone(), ()),
            column((
                text("main"),
                foreign_window("Plugin", open)
                    .on_ready({
                        let seen = seen.clone();
                        move |child| seen.borrow_mut().child = Some(child)
                    })
                    .on_destroy({
                        let seen = seen.clone();
                        move || {
                            let child = seen.borrow().child.expect("ready ran first");
                            seen.borrow_mut().alive_in_destroy =
                                Some(kivact::test::is_native_window_alive(child.window()));
                            seen.borrow_mut().destroyed += 1;
                        }
                    }),
            )),
        )
    });
    let window = screen.window("Plugin");
    let child = seen.borrow().child.expect("child");
    open.set(false);
    screen.run_until_idle();
    assert!(!window.is_open());
    assert_eq!(seen.borrow().destroyed, 1);
    assert_eq!(seen.borrow().alive_in_destroy, Some(true));
    assert!(!kivact::test::is_native_window_alive(child.window()));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn closing_from_the_title_bar_and_from_the_declaring_branch_destroy_once() {
    let options = RenderOptions::from_env().foreign_windows();
    let (screen, (declared, seen, ())) = mount_with(options, || {
        let declared = signal(true);
        let seen = seen();
        (
            (declared, seen.clone(), ()),
            column((
                text("main"),
                show(declared, {
                    let seen = seen.clone();
                    move || {
                        let open = signal(true);
                        foreign_window("Plugin", open).on_destroy({
                            let seen = seen.clone();
                            move || {
                                seen.borrow_mut().destroyed += 1;
                            }
                        })
                    }
                }),
            )),
        )
    });
    let window = screen.window("Plugin");
    assert!(window.request_close());
    assert_eq!(seen.borrow().destroyed, 1);
    declared.set(false);
    screen.run_until_idle();
    assert_eq!(seen.borrow().destroyed, 1, "a second destruction is wrong");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_app_exit_closes_a_foreign_window_in_the_exit_turn() {
    let options = RenderOptions::from_env().foreign_windows();
    let (screen, seen) = mount_with(options, || {
        let open = signal(true);
        let seen = seen();
        (
            seen.clone(),
            column((
                text("main"),
                foreign_window("Plugin", open)
                    .on_ready({
                        let seen = seen.clone();
                        move |child| seen.borrow_mut().child = Some(child)
                    })
                    .on_destroy({
                        let seen = seen.clone();
                        move || {
                            let child = seen.borrow().child.expect("ready ran first");
                            seen.borrow_mut().alive_in_destroy =
                                Some(kivact::test::is_native_window_alive(child.window()));
                            seen.borrow_mut().destroyed += 1;
                        }
                    }),
            )),
        )
    });
    let _window = screen.window("Plugin");
    assert!(screen.request_close());
    assert!(screen.has_exited());
    assert_eq!(seen.borrow().destroyed, 1);
    assert_eq!(seen.borrow().alive_in_destroy, Some(true));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn reopening_gives_a_new_handle_and_a_second_ready() {
    let options = RenderOptions::from_env().foreign_windows();
    let (screen, (open, seen, ())) = mount_with(options, || {
        let open = signal(true);
        let seen = seen();
        (
            (open, seen.clone(), ()),
            column((
                text("main"),
                foreign_window("Plugin", open).on_ready({
                    let seen = seen.clone();
                    move |child| {
                        let mut seen = seen.borrow_mut();
                        seen.ready += 1;
                        seen.child = Some(child);
                    }
                }),
            )),
        )
    });
    let window = screen.window("Plugin");
    let _ = window.is_open();
    let first = seen.borrow().child.expect("first child");
    open.set(false);
    screen.run_until_idle();
    open.set(true);
    screen.run_until_idle();
    let reopened = screen.window("Plugin");
    assert_eq!(seen.borrow().ready, 2);
    let second = seen.borrow().child.expect("second child");
    assert_ne!(first.window(), second.window());
    assert_eq!(Some(second.window()), reopened.foreign_child());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
