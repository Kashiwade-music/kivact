//! Role: Environment variable configuration for Arvane.
//!
//! The environment variables Arvane reads, all read here: once per native
//! window loop ([`crate::Shell::run`]) or test window, never on a hot
//! path.
//!
//! - `ARVANE_RENDERER`: `cpu` draws a native window on the CPU, `gpu` on
//!   the GPU, overriding the host's preference. Test windows do not read
//!   it (their frames are the same whatever the environment); their
//!   option `VirtualWindowOptions::renderer` stands for it, by the same
//!   rule ([`starts_gpu`]).
//! - `ARVANE_TEST_HEADLESS` (any value but `0`): test windows in
//!   `WindowMode::Auto` are headless (feature `testing`).
//! - `WGPU_BACKEND`, `WGPU_POWER_PREF` and the instance flags wgpu reads
//!   (`WGPU_VALIDATION`, `WGPU_DEBUG` ..): how GPU devices are made
//!   (feature `gpu`; read by `arvane-render`, which alone knows wgpu). The
//!   first two override the host's [`ui_types::GpuPreference`].

use crate::presentation::Renderer;

/// Whether a window starts the GPU renderer: as `forced` says
/// (`ARVANE_RENDERER`, or a test window's option standing for it), else as
/// the host prefers (`prefer_gpu`).
pub(crate) fn starts_gpu(forced: Option<Renderer>, prefer_gpu: bool) -> bool {
    forced.map_or(prefer_gpu, |renderer| renderer == Renderer::Gpu)
}

/// What the environment asks of Arvane.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Env {
    /// `ARVANE_RENDERER`: the renderer native windows draw with, whatever
    /// the host prefers.
    pub(crate) renderer: Option<Renderer>,
    /// `ARVANE_TEST_HEADLESS`: test windows that may be real are headless.
    #[cfg(feature = "testing")]
    pub(crate) test_headless: bool,
    /// How GPU devices are made.
    #[cfg(feature = "gpu")]
    pub(crate) gpu: arvane_render::GpuEnv,
}

impl Env {
    /// Read the environment now, for GPU devices made as `preference`
    /// wishes where the environment does not say otherwise.
    pub(crate) fn read(preference: ui_types::GpuPreference) -> Self {
        #[cfg(not(feature = "gpu"))]
        let _ = preference;
        Self {
            renderer: match std::env::var("ARVANE_RENDERER").as_deref() {
                Ok("cpu") => Some(Renderer::Cpu),
                Ok("gpu") => Some(Renderer::Gpu),
                _ => None,
            },
            #[cfg(feature = "testing")]
            test_headless: std::env::var_os("ARVANE_TEST_HEADLESS").is_some_and(|v| v != "0"),
            #[cfg(feature = "gpu")]
            gpu: arvane_render::GpuEnv::from_env(preference),
        }
    }
}
