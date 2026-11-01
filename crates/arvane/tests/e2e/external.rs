//! Role: Textures an external render engine shares, drawn on GPU test windows.
//!
//! The engine is `gpu-interop-producer` (a device of its own on the
//! window's adapter, its frames a known picture with the frame number in
//! it). Each test runs once per graphics API: Direct3D 12 (Windows) and
//! Vulkan. A test is skipped, saying why, when the window cannot have a
//! GPU device of that API that imports (no hardware adapter, an adapter
//! without the extensions, or `WGPU_BACKEND` choosing another API).

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use arvane::test_display::{VirtualWindow, VirtualWindowOptions};
use arvane::{ErrorKind, Presented, Renderer, RendererChange, Window as _};
use gpu_interop_producer::{Producer, pattern};
use kivact_types::{
    AdapterMatch, DrawCommand, ExternalFrame, ExternalImageId, GpuAdapter, GpuBackend,
    GpuPreference,
};

use crate::support::{WHITE, open, options, rect, show};

/// The engine's frame size: a window shows four frames.
const W: u32 = 256;
const H: u32 = 128;
const FRAMES: u64 = 60;
const IMAGE: ExternalImageId = ExternalImageId(1);

/// A GPU test window of `backend` that can import, and its adapter; why
/// not when there is none here (the test is skipped).
fn gpu_window(backend: GpuBackend) -> Result<(VirtualWindow, GpuAdapter), String> {
    let opened = VirtualWindow::open(VirtualWindowOptions {
        gpu: true,
        gpu_preference: GpuPreference {
            backend: Some(backend),
            ..GpuPreference::default()
        },
        ..options(2.0 * W as f32, 2.0 * H as f32)
    });
    let window = match opened {
        Ok(window) => window,
        Err(error) if error.kind() == ErrorKind::Unavailable => return Err(error.to_string()),
        Err(error) => panic!("the GPU test window opens: {error}"),
    };
    let Some(adapter) = window.gpu_adapter() else {
        return Err(format!(
            "the {backend:?} device cannot import shared textures"
        ));
    };
    if adapter.backend != backend {
        return Err(format!(
            "the window's device is {:?} (WGPU_BACKEND?)",
            adapter.backend
        ));
    }
    Ok((window, adapter))
}

/// An engine for the window's adapter, its image imported as `image`.
fn imported(window: &mut VirtualWindow, adapter: &GpuAdapter, image: ExternalImageId) -> Producer {
    let mut producer = Producer::new(adapter, image, W, H).expect("the engine starts");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    window
        .import_external_image(image, shared)
        .expect("the window imports the pool");
    producer
}

/// `frame` drawn at its own size at `(x, y)`.
fn external(frame: ExternalFrame, x: u32, y: u32, pixelated: bool) -> DrawCommand {
    DrawCommand::External {
        frame,
        rect: rect(x as f32, y as f32, W as f32, H as f32),
        source: None,
        opacity: 1.0,
        pixelated,
    }
}

/// The window's four places: nearest on top, bilinear below.
const PLACES: [(u32, u32, bool); 4] = [(0, 0, true), (W, 0, true), (0, H, false), (W, H, false)];

/// `frame` in all four places.
fn four(frame: ExternalFrame) -> Vec<DrawCommand> {
    PLACES
        .iter()
        .map(|&(x, y, pixelated)| external(frame, x, y, pixelated))
        .collect()
}

/// What the last frame shows at the place `(x, y)`, exactly.
fn verdict(window: &VirtualWindow, (x, y): (u32, u32), number: u64) -> pattern::Verdict {
    verdict_within(window, (x, y), number, 0)
}

/// What the last frame shows at the place `(x, y)`, channels at most
/// `tolerance` off.
fn verdict_within(
    window: &VirtualWindow,
    (x, y): (u32, u32),
    number: u64,
    tolerance: u8,
) -> pattern::Verdict {
    let (width, _, pixels) = window.frame().expect("a frame was presented");
    pattern::check(pixels, width as usize, (x, y), (W, H), number, tolerance)
}

/// Whether the place `(x, y)` shows nothing (the white the frame is
/// composited over).
fn blank(window: &VirtualWindow, (x, y): (u32, u32)) -> bool {
    let (width, _, pixels) = window.frame().expect("a frame was presented");
    (y..y + H).all(|row| {
        pixels[(row * width + x) as usize..(row * width + x + W) as usize]
            .iter()
            .all(|&p| p == WHITE)
    })
}

