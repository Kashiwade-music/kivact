//! Role: GPU adapter choice and textures shared with an external render engine.
//!
//! The vocabulary of GPU texture sharing: an application's render engine
//! (its own device, not Arvane's) draws frames into a pool of textures and
//! hands their OS handles to a window ([`SharedTextures`]); a scene draws
//! one of its frames with `RenderCmd::External`. The types exist
//! on every platform; only Windows and Linux can make a [`SharedHandle`].
//!
//! The adapter is negotiated: the application states a wish
//! ([`GpuPreference`]) and Arvane reports the adapter it uses
//! ([`GpuAdapter`]), which the engine opens its device on.

/// A graphics API whose textures can be shared.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GpuBackend {
    /// Direct3D 12 (Windows).
    Dx12,
    /// Vulkan (Windows and Linux).
    Vulkan,
}

/// Which kind of adapter an application prefers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GpuPower {
    /// The integrated adapter, when there is one (the default).
    #[default]
    LowPower,
    /// The discrete adapter, when there is one.
    HighPerformance,
}

/// An adapter's identity, as the graphics APIs report it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AdapterMatch {
    /// DXGI's adapter LUID (also Vulkan's `deviceLUID` on Windows), as the
    /// 8 bytes of `LowPart` then `HighPart`, little-endian.
    Luid([u8; 8]),
    /// Vulkan's `deviceUUID` and `driverUUID`
    /// (`VkPhysicalDeviceIDProperties`).
    Uuid {
        /// `deviceUUID`.
        device: [u8; 16],
        /// `driverUUID`.
        driver: [u8; 16],
    },
}

/// When the window's GPU device is started.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GpuStart {
    /// Frames are drawn on the CPU until the window needs the GPU: a
    /// composition with a user shader or a GPU export, or CPU frames that
    /// keep missing half the display period (the default).
    #[default]
    OnDemand,
    /// Start the device with the window (frames are drawn on the CPU until
    /// it is ready), as before.
    AtLaunch,
}

/// What an application wishes of the GPU device Arvane makes. A wish, not
/// a requirement: Arvane takes the adapter asked for when it has one, else
/// the one the power preference picks, and reports what it took
/// ([`GpuAdapter`]). The environment variables `WGPU_BACKEND` and
/// `WGPU_POWER_PREF` override `backend` and `power`.
///
/// The default is Arvane's own choice: low power, any backend (Vulkan,
/// then Direct3D 12 on Windows), any adapter, started when it is needed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GpuPreference {
    /// The kind of adapter.
    pub power: GpuPower,
    // xtask-allow(state-fields): option: absent means the default (Arvane's backend order)
    /// The graphics API (`None`: Arvane's order).
    pub backend: Option<GpuBackend>,
    // xtask-allow(state-fields): option: absent means the default (as `power` picks)
    /// The adapter (`None`: as `power` picks). Only builds with GPU
    /// texture sharing look adapters up by identity; others ignore it.
    pub adapter: Option<AdapterMatch>,
    /// When the device is started.
    pub start: GpuStart,
}

/// The adapter a window's GPU device is on, when the device can import
/// shared textures. Direct3D 12 reports the LUID; Vulkan reports the device
/// and driver UUIDs, and on Windows the LUID as well.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GpuAdapter {
    /// The graphics API of the device.
    pub backend: GpuBackend,
    /// The adapter's name, for diagnostics.
    pub name: String,
    // xtask-allow(state-fields): identity: DXGI's (or Vulkan's) adapter LUID, in `AdapterMatch::Luid`'s bytes
    /// DXGI's (or Vulkan's) adapter LUID ([`AdapterMatch::Luid`]'s bytes).
    pub luid: Option<[u8; 8]>,
    // xtask-allow(state-fields): identity: Vulkan's `deviceUUID`
    /// Vulkan's `deviceUUID`.
    pub device_uuid: Option<[u8; 16]>,
    // xtask-allow(state-fields): identity: Vulkan's `driverUUID`
    /// Vulkan's `driverUUID`.
    pub driver_uuid: Option<[u8; 16]>,
    // xtask-allow(state-fields): identity: the device instance; a new generation means every image imported before is gone
    /// The device's generation: 1 for the process's first GPU device, one
    /// more for each device made after it. A new generation means every
    /// image imported into the window before is gone.
    pub generation: u64,
}

impl GpuAdapter {
    /// Whether this is the adapter `identity` names.
    pub fn matches(&self, identity: &AdapterMatch) -> bool {
        match identity {
            AdapterMatch::Luid(luid) => self.luid.as_ref() == Some(luid),
            AdapterMatch::Uuid { device, driver } => {
                self.device_uuid.as_ref() == Some(device)
                    && self.driver_uuid.as_ref() == Some(driver)
            }
        }
    }
}

/// A window's name for an image imported into it
/// ([`SharedTextures`]); the host chooses it. An image belongs to the
/// window it was imported into: other windows do not know the id.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExternalImageId(pub u64);

