//! Role: Locate the owner and runtime of public setup calls.
use std::panic::Location;
use std::rc::Rc;

use crate::arena::Key;
use crate::data::Runtime;
use crate::error::{ErrorKind, misuse_at};
use crate::registry::current_runtime;

/// The executing runtime and its setup owner, or a located `SetupOnly`
/// misuse naming `api` (also when no App exists).
pub(crate) fn setup_scope(api: &'static str, at: &'static Location<'static>) -> (Rc<Runtime>, Key) {
    match current_runtime() {
        Some(rt) => {
            let owner = rt.require_setup(api, at);
            (rt, owner)
        }
        None => misuse_at(
            ErrorKind::SetupOnly { api },
            format!("`{api}` can only be called during component setup"),
            at,
        ),
    }
}

/// The executing runtime and the nearest owner of its frame stack (setup,
/// handler, task, frame callback, effect or cleanup). A located `Disposed`
/// misuse when the code runs for an owner that is already disposed (a
/// queue job draining after its owner went away); a located `SetupOnly`
/// misuse naming `api` when no owner applies at all.
pub(crate) fn owner_scope(api: &'static str, at: &'static Location<'static>) -> (Rc<Runtime>, Key) {
    if let Some(rt) = current_runtime()
        && let Some(owner) = rt.current_owner()
    {
        if rt.owner_alive(owner) {
            return (rt, owner);
        }
        let tomb = rt.owners.borrow().tomb(owner);
        misuse_at(
            ErrorKind::Disposed {
                created_at: tomb.map(|t| t.created_at),
                disposed_by: tomb.and_then(|t| t.disposed_by),
            },
            format!("`{api}` ran for an owner that was already disposed"),
            at,
        )
    }
    misuse_at(
        ErrorKind::SetupOnly { api },
        format!(
            "`{api}` needs an owner: call it during component setup or in a handler, task or effect"
        ),
        at,
    )
}