/// The last frame shows `number` in all four places, on the GPU: nearest
/// exactly, bilinear within one level (the filter's rounding at texel
/// centres).
#[track_caller]
fn assert_four(window: &VirtualWindow, number: u64) {
    assert_eq!(window.renderer(), Some(Renderer::Gpu));
    for (x, y, pixelated) in PLACES {
        let tolerance = if pixelated { 0 } else { 1 };
        assert_eq!(
            verdict_within(window, (x, y), number, tolerance),
            pattern::Verdict::Ok,
            "frame {number} at ({x}, {y}), {}",
            if pixelated { "nearest" } else { "bilinear" }
        );
    }
}

fn frames_match_in_four_places(backend: GpuBackend) -> Result<(), String> {
    let (mut window, adapter) = gpu_window(backend)?;
    let mut producer = imported(&mut window, &adapter, IMAGE);
    for number in 0..FRAMES {
        let frame = producer.draw(number).expect("the engine draws");
        show(&mut window, four(frame));
        assert_four(&window, number);
    }
    assert!(
        window
            .take_renderer_changes()
            .iter()
            .all(|c| !matches!(c, RendererChange::ExternalFrameMissed { .. }))
    );
    drop(window);
    Ok(())
}

/// Every frame, drawn at its own size in four places of one scene (two
/// nearest, two bilinear), shows the engine's picture and number exactly.
#[cfg(windows)]
#[test]
fn frames_match_in_four_places_on_dx12() {
    if let Err(reason) = frames_match_in_four_places(GpuBackend::Dx12) {
        eprintln!("SKIPPED frames_match_in_four_places_on_dx12: {reason}");
    }
}

#[test]
fn frames_match_in_four_places_on_vulkan() {
    if let Err(reason) = frames_match_in_four_places(GpuBackend::Vulkan) {
        eprintln!("SKIPPED frames_match_in_four_places_on_vulkan: {reason}");
    }
}

fn frames_match_with_a_wgpu_engine(backend: GpuBackend) -> Result<(), String> {
    let (mut window, adapter) = gpu_window(backend)?;
    let mut producer = Producer::with_wgpu(&adapter, IMAGE, W, H).expect("the engine starts");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    window
        .import_external_image(IMAGE, shared)
        .expect("the window imports the pool");
    for number in 0..FRAMES {
        let frame = producer.draw(number).expect("the engine draws");
        show(&mut window, four(frame));
        assert_four(&window, number);
    }
    drop(window);
    Ok(())
}

/// The engine written with Kivact's wgpu (`kivact_gpu_share::wgpu`,
/// its own wgpu device next to the window's in this process; on Direct3D
/// 12 both wrap the process's one device of the adapter) shows every
/// frame exactly, as an engine on the graphics API itself.
#[cfg(windows)]
#[test]
fn frames_match_with_a_wgpu_engine_on_dx12() {
    if let Err(reason) = frames_match_with_a_wgpu_engine(GpuBackend::Dx12) {
        eprintln!("SKIPPED frames_match_with_a_wgpu_engine_on_dx12: {reason}");
    }
}

#[test]
fn frames_match_with_a_wgpu_engine_on_vulkan() {
    if let Err(reason) = frames_match_with_a_wgpu_engine(GpuBackend::Vulkan) {
        eprintln!("SKIPPED frames_match_with_a_wgpu_engine_on_vulkan: {reason}");
    }
}

fn drawing_ahead_waits_for_release(backend: GpuBackend) -> Result<(), String> {
    let (mut window, adapter) = gpu_window(backend)?;
    let mut producer = imported(&mut window, &adapter, IMAGE);
    let mut ahead: VecDeque<ExternalFrame> = (0..2)
        .map(|number| producer.draw(number).expect("the engine draws"))
        .collect();
    for number in 0..FRAMES {
        // Two frames ahead: the engine reuses a texture the window drew
        // two frames ago only once the window released it.
        ahead.push_back(producer.draw(number + 2).expect("the engine draws"));
        let frame = ahead.pop_front().expect("a frame drawn ahead");
        show(&mut window, four(frame));
        assert_four(&window, number);
    }
    drop(window);
    Ok(())
}

/// An engine drawing two frames ahead of the window never shows torn or
/// stale frames: it waits for the window's release.
#[cfg(windows)]
#[test]
fn drawing_ahead_waits_for_release_on_dx12() {
    if let Err(reason) = drawing_ahead_waits_for_release(GpuBackend::Dx12) {
        eprintln!("SKIPPED drawing_ahead_waits_for_release_on_dx12: {reason}");
    }
}

