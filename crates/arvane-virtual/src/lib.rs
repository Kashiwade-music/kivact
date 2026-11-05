//! Role: Runs scripted input and virtual time with optional mirror windows.

#![deny(clippy::too_many_lines, clippy::disallowed_types)]
#![forbid(unsafe_code)]

mod backend;
mod clock;
#[cfg(feature = "mirror")]
mod mirror;
pub mod raw;
mod script;
mod surface;
mod window;

#[cfg(feature = "mirror")]
pub use arvane_native::mirror::MirrorStats;
pub use backend::{GpuMode, MirrorMode, Probe, VirtualBackend, VirtualOptions};
#[cfg(feature = "mirror")]
pub use mirror::mirror_stats;
pub use script::Script;
