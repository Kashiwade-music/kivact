//! Role: Owner-scoped lifecycle order, environment visibility and error routing.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::platform::{self, LifecycleEvent, on_lifecycle};
use kivact::prelude::*;
use kivact::view::{error_boundary, show};

#[test]
fn lifecycle_reports_transitions_after_environment_and_unregisters_removed_owners() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let out = log.clone();
    let (screen, shown) = mount(move || {
        let shown = signal(true);
        let root = out.clone();
        on_lifecycle(move |event| {
            root.borrow_mut()
                .push(("root", event, platform::is_surface_active().get()))
        });
        let child = out.clone();
        (
            shown,
            show(shown, move || {
                let child = child.clone();
                on_lifecycle(move |event| {
                    child
                        .borrow_mut()
                        .push(("child", event, platform::is_surface_active().get()))
                });
                text("child")
            }),
        )
    });
    assert!(log.borrow().is_empty(), "no initial synthetic resume");
    screen.lose_surface();
    screen.restore_surface();
    assert!(
        log.borrow().is_empty(),
        "surface changes are environment only"
    );
    screen.suspend(false);
    screen.suspend(false);
    assert_eq!(
        *log.borrow(),
        [
            ("root", LifecycleEvent::Suspended, false),
            ("child", LifecycleEvent::Suspended, false)
        ]
    );
    screen.resume();
    screen.resume();
    assert_eq!(
        &log.borrow()[2..],
        [
            ("root", LifecycleEvent::Resumed, true),
            ("child", LifecycleEvent::Resumed, true)
        ]
    );
    shown.set(false);
    screen.run_until_idle();
    screen.memory_warning();
    screen.memory_warning();
    assert_eq!(
        &log.borrow()[4..],
        [
            ("root", LifecycleEvent::MemoryWarning, true),
            ("root", LifecycleEvent::MemoryWarning, true)
        ]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn lifecycle_failure_routes_to_its_owner_and_disposal_suppresses_later_events() {
    let later = Rc::new(Cell::new(0));
    let out = later.clone();
    let (screen, ()) = mount(move || {
        let out = out.clone();
        (
            (),
            error_boundary(move || {
                on_lifecycle(|_| -> Result<(), UiError> {
                    Err(UiError::new(
                        kivact::ErrorKind::Platform,
                        "lifecycle failure",
                        None,
                    ))
                });
                let later = out.clone();
                on_lifecycle(move |_| later.set(later.get() + 1));
                text("before")
            })
            .fallback(|error, _| {
                assert!(error.to_string().contains("lifecycle failure"));
                text("lifecycle failed")
            }),
        )
    });
    screen.memory_warning();
    screen.get_by_text("lifecycle failed");
    assert_eq!(
        later.get(),
        1,
        "ordinary event batching defers fallback disposal"
    );
    screen.memory_warning();
    assert_eq!(later.get(), 1, "disposed owner receives no later event");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn component_initializes_once_in_mounting_context_and_cleans_up_on_removal() {
    let started = Rc::new(Cell::new(0));
    let cleaned = Rc::new(Cell::new(0));
    let starts = started.clone();
    let cleanups = cleaned.clone();
    let (screen, (shown, value)) = mount(move || {
        let shown = signal(true);
        let value = signal(1);
        let view = provide(String::from("inherited"), move || {
            show(shown, move || {
                let starts = starts.clone();
                let cleanups = cleanups.clone();
                component(move || {
                    starts.set(starts.get() + 1);
                    let context = use_context::<String>();
                    on_cleanup(move || cleanups.set(cleanups.get() + 1));
                    text(move || format!("{} {}", context.get(), value.get()))
                })
            })
        });
        ((shown, value), view)
    });
    assert!(screen.get_by_text("inherited 1").is_visible());
    value.set(2);
    screen.run_until_idle();
    assert!(screen.get_by_text("inherited 2").is_visible());
    assert_eq!(started.get(), 1);
    shown.set(false);
    screen.run_until_idle();
    assert_eq!(cleaned.get(), 1);
    shown.set(true);
    screen.run_until_idle();
    assert!(screen.get_by_text("inherited 2").is_visible());
    assert_eq!(started.get(), 2);
    assert_eq!(cleaned.get(), 1);
}
