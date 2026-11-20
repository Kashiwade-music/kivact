//! Role: Contracts of `on_close_request`, which may keep the App open.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::platform::{self, on_close_request};
use kivact::prelude::*;

use kivact_public_api_tests::support::mount;

#[test]
fn without_a_handler_closing_the_main_window_exits_the_app() {
    let (screen, ()) = mount(|| ((), text("main")));
    assert!(screen.request_close(), "no handler: the App exits");
    assert!(screen.has_exited());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_handler_that_answers_yes_keeps_the_app_running() {
    let (screen, count) = mount(|| {
        let count = signal(0);
        on_close_request(|| Handled::Yes);
        (
            count,
            column((
                text(move || format!("count: {}", count.get())),
                button("Tick").on_press(move || count.update(|n| *n += 1)),
            )),
        )
    });
    assert!(!screen.request_close(), "Handled::Yes keeps the App open");
    assert!(!screen.has_exited());
    screen.get_by_role(Role::Button, "Tick").click();
    assert_eq!(count.get_untracked(), 1);
    screen.get_by_text("count: 1");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_app_closes_once_the_handler_allows_it() {
    let (screen, dirty) = mount(|| {
        let dirty = signal(true);
        on_close_request(move || {
            if dirty.get() {
                Handled::Yes
            } else {
                Handled::No
            }
        });
        (dirty, text("main"))
    });
    assert!(!screen.request_close(), "dirty: the App stays open");
    dirty.set(false);
    assert!(screen.request_close(), "clean: the close exits the App");
    assert!(screen.has_exited());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn every_handler_runs_in_registration_order_and_one_veto_keeps_the_app_open() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let (screen, ()) = mount({
        let log = log.clone();
        move || {
            for (name, answer) in [("first", Handled::No), ("second", Handled::Yes)] {
                let log = log.clone();
                on_close_request(move || {
                    log.borrow_mut().push(name);
                    answer
                });
            }
            let log = log.clone();
            on_close_request(move || {
                log.borrow_mut().push("third");
                Handled::No
            });
            ((), text("main"))
        }
    });
    assert!(!screen.request_close(), "the second handler keeps it open");
    assert_eq!(*log.borrow(), ["first", "second", "third"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_failing_handler_keeps_the_app_open_and_reports_its_error() {
    let (screen, ()) = mount(|| {
        on_close_request(|| -> Result<(), UiError> { Err(UiError::msg("save check failed")) });
        ((), text("main"))
    });
    assert!(
        !screen.request_close(),
        "a failed handler keeps the App open"
    );
    assert!(!screen.has_exited());
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].message().contains("save check failed"),
        "{errors:?}"
    );
}

#[test]
fn app_handle_close_exits_even_while_a_handler_would_veto() {
    let (screen, ()) = mount(|| {
        on_close_request(|| Handled::Yes);
        let quit = platform::app_handle();
        ((), button("Quit").on_press(move || quit.close()))
    });
    screen.get_by_role(Role::Button, "Quit").click();
    assert!(
        screen.has_exited(),
        "AppHandle::close does not ask the handlers"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_handler_in_an_unmounted_scope_no_longer_vetoes() {
    let (screen, shown) = mount(|| {
        let shown = signal(true);
        (
            shown,
            show(shown, || {
                on_close_request(|| Handled::Yes);
                text("scoped")
            }),
        )
    });
    assert!(!screen.request_close(), "the mounted handler vetoes");
    shown.set(false);
    screen.run_until_idle();
    assert!(screen.request_close(), "the unmounted handler is gone");
    assert!(screen.has_exited());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[component]
fn guard(on_close: Callback<()>) -> impl IntoView {
    on_close_request(on_close);
    text("guard")
}

#[test]
fn close_request_handled_is_the_same_inline_shared_and_through_a_prop() {
    let (screen, ()) = mount(|| {
        let keep_inline = signal(true);
        let keep_shared = signal(true);
        let keep_prop = signal(true);
        let close_inline = signal(true);
        let close_shared = signal(true);
        let close_prop = signal(true);
        let view = column((
            window("Keep inline", keep_inline).content(|| {
                on_close_request(|| Handled::Yes);
                text("keep inline")
            }),
            window("Keep shared", keep_shared).content(|| {
                let keep = callback(move |_: ()| Handled::Yes);
                on_close_request(keep);
                text("keep shared")
            }),
            window("Keep prop", keep_prop).content(|| guard(|| Handled::Yes)),
            window("Close inline", close_inline).content(|| {
                on_close_request(|| Handled::No);
                text("close inline")
            }),
            window("Close shared", close_shared).content(|| {
                let allow = callback(move |_: ()| Handled::No);
                on_close_request(allow);
                text("close shared")
            }),
            window("Close prop", close_prop).content(|| guard(|| Handled::No)),
        ));
        ((), view)
    });
    for title in ["Keep inline", "Keep shared", "Keep prop"] {
        let handle = screen.window(title);
        assert!(
            !handle.request_close(),
            "{title}: Handled::Yes keeps the window open"
        );
        assert!(handle.is_open(), "{title}");
    }
    for title in ["Close inline", "Close shared", "Close prop"] {
        let handle = screen.window(title);
        assert!(
            handle.request_close(),
            "{title}: Handled::No lets the window close"
        );
        assert!(!handle.is_open(), "{title}");
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
