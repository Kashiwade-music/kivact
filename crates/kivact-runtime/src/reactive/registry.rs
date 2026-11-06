//! Role: Thread-local app registry and handle lookup.
//!
//! The thread-local registry of the Apps on this thread, the App whose
//! code is executing now, and the lookups every handle goes through.
//!
//! The registry and execution modules hold the reactive runtime's two
//! mutable thread-locals (keep it at these two; do not add more):
//! - `REGISTRY` maps an `AppId` (a 1-based index, never reused) to its
//!   runtime. Handles are `(app, key)` pairs without a pointer, so every
//!   use looks the runtime up here; an unregistered App's slot stays
//!   `None`, which makes its handles `WrongApp`.
//! - `EXECUTING` is the App whose frame is on top (0: none). Every pushed
//!   frame sets it and its guard restores it, so a handle of another live
//!   App used inside this App's code is rejected (`WrongApp`) instead of
//!   silently skipping tracking and purity checks.
//!
//! Both are per thread: runtimes and handles are `!Send`.

use crate::reactive::AppId;
use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use crate::reactive::data::Runtime;
use crate::reactive::error::{ErrorKind, misuse_at};
use crate::reactive::execution::executing_app;
use crate::reactive::host_types::Host;

thread_local! {
    static REGISTRY: RefCell<Vec<Option<Rc<Runtime>>>> = const { RefCell::new(Vec::new()) };

}

/// Located `WrongApp` misuse if another App is executing.
pub(crate) fn check_not_foreign(app: AppId, at: &'static Location<'static>) {
    let cur = executing_app();
    if cur != AppId::NONE && cur != app {
        misuse_at(
            ErrorKind::WrongApp,
            format!("a handle of App {app} was used while App {cur} is running"),
            at,
        );
    }
}

impl Runtime {
    /// Create and register a new runtime (App) with its extensions. Ids are
    /// never reused.
    pub(crate) fn register(host: Rc<dyn Host>, start_ns: u64) -> Rc<Runtime> {
        REGISTRY.with(|r| {
            let mut reg = r.borrow_mut();
            let id = AppId(reg.len() as u32 + 1);
            let rt = Rc::new(Runtime::with_id(id, host, start_ns));
            reg.push(Some(rt.clone()));
            rt
        })
    }

    /// Unregister: every handle of this App becomes `WrongApp`/`Disposed`.
    pub(crate) fn unregister(id: AppId) {
        let removed = REGISTRY.with(|r| {
            let mut reg = r.borrow_mut();
            reg.get_mut(id.0 as usize - 1).and_then(|slot| slot.take())
        });
        drop(removed);
    }
}

/// Look up a live runtime (internal: no foreign-App check).
pub(crate) fn try_runtime(app: AppId) -> Option<Rc<Runtime>> {
    REGISTRY.with(|r| {
        r.borrow()
            .get((app.0 as usize).wrapping_sub(1))
            .and_then(|s| s.clone())
    })
}

/// Look up a runtime for a handle, or raise a located `WrongApp` panic when
/// the App no longer exists or another App is executing.
pub(crate) fn runtime(app: AppId, at: &'static Location<'static>) -> Rc<Runtime> {
    check_not_foreign(app, at);
    match try_runtime(app) {
        Some(rt) => rt,
        None => misuse_at(
            ErrorKind::WrongApp,
            "handle belongs to an App that no longer exists",
            at,
        ),
    }
}

/// The runtime whose frame is currently executing on this thread.
/// Returns `None` outside every frame, even when live Apps are registered.
pub(crate) fn current_runtime() -> Option<Rc<Runtime>> {
    try_runtime(executing_app())
}