/// One frame of an imported image: the texture of the pool it is in
/// (`slot`, an index into [`SharedTextures::textures`]) and the value the
/// engine signaled the ready fence to once it was drawn (`value`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExternalFrame {
    /// The image.
    pub image: ExternalImageId,
    /// The texture of the pool holding the frame.
    pub slot: u32,
    /// The ready fence's value that says the frame is drawn.
    pub value: u64,
}

/// The pixel format of shared textures.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SharedTextureFormat {
    /// RGBA, 8-bit unsigned normalized channels, premultiplied alpha
    /// (`DXGI_FORMAT_R8G8B8A8_UNORM`, `VK_FORMAT_R8G8B8A8_UNORM`).
    ///
    /// The values are sRGB-encoded, as `Pixels` and CSS colors are: the
    /// window composites them as they are (premultiplied by alpha), and
    /// does not sample the texture through an sRGB view. An engine that
    /// computes in linear light encodes to sRGB as the last step before
    /// the values reach the texture.
    #[default]
    Rgba8UnormPremultiplied,
}

#[cfg(windows)]
type OsHandle = std::os::windows::io::OwnedHandle;
#[cfg(target_os = "linux")]
type OsHandle = std::os::fd::OwnedFd;
/// No platform handle: the type cannot be made.
#[cfg(not(any(windows, target_os = "linux")))]
#[derive(Debug)]
enum OsHandle {}

/// An owned OS handle of a shared texture or fence: an NT handle on
/// Windows ([`std::os::windows::io::OwnedHandle`]), a file descriptor on
/// Linux (`std::os::fd::OwnedFd`). Other platforms cannot make one.
#[derive(Debug)]
pub struct SharedHandle(OsHandle);

#[cfg(windows)]
impl From<std::os::windows::io::OwnedHandle> for SharedHandle {
    fn from(handle: std::os::windows::io::OwnedHandle) -> Self {
        Self(handle)
    }
}

#[cfg(windows)]
impl From<SharedHandle> for std::os::windows::io::OwnedHandle {
    fn from(handle: SharedHandle) -> Self {
        handle.0
    }
}

#[cfg(windows)]
impl std::os::windows::io::AsHandle for SharedHandle {
    fn as_handle(&self) -> std::os::windows::io::BorrowedHandle<'_> {
        self.0.as_handle()
    }
}

#[cfg(target_os = "linux")]
impl From<std::os::fd::OwnedFd> for SharedHandle {
    fn from(fd: std::os::fd::OwnedFd) -> Self {
        Self(fd)
    }
}

#[cfg(target_os = "linux")]
impl From<SharedHandle> for std::os::fd::OwnedFd {
    fn from(handle: SharedHandle) -> Self {
        handle.0
    }
}

#[cfg(target_os = "linux")]
impl std::os::fd::AsFd for SharedHandle {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.0.as_fd()
    }
}

/// The value the `release` fence reaches once the shared textures are
/// prepared for the hand-off; the producer writes a texture for the first
/// time only after `release` has reached it. Frame values start above it.
pub const SHARED_PREPARED: u64 = 1;

