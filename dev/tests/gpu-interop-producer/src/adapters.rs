//! Role: Lists this machine's hardware adapters of a backend, for window-less tests.

use kivact_types::{GpuAdapter, GpuBackend};

/// This machine's hardware adapters of `backend` that a window could
/// report (generation 1), for tests without a window; none where the
/// backend is missing.
pub fn adapters(backend: GpuBackend) -> Vec<GpuAdapter> {
    match backend {
        #[cfg(windows)]
        GpuBackend::Dx12 => crate::d3d12::adapters(),
        #[cfg(any(windows, target_os = "linux"))]
        GpuBackend::Vulkan => crate::vulkan::adapters(),
        #[allow(unreachable_patterns, reason = "the backends of other platforms")]
        _ => Vec::new(),
    }
}
