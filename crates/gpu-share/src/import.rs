//! Role: Imports an external engine's GPU textures and fences into wgpu.
//!
//! Responsibility: what GPU texture sharing needs below wgpu's safe API: reading an adapter's identity (DXGI's LUID, Vulkan's
//! device and driver UUIDs), opening a Vulkan device with the external
//! semaphore extensions wgpu does not enable, importing a pool of shared
//! textures and its two timeline fences ([`ui_types::SharedTextures`],
//! whose rustdoc is the engine's contract) into a wgpu device, and the
//! raw work on wgpu's queue around wgpu's own submissions.
//! `arvane-render` uses it (feature `gpu-interop`); the facade never names
//! a wgpu, ash or windows type.
//!
//! Hand-off, per frame of the presenting thread:
//! - [`Import::ready_value`] and [`Import::wait_ready`] read the ready
//!   fence on the CPU (bounded), so the queue is never made to wait on the
//!   GPU for a frame that has not arrived.
//! - [`Import::acquire`], before the frame's submission: a GPU-side wait
//!   for the ready value (already reached), then the texture taken from the
//!   engine (Vulkan: owned by `VK_QUEUE_FAMILY_EXTERNAL` in
//!   `SHADER_READ_ONLY_OPTIMAL`, acquired keeping the layout; Direct3D 12:
//!   `COMMON` to the shader-resource state wgpu's tracker holds).
//! - [`Import::release`] after it: the texture given back, and
//!   [`Import::signal`] the release fence (after every earlier submission,
//!   by submission order).
//!
//! An import is prepared once ([`Import::new`]): wgpu's tracker is moved
//! to the sampled state (`transition_resources`) while the textures hold
//! nothing yet, and each texture is given to the engine, so wgpu's first
//! use does not start from an undefined layout.
//!
//! Invariants:
//! - `unsafe` lives in the backend modules (`dx12`, `vulkan`), each block
//!   with its `SAFETY` note; the crate root denies it.
//! - The raw submissions go to wgpu's own queue (`VkQueue`,
//!   `ID3D12CommandQueue`), which needs external synchronization with
//!   wgpu's `Queue::submit`, `present` and `poll`: they are made only on
//!   the thread that owns the device's context, between its calls to wgpu
//!   (`arvane-render`'s `GpuContext` is used serially on its owner's
//!   thread).
//! - An import keeps a clone of its `wgpu::Device`, so the native device
//!   outlives the raw objects; dropping it waits (bounded) for the queue
//!   before destroying them, and leaks them instead when the queue does
//!   not finish.
//! - Waits are bounded; failures are `Err`, never panics.
//!
//! Platforms: Direct3D 12 (Windows) and Vulkan (Windows, Linux with
//! `OPAQUE_FD` handles). Elsewhere [`identity`] is `None`, devices open as
//! wgpu opens them and every import fails.

#![deny(unsafe_code)]

#[cfg(windows)]
mod dx12;
#[cfg(any(windows, target_os = "linux"))]
mod vulkan;

use crate::common::{Error, Identity, api};

/// A device opened with the shared-texture extensions when available.
pub struct Opened {
    /// The device.
    pub device: ::wgpu::Device,
    /// Its queue.
    pub queue: ::wgpu::Queue,
    /// Whether shared textures can be imported.
    pub interop: bool,
}
use std::time::Duration;
use ui_types::{SharedTextureFormat, SharedTextures};

/// The longest the queue is waited for when an import is prepared or
/// dropped.
const IDLE: Duration = Duration::from_secs(5);

/// The identity of `adapter` (`None`: neither Direct3D 12 nor Vulkan, or
/// unreadable).
pub fn identity(adapter: &::wgpu::Adapter) -> Option<Identity> {
    #[cfg(windows)]
    if let Some(identity) = dx12::identity(adapter) {
        return Some(identity);
    }
    #[cfg(any(windows, target_os = "linux"))]
    if let Some(identity) = vulkan::identity(adapter) {
        return Some(identity);
    }
    let _ = adapter;
    None
}

/// `adapter.request_device(desc)`, able to import shared textures where
/// the adapter can: a Vulkan device gets the external memory and
/// semaphore extensions (and needs Vulkan 1.2's timeline semaphores); an
/// adapter without them gets an ordinary device that cannot import.
pub fn request_device(
    adapter: &::wgpu::Adapter,
    desc: &::wgpu::DeviceDescriptor<'_>,
) -> Result<Opened, Error> {
    #[cfg(any(windows, target_os = "linux"))]
    if let Some(opened) = vulkan::request_device(adapter, desc) {
        let (device, queue) = opened?;
        return Ok(Opened {
            device,
            queue,
            interop: true,
        });
    }
    let (device, queue) =
        pollster::block_on(adapter.request_device(desc)).map_err(api("request_device"))?;
    let interop = cfg!(windows) && adapter.get_info().backend == ::wgpu::Backend::Dx12;
    Ok(Opened {
        device,
        queue,
        interop,
    })
}

