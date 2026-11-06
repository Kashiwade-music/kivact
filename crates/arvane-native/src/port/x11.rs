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
#[cfg(feature = "test")]
pub(crate) mod test_parent;
mod thread;
mod window;

pub(in crate::port) use thread::open;
