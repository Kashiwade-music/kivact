//! Role: Environment variable configuration for Arvane.
//!
//! The environment variables Arvane reads, all read here: once per native
//! window loop ([`crate::Shell::run`]) or test window, never on a hot
//! path.
//!
//! - `ARVANE_RENDERER`: `cpu` draws a native window on the CPU, `gpu` on
//!   the GPU with the device started with the window, `auto` starts the
//!   device when it is needed, overriding the host's preference. Stepped
//!   shells read it through the same rendering driver.
//! - `WGPU_BACKEND`, `WGPU_POWER_PREF` and the instance flags wgpu reads
//!   (`WGPU_VALIDATION`, `WGPU_DEBUG` ..): how GPU devices are made
//!   (feature `gpu`; read by `arvane-render`, which alone knows wgpu). The
//!   first two override the host's [`kivact_types::GpuPreference`].

use kivact_types::gpu::GpuStart;

/// How a native window's renderer starts, decided once per loop
/// ([`Env::read`]): `ARVANE_RENDERER` (`cpu`, `gpu`, `auto`), else the host's
/// `prefer_gpu` and `GpuPreference::start`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RendererStart {
    /// Frames are drawn on the CPU only; no device ever starts.
    Cpu,
    /// The device starts on a worker with the window.
    GpuAtLaunch,
    /// The device starts when the window needs it.
    GpuOnDemand,
}

impl RendererStart {
    fn resolve(forced: Option<&str>, prefer_gpu: bool, start: GpuStart) -> Self {
        match forced {
            Some("cpu") => Self::Cpu,
            Some("gpu") => Self::GpuAtLaunch,
            Some("auto") => Self::GpuOnDemand,
            _ if !prefer_gpu => Self::Cpu,
            _ => match start {
                GpuStart::AtLaunch => Self::GpuAtLaunch,
                GpuStart::OnDemand => Self::GpuOnDemand,
            },
        }
    }
}

/// What the environment and the host ask of Arvane's renderers.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Env {
    /// How every window's renderer starts.
    pub(crate) start: RendererStart,
    /// How GPU devices are made.
    pub(crate) gpu: arvane_compose::GpuEnv,
}

impl Env {
    /// Read the environment now, for windows the host wants drawn on the GPU
    /// (`prefer_gpu`) with devices made and started as `preference` wishes.
    pub(crate) fn read(prefer_gpu: bool, preference: kivact_types::GpuPreference) -> Self {
        let forced = std::env::var("ARVANE_RENDERER").ok();
        Self {
            start: RendererStart::resolve(forced.as_deref(), prefer_gpu, preference.start),
            gpu: arvane_compose::GpuEnv::from_env(preference),
        }
    }
}