/// The backend half of an import: its fences and the raw work around
/// wgpu's submissions.
enum Raw {
    #[cfg(windows)]
    Dx12(dx12::Shared),
    #[cfg(any(windows, target_os = "linux"))]
    Vulkan(Box<vulkan::Shared>),
    /// No backend imports on this platform.
    #[cfg(not(any(windows, target_os = "linux")))]
    #[expect(dead_code, reason = "never made")]
    Unsupported(std::convert::Infallible),
}

/// The textures wgpu sees and the backend half, as a backend imports them.
type Imported = (Vec<::wgpu::Texture>, Raw);

/// A pool of shared textures and its fences, imported into a wgpu device
/// (see this module's documentation for the hand-off).
pub struct Import {
    textures: Vec<::wgpu::Texture>,
    width: u32,
    height: u32,
    raw: Raw,
}

impl Import {
    /// Import `shared` into `device` (a device of [`request_device`] that
    /// can import), checking what can be checked against it, and prepare
    /// the textures for the hand-off (waits for the queue, bounded). The
    /// handles are taken: closed once opened (Windows) or given to the
    /// driver (Linux; closed when the import fails).
    pub fn new(
        device: &::wgpu::Device,
        queue: &::wgpu::Queue,
        shared: SharedTextures,
    ) -> Result<Self, Error> {
        let SharedTextureFormat::Rgba8UnormPremultiplied = shared.format;
        let (width, height) = (shared.width, shared.height);
        let max = device
            .limits()
            .max_texture_dimension_2d
            .min(u32::from(u16::MAX));
        if width == 0 || height == 0 || width > max || height > max {
            return Err(Error::Mismatch(format!(
                "the textures' size {width} x {height} is outside 1 ..= {max}"
            )));
        }
        if shared.textures.is_empty() || u32::try_from(shared.textures.len()).is_err() {
            return Err("no textures to import".into());
        }
        let desc = ::wgpu::TextureDescriptor {
            label: Some("Arvane shared texture"),
            size: ::wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: ::wgpu::TextureDimension::D2,
            format: ::wgpu::TextureFormat::Rgba8Unorm,
            usage: ::wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let (textures, raw) = import(device, shared, &desc)?;
        // Made first, so a failure below still waits for the queue before
        // the raw objects go.
        let mut import = Self {
            textures,
            width,
            height,
            raw,
        };
        // wgpu's tracker holds the sampled state from now on, reached
        // before the engine writes anything; then the engine owns them.
        let mut encoder = device.create_command_encoder(&::wgpu::CommandEncoderDescriptor {
            label: Some("Arvane shared texture preparation"),
        });
        encoder.transition_resources(
            std::iter::empty(),
            import
                .textures
                .iter()
                .map(|texture| ::wgpu::TextureTransition {
                    texture,
                    selector: None,
                    state: ::wgpu::TextureUses::RESOURCE,
                }),
        );
        queue.submit([encoder.finish()]);
        for slot in 0..import.textures.len() {
            import.raw.release(slot)?;
        }
        if !import.raw.idle(IDLE) {
            return Err(Error::Timeout("shared texture preparation"));
        }
        Ok(import)
    }

    /// The pool's textures, by slot.
    pub fn textures(&self) -> &[::wgpu::Texture] {
        &self.textures
    }

    /// The textures' pixel size.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// The ready fence's value now (0 when it cannot be read).
    pub fn ready_value(&self) -> u64 {
        self.raw.ready_value()
    }

    /// Wait at most `timeout` for the ready fence to reach `value`;
    /// whether it did.
    pub fn wait_ready(&self, value: u64, timeout: Duration) -> bool {
        self.raw.ready_value() >= value || self.raw.wait_ready(value, timeout)
    }

    /// Before the submissions that sample texture `slot`: wait on the GPU
    /// for the ready fence to reach `value` and take the texture from the
    /// engine. Call it only once [`Import::wait_ready`] said `value` was
    /// reached, on the thread that submits to the device's queue.
    pub fn acquire(&mut self, slot: usize, value: u64) -> Result<(), Error> {
        self.raw.acquire(slot, value)
    }

    /// After the last submission that samples texture `slot` (of those
    /// since its [`Import::acquire`]): give it back to the engine.
    pub fn release(&mut self, slot: usize) -> Result<(), Error> {
        self.raw.release(slot)
    }

    /// Signal the release fence to `value` after every earlier submission
    /// (a value above the last one signaled).
    pub fn signal(&mut self, value: u64) -> Result<(), Error> {
        self.raw.signal(value)
    }
}

impl Drop for Import {
    fn drop(&mut self) {
        // The raw objects must outlive the GPU's use of them.
        if !self.raw.idle(IDLE) {
            self.raw.leak();
        }
    }
}

/// Import `shared` with `device`'s backend.
fn import(
    device: &::wgpu::Device,
    shared: SharedTextures,
    desc: &::wgpu::TextureDescriptor<'_>,
) -> Result<Imported, Error> {
    #[cfg(windows)]
    if dx12::is_dx12(device) {
        return dx12::import(device, shared, desc)
            .map(|(textures, raw)| (textures, Raw::Dx12(raw)));
    }
    #[cfg(any(windows, target_os = "linux"))]
    if vulkan::is_vulkan(device) {
        return vulkan::import(device, shared, desc)
            .map(|(textures, raw)| (textures, Raw::Vulkan(Box::new(raw))));
    }
    let _ = (device, shared, desc);
    Err("this device cannot import shared textures".into())
}

impl Raw {
    fn ready_value(&self) -> u64 {
        match self {
            #[cfg(windows)]
            Self::Dx12(raw) => raw.ready_value(),
            #[cfg(any(windows, target_os = "linux"))]
            Self::Vulkan(raw) => raw.ready_value(),
            #[cfg(not(any(windows, target_os = "linux")))]
            Self::Unsupported(never) => match *never {},
        }
    }

