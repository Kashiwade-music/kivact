//! Role: A shared pool on the engine's own ash device: images, semaphores, barriers.
//!
//! The engine brings its Vulkan instance, physical device, device and
//! queue family ([`find_physical_device`] finds the physical device the
//! window reported; the device enables [`DEVICE_EXTENSIONS`], API
//! [`API_VERSION`] and the `timelineSemaphore` feature). [`Pool::new`]
//! makes the images and the two timeline semaphores as
//! [`ui_types::SharedTextures`] says and exports them once. A frame:
//! [`Pool::acquire`], [`Pool::record_acquire`], the engine's own drawing,
//! [`Pool::record_release`], then [`Pool::submit`] (or the engine's own
//! submission with [`Pool::finish`]'s semaphores).
//!
//! Invariants:
//! - A texture is written again only after the GPU waited for `release`
//!   to reach its last frame's value (the submission's timeline wait);
//!   the CPU waits, bounded, only for the texture's last submission to
//!   have signaled `ready`.
//! - Every `unsafe` block has its `SAFETY` note; the raw handles a caller
//!   passes are covered by the `# Safety` sections of the `unsafe fn`s.
//! - Dropping the pool waits (bounded) for the last frame's `ready` and
//!   leaks the images and semaphores instead when it does not arrive.

#![allow(
    unsafe_code,
    reason = "the Vulkan calls of an engine's shared pool, below any safe API"
)]

mod raw;
mod read_back;

pub use raw::FORMAT;
use raw::{barrier, memory_type};
pub use read_back::{ReadImage, read_back};

use crate::common::{Error, api};
use crate::pool::{DROP_WAIT, FrameReady, PoolConfig, Slots};
use ash::vk;
use std::time::Duration;
use ui_types::{
    AdapterMatch, GpuAdapter, GpuBackend, SharedHandle, SharedTextureFormat, SharedTextures,
};

pub use crate::vulkan_contract::{API_VERSION, DEVICE_EXTENSIONS, HAND_OFF_LAYOUT};
use crate::vulkan_contract::{MEMORY_HANDLE, SEMAPHORE_HANDLE, usage};

/// Fail unless `physical` exports images of the contract's usage (with
/// storage or not) at `config`'s size.
///
/// # Safety
/// `physical` is a physical device of the live `instance`.
unsafe fn check_export(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    config: &PoolConfig,
) -> Result<(), Error> {
    let mut external_info =
        vk::PhysicalDeviceExternalImageFormatInfo::default().handle_type(MEMORY_HANDLE);
    let info = vk::PhysicalDeviceImageFormatInfo2::default()
        .format(FORMAT)
        .ty(vk::ImageType::TYPE_2D)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(usage(config.storage))
        .push_next(&mut external_info);
    let mut external = vk::ExternalImageFormatProperties::default();
    let which = if config.storage { " with storage" } else { "" };
    let extent = {
        let mut properties = vk::ImageFormatProperties2::default().push_next(&mut external);
        // SAFETY: a physical device of the instance (the caller); valid
        // chains.
        let supported = unsafe {
            instance.get_physical_device_image_format_properties2(physical, &info, &mut properties)
        };
        match supported {
            Ok(()) => {}
            Err(vk::Result::ERROR_FORMAT_NOT_SUPPORTED) => {
                return Err(Error::Mismatch(format!(
                    "the adapter has no shared images{which}"
                )));
            }
            Err(e) => return Err(api("vkGetPhysicalDeviceImageFormatProperties2")(e)),
        }
        properties.image_format_properties.max_extent
    };
    let features = external.external_memory_properties.external_memory_features;
    if !features.contains(vk::ExternalMemoryFeatureFlags::EXPORTABLE) {
        return Err(Error::Mismatch(format!(
            "the adapter cannot export shared images{which}"
        )));
    }
    if config.width > extent.width || config.height > extent.height {
        return Err(Error::Mismatch(format!(
            "the adapter exports images up to {} x {}",
            extent.width, extent.height
        )));
    }
    Ok(())
}

