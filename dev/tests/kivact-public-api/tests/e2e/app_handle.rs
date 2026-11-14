//! Role: Contracts of the AppHandle that ends an App's run.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use kivact::platform;
use kivact::prelude::*;
use kivact::test::Screen;

use crate::support::mount;

#[test]
fn close_from_a_handler_exits_the_app() {
    let (screen, ()) = mount(|| {
        (
            (),
            button("Quit").on_press(|| platform::app_handle().close()),
        )
    });
    screen.get_by_role(Role::Button, "Quit").click();
    assert!(screen.has_exited(), "the next turn ends the App");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn close_from_another_thread_exits_the_app() {
    let (screen, handle) = mount(|| (platform::app_handle(), text("main")));
    std::thread::spawn(move || handle.close())
        .join()
        .expect("the closing thread does not panic");
    screen.wait_for(Duration::from_secs(3), Screen::has_exited);
    assert!(screen.has_exited());
}

#[test]
fn close_during_setup_exits_after_the_first_turn() {
    let cleaned = Rc::new(Cell::new(false));
    let cleanup = cleaned.clone();
    let (screen, handle) = mount(move || {
        on_cleanup(move || cleanup.set(true));
        let handle = platform::app_handle();
        handle.close();
        (handle, text("main"))
    });
    assert!(screen.has_exited(), "the first turn exits");
    drop(screen);
    assert!(cleaned.get(), "owners are disposed when the run ends");
    handle.close(); // after the run: nothing happens
}

#[test]
fn queued_actions_finish_before_the_app_ends() {
    let (screen, log) = mount(|| {
        let log = Rc::new(RefCell::new(Vec::new()));
        let saved = log.clone();
        let save = action(ActionMode::Queue, move |n: u32| {
            let saved = saved.clone();
            async move {
                kivact::task::sleep(Duration::from_secs(1)).await;
                saved.borrow_mut().push(n);
                Ok(())
            }
        });
        let quitter = platform::app_handle();
        let view = button("Save and quit").on_press(move || {
            save.run(1);
            quitter.close();
        });
        (log, view)
    });
    screen.get_by_role(Role::Button, "Save and quit").click();
    screen.advance(Duration::from_secs(1));
    assert_eq!(*log.borrow(), vec![1u32], "the queued action drained");
    assert!(screen.has_exited(), "then the App ended");
}

#[test]
fn closing_twice_and_after_the_run_is_harmless() {
    let (screen, handle) = mount(|| (platform::app_handle(), text("main")));
    handle.close();
    handle.close();
    screen.wait_for(Duration::from_secs(3), Screen::has_exited);
    handle.close();
    assert!(screen.has_exited());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
