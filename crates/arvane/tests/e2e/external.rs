//! Role: Textures an external render engine shares, drawn on GPU test windows.
//!
//! The engine is `gpu-interop-producer` (a device of its own on the
//! window's adapter, its frames a known picture with the frame number in
//! it). Each test runs once per graphics API: Direct3D 12 (Windows) and
//! Vulkan. A test is skipped, saying why, when the window cannot have a
//! GPU device of that API that imports (no hardware adapter, an adapter
//! without the extensions, or `WGPU_BACKEND` choosing another API).

use crate::support::Harness;
use arvane::{Renderer, RendererChange};
use arvane_virtual::VirtualOptions;
use gpu_interop_producer::{Producer, pattern};
use kivact_types::render::{Composition, ExternalItem, LayerItem};
use kivact_types::{
    ExternalFrame, ExternalImageId, GpuAdapter, GpuBackend, GpuPreference, Transform,
};

use crate::support::{options, rect, root_layer, show_composition};

/// The engine's frame size: a window shows four frames.
const W: u32 = 256;
const H: u32 = 128;
const FRAMES: u64 = 60;
const IMAGE: ExternalImageId = ExternalImageId(1);

/// A GPU test window of `backend` that can import, and its adapter; why
/// not when there is none here (the test is skipped).
fn gpu_window(backend: GpuBackend) -> Result<(Harness, GpuAdapter), String> {
    let opened = Harness::configured(
        VirtualOptions {
            gpu: arvane_virtual::GpuMode::Required,
            ..options(2.0 * W as f64, 2.0 * H as f64)
        },
        GpuPreference {
            backend: Some(backend),
            ..GpuPreference::default()
        },
    );
    let mut window = match opened {
        Ok(window) => window,
        Err(error) => return Err(error.to_string()),
    };
    let Some(adapter) = window.with(move |w| w.gpu_adapter()) else {
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
fn imported(window: &mut Harness, adapter: &GpuAdapter, image: ExternalImageId) -> Producer {
    let mut producer = Producer::new(adapter, image, W, H).expect("the engine starts");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    window
        .with(move |w| w.import_external_image(image, shared))
        .expect("the window imports the pool");
    producer
}

/// `frame` drawn at its own size at `(x, y)`.
fn external(frame: ExternalFrame, x: u32, y: u32, pixelated: bool) -> LayerItem {
    LayerItem::External(ExternalItem {
        frame,
        rect: rect(x as f64, y as f64, W as f64, H as f64),
        source: None,
        transform: Transform::IDENTITY,
        clips: Vec::new(),
        opacity: 1.0,
        pixelated,
        cpu: None,
    })
}

/// The window's four places: nearest on top, bilinear below.
const PLACES: [(u32, u32, bool); 4] = [(0, 0, true), (W, 0, true), (0, H, false), (W, H, false)];

/// `frame` in all four places.
fn four(frame: ExternalFrame) -> Vec<LayerItem> {
    PLACES
        .iter()
        .map(|&(x, y, pixelated)| external(frame, x, y, pixelated))
        .collect()
}

/// What the last frame shows at the place `(x, y)`, channels at most
/// `tolerance` off.
fn verdict_within(
    window: &Harness,
    (x, y): (u32, u32),
    number: u64,
    tolerance: u8,
) -> pattern::Verdict {
    let (width, _, pixels) = window.probe().frame().expect("a frame was presented");
    pattern::check(&pixels, width as usize, (x, y), (W, H), number, tolerance)
}

/// The last frame shows `number` in all four places, on the GPU: nearest
/// exactly, bilinear within one level (the filter's rounding at texel
/// centres).
#[track_caller]
fn assert_four(window: &Harness, number: u64) {
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
        let items = four(frame);
        show_composition(&mut window, move |size| Composition {
            layers: vec![root_layer(size, items)],
            exports: Vec::new(),
        });
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