/// The device and driver UUIDs and API version of `physical`.
///
/// # Safety
/// `physical` is a physical device of the live `instance`.
pub(crate) unsafe fn identity(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
) -> ([u8; 16], [u8; 16], u32) {
    // SAFETY: the caller provides a physical device of the live instance.
    let (identity, version) = unsafe { crate::vulkan_raw::identity(instance, physical) };
    (
        identity
            .device_uuid
            .expect("invariant: Vulkan reports a device UUID"),
        identity
            .driver_uuid
            .expect("invariant: Vulkan reports a driver UUID"),
        version,
    )
}

/// The UUIDs `adapter` reports for Vulkan.
pub(crate) fn uuids(adapter: &GpuAdapter) -> Result<([u8; 16], [u8; 16]), Error> {
    if adapter.backend != GpuBackend::Vulkan {
        return Err(Error::Mismatch(format!(
            "the window's device is {:?}, not Vulkan",
            adapter.backend
        )));
    }
    match (adapter.device_uuid, adapter.driver_uuid) {
        (Some(device), Some(driver)) => Ok((device, driver)),
        _ => Err(Error::Mismatch("the adapter has no Vulkan UUIDs".into())),
    }
}

/// The physical device of `instance` that is `adapter` (its device and
/// driver UUIDs) with Vulkan [`API_VERSION`] or later.
///
/// # Safety
/// `instance` is a live instance of API 1.1 or later.
pub unsafe fn find_physical_device(
    instance: &ash::Instance,
    adapter: &GpuAdapter,
) -> Result<vk::PhysicalDevice, Error> {
    let wanted = uuids(adapter)?;
    // SAFETY: a live instance (the caller).
    let physicals = unsafe { instance.enumerate_physical_devices() }
        .map_err(api("vkEnumeratePhysicalDevices"))?;
    for physical in physicals {
        // SAFETY: a physical device of the instance.
        let (device, driver, version) = unsafe { identity(instance, physical) };
        if (device, driver) == wanted && version >= API_VERSION {
            return Ok(physical);
        }
    }
    Err(Error::NoAdapter)
}

/// One texture of the pool.
#[derive(Default)]
struct Slot {
    image: vk::Image,
    memory: vk::DeviceMemory,
}

/// The texture the next frame is drawn into, from [`Pool::acquire`], until
/// [`Pool::submit`] or [`Pool::finish`] takes it.
#[derive(Debug)]
pub struct Target {
    slot: u32,
    image: vk::Image,
    extent: vk::Extent2D,
    /// The value `release` reaches before the GPU writes (0: no wait).
    wait: u64,
}

impl Target {
    /// The texture's index in the pool.
    pub fn slot(&self) -> u32 {
        self.slot
    }

    /// The image to draw into.
    pub fn image(&self) -> vk::Image {
        self.image
    }

    /// The image's size.
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    /// The image's format ([`FORMAT`]).
    pub fn format(&self) -> vk::Format {
        FORMAT
    }

    /// The value `release` reaches before the GPU writes (0: no wait).
    #[cfg(feature = "wgpu")]
    pub(crate) fn release_wait(&self) -> u64 {
        self.wait
    }
}

/// The timeline semaphores an engine's own submission of a frame waits
/// for and signals ([`Pool::finish`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Submission {
    /// Wait (`VkTimelineSemaphoreSubmitInfo`) for `release` to reach this
    /// value before the frame's first command touching the image (`None`:
    /// the texture was never drawn).
    pub wait: Option<(vk::Semaphore, u64)>,
    /// Signal `ready` to this value after the frame's last command.
    pub signal: (vk::Semaphore, u64),
    /// The frame to publish to the UI.
    pub frame: FrameReady,
}

