//! Role: Environment variable configuration for Arvane.
//!
//! The environment variables Arvane reads, all read here: once per native
//! window loop ([`crate::Shell::run`]) or test window, never on a hot
//! path.
//!
//! - `ARVANE_RENDERER`: `cpu` draws a native window on the CPU, `gpu` on
//!   the GPU, overriding the host's preference. Stepped shells read
//!   it through the same rendering driver.
//! - `WGPU_BACKEND`, `WGPU_POWER_PREF` and the instance flags wgpu reads
//!   (`WGPU_VALIDATION`, `WGPU_DEBUG` ..): how GPU devices are made
//!   (feature `gpu`; read by `arvane-render`, which alone knows wgpu). The
//!   first two override the host's [`kivact_types::GpuPreference`].

use crate::presentation::Renderer;

/// Whether a window starts the GPU renderer: as `forced` says
/// (`ARVANE_RENDERER`), else as
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
    /// How GPU devices are made.
    pub(crate) gpu: arvane_compose::GpuEnv,
}

impl Env {
    /// Read the environment now, for GPU devices made as `preference`
    /// wishes where the environment does not say otherwise.
    pub(crate) fn read(preference: kivact_types::GpuPreference) -> Self {
        Self {
            renderer: match std::env::var("ARVANE_RENDERER").as_deref() {
                Ok("cpu") => Some(Renderer::Cpu),
                Ok("gpu") => Some(Renderer::Gpu),
                _ => None,
            },
            gpu: arvane_compose::GpuEnv::from_env(preference),
        }
    }
}