#[test]
fn drawing_ahead_waits_for_release_on_vulkan() {
    if let Err(reason) = drawing_ahead_waits_for_release(GpuBackend::Vulkan) {
        eprintln!("SKIPPED drawing_ahead_waits_for_release_on_vulkan: {reason}");
    }
}

fn a_frame_never_ready_is_not_drawn(backend: GpuBackend) -> Result<(), String> {
    let (mut window, adapter) = gpu_window(backend)?;
    let mut producer = imported(&mut window, &adapter, IMAGE);
    let drawn = producer.draw(0).expect("the engine draws");
    // The engine never signals this value.
    let never = ExternalFrame {
        image: IMAGE,
        slot: 1,
        value: 10,
    };
    let commands = vec![external(never, 0, 0, true), external(drawn, W, 0, true)];
    window.take_renderer_changes();
    let started = Instant::now();
    show(&mut window, commands.clone());
    let first = started.elapsed();
    assert!(
        first >= Duration::from_millis(95),
        "the frame waited for the ready value ({first:?})"
    );
    assert!(
        blank(&window, (0, 0)),
        "the frame never ready draws nothing"
    );
    assert_eq!(verdict(&window, (W, 0), 0), pattern::Verdict::Ok);
    assert_eq!(
        window.take_renderer_changes(),
        vec![RendererChange::ExternalFrameMissed { frame: never }],
        "the missing frame is noted once"
    );
    // Noted once: the next frame does not note it again (nor wait again,
    // which a time bound under a loaded test run cannot tell).
    show(&mut window, commands);
    assert!(blank(&window, (0, 0)));
    assert_eq!(window.take_renderer_changes(), Vec::new());
    drop(window);
    Ok(())
}

/// A frame whose ready value does not arrive within 100 ms is given up:
/// that command draws nothing (the others draw), noted once.
#[cfg(windows)]
#[test]
fn a_frame_never_ready_is_not_drawn_on_dx12() {
    if let Err(reason) = a_frame_never_ready_is_not_drawn(GpuBackend::Dx12) {
        eprintln!("SKIPPED a_frame_never_ready_is_not_drawn_on_dx12: {reason}");
    }
}

#[test]
fn a_frame_never_ready_is_not_drawn_on_vulkan() {
    if let Err(reason) = a_frame_never_ready_is_not_drawn(GpuBackend::Vulkan) {
        eprintln!("SKIPPED a_frame_never_ready_is_not_drawn_on_vulkan: {reason}");
    }
}

fn a_dropped_image_draws_nothing_and_releases(backend: GpuBackend) -> Result<(), String> {
    let (mut window, adapter) = gpu_window(backend)?;
    let mut producer = imported(&mut window, &adapter, IMAGE);
    let mut last = None;
    for number in 0..4 {
        let frame = producer.draw(number).expect("the engine draws");
        show(&mut window, four(frame));
        assert_four(&window, number);
        last = Some(frame);
    }
    let last = last.expect("frames were drawn");
    // While shown, the last frame is held: release is one below it.
    assert!(producer.wait_release(last.value - 1, Duration::from_secs(5)));
    assert_eq!(producer.release_value(), last.value - 1);
    window.drop_external_image(IMAGE);
    assert!(
        producer.wait_release(last.value, Duration::from_secs(5)),
        "dropping the image releases its last frame (release {})",
        producer.release_value()
    );
    show(&mut window, four(last));
    for (x, y, _) in PLACES {
        assert!(blank(&window, (x, y)), "a dropped image draws nothing");
    }
    drop(window);
    Ok(())
}

/// After `drop_external_image`, its frames draw nothing and the release
/// fence reaches the last frame's value.
#[cfg(windows)]
#[test]
fn a_dropped_image_draws_nothing_and_releases_on_dx12() {
    if let Err(reason) = a_dropped_image_draws_nothing_and_releases(GpuBackend::Dx12) {
        eprintln!("SKIPPED a_dropped_image_draws_nothing_and_releases_on_dx12: {reason}");
    }
}

#[test]
fn a_dropped_image_draws_nothing_and_releases_on_vulkan() {
    if let Err(reason) = a_dropped_image_draws_nothing_and_releases(GpuBackend::Vulkan) {
        eprintln!("SKIPPED a_dropped_image_draws_nothing_and_releases_on_vulkan: {reason}");
    }
}