/// A pool of shared images and its `ready` and `release` timeline
/// semaphores, on the engine's own device.
pub struct Pool {
    instance: ash::Instance,
    physical: vk::PhysicalDevice,
    device: ash::Device,
    family: u32,
    extent: vk::Extent2D,
    storage: bool,
    slots: Vec<Slot>,
    /// The pool destroys the images and their memory (not handed over).
    owns_images: bool,
    ready: vk::Semaphore,
    release: vk::Semaphore,
    frames: Slots,
}

impl Pool {
    /// A pool of `config`'s images on `device`, and its handles for the
    /// window to import ([`ui_types::SharedTextures`], exported here and
    /// only here: an NVIDIA driver refuses memory exported twice).
    ///
    /// # Safety
    /// - `instance`, `physical` and `device` are live, `device` made from
    ///   `physical` with [`DEVICE_EXTENSIONS`], API [`API_VERSION`] and the
    ///   `timelineSemaphore` feature, and `device` outlives the pool.
    /// - `queue_family` is a family of `device` the engine draws on.
    pub unsafe fn new(
        instance: &ash::Instance,
        physical: vk::PhysicalDevice,
        device: &ash::Device,
        queue_family: u32,
        adapter: &GpuAdapter,
        config: PoolConfig,
    ) -> Result<(Self, SharedTextures), Error> {
        config.check()?;
        let wanted = uuids(adapter)?;
        // SAFETY: a physical device of the instance (the caller).
        let (device_uuid, driver_uuid, _) = unsafe { identity(instance, physical) };
        if (device_uuid, driver_uuid) != wanted {
            return Err(Error::Mismatch(
                "the device is not on the window's adapter".into(),
            ));
        }
        // SAFETY: as above.
        unsafe { check_export(instance, physical, &config) }?;
        let mut pool = Self {
            instance: instance.clone(),
            physical,
            device: device.clone(),
            family: queue_family,
            extent: vk::Extent2D {
                width: config.width,
                height: config.height,
            },
            storage: config.storage,
            slots: Vec::new(),
            owns_images: true,
            ready: vk::Semaphore::null(),
            release: vk::Semaphore::null(),
            frames: Slots::new(config.slots),
        };
        for _ in 0..config.slots {
            pool.slots.push(Slot::default());
            pool.make_slot()?;
        }
        pool.ready = pool.semaphore()?;
        pool.release = pool.semaphore()?;
        let shared = pool.share()?;
        Ok((pool, shared))
    }

