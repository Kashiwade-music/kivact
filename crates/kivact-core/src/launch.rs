//! Role: Shared application launch and read-only inspection boundary.
mod inspect;
mod runtime;
mod settings;
pub use inspect::{Inspect, LaunchHandle};
pub(crate) use runtime::run_native;
pub use runtime::{ClientMode, Launch, TestTiming};
pub use settings::AppOptions;
#[cfg(test)]
pub(crate) use settings::check_window_sizes;