/// A pool of textures an external render engine draws into, and its two
/// timeline fences, handed to a window to import. The window takes the
/// handles: on Windows it closes them once opened, on Linux a successful
/// import gives the descriptors to the driver (and a failed one closes
/// them).
///
/// # The engine's contract
///
/// Breaking it is undefined behavior of the graphics API (Vulkan and
/// Direct3D 12): a broken picture, a lost device, a crashed process, or
/// access to GPU memory outside the resource are all possible. Arvane and
/// Kivact cannot check most of it (an imported texture is only memory to
/// them), which is why importing is an `unsafe fn`.
///
/// Device: on the adapter the window reports ([`GpuAdapter`], named by
/// `adapter`), with the window's graphics API (`backend`): Direct3D 12
/// with Direct3D 12, Vulkan with Vulkan.
///
/// Textures: 2D, `format`'s RGBA8 unorm with premultiplied alpha,
/// `width` × `height`, 1 mip level, 1 array layer, 1 sample. The values
/// are sRGB-encoded (see [`SharedTextureFormat`]). The engine
/// may render into them, copy into and out of them and sample them, and,
/// when `storage` is true, write them as storage textures.
/// - Direct3D 12: a committed resource in a `D3D12_HEAP_TYPE_DEFAULT`
///   heap with `D3D12_HEAP_FLAG_SHARED`, `D3D12_RESOURCE_DIMENSION_TEXTURE2D`,
///   `DXGI_FORMAT_R8G8B8A8_UNORM`, `D3D12_TEXTURE_LAYOUT_UNKNOWN`, flags
///   exactly `D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET` (with
///   `| D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS` when `storage` is true;
///   never `D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS`), created in
///   `D3D12_RESOURCE_STATE_COMMON`; its handle from
///   `ID3D12Device::CreateSharedHandle` (`GENERIC_ALL`). Arvane checks the
///   flags against `storage` (a resource describes itself).
/// - Vulkan: a `VkImage` of `VK_IMAGE_TYPE_2D`, `VK_FORMAT_R8G8B8A8_UNORM`,
///   extent `width` × `height` × 1, `mipLevels` 1, `arrayLayers` 1,
///   `VK_SAMPLE_COUNT_1_BIT`, `VK_IMAGE_TILING_OPTIMAL`, usage exactly
///   `TRANSFER_SRC | TRANSFER_DST | SAMPLED | COLOR_ATTACHMENT` (with
///   `| STORAGE` when `storage` is true), no flags,
///   `VK_SHARING_MODE_EXCLUSIVE`, initial layout `UNDEFINED`, with a
///   `VkExternalMemoryImageCreateInfo` of the handle type
///   (`OPAQUE_WIN32` on Windows, `OPAQUE_FD` on Linux); bound at offset 0
///   to a dedicated allocation (`VkMemoryDedicatedAllocateInfo`) exported
///   as that handle type, whose `allocationSize` is the image's
///   `vkGetImageMemoryRequirements` size and whose memory type is the
///   lowest index in its `memoryTypeBits` with `DEVICE_LOCAL` (Arvane
///   makes the same image and allocation info to import it); the handle
///   from `vkGetMemoryWin32HandleKHR` or `vkGetMemoryFdKHR`. Before it
///   makes the image, the engine checks with
///   `vkGetPhysicalDeviceImageFormatProperties2` (that usage, the handle
///   type in `VkPhysicalDeviceExternalImageFormatInfo`) that the adapter
///   exports such an image; Arvane checks the same for importing and
///   refuses the textures otherwise. Arvane cannot tell an image made with
///   another usage than `storage` says (the memory does not describe its
///   image): that breaks the contract.
///
/// Handles: one export of each texture and fence per import (an NVIDIA
/// Vulkan driver refused to import memory whose handle had been exported
/// and closed before).
///
/// Fences (`ready`, `release`): timeline fences starting at 0, Direct3D
/// 12's shared fences (`D3D12_FENCE_FLAG_SHARED`) or Vulkan's timeline
/// semaphores exported as the handle type above.
///
/// Order: the engine hands over textures it has not written yet (what a
/// texture holds before the engine's first frame in it is undefined), and
/// writes a texture for the first time only after `release` reached
/// [`SHARED_PREPARED`] (1): the importer signals it once the textures are
/// prepared for the hand-off. Between Arvane and the engine a
/// texture is in `D3D12_RESOURCE_STATE_COMMON` (Direct3D 12), or in
/// `VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL` and owned by
/// `VK_QUEUE_FAMILY_EXTERNAL` (Vulkan): the engine's first barrier of a
/// write acquires it from `VK_QUEUE_FAMILY_EXTERNAL` in that layout, and its
/// last releases it there in that layout (both keep the layout).
///
/// Frames: when the engine has drawn a texture, it signals `ready` to a
/// value `v`, larger for each frame of the image, and names the frame
/// `ExternalFrame { slot, value: v }`. Frame values start at 2 (1 is the
/// preparation signal, [`SHARED_PREPARED`]).
///
/// Release: `release` ≥ `v` means Arvane never reads a frame of value `v`
/// or less again. Before it writes a texture again, the engine makes sure
/// the value of the last frame it drew there is at most `release`. After
/// a lost device, `release` may never arrive: the engine waits for it
/// with a time limit. When the window drops the image (or closes), it
/// signals `release` to the last value it saw.
///
/// After a new device ([`GpuAdapter::generation`]) every import is gone:
/// the engine makes new textures and they are imported again.
#[derive(Debug)]
pub struct SharedTextures {
    /// Pixel width (at least 1).
    pub width: u32,
    /// Pixel height (at least 1).
    pub height: u32,
    /// The textures' pixel format.
    pub format: SharedTextureFormat,
    /// The graphics API the engine made the textures and fences with.
    pub backend: GpuBackend,
    /// The adapter the engine's device is on.
    pub adapter: AdapterMatch,
    /// Whether the textures were also made for storage writes (Direct3D
    /// 12's `ALLOW_UNORDERED_ACCESS`, Vulkan's `STORAGE` usage); see the
    /// contract's texture rules.
    pub storage: bool,
    /// The pool's textures (at least one), indexed by
    /// [`ExternalFrame::slot`].
    pub textures: Vec<SharedHandle>,
    /// The fence the engine signals when a frame is drawn.
    pub ready: SharedHandle,
    /// The fence Arvane signals when it is done with frames.
    pub release: SharedHandle,
}

/// How the window answered an import of shared textures
/// (`HostEvent::ExternalImported`).
#[derive(Clone, Debug, PartialEq)]
pub enum ExternalImport {
    /// Imported into the device of this generation
    /// (`GpuAdapter::generation`).
    Ready { generation: u64 },
    /// The window has no device that can import (it draws on the CPU, its
    /// device is not ready, or the platform has none): Arvane's
    /// `ErrorKind::Unavailable`, with its message.
    Unavailable(String),
    /// The textures do not fit what they say or the window's device, or
    /// the device failed (Arvane's `Content` and `Backend`), with its
    /// message.
    Failed(String),
}