    /// The last slot's image, bound to an exportable dedicated allocation.
    fn make_slot(&mut self) -> Result<(), Error> {
        let mut external = vk::ExternalMemoryImageCreateInfo::default().handle_types(MEMORY_HANDLE);
        let info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(FORMAT)
            .extent(self.extent.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage(self.storage))
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .push_next(&mut external);
        let slot = self.slots.len() - 1;
        // SAFETY: a valid create info for the pool's device.
        let image =
            unsafe { self.device.create_image(&info, None) }.map_err(api("vkCreateImage"))?;
        self.slots[slot].image = image;
        // SAFETY: a live image.
        let required = unsafe { self.device.get_image_memory_requirements(image) };
        // SAFETY: the pool's physical device, of its instance.
        let index = unsafe {
            memory_type(
                &self.instance,
                self.physical,
                required.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
        }
        .ok_or_else(|| Error::Mismatch("no device-local memory for the image".into()))?;
        let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().image(image);
        let mut export = vk::ExportMemoryAllocateInfo::default().handle_types(MEMORY_HANDLE);
        let allocate = vk::MemoryAllocateInfo::default()
            .allocation_size(required.size)
            .memory_type_index(index)
            .push_next(&mut dedicated)
            .push_next(&mut export);
        // SAFETY: a valid allocate info.
        let memory = unsafe { self.device.allocate_memory(&allocate, None) }
            .map_err(api("vkAllocateMemory"))?;
        self.slots[slot].memory = memory;
        // SAFETY: a dedicated allocation of the image's size.
        unsafe { self.device.bind_image_memory(image, memory, 0) }
            .map_err(api("vkBindImageMemory"))?;
        Ok(())
    }

    /// An exportable timeline semaphore at 0.
    fn semaphore(&self) -> Result<vk::Semaphore, Error> {
        // SAFETY: the pool's device satisfies the Vulkan sharing contract.
        unsafe { crate::vulkan_raw::timeline(&self.device, true) }
    }

    /// The handles of the images and semaphores, exported once.
    fn share(&self) -> Result<SharedTextures, Error> {
        let textures = self
            .slots
            .iter()
            .map(|slot| self.export_memory(slot.memory))
            .collect::<Result<_, _>>()?;
        // SAFETY: the pool's physical device, of its instance.
        let (device, driver, _) = unsafe { identity(&self.instance, self.physical) };
        Ok(SharedTextures {
            width: self.extent.width,
            height: self.extent.height,
            format: SharedTextureFormat::Rgba8UnormPremultiplied,
            backend: GpuBackend::Vulkan,
            adapter: AdapterMatch::Uuid { device, driver },
            storage: self.storage,
            textures,
            ready: self.export_semaphore(self.ready)?,
            release: self.export_semaphore(self.release)?,
        })
    }

    #[cfg(windows)]
    fn export_memory(&self, memory: vk::DeviceMemory) -> Result<SharedHandle, Error> {
        use std::os::windows::io::{FromRawHandle, OwnedHandle};
        let ext = ash::khr::external_memory_win32::Device::new(&self.instance, &self.device);
        let info = vk::MemoryGetWin32HandleInfoKHR::default()
            .memory(memory)
            .handle_type(MEMORY_HANDLE);
        // SAFETY: the memory was allocated exportable as this handle type.
        let raw = unsafe { ext.get_memory_win32_handle(&info) }
            .map_err(api("vkGetMemoryWin32HandleKHR"))?;
        // SAFETY: a new NT handle, owned by the result alone.
        Ok(unsafe { OwnedHandle::from_raw_handle(raw as *mut std::ffi::c_void) }.into())
    }

    #[cfg(target_os = "linux")]
    fn export_memory(&self, memory: vk::DeviceMemory) -> Result<SharedHandle, Error> {
        use std::os::fd::{FromRawFd, OwnedFd};
        let ext = ash::khr::external_memory_fd::Device::new(&self.instance, &self.device);
        let info = vk::MemoryGetFdInfoKHR::default()
            .memory(memory)
            .handle_type(MEMORY_HANDLE);
        // SAFETY: the memory was allocated exportable as this handle type.
        let fd = unsafe { ext.get_memory_fd(&info) }.map_err(api("vkGetMemoryFdKHR"))?;
        // SAFETY: a new descriptor, owned by the result alone.
        Ok(unsafe { OwnedFd::from_raw_fd(fd) }.into())
    }

    #[cfg(windows)]
    fn export_semaphore(&self, semaphore: vk::Semaphore) -> Result<SharedHandle, Error> {
        use std::os::windows::io::{FromRawHandle, OwnedHandle};
        let ext = ash::khr::external_semaphore_win32::Device::new(&self.instance, &self.device);
        let info = vk::SemaphoreGetWin32HandleInfoKHR::default()
            .semaphore(semaphore)
            .handle_type(SEMAPHORE_HANDLE);
        // SAFETY: the semaphore was made exportable as this handle type.
        let raw = unsafe { ext.get_semaphore_win32_handle(&info) }
            .map_err(api("vkGetSemaphoreWin32HandleKHR"))?;
        // SAFETY: a new NT handle, owned by the result alone.
        Ok(unsafe { OwnedHandle::from_raw_handle(raw as *mut std::ffi::c_void) }.into())
    }

    #[cfg(target_os = "linux")]
    fn export_semaphore(&self, semaphore: vk::Semaphore) -> Result<SharedHandle, Error> {
        use std::os::fd::{FromRawFd, OwnedFd};
        let ext = ash::khr::external_semaphore_fd::Device::new(&self.instance, &self.device);
        let info = vk::SemaphoreGetFdInfoKHR::default()
            .semaphore(semaphore)
            .handle_type(SEMAPHORE_HANDLE);
        // SAFETY: the semaphore was made exportable as this handle type.
        let fd = unsafe { ext.get_semaphore_fd(&info) }.map_err(api("vkGetSemaphoreFdKHR"))?;
        // SAFETY: a new descriptor, owned by the result alone.
        Ok(unsafe { OwnedFd::from_raw_fd(fd) }.into())
    }

    /// Wait at most `timeout` on the CPU for the timeline `semaphore` to
    /// reach `value`.
    fn wait(
        &self,
        semaphore: vk::Semaphore,
        value: u64,
        timeout: Duration,
        what: &'static str,
    ) -> Result<(), Error> {
        if value == 0 {
            return Ok(());
        }
        let semaphores = [semaphore];
        let values = [value];
        let info = vk::SemaphoreWaitInfo::default()
            .semaphores(&semaphores)
            .values(&values);
        let nanos = u64::try_from(timeout.as_nanos()).unwrap_or(u64::MAX);
        // SAFETY: a timeline semaphore of the pool's device; a bounded wait.
        match unsafe { self.device.wait_semaphores(&info, nanos) } {
            Ok(()) => Ok(()),
            Err(vk::Result::TIMEOUT) => Err(Error::Timeout(what)),
            Err(e) => Err(api("vkWaitSemaphores")(e)),
        }
    }

    /// The texture of the next frame, once its last frame's submission
    /// signaled `ready` (waited at most `timeout` on the CPU). The GPU
    /// waits for the window's `release` of that frame: the submission
    /// ([`Pool::submit`], or [`Pool::finish`]'s `wait`) does, so the
    /// engine can draw ahead of the window.
    ///
    /// A [`Error::Timeout`] means the texture's last submission did not
    /// finish, usually because it still waits for a release the window
    /// never gives (a lost device, a window that stopped presenting).
    pub fn acquire(&mut self, timeout: Duration) -> Result<Target, Error> {
        let (slot, last) = self.frames.acquire()?;
        if let Err(error) = self.wait(self.ready, last, timeout, "the texture's last frame") {
            self.frames.cancel();
            return Err(error);
        }
        Ok(Target {
            slot,
            image: self.slots[slot as usize].image,
            extent: self.extent,
            wait: last,
        })
    }

    /// Give `target` up without a frame (nothing of it was submitted).
    pub fn cancel(&mut self, target: Target) {
        let _ = target;
        self.frames.cancel();
    }

    /// Record into `command` the barriers that take `target`'s image from
    /// the window: acquired from `VK_QUEUE_FAMILY_EXTERNAL` to the pool's
    /// queue family keeping [`HAND_OFF_LAYOUT`] (as the window released
    /// it), then moved to `layout` (contents kept) for the engine's first
    /// use at `stage` with `access`. The frame's first commands on the
    /// image.
    ///
    /// # Safety
    /// `command` is a command buffer of the pool's device, recording, for
    /// a queue of the pool's queue family; `target` is of this pool.
    pub unsafe fn record_acquire(
        &self,
        command: vk::CommandBuffer,
        target: &Target,
        layout: vk::ImageLayout,
        (stage, access): (vk::PipelineStageFlags, vk::AccessFlags),
    ) {
        let all = vk::PipelineStageFlags::ALL_COMMANDS;
        let none = vk::AccessFlags::empty();
        let memory = vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE;
        // SAFETY: a recording buffer of the device (the caller).
        unsafe {
            barrier(
                &self.device,
                command,
                target.image,
                (HAND_OFF_LAYOUT, HAND_OFF_LAYOUT),
                (vk::QUEUE_FAMILY_EXTERNAL, self.family),
                (all, none),
                (all, memory),
            );
        }
        if layout != HAND_OFF_LAYOUT {
            // SAFETY: as above.
            unsafe {
                barrier(
                    &self.device,
                    command,
                    target.image,
                    (HAND_OFF_LAYOUT, layout),
                    (vk::QUEUE_FAMILY_IGNORED, vk::QUEUE_FAMILY_IGNORED),
                    (all, none),
                    (stage, access),
                );
            }
        }
    }

    /// Record into `command` the barriers that give `target`'s image back
    /// to the window: from `layout` (last used at `stage` with `access`)
    /// to [`HAND_OFF_LAYOUT`], then released to
    /// `VK_QUEUE_FAMILY_EXTERNAL` keeping it. The frame's last commands
    /// on the image.
    ///
    /// # Safety
    /// As [`Pool::record_acquire`].
    pub unsafe fn record_release(
        &self,
        command: vk::CommandBuffer,
        target: &Target,
        layout: vk::ImageLayout,
        (stage, access): (vk::PipelineStageFlags, vk::AccessFlags),
    ) {
        let all = vk::PipelineStageFlags::ALL_COMMANDS;
        let none = vk::AccessFlags::empty();
        let (src_stage, src_access) = if layout == HAND_OFF_LAYOUT {
            (stage, access)
        } else {
            // SAFETY: a recording buffer of the device (the caller).
            unsafe {
                barrier(
                    &self.device,
                    command,
                    target.image,
                    (layout, HAND_OFF_LAYOUT),
                    (vk::QUEUE_FAMILY_IGNORED, vk::QUEUE_FAMILY_IGNORED),
                    (stage, access),
                    (all, none),
                );
            }
            (all, vk::AccessFlags::MEMORY_WRITE)
        };
        // SAFETY: as above.
        unsafe {
            barrier(
                &self.device,
                command,
                target.image,
                (HAND_OFF_LAYOUT, HAND_OFF_LAYOUT),
                (self.family, vk::QUEUE_FAMILY_EXTERNAL),
                (src_stage, src_access),
                (vk::PipelineStageFlags::BOTTOM_OF_PIPE, none),
            );
        }
    }

    /// The semaphores the engine's own submission of `target`'s frame
    /// waits for and signals; the frame counts as submitted. The
    /// submission must follow: until `ready` reaches the frame's value,
    /// acquiring this texture again times out.
    pub fn finish(&mut self, target: Target) -> Submission {
        let frame = self.frames.finish(target.slot);
        Submission {
            wait: (target.wait > 0).then_some((self.release, target.wait)),
            signal: (self.ready, frame.value),
            frame,
        }
    }

    /// Submit `commands` (which draw `target`'s frame between
    /// [`Pool::record_acquire`] and [`Pool::record_release`]) to `queue`,
    /// waiting on the GPU for the window's release first, signaling
    /// `ready` after, and `fence` (may be null) once done: the frame to
    /// publish. When the submission fails, the frame is given up.
    ///
    /// # Safety
    /// `queue` is a queue of the pool's device and queue family, not used
    /// by another thread during the call; `commands` are executable
    /// command buffers of the device; `fence` is null or an unsignaled
    /// fence of the device.
    pub unsafe fn submit(
        &mut self,
        queue: vk::Queue,
        target: Target,
        commands: &[vk::CommandBuffer],
        fence: vk::Fence,
    ) -> Result<FrameReady, Error> {
        let value = self.frames.last() + 1;
        let (waits, wait_values) = if target.wait > 0 {
            (vec![self.release], vec![target.wait])
        } else {
            (Vec::new(), Vec::new())
        };
        let stages = vec![vk::PipelineStageFlags::ALL_COMMANDS; waits.len()];
        let signals = [self.ready];
        let signal_values = [value];
        let mut values = vk::TimelineSemaphoreSubmitInfo::default()
            .wait_semaphore_values(&wait_values)
            .signal_semaphore_values(&signal_values);
        let info = vk::SubmitInfo::default()
            .command_buffers(commands)
            .wait_semaphores(&waits)
            .wait_dst_stage_mask(&stages)
            .signal_semaphores(&signals)
            .push_next(&mut values);
        // SAFETY: the caller's queue and buffers of the pool's device.
        let submitted = unsafe { self.device.queue_submit(queue, &[info], fence) };
        match submitted {
            Ok(()) => Ok(self.finish(target).frame),
            Err(e) => {
                self.cancel(target);
                Err(api("vkQueueSubmit")(e))
            }
        }
    }

    /// The `release` timeline semaphore.
    #[cfg(feature = "wgpu")]
    pub(crate) fn release_semaphore(&self) -> vk::Semaphore {
        self.release
    }

    /// The images and their dedicated memory, by slot, from now on owned
    /// by the caller (who destroys them only after the pool's last frame
    /// finished); the pool still uses them.
    #[cfg(feature = "wgpu")]
    pub(crate) fn hand_over_images(&mut self) -> Vec<(vk::Image, vk::DeviceMemory)> {
        self.owns_images = false;
        self.slots.iter().map(|s| (s.image, s.memory)).collect()
    }

    /// The `release` semaphore's value now: the window reads no frame of
    /// this value or less again.
    pub fn release_value(&self) -> Result<u64, Error> {
        // SAFETY: a timeline semaphore of the pool's device.
        unsafe { self.device.get_semaphore_counter_value(self.release) }
            .map_err(api("vkGetSemaphoreCounterValue"))
    }

    /// Wait at most `timeout` on the CPU for `release` to reach `value`.
    pub fn wait_release(&self, value: u64, timeout: Duration) -> Result<(), Error> {
        self.wait(self.release, value, timeout, "the window's release")
    }

    /// Wait at most `timeout` on the CPU for every submitted frame's
    /// `ready` (the engine's work on the pool finished).
    pub fn wait_idle(&self, timeout: Duration) -> Result<(), Error> {
        self.wait(self.ready, self.frames.last(), timeout, "the last frame")
    }

    /// Texture `slot`'s last frame as straight-alpha RGBA8 (`width` ×
    /// `height` × 4 bytes, rows top to bottom), once its `ready` arrived
    /// (each wait at most `timeout`). A thin wrapper of [`read_back`].
    ///
    /// # Safety
    /// - `queue` is a queue of the pool's device and queue family, not
    ///   used by another thread during the call.
    /// - No window uses the texture meanwhile: its frame is released, or
    ///   the pool was never imported (then the texture never was in the
    ///   hand-off state, and acquiring it from `VK_QUEUE_FAMILY_EXTERNAL`
    ///   is out of Vulkan's rules, though drivers take it; see the crate
    ///   documentation).
    pub unsafe fn read_back(
        &self,
        queue: vk::Queue,
        slot: u32,
        timeout: Duration,
    ) -> Result<Vec<u8>, Error> {
        let last = self.frames.drawn(slot)?;
        self.wait(self.ready, last, timeout, "the texture's last frame")?;
        let image = ReadImage {
            image: self.slots[slot as usize].image,
            layout: HAND_OFF_LAYOUT,
            width: self.extent.width,
            height: self.extent.height,
            owner: vk::QUEUE_FAMILY_EXTERNAL,
        };
        // SAFETY: the pool's live objects; the queue and the texture's use
        // as the caller says.
        unsafe {
            read_back(
                &self.instance,
                self.physical,
                &self.device,
                (queue, self.family),
                &image,
                timeout,
            )
        }
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        if self.ready != vk::Semaphore::null() && self.wait_idle(DROP_WAIT).is_err() {
            // A submission may still wait for a release: keep everything.
            return;
        }
        for slot in self.slots.iter().filter(|_| self.owns_images) {
            // SAFETY: no submission uses the image any more (`ready`);
            // destroying a null handle is a no-op.
            unsafe { self.device.destroy_image(slot.image, None) };
            // SAFETY: as above, after its image.
            unsafe { self.device.free_memory(slot.memory, None) };
        }
        // SAFETY: no submission waits for or signals it any more.
        unsafe { self.device.destroy_semaphore(self.ready, None) };
        // SAFETY: as above.
        unsafe { self.device.destroy_semaphore(self.release, None) };
    }
}