fn mismatches_are_errors(backend: GpuBackend) -> Result<(), String> {
    let (mut window, adapter) = gpu_window(backend)?;
    // A new engine each time: an engine shares its pool once (a Vulkan
    // driver refuses to import memory exported a second time).
    let mut refused = |change: &dyn Fn(&mut kivact_types::SharedTextures), what: &str| {
        let mut producer = Producer::new(&adapter, IMAGE, W, H).expect("the engine starts");
        let mut shared = producer
            .shared_textures()
            .expect("the engine shares its pool");
        change(&mut shared);
        let error = window.import_external_image(IMAGE, shared).expect_err(what);
        assert_eq!(error.kind(), ErrorKind::Content, "{what}: {error}");
    };
    refused(
        &|shared| shared.adapter = AdapterMatch::Luid([0xa5; 8]),
        "textures of another adapter",
    );
    refused(
        &|shared| {
            shared.backend = match shared.backend {
                GpuBackend::Dx12 => GpuBackend::Vulkan,
                GpuBackend::Vulkan => GpuBackend::Dx12,
            }
        },
        "textures of another graphics API",
    );
    refused(&|shared| shared.width = 0, "an empty size");
    refused(
        &|shared| shared.height = 1 << 20,
        "a size beyond the device",
    );
    refused(&|shared| shared.textures.clear(), "no textures");
    if backend == GpuBackend::Dx12 {
        // Direct3D 12 tells a resource's size (Vulkan's memory does not).
        refused(
            &|shared| shared.width += 1,
            "another width than the textures'",
        );
    }
    // The window still imports the right textures.
    let mut producer = Producer::new(&adapter, IMAGE, W, H).expect("the engine starts");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    window
        .import_external_image(IMAGE, shared)
        .expect("the window imports the pool");
    let frame = producer.draw(0).expect("the engine draws");
    show(&mut window, four(frame));
    assert_four(&window, 0);
    drop(window);
    Ok(())
}

/// Textures that do not fit what they say or the window's device
/// (adapter, graphics API, size) are refused with `Content`.
#[cfg(windows)]
#[test]
fn mismatches_are_errors_on_dx12() {
    if let Err(reason) = mismatches_are_errors(GpuBackend::Dx12) {
        eprintln!("SKIPPED mismatches_are_errors_on_dx12: {reason}");
    }
}

#[test]
fn mismatches_are_errors_on_vulkan() {
    if let Err(reason) = mismatches_are_errors(GpuBackend::Vulkan) {
        eprintln!("SKIPPED mismatches_are_errors_on_vulkan: {reason}");
    }
}

fn a_storage_pool_is_imported(backend: GpuBackend) -> Result<(), String> {
    let (mut window, adapter) = gpu_window(backend)?;
    // A Vulkan adapter may not export images of the storage usage; every
    // Direct3D 12 adapter does.
    let mut producer = match Producer::with_storage(&adapter, IMAGE, W, H) {
        Ok(producer) => producer,
        Err(reason) if backend == GpuBackend::Vulkan => return Err(reason),
        Err(reason) => panic!("a Direct3D 12 storage pool: {reason}"),
    };
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    assert!(shared.storage);
    window
        .import_external_image(IMAGE, shared)
        .expect("the window imports the storage pool");
    for number in 0..4 {
        let frame = producer.draw(number).expect("the engine draws");
        show(&mut window, four(frame));
        assert_four(&window, number);
    }
    drop(window);
    Ok(())
}

/// A pool made for storage writes as well (`SharedTextures::storage`) is
/// imported and its frames draw as any pool's. Skipped where the adapter
/// cannot export such textures (Vulkan's storage usage is optional with
/// external memory).
#[cfg(windows)]
#[test]
fn a_storage_pool_is_imported_on_dx12() {
    if let Err(reason) = a_storage_pool_is_imported(GpuBackend::Dx12) {
        eprintln!("SKIPPED a_storage_pool_is_imported_on_dx12: {reason}");
    }
}

#[test]
fn a_storage_pool_is_imported_on_vulkan() {
    if let Err(reason) = a_storage_pool_is_imported(GpuBackend::Vulkan) {
        eprintln!("SKIPPED a_storage_pool_is_imported_on_vulkan: {reason}");
    }
}

