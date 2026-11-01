//! Role: Optional GPU rasterization (feature `gpu`).
//! Handles remain private and
//! frames use core vocabulary.
//!
//! Responsibility: one wgpu device and vello_hybrid renderer
//! ([`GpuContext`]), presenting scenes to window surfaces ([`GpuSurface`],
//! `surface.rs`) without reading pixels back, and reading frames back only
//! where pixels are needed (tests, screenshots).
//!
//! vello_hybrid is vello_cpu's pipeline with its last stage on the GPU: the
//! CPU flattens paths and makes sparse strips exactly as vello_cpu does
//! (`draw.rs` records the same calls for both), and ordinary render
//! pipelines (no compute shaders) fill the strips' pixels into the target.
//! Its GPU memory follows the window and the scene (strips, a depth
//! texture, glyph atlas pages once text is drawn, a texture per image,
//! layer textures once a layer is used), with no fixed working buffers.
//!
//! Adapter choice: hardware adapters only, through the backends that start
//! fastest first (Vulkan, then DirectX 12 on Windows; the platform's
//! primary backends elsewhere), or the backend the application asks for
//! (`GpuPreference`; `WGPU_BACKEND` overrides both); among them the
//! adapter the application names (feature `gpu-interop`, looked up by
//! LUID or UUIDs), else the one its power preference picks
//! (`WGPU_POWER_PREF` overrides). With `gpu-interop`, a Vulkan device is
//! opened with the extensions shared textures need when the adapter has
//! them, and the context reports its adapter (`interop_adapter`) when it
//! can import. Software adapters
//! (WARP, lavapipe) are refused: vello_hybrid runs on WARP, but a screen of
//! text takes 2.5 times as long as vello_cpu takes for it and the adapter
//! adds ~90 MB. The facade falls back to the CPU renderer instead (on
//! Linux the GPU is a hardware Vulkan adapter; WSLg offers only lavapipe,
//! without D3D12's Vulkan driver dozen, so it draws on the CPU). wgpu is
//! built with the Vulkan, DirectX 12 and Metal backends only; Vulkan
//! drivers cache the compiled pipelines between runs. The device asks for
//! `MemoryHints::MemoryUsage` (smaller allocation blocks: a UI needs
//! little GPU memory), and a new context draws one small blank frame, so
//! the renderer's first buffers are made then rather than in a window's
//! first frame. How devices are made is read from the environment once
//! ([`GpuEnv::from_env`]).
//!
//! The feature `gpu` of `kivact` (on by default) is forwarded through
//! `kivact-core`, `kivact-engine` and `arvane` to this crate's; without it
//! only the CPU renderer exists.
//!
//! Invariants:
//! - GPU resources are serially owned by their context; a context may be
//!   created on a worker thread and moved to the presenting thread.
//! - Validation errors and device loss never panic: they are recorded and
//!   reported by [`GpuContext::failure`] and by the next call, so the
//!   facade can recreate the device or fall back to the CPU.
//! - Readback waits are bounded.

mod encode;
pub(crate) mod external;
pub(crate) mod state;
mod surface;
mod texture;

use state::{FORMAT, Failure, GpuContext, GpuEnv, GpuSurface};
use texture::target_texture;

