//! Role: A driver's GPU device: adapter, queue, pipelines, rasterizer, shaders and imported images.
//!
//! The device is made once on a worker thread and shared by the windows' contexts
//! ([`super::state::GpuContext`]). The windows' surfaces are made from it, and configured
//! by a [`SurfaceConfigurer`] on a worker thread.

use super::external::External;
use super::pipelines::Pipelines;
use super::shader_cache::Shaders;
use super::state::{DeviceLost, GpuEnv, GpuSurface};
use super::surface::{configure as configure_surface, make_surface};
use crate::RenderError;
use arvane_render::GpuRaster;
use kivact_types::GpuAdapter;
use std::cell::RefCell;
use std::sync::Arc;

// xtask-allow(state): raster, shaders and external are independent caches of one device, each borrowed alone; any combination of their contents is valid
/// One GPU device shared by every window of a driver (PERF-8). Made on a
/// worker thread (`GpuDevice::new` blocks), then used on the UI thread.
pub struct GpuDevice {
    /// The images external engines share (first: dropped, and their
    /// fences signaled, before the device).
    pub(crate) external: RefCell<External>,
    /// The adapter, when the device can import shared textures.
    pub(crate) interop: Option<GpuAdapter>,
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) failure: Arc<DeviceLost>,
    pub(crate) pipelines: Pipelines,
    /// Draws the chunks (Vello's GPU stage).
    pub(crate) raster: RefCell<GpuRaster>,
    /// The user shaders' pipelines.
    pub(crate) shaders: RefCell<Shaders>,
}

/// What configuring a window's surface needs, sendable to a worker thread.
#[derive(Clone)]
pub struct SurfaceConfigurer {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    failure: Arc<DeviceLost>,
}

impl SurfaceConfigurer {
    /// Configure `surface` for a `width` × `height` window (a new size makes a new swapchain).
    pub fn configure(
        &self,
        surface: &mut GpuSurface,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        configure_surface(
            &self.device,
            &self.adapter,
            surface,
            width.max(1),
            height.max(1),
            true,
        )?;
        self.failure.check()
    }
}

impl GpuDevice {
    /// Requests a hardware adapter and creates the device's pipelines, as `env` says. Any
    /// thread may do it (windows' surfaces are made later: [`GpuDevice::surface`]). Fails
    /// with [`RenderError::NoAdapter`] when no backend has a hardware adapter.
    pub fn new(env: &GpuEnv) -> Result<Self, RenderError> {
        let mut failures = Vec::new();
        let mut adapter_found = false;
        for backends in backend_order(env) {
            match Self::with_backends(backends, env) {
                Ok(device) => return Ok(device),
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
            external: RefCell::new(External::default()),
            interop,
            instance,
            adapter,
            device,
            queue,
            failure,
            pipelines,
            raster: RefCell::new(raster),
            shaders: RefCell::new(shaders),
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
    pub fn interop_adapter(&self) -> Option<GpuAdapter> {
        self.interop.clone()
    }

    /// The first device failure (validation error or device loss), if any:
    /// the device is unusable once one happened.
    pub fn failure(&self) -> Option<String> {
        self.failure.message()
    }

    /// Fail when the device reported a failure.
    pub(crate) fn check(&self) -> Result<(), RenderError> {
        self.failure.check()
    }

    /// What configuring a window's surface on this device needs, for a worker thread.
    pub fn configurer(&self) -> SurfaceConfigurer {
        SurfaceConfigurer {
            adapter: self.adapter.clone(),
            device: self.device.clone(),
            failure: Arc::clone(&self.failure),
        }
    }

    /// A window surface made on this device's instance and adapter.
    pub fn surface<T>(&self, target: T) -> Result<GpuSurface, RenderError>
    where
        T: raw_window_handle::HasWindowHandle
            + raw_window_handle::HasDisplayHandle
            + Send
            + Sync
            + 'static,
    {
        make_surface(&self.instance, &self.adapter, target)
    }

    /// A function making a window surface on this device's instance, for another thread.
    pub fn surface_maker<T>(
        &self,
    ) -> impl FnOnce(T) -> Result<GpuSurface, RenderError> + Send + 'static + use<T>
    where
        T: raw_window_handle::HasWindowHandle
            + raw_window_handle::HasDisplayHandle
            + Send
            + Sync
            + 'static,
    {
        let (instance, adapter) = (self.instance.clone(), self.adapter.clone());
        move |target| make_surface(&instance, &adapter, target)
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
                kivact_gpu_share::adapter::identity(adapter)
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
            .then(|| kivact_gpu_share::adapter::identity(adapter))
            .flatten()
            .map(|ids| gpu_adapter(adapter, ids, next_generation()));
        Ok((opened.device, opened.queue, interop))
    }
    #[cfg(not(feature = "gpu-interop"))]
    {
        let (device, queue) =
            pollster::block_on(adapter.request_device(desc)).map_err(|e| e.to_string())?;
        Ok((device, queue, None))
    }
}

/// The report of `adapter`, whose identity is `ids`.
fn gpu_adapter(
    adapter: &wgpu::Adapter,
    ids: kivact_types::AdapterIds,
    generation: u64,
) -> kivact_types::GpuAdapter {
    kivact_types::GpuAdapter {
        ids,
        name: adapter.get_info().name,
        generation,
    }
}

/// The generation of a new interop device: 1 for the process's first.
#[cfg(feature = "gpu-interop")]
fn next_generation() -> u64 {
    static DEVICES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    DEVICES.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
}

/// The name of the adapter [`GpuDevice::new`] would open, without opening a
/// device. `Err` when no backend has a hardware adapter.
pub fn probe_adapter(env: &GpuEnv) -> Result<String, RenderError> {
    let mut failures = Vec::new();
    for backends in backend_order(env) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: env.flags,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        match adapter(&instance, backends, env) {
            Ok(adapter) => {
                let info = adapter.get_info();
                if info.device_type != wgpu::DeviceType::Cpu {
                    return Ok(format!("{} ({:?})", info.name, info.backend));
                }
                failures.push(format!(
                    "{backends:?}: only the software adapter {} ({:?})",
                    info.name, info.backend
                ));
            }
            Err(Attempt::NoAdapter(message) | Attempt::Failed(message)) => failures.push(message),
        }
    }
    Err(RenderError::NoAdapter(failures.join("; ")))
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