/// Textures whose `storage` says otherwise than the resources' flags are
/// refused with `Content`, both ways; the window still imports the right
/// ones. Direct3D 12 only: a Direct3D 12 resource describes its flags,
/// while Vulkan's imported memory does not tell the usage of the image it
/// was made for (Arvane makes the image as `storage` says; a mismatch
/// there breaks the contract undetected).
#[cfg(windows)]
#[test]
fn a_usage_mismatch_is_refused_on_dx12() {
    let (mut window, adapter) = match gpu_window(GpuBackend::Dx12) {
        Ok(found) => found,
        Err(reason) => {
            eprintln!("SKIPPED a_usage_mismatch_is_refused_on_dx12: {reason}");
            return;
        }
    };
    for storage in [false, true] {
        let mut producer = if storage {
            Producer::with_storage(&adapter, IMAGE, W, H)
        } else {
            Producer::new(&adapter, IMAGE, W, H)
        }
        .expect("the engine starts");
        let mut shared = producer
            .shared_textures()
            .expect("the engine shares its pool");
        shared.storage = !storage;
        let error = window
            .import_external_image(IMAGE, shared)
            .expect_err("a pool whose storage says otherwise");
        assert_eq!(
            error.kind(),
            ErrorKind::Content,
            "storage {storage}: {error}"
        );
    }
    let mut producer = Producer::with_storage(&adapter, IMAGE, W, H).expect("the engine starts");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    window
        .import_external_image(IMAGE, shared)
        .expect("the window imports the pool");
    let frame = producer.draw(0).expect("the engine draws");
    show(&mut window, four(frame));
    assert_four(&window, 0);
    drop(window);
}

fn two_windows_draw_only_their_own_images(backend: GpuBackend) -> Result<(), String> {
    let (mut first, first_adapter) = gpu_window(backend)?;
    let (mut second, second_adapter) = gpu_window(backend)?;
    let (one, two) = (ExternalImageId(1), ExternalImageId(2));
    let mut first_engine = imported(&mut first, &first_adapter, one);
    let mut second_engine = imported(&mut second, &second_adapter, two);
    for number in 0..3 {
        let a = first_engine.draw(number).expect("the engine draws");
        let b = second_engine.draw(number).expect("the engine draws");
        let commands = vec![external(a, 0, 0, true), external(b, W, 0, true)];
        show(&mut first, commands.clone());
        show(&mut second, commands);
        assert_eq!(verdict(&first, (0, 0), number), pattern::Verdict::Ok);
        assert!(blank(&first, (W, 0)), "the second window's image");
        assert!(blank(&second, (0, 0)), "the first window's image");
        assert_eq!(verdict(&second, (W, 0), number), pattern::Verdict::Ok);
    }
    drop((first, second));
    Ok(())
}

/// Two windows each import their own engine's pool; neither draws the
/// other's image (an image belongs to the window it was imported into).
#[cfg(windows)]
#[test]
fn two_windows_draw_only_their_own_images_on_dx12() {
    if let Err(reason) = two_windows_draw_only_their_own_images(GpuBackend::Dx12) {
        eprintln!("SKIPPED two_windows_draw_only_their_own_images_on_dx12: {reason}");
    }
}

#[test]
fn two_windows_draw_only_their_own_images_on_vulkan() {
    if let Err(reason) = two_windows_draw_only_their_own_images(GpuBackend::Vulkan) {
        eprintln!("SKIPPED two_windows_draw_only_their_own_images_on_vulkan: {reason}");
    }
}

/// A CPU window draws nothing for an external frame and the scene is
/// still valid; it has no adapter to import into.
#[test]
fn a_cpu_window_draws_no_external_frame() {
    let mut window = open(2.0 * W as f32, 2.0 * H as f32);
    let frame = ExternalFrame {
        image: IMAGE,
        slot: 0,
        value: 1,
    };
    let plan = window.frame_plan();
    let presented = window
        .present(
            &kivact_types::Scene {
                commands: four(frame),
            },
            plan,
        )
        .expect("the window can present");
    assert!(
        matches!(
            presented,
            Presented::Shown {
                renderer: Renderer::Cpu,
                ..
            }
        ),
        "{presented:?}"
    );
    for (x, y, _) in PLACES {
        assert!(blank(&window, (x, y)));
    }
    assert_eq!(window.gpu_adapter(), None);
    // Textures from an engine on this machine's GPU are refused as
    // unavailable, not as invalid.
    let (gpu, adapter) = match gpu_window(GpuBackend::Vulkan) {
        Ok(found) => found,
        Err(reason) => {
            eprintln!("SKIPPED a CPU window refusing an import: {reason}");
            return;
        }
    };
    drop(gpu);
    let mut producer = Producer::new(&adapter, IMAGE, W, H).expect("the engine starts");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    let error = window
        .import_external_image(IMAGE, shared)
        .expect_err("a CPU window cannot import");
    assert_eq!(error.kind(), ErrorKind::Unavailable);
}