use crate::error::RenderError;
use crate::{draw, validate};
use kivact_types::{CpuFrame, Scene, Size};
use std::sync::{Arc, Mutex, PoisonError, mpsc};
use std::time::Duration;
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
                    atlas_size: (encode::GLYPH_PAGE, encode::GLYPH_PAGE),
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
            external: external::External::default(),
            interop,
            instance,
            adapter,
            device,
            queue,
            renderer,
            resources,
            scene: vello_hybrid::Scene::new(1, 1),
            caches: draw::Caches::default(),
            images: state::Images::new(),
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

    /// Import the pool of textures `shared` as the image `id` (replacing
    /// an image imported as `id` before), for scenes drawn with this
    /// context. Fails with [`RenderError::External`] when the textures do
    /// not fit the device (another adapter or graphics API, a size beyond
    /// its limits, handles it refuses) or it cannot import.
    #[cfg(feature = "gpu-interop")]
    pub fn import_external(
        &mut self,
        id: kivact_types::ExternalImageId,
        shared: kivact_types::SharedTextures,
    ) -> Result<(), RenderError> {
        self.check()?;
        let Some(adapter) = &self.interop else {
            return Err(RenderError::External(
                "the GPU device cannot import shared textures".into(),
            ));
        };
        self.external
            .import(&self.device, &self.queue, adapter, id, shared)?;
        self.check()
    }

    /// Drop the image `id` (nothing when it is not imported): its frames
    /// draw nothing from now on, and its release fence is signaled past
    /// every frame of it.
    #[cfg(feature = "gpu-interop")]
    pub fn drop_external(&mut self, id: kivact_types::ExternalImageId) {
        self.external.drop_image(id);
    }

    /// The external frames scenes named that were not drawn since the last
    /// call: not ready within the frame's wait, or no longer held (each
    /// noted once).
    pub fn take_missed_external(&mut self) -> Vec<kivact_types::ExternalFrame> {
        self.external.take_missed()
    }

    /// The first device failure (validation error or device loss), if any:
    /// the context is unusable once one happened.
    pub fn failure(&self) -> Option<String> {
        self.failure
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// A surface presenting to the window `target` (any window handle that
    /// keeps its window alive on any thread) with this context's device.
    /// Call it on the thread that runs the window's event loop (windowing
    /// systems hand out window handles there only), or through
    /// [`GpuContext::surface_maker`].
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

    /// [`GpuContext::surface`] for another thread: a window's own thread
    /// makes the surface and hands it back.
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

    /// Renders a validated scene offscreen into premultiplied RGBA8 and
    /// reads it back (tests and screenshots; windows use
    /// [`GpuContext::present`]).
    pub fn render(
        &mut self,
        scene: &Scene,
        size: Size,
        scale: f32,
    ) -> Result<CpuFrame, RenderError> {
        // Whatever happens, the frame gives back what it acquired.
        let rendered = match self.external.begin(scene) {
            Ok(()) => self.render_frame(scene, size, scale),
            Err(error) => Err(error),
        };
        let ended = self.external.end();
        let frame = rendered?;
        ended?;
        Ok(frame)
    }

    /// [`GpuContext::render`] between the external frames' hand-offs.
    fn render_frame(
        &mut self,
        scene: &Scene,
        size: Size,
        scale: f32,
    ) -> Result<CpuFrame, RenderError> {
        let ((width, height), info) = self.dimensions(scene, size, scale)?;
        let reuse = self
            .offscreen
            .as_ref()
            .is_some_and(|texture| (texture.width(), texture.height()) == (width, height));
        if !reuse {
            self.offscreen = Some(target_texture(&self.device, width, height));
        }
        let texture = self
            .offscreen
            .clone()
            .expect("invariant: the offscreen texture was just made");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Arvane offscreen"),
            });
        self.encode(scene, scale, (width, height), &info, false);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let drawn = self.draw(&mut encoder, (width, height), &view);
        self.queue.submit([encoder.finish()]);
        self.external.submitted();
        drawn?;
        self.read_texture(&texture)
    }

    /// Validate `scene` and the frame size against the renderer's budget
    /// and the device's texture limit, returning the pixel dimensions and
    /// what drawing the scene needs to know.
    pub(crate) fn dimensions(
        &mut self,
        scene: &Scene,
        size: Size,
        scale: f32,
    ) -> Result<((u32, u32), validate::SceneInfo), RenderError> {
        let (width, height) = validate::dimensions(size, scale)?;
        let info = validate::scene(scene, &mut self.caches.validation)?;
        let (width, height) = (u32::from(width), u32::from(height));
        let max = self.device.limits().max_texture_dimension_2d;
        if width > max || height > max {
            return Err(RenderError::InvalidSize);
        }
        self.check()?;
        Ok(((width, height), info))
    }

    /// Draw the recorded scene into `view` (a `width` × `height` texture
    /// in [`FORMAT`]) with `encoder`.
    pub(crate) fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        (width, height): (u32, u32),
        view: &wgpu::TextureView,
    ) -> Result<(), RenderError> {
        self.renderer
            .render(
                &self.scene,
                &mut self.resources,
                &self.device,
                &self.queue,
                encoder,
                &vello_hybrid::RenderSize { width, height },
                view,
                &self.images.bindings,
            )
            .map_err(|e| match e {
                // What the scene asks of the renderer's atlases and
                // textures, not a device failure.
                vello_hybrid::RenderError::AtlasError(_)
                | vello_hybrid::RenderError::MissingTextureBinding(_)
                | vello_hybrid::RenderError::IntermediateTexture(_) => {
                    RenderError::GpuLimit(e.to_string())
                }
            })
    }

    /// Fail when the device reported a failure.
    pub(crate) fn check(&self) -> Result<(), RenderError> {
        match self.failure() {
            Some(message) => Err(RenderError::Backend(message)),
            None => Ok(()),
        }
    }

    /// Copy `texture` (in [`FORMAT`]) into an RGBA frame, waiting a
    /// bounded time.
    pub(crate) fn read_texture(&self, texture: &wgpu::Texture) -> Result<CpuFrame, RenderError> {
        let (width, height) = (texture.width(), texture.height());
        // wgpu requires rows aligned to COPY_BYTES_PER_ROW_ALIGNMENT during copy.
        let row_bytes = width * 4;
        let stride = row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Arvane raster readback"),
            size: u64::from(stride) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Arvane readback"),
            });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(height),
                },
            },
            texture.size(),
        );
        let submission = self.queue.submit([encoder.finish()]);
        let slice = buffer.slice(..);
        let (sender, receiver) = mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(30)),
            })
            .map_err(backend)?;
        receiver
            .recv_timeout(Duration::from_secs(1))
            .map_err(backend)?
            .map_err(backend)?;
        let data = slice.get_mapped_range();
        let mut rgba = Vec::with_capacity((row_bytes * height) as usize);
        for row in data.chunks_exact(stride as usize) {
            // BGRA to RGBA.
            for [b, g, r, a] in row[..row_bytes as usize].as_chunks::<4>().0 {
                rgba.extend_from_slice(&[*r, *g, *b, *a]);
            }
        }
        drop(data);
        buffer.unmap();
        self.check()?;
        Ok(CpuFrame {
            width,
            height,
            rgba,
        })
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

fn make_surface<T>(
    instance: &wgpu::Instance,
    adapter: &wgpu::Adapter,
    target: T,
) -> Result<GpuSurface, RenderError>
where
    T: raw_window_handle::HasWindowHandle
        + raw_window_handle::HasDisplayHandle
        + Send
        + Sync
        + 'static,
{
    let surface = instance
        .create_surface(target)
        .map_err(|e| RenderError::Surface(e.to_string()))?;
    if !adapter.is_surface_supported(&surface) {
        return Err(RenderError::Surface(
            "the GPU adapter cannot present to this window".into(),
        ));
    }
    Ok(GpuSurface::new(surface))
}

fn record(failure: &Mutex<Option<String>>, message: String) {
    failure
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_or_insert(message);
}

pub(crate) fn backend(error: impl std::fmt::Display) -> RenderError {
    RenderError::Backend(error.to_string())
}
