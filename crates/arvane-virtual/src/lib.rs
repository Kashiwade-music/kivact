//! Role: Virtual Arvane backend with scripted input, a virtual clock and optional mirror windows.
#![forbid(unsafe_code)]

mod backend;
mod clock;
#[cfg(feature = "mirror")]
mod mirror;
mod script;
mod surface;
mod window;

#[cfg(feature = "mirror")]
pub use arvane_native::mirror::MirrorStats;
pub use backend::{GpuMode, MirrorMode, Probe, VirtualBackend, VirtualOptions};
#[cfg(feature = "mirror")]
pub use mirror::mirror_stats;
pub use script::Script;
