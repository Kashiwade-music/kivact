//! Role: A pool's texture read back to the CPU is the picture the engine drew.
//!
//! No window: the engine runs on an adapter of this machine and its pool
//! is never imported, as for a window that shows the frames as `Pixels`.
//! The test sends the preparation signal a window sends on import
//! (`SHARED_PREPARED`), which the engine's first frames wait for.
//! With the feature `wgpu`, the same for the engine on wgpu, and a texture
//! of its own read back with wgpu alone. Each test is skipped, saying why, when the machine has no hardware
//! adapter of that graphics API.

use gpu_interop_producer::{POOL, Producer, pattern};
use kivact_types::{ExternalImageId, GpuBackend};

const W: u32 = 256;
const H: u32 = 128;

fn reads_back_the_pattern(backend: GpuBackend) -> Result<(), String> {
    let adapter = gpu_interop_producer::adapters(backend)
        .into_iter()
        .next()
        .ok_or_else(|| format!("no {backend:?} hardware adapter"))?;
    let mut producer =
        Producer::new(&adapter, ExternalImageId(1), W, H).expect("the engine starts");
    producer.signal_prepared().expect("the pool is prepared");
    // One frame per texture: without a window nothing releases a frame,
    // so a texture is not written twice.
    for number in 0..POOL as u64 {
        producer.draw(number).expect("the engine draws");
    }
    for number in 0..POOL as u64 {
        let slot = number as u32;
        let rgba = producer.read_back(slot).expect("the texture reads back");
        assert_eq!(
            rgba,
            pattern::rgba(W, H, number, W as usize * 4),
            "texture {slot} holds frame {number} on {}",
            adapter.name
        );
    }
    Ok(())
}

/// Direct3D 12: the read back of each texture is its last frame's
/// picture, straight RGBA8.
#[cfg(windows)]
#[test]
fn reads_back_the_pattern_on_dx12() {
    if let Err(reason) = reads_back_the_pattern(GpuBackend::Dx12) {
        eprintln!("SKIPPED reads_back_the_pattern_on_dx12: {reason}");
    }
}

/// Vulkan: as on Direct3D 12.
#[test]
fn reads_back_the_pattern_on_vulkan() {
    if let Err(reason) = reads_back_the_pattern(GpuBackend::Vulkan) {
        eprintln!("SKIPPED reads_back_the_pattern_on_vulkan: {reason}");
    }
}

#[cfg(feature = "wgpu")]
fn reads_back_the_pattern_with_wgpu(backend: GpuBackend) -> Result<(), String> {
    let adapter = gpu_interop_producer::adapters(backend)
        .into_iter()
        .next()
        .ok_or_else(|| format!("no {backend:?} hardware adapter"))?;
    let mut producer =
        Producer::with_wgpu(&adapter, ExternalImageId(1), W, H).expect("the engine starts");
    producer.signal_prepared().expect("the pool is prepared");
    for number in 0..POOL as u64 {
        producer.draw(number).expect("the engine draws");
    }
    for number in 0..POOL as u64 {
        let slot = number as u32;
        let rgba = producer.read_back(slot).expect("the texture reads back");
        assert_eq!(
            rgba,
            pattern::rgba(W, H, number, W as usize * 4),
            "texture {slot} holds frame {number} on {}",
            adapter.name
        );
    }
    Ok(())
}

/// Direct3D 12, the engine on wgpu: as on Direct3D 12 itself.
#[cfg(all(feature = "wgpu", windows))]
#[test]
fn reads_back_the_pattern_with_wgpu_on_dx12() {
    if let Err(reason) = reads_back_the_pattern_with_wgpu(GpuBackend::Dx12) {
        eprintln!("SKIPPED reads_back_the_pattern_with_wgpu_on_dx12: {reason}");
    }
}

/// Vulkan, the engine on wgpu: as on Vulkan itself.
#[cfg(feature = "wgpu")]
#[test]
fn reads_back_the_pattern_with_wgpu_on_vulkan() {
    if let Err(reason) = reads_back_the_pattern_with_wgpu(GpuBackend::Vulkan) {
        eprintln!("SKIPPED reads_back_the_pattern_with_wgpu_on_vulkan: {reason}");
    }
}

#[cfg(feature = "wgpu")]
fn reads_back_a_texture_of_its_own(backend: GpuBackend) -> Result<(), String> {
    use kivact_gpu_share::wgpu::{FORMAT, read_back, request_device};
    let adapter = gpu_interop_producer::adapters(backend)
        .into_iter()
        .next()
        .ok_or_else(|| format!("no {backend:?} hardware adapter"))?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: match backend {
            GpuBackend::Dx12 => wgpu::Backends::DX12,
            GpuBackend::Vulkan => wgpu::Backends::VULKAN,
        },
        flags: wgpu::InstanceFlags::empty().with_env(),
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let (_, device, queue) =
        request_device(&instance, &adapter, &wgpu::DeviceDescriptor::default())
            .expect("the device opens on the adapter");
    let size = wgpu::Extent3d {
        width: W,
        height: H,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    // The pattern, with a premultiplied half-transparent pixel and a
    // transparent one at the start.
    let mut rgba = pattern::rgba(W, H, 5, W as usize * 4);
    rgba[..8].copy_from_slice(&[100, 50, 0, 128, 7, 7, 7, 0]);
    queue.write_texture(
        texture.as_image_copy(),
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(W * 4),
            rows_per_image: Some(H),
        },
        size,
    );
    let read = read_back(&device, &queue, &texture, std::time::Duration::from_secs(5))
        .expect("the texture reads back");
    let mut straight = rgba;
    straight[..8].copy_from_slice(&[199, 100, 0, 128, 0, 0, 0, 0]);
    assert_eq!(read, straight, "straight alpha on {}", adapter.name);
    Ok(())
}

/// A texture of the engine's own (not shared) reads back with wgpu alone
/// as straight-alpha RGBA8, on Direct3D 12.
#[cfg(all(feature = "wgpu", windows))]
#[test]
fn reads_back_a_texture_of_its_own_on_dx12() {
    if let Err(reason) = reads_back_a_texture_of_its_own(GpuBackend::Dx12) {
        eprintln!("SKIPPED reads_back_a_texture_of_its_own_on_dx12: {reason}");
    }
}

/// As on Direct3D 12, on Vulkan.
#[cfg(feature = "wgpu")]
#[test]
fn reads_back_a_texture_of_its_own_on_vulkan() {
    if let Err(reason) = reads_back_a_texture_of_its_own(GpuBackend::Vulkan) {
        eprintln!("SKIPPED reads_back_a_texture_of_its_own_on_vulkan: {reason}");
    }
}
