//! Role: Shared embedded window lifecycle and entry points.
mod open;
#[cfg(all(feature = "test", any(windows, target_os = "linux")))]
pub use open::TestParent;
pub use open::open;
#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
pub(crate) mod child;
#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
pub(crate) mod geometry;
#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
pub(crate) mod session;
