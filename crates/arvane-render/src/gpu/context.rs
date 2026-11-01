//! Role: Creates a GpuContext on a hardware adapter and reports device failures.

use super::state::{FORMAT, Failure, GpuContext, GpuEnv};
use crate::draw;
use crate::error::RenderError;
use kivact_types::{Scene, Size};
use std::sync::{Arc, Mutex};
use vello_hybrid::{AtlasConfig, MemorySettings, RenderSettings, RenderTargetConfig, Renderer};

/// Glyph atlas pages at most (added as needed, `encode.rs`).
const GLYPH_PAGES: usize = 64;

impl GpuContext {
    /// Release rebuildable validation metadata without releasing imports or leases.
    pub fn release_caches(&mut self) {
        self.caches.validation.clear();
    }

    /// Requests a hardware adapter and creates the renderer's pipelines,
    /// as `env` says. Any thread may do it (windows' surfaces are made
    /// later, on their windows' threads: [`GpuContext::surface`]). Fails
    /// with [`RenderError::NoAdapter`] when no backend has a hardware
    /// adapter.
    pub fn new(env: &GpuEnv) -> Result<Self, RenderError> {
        let mut failures = Vec::new();
        let mut adapter_found = false;
        for backends in backend_order(env) {
            match Self::with_backends(backends, env) {
                Ok(context) => return Ok(context),
                Err(Attempt::NoAdapter(message)) => failures.push(message),
                Err(Attempt::Failed(message)) => {
                    adapter_found = true;
                    failures.push(message);
                }
            }
        }
        let message = failures.join("; ");
        Err(if adapter_found {
            RenderError::Backend(message)
        } else {
            RenderError::NoAdapter(message)
        })
    }

    fn with_backends(backends: wgpu::Backends, env: &GpuEnv) -> Result<Self, Attempt> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: env.flags,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = adapter(&instance, backends, env)?;
        let info = adapter.get_info();
        if info.device_type == wgpu::DeviceType::Cpu {
            return Err(Attempt::NoAdapter(format!(
                "{backends:?}: only the software adapter {} ({:?})",
                info.name, info.backend
            )));
        }
        let failed = |e: &dyn std::fmt::Display| {
            Attempt::Failed(format!("{} ({:?}): {e}", info.name, info.backend))
        };
        let desc = wgpu::DeviceDescriptor {
            label: Some("Arvane GPU"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
            // Smaller allocation blocks: a UI needs little GPU memory.
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        };
        let (device, queue, interop) = open_device(&adapter, &desc).map_err(|e| failed(&e))?;
        let failure = Failure::default();
        let lost = Arc::clone(&failure);
        device.set_device_lost_callback(move |reason, message| {
            record(&lost, format!("device lost ({reason:?}): {message}"));
        });
        let uncaptured = Arc::clone(&failure);
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            record(&uncaptured, error.to_string());
        }));
        let settings = RenderSettings {
            memory_settings: MemorySettings {
                image_atlas_config: AtlasConfig {
                    atlas_size: (super::encode::GLYPH_PAGE, super::encode::GLYPH_PAGE),
                    max_atlases: GLYPH_PAGES,
                    initial_atlas_count: 0,
                    ..AtlasConfig::default()
                },
                ..MemorySettings::default()
            },
            ..RenderSettings::default()
        };
        let (renderer, resources) = Renderer::new_with(
            &device,
            &RenderTargetConfig {
                format: FORMAT,
                width: 1,
                height: 1,
            },
            settings,
        );
        let mut context = Self {
            external: super::external::External::default(),
            interop,
            instance,
            adapter,
            device,
            queue,
            renderer,
            resources,
            scene: vello_hybrid::Scene::new(1, 1),
            caches: draw::Caches::default(),
            images: super::state::Images::new(),
            failure,
            offscreen: None,
        };
        // One small frame makes the renderer's first buffers here, not in
        // the first frame of a window.
        let blank = Scene::default();
        context
            .render(
                &blank,
                Size {
                    width: 16.0,
                    height: 16.0,
                },
                1.0,
            )
            .map_err(|e| failed(&e))?;
        context.offscreen = None;
        Ok(context)
    }

    /// The adapter and backend, for diagnostics ("NVIDIA ... (Vulkan)").
    pub fn describe(&self) -> String {
        let info = self.adapter.get_info();
        format!("{} ({:?})", info.name, info.backend)
    }

    /// The adapter, when the device can import textures an external
    /// render engine shares (feature `gpu-interop`, Direct3D 12 or Vulkan
    /// on Windows and Linux).
    pub fn interop_adapter(&self) -> Option<kivact_types::GpuAdapter> {
        self.interop.clone()
    }

    /// The first device failure (validation error or device loss), if any:
    /// the context is unusable once one happened.
    pub fn failure(&self) -> Option<String> {
        self.failure
            .lock()
            .expect("GPU failure lock poisoned: a panic already happened")
            .clone()
    }

    /// Fail when the device reported a failure.
    pub(crate) fn check(&self) -> Result<(), RenderError> {
        match self.failure() {
            Some(message) => Err(RenderError::Backend(message)),
            None => Ok(()),
        }
    }
}

