//! Role: Creates a GPU context on a hardware adapter and device.
//!
//! It also reports the device failures the context keeps.

use super::export_pool::Exports;
use super::external::External;
use super::pipelines::Pipelines;
use super::shader_cache::Shaders;
use super::state::{DeviceLost, GpuContext, GpuEnv};
use super::targets::Pool;
use crate::ComposeError;
use crate::cache::CacheIndex;
use arvane_render::GpuRaster;
use std::collections::HashMap;
use std::sync::Arc;

impl GpuContext {
    /// Release rebuildable metadata: the previous frame's records, the free
    /// textures and the kept surfaces and chunks they describe. Imports stay.
    pub fn release_caches(&mut self) {
        self.cache = CacheIndex::default();
        self.pool.clear();
        self.chunks.clear();
        self.kept.clear();
    }

    /// Requests a hardware adapter and creates the executor's pipelines,
    /// as `env` says. Any thread may do it (windows' surfaces are made
    /// later, on their windows' threads: [`GpuContext::surface`]). Fails
    /// with [`ComposeError::NoAdapter`] when no backend has a hardware
    /// adapter.
    pub fn new(env: &GpuEnv) -> Result<Self, ComposeError> {
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
            ComposeError::Backend(message)
        } else {
            ComposeError::NoAdapter(message)
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
        let failure = Arc::new(DeviceLost::default());
        let lost = Arc::clone(&failure);
        device.set_device_lost_callback(move |reason, message| {
            lost.record(format!("device lost ({reason:?}): {message}"));
        });
        let uncaptured = Arc::clone(&failure);
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            uncaptured.record(error.to_string());
        }));
        let raster = GpuRaster::new(&device, &queue).map_err(|e| failed(&e))?;
        let pipelines = Pipelines::new(&device, &queue);
        let shaders = Shaders::new(&device, &queue);
        Ok(Self {
            external: External::default(),
            interop,
            instance,
            adapter,
            device,
            queue,
            raster,
            pipelines,
            failure,
            cache: CacheIndex::default(),
            pool: Pool::default(),
            kept: HashMap::new(),
            chunks: HashMap::new(),
            shaders,
            shader_failures: Vec::new(),
            exports: Exports::default(),
        })
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
        self.failure.message()
    }

    /// Fail when the device reported a failure.
    pub(crate) fn check(&self) -> Result<(), ComposeError> {
        match self.failure() {
            Some(message) => Err(ComposeError::Backend(message)),
            None => Ok(()),
        }
    }
}

/// One backend set's failure.
enum Attempt {
    /// No usable hardware adapter there.
    NoAdapter(String),
    /// An adapter was found, but its device or executor failed.
    Failed(String),
}

/// The adapter of `backends` the device is made on: the one the
/// application named, when it is there, else the one its power preference
/// picks.
fn adapter(
    instance: &wgpu::Instance,
    backends: wgpu::Backends,
    env: &GpuEnv,
) -> Result<wgpu::Adapter, Attempt> {
    if let Some(wanted) = env.preference.adapter {
        let named = pollster::block_on(instance.enumerate_adapters(backends))
            .into_iter()
            .filter(|adapter| adapter.get_info().device_type != wgpu::DeviceType::Cpu)
            .find(|adapter| {
                kivact_gpu_share::device::identity(adapter)
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
        let opened = kivact_gpu_share::device::request_device(adapter, desc)
            .map_err(|error| error.to_string())?;
        let interop = opened
            .interop
            .then(|| kivact_gpu_share::device::identity(adapter))
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
