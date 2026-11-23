//! Role: Shared application launch and read-only inspection boundary.
mod close;
mod inspect;
mod policy;
mod runtime;
mod settings;
mod turn;
pub use inspect::{Inspect, LaunchHandle};
pub use policy::LaunchPolicy;
pub(crate) use runtime::{Launch, run_native};
pub(crate) use settings::{AppSettings, LaunchApp};
