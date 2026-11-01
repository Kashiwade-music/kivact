//! Role: Import: an engine's shared texture pool and its fences, opened on wgpu.

use crate::common::Error;
use kivact_types::{SharedTextureFormat, SharedTextures};
use std::time::Duration;

/// The longest the queue is waited for when an import is prepared or
/// dropped.
const IDLE: Duration = Duration::from_secs(5);

/// The backend half of an import: its fences and the raw work around
/// wgpu's submissions.
enum Raw {
    #[cfg(windows)]
    Dx12(super::dx12::Shared),
    #[cfg(any(windows, target_os = "linux"))]
    Vulkan(Box<super::vulkan::Shared>),
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
    if super::dx12::is_dx12(device) {
        return super::dx12::import(device, shared, desc)
            .map(|(textures, raw)| (textures, Raw::Dx12(raw)));
    }
    #[cfg(any(windows, target_os = "linux"))]
    if super::vulkan::is_vulkan(device) {
        return super::vulkan::import(device, shared, desc)
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
