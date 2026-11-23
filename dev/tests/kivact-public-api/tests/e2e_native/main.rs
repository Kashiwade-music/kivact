//! Role: End-to-end tests of embedding, GPU interop, foreign windows and DevTools.

#[cfg(feature = "devtools")]
mod devtools;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod drawable_export_gpu;
#[cfg(feature = "embed")]
mod embed;
mod foreign_window;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod gpu_interop;
mod gpu_on_demand;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod gpu_on_demand_shared;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod gpu_on_demand_windows;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod viewport_gpu_interop;
