//! Role: Runs each embedded X11 window on its own UI thread.
//!
//! Invariants: each child owns an independent XCB connection; host process
//! settings are never changed. Commands and wakes use the same eventfd.

mod a11y;
mod conn;
mod cursor;
mod dnd;
mod error;
mod event;
mod ime;
mod keyboard;
mod notifier;
mod poll;
mod services;
#[cfg(feature = "testing")]
pub(crate) mod test_parent;
mod thread;
mod window;

use arvane_shell::{ChildOptions, MakeHandler, ShellChild, ShellError};

pub(super) fn open(
    parent: kivact_types::ParentWindow,
    options: ChildOptions,
    make: MakeHandler,
) -> Result<Box<dyn ShellChild>, ShellError> {
    thread::open(parent, options, make)
}