    fn wait_ready(&self, value: u64, timeout: Duration) -> bool {
        match self {
            #[cfg(windows)]
            Self::Dx12(raw) => raw.wait_ready(value, timeout),
            #[cfg(any(windows, target_os = "linux"))]
            Self::Vulkan(raw) => raw.wait_ready(value, timeout),
            #[cfg(not(any(windows, target_os = "linux")))]
            Self::Unsupported(never) => match *never {},
        }
    }

    fn acquire(&mut self, slot: usize, value: u64) -> Result<(), Error> {
        match self {
            #[cfg(windows)]
            Self::Dx12(raw) => raw.acquire(slot, value),
            #[cfg(any(windows, target_os = "linux"))]
            Self::Vulkan(raw) => raw.acquire(slot, value),
            #[cfg(not(any(windows, target_os = "linux")))]
            Self::Unsupported(never) => match *never {},
        }
    }

    fn release(&mut self, slot: usize) -> Result<(), Error> {
        match self {
            #[cfg(windows)]
            Self::Dx12(raw) => raw.release(slot),
            #[cfg(any(windows, target_os = "linux"))]
            Self::Vulkan(raw) => raw.release(slot),
            #[cfg(not(any(windows, target_os = "linux")))]
            Self::Unsupported(never) => match *never {},
        }
    }

    fn signal(&mut self, value: u64) -> Result<(), Error> {
        match self {
            #[cfg(windows)]
            Self::Dx12(raw) => raw.signal(value),
            #[cfg(any(windows, target_os = "linux"))]
            Self::Vulkan(raw) => raw.signal(value),
            #[cfg(not(any(windows, target_os = "linux")))]
            Self::Unsupported(never) => match *never {},
        }
    }

    /// Wait at most `timeout` until the queue finished everything
    /// submitted so far, raw submissions included; whether it did.
    fn idle(&mut self, timeout: Duration) -> bool {
        match self {
            #[cfg(windows)]
            Self::Dx12(raw) => raw.idle(timeout),
            #[cfg(any(windows, target_os = "linux"))]
            Self::Vulkan(raw) => raw.idle(timeout),
            #[cfg(not(any(windows, target_os = "linux")))]
            Self::Unsupported(never) => match *never {},
        }
    }

    /// Keep the raw objects alive for the process (the GPU may still use
    /// them).
    fn leak(&mut self) {
        match self {
            #[cfg(windows)]
            Self::Dx12(raw) => raw.leak(),
            #[cfg(any(windows, target_os = "linux"))]
            Self::Vulkan(raw) => raw.leak(),
            #[cfg(not(any(windows, target_os = "linux")))]
            Self::Unsupported(never) => match *never {},
        }
    }
}