/// One backend set's failure.
enum Attempt {
    /// No usable hardware adapter there.
    NoAdapter(String),
    /// An adapter was found, but its device or renderer failed.
    Failed(String),
}

/// The adapter of `backends` the device is made on: the one the
/// application named, when it is there (feature `gpu-interop`), else the
/// one its power preference picks.
fn adapter(
    instance: &wgpu::Instance,
    backends: wgpu::Backends,
    env: &GpuEnv,
) -> Result<wgpu::Adapter, Attempt> {
    #[cfg(feature = "gpu-interop")]
    if let Some(wanted) = env.preference.adapter {
        let named = pollster::block_on(instance.enumerate_adapters(backends))
            .into_iter()
            .filter(|adapter| adapter.get_info().device_type != wgpu::DeviceType::Cpu)
            .find(|adapter| {
                kivact_gpu_share::import::identity(adapter)
                    .is_some_and(|identity| gpu_adapter(adapter, identity, 0).matches(&wanted))
            });
        if let Some(adapter) = named {
            return Ok(adapter);
        }
    }
    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: env.power(),
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .map_err(|e| Attempt::NoAdapter(format!("{backends:?}: {e}")))
}

/// The device and queue of `adapter`, and the adapter's report when the
/// device can import shared textures (a Vulkan device gets the extensions
/// that needs where the adapter has them).
fn open_device(
    adapter: &wgpu::Adapter,
    desc: &wgpu::DeviceDescriptor<'_>,
) -> Result<(wgpu::Device, wgpu::Queue, Option<kivact_types::GpuAdapter>), String> {
    #[cfg(feature = "gpu-interop")]
    {
        let opened = kivact_gpu_share::import::request_device(adapter, desc)
            .map_err(|error| error.to_string())?;
        let interop = opened
            .interop
            .then(|| kivact_gpu_share::import::identity(adapter))
            .flatten()
            .map(|identity| gpu_adapter(adapter, identity, next_generation()));
        Ok((opened.device, opened.queue, interop))
    }
    #[cfg(not(feature = "gpu-interop"))]
    {
        let (device, queue) =
            pollster::block_on(adapter.request_device(desc)).map_err(|e| e.to_string())?;
        Ok((device, queue, None))
    }
}

/// The report of `adapter`, whose identity is `identity`.
#[cfg(feature = "gpu-interop")]
fn gpu_adapter(
    adapter: &wgpu::Adapter,
    identity: kivact_gpu_share::Identity,
    generation: u64,
) -> kivact_types::GpuAdapter {
    kivact_types::GpuAdapter {
        backend: identity.backend,
        name: adapter.get_info().name,
        luid: identity.luid,
        device_uuid: identity.device_uuid,
        driver_uuid: identity.driver_uuid,
        generation,
    }
}

/// The generation of a new interop device: 1 for the process's first.
#[cfg(feature = "gpu-interop")]
fn next_generation() -> u64 {
    static DEVICES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    DEVICES.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
}

/// The backend sets tried in order: the environment's or the
/// application's, else Arvane's.
fn backend_order(env: &GpuEnv) -> Vec<wgpu::Backends> {
    if let Some(backends) = env.backends() {
        return vec![backends];
    }
    if cfg!(windows) {
        // Vulkan starts fastest and its drivers cache compiled pipelines.
        vec![wgpu::Backends::VULKAN, wgpu::Backends::DX12]
    } else {
        vec![wgpu::Backends::PRIMARY]
    }
}

fn record(failure: &Mutex<Option<String>>, message: String) {
    failure
        .lock()
        .expect("GPU failure lock poisoned: a panic already happened")
        .get_or_insert(message);
}
