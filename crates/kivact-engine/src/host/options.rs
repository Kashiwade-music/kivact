//! Role: Passive initial native window settings consumed by the host.
use kivact_types::Size;

/// Initial native window settings, after the application validates them.
#[derive(Clone, Copy, Debug, Default)]
pub struct HostOptions {
    /// Logical pixels; `None` selects the host's initial window size
    /// (960×720). kivact's `App::run` checks a given size once, before the
    /// window opens; the host does not check it again. The minimum size
    /// comes later, with the first revision (`Writer::set_window`).
    pub window_size: Option<Size>,
    /// How the window's GPU device is made (`Shell::gpu_preference`). Used
    /// with feature `gpu-interop`; ignored without it.
    pub gpu_preference: kivact_types::GpuPreference,
}
