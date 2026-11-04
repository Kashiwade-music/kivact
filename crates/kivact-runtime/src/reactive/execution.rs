//! Role: Thread-local identity of the executing app.
use crate::reactive::AppId;
use std::cell::Cell;

thread_local! {
    /// Id of the App whose code is executing on this thread (0: none). Set
    /// by every pushed frame and restored when the frame is popped.
    static EXECUTING: Cell<AppId> = const { Cell::new(AppId::NONE) };
}
/// The App currently executing code on this thread (0 outside every App).
#[inline]
pub(crate) fn executing_app() -> AppId {
    EXECUTING.with(|c| c.get())
}

/// Whether a handle of `app` may be used here: always outside Apps, and only
/// by its own App while an App runs (another live App's handles are
/// rejected instead of silently bypassing tracking and purity).
#[inline]
pub(crate) fn handle_usable_here(app: AppId) -> bool {
    let cur = executing_app();
    cur == AppId::NONE || cur == app
}

/// Make `app` the executing App; returns the one it replaces (frame push).
#[inline]
pub(crate) fn enter_app(app: AppId) -> AppId {
    EXECUTING.with(|c| c.replace(app))
}

/// Make `app` the executing App again (frame pop).
#[inline]
pub(crate) fn leave_app(app: AppId) {
    EXECUTING.with(|c| c.set(app));
}
