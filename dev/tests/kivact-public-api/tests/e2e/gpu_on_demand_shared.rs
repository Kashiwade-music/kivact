//! Role: The GPU device a `shared_image` starts when its window waits for one.
//!
//! `shared_image` reads the window's adapter: a window that waits starts
//! its device, and the engine's GPU frames arrive; a window on the CPU
//! only asks for pixels and starts none.
#![allow(
    unsafe_code,
    reason = "providing shared textures is an unsafe fn: the test engine keeps its contract"
)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpu_interop_producer::{Producer, pattern};
use kivact::draw::ImageOptions;
use kivact::geometry::Rect;
use kivact::gpu_interop::{self, ShareMode, ShareStatus, SharedImage};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Renderer, Screen};
use kivact::view::canvas;

use super::gpu_on_demand::{GRACE, cpu_forced, cpu_only, gpu_lane, on_demand};
use super::support::mount_with;

/// The image's size, and the window's: the producer's pattern and its check
/// fit it (`pattern::fits`, and wider than 200 for the check's clear column).
const W: u32 = 224;
const H: u32 = 96;
/// The longest a step of the exchange waits (real time).
const WAIT: Duration = Duration::from_secs(10);

/// A window of the image's size, started with `options`, that makes a
/// `shared_image` once `go` is set and draws its GPU frame at its own size.
#[track_caller]
fn mount(options: RenderOptions) -> (Screen, Signal<bool>, Rc<RefCell<Option<SharedImage>>>) {
    let options = options.window_size(f64::from(W), f64::from(H));
    let image = Rc::new(RefCell::new(None::<SharedImage>));
    let handed = image.clone();
    let (screen, go) = mount_with(options, move || {
        let go = signal(false);
        let view = show(go, move || {
            let shared = gpu_interop::shared_image(W, H);
            *handed.borrow_mut() = Some(shared.clone());
            canvas(move |d, _| {
                if let Some(f) = shared.frame() {
                    let rect = Rect::new(0.0, 0.0, f64::from(W), f64::from(H));
                    let options = ImageOptions {
                        pixelated: true,
                        ..ImageOptions::default()
                    };
                    d.gpu_frame(&f, rect, options);
                }
            })
            .style(style().size(W, H))
        });
        (go, view)
    });
    (screen, go, image)
}

/// A shared image in a window started with `options` asks for pixels,
/// and no device starts, given the time to.
#[track_caller]
fn shares_pixels_on_cpu(options: RenderOptions) {
    let (screen, go, image) = mount(options);
    go.set(true);
    screen.run_until_idle();
    std::thread::sleep(GRACE);
    screen.run_until_idle();
    let image = image.borrow().clone().expect("the window made its image");
    assert_eq!(
        image.status(),
        ShareStatus::Pixels,
        "a window on the CPU asks for pixels"
    );
    assert_eq!(screen.renderer(), Renderer::Cpu, "and starts no device");
    let request = image.link().poll_request().expect("the image asks");
    assert!(
        matches!(request.mode, ShareMode::Pixels),
        "{:?}",
        request.mode
    );
}

/// Making a shared image starts the waiting device; the image then asks
/// for textures on the window's adapter, and the engine's frame arrives
/// and is drawn by the GPU.
#[test]
fn a_shared_image_starts_the_gpu_on_demand() {
    if !gpu_lane("a_shared_image_starts_the_gpu_on_demand") {
        return;
    }
    if cpu_forced() {
        return shares_pixels_on_cpu(on_demand());
    }
    let (screen, go, image) = mount(on_demand());
    assert_eq!(
        screen.renderer(),
        Renderer::Cpu,
        "nothing needs the GPU yet"
    );
    go.set(true);
    screen.run_until_idle();
    assert!(
        matches!(screen.renderer(), Renderer::GpuStarting | Renderer::Gpu),
        "making a shared image starts the device, not {:?}",
        screen.renderer()
    );
    screen.wait_for_gpu();
    let image = image.borrow().clone().expect("the window made its image");
    let link = image.link();
    // A request of pixels may come first, while the device started.
    let start = Instant::now();
    let (request, adapter) = loop {
        screen.run_until_idle();
        if let Some(request) = link.poll_request()
            && let ShareMode::Gpu(adapter) = &request.mode
        {
            let adapter = adapter.clone();
            break (request, adapter);
        }
        assert!(
            start.elapsed() < WAIT,
            "the image asks for textures on the ready device"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!((request.width, request.height), (W, H));
    let mut producer = Producer::new(&adapter, Default::default(), W, H)
        .expect("the engine starts on the adapter");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    // SAFETY: the test engine keeps the contract of `SharedTextures`.
    unsafe { link.provide(&request, shared) }.expect("the newest request takes the pool");
    let start = Instant::now();
    while image.status() != ShareStatus::Ready {
        assert!(
            start.elapsed() < WAIT,
            "the pool is imported: {:?}",
            image.status()
        );
        std::thread::sleep(Duration::from_millis(5));
        screen.run_until_idle();
    }
    let frame = producer.draw(0).expect("the engine draws");
    link.publish(&request, frame.slot, frame.value);
    screen.run_until_idle();
    assert!(image.frame().is_some(), "the GPU frame arrived");
    let shot = screen.screenshot();
    assert_eq!(
        pattern::check(&shot.pixels, shot.width as usize, (0, 0), (W, H), 0, 0),
        pattern::Verdict::Ok,
        "the window draws the engine's frame"
    );
    assert_eq!(screen.renderer(), Renderer::Gpu);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    go.set(false);
    screen.run_until_idle();
    assert!(
        producer.wait_release(frame.value, Duration::from_secs(5)),
        "the dropped image releases its frame (release {})",
        producer.release_value()
    );
}

/// A shared image in a window on the CPU only asks for pixels and starts
/// no device (the App's demand is ignored on the CPU).
#[test]
fn a_cpu_window_shares_pixels_without_a_device() {
    if !gpu_lane("a_cpu_window_shares_pixels_without_a_device") {
        return;
    }
    shares_pixels_on_cpu(cpu_only());
}
