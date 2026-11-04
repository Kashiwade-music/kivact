//! Role: Shared GPU frames and CPU fallback in viewport_surface.
//!
//! The App is the recommended one: a `gpu_interop::shared_image` whose
//! `frame()` and `pixels()` go to `viewport_surface(..).gpu_frame(..)`. The
//! engine is `gpu-interop-producer` (a device of its own on the window's
//! adapter, its frames a known picture), on a thread of its own with the
//! image's `ShareLink`, as in the public API's `gpu_interop` e2e; the
//! test's thread (the UI thread) answers what the engine's thread asks it
//! to check. The surface is as large as the frame and gets no transform,
//! so it fits the frame at scale 1; the `Pixels` the engine publishes on
//! the CPU are small and solid, so the two cannot be confused and their
//! size shows which image the surface took its natural size from. The GPU
//! tests run on the default adapter (Direct3D 12 or Vulkan) and are
//! skipped, saying why, when the window cannot have a GPU device that
//! imports.
#![allow(
    unsafe_code,
    reason = "providing shared textures is an unsafe fn: the test engine keeps its contract"
)]
#![allow(
    clippy::print_stderr,
    reason = "report unavailable optional hardware explicitly"
)]

use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use gpu_interop_producer::{Producer, pattern};
use kivact::draw::Pixels;
use kivact::gpu_interop::{self, GpuAdapter, ShareLink, ShareMode, ShareRequest, ShareStatus};
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions, Screen, gpu_unavailable};
use kivact_ui_creative as ui;

/// The frame's size, and the surface's.
const W: u32 = 256;
const H: u32 = 128;
/// The side of the solid `Pixels` the engine publishes on the CPU.
const SIDE: u32 = 16;
/// A channel's distance from the pattern: the frame is sampled bilinearly at scale 1
/// (`pixelated` applies to an enlarged image only), within one level as the
/// public API's bilinear case.
const TOLERANCE: u8 = 1;
/// The longest the engine's thread waits for the UI thread.
const UI_WAIT: Duration = Duration::from_secs(10);

/// What the engine's thread asks the test's (UI) thread to do, with the
/// surface.
type Ask = Box<dyn FnOnce(&Surface) + Send>;

/// The engine thread's way to the test's thread: each ask waits for its
/// answer.
struct Ui {
    ask: mpsc::Sender<Ask>,
    answered: mpsc::Receiver<()>,
}

impl Ui {
    fn ask(&self, ask: impl FnOnce(&Surface) + Send + 'static) {
        self.ask.send(Box::new(ask)).expect("the UI thread listens");
        self.answered.recv().expect("the UI thread answers");
    }
}

/// An App whose surface shows the frame of a `shared_image` over its
/// `Pixels`.
struct Surface {
    screen: Screen,
    adapter: Value<Option<GpuAdapter>>,
    image: gpu_interop::SharedImage,
}

impl Surface {
    #[track_caller]
    fn mount(options: RenderOptions) -> Surface {
        let (screen, (adapter, image)) = kivact::test::render_with_handle(options, move || {
            let adapter = memo(gpu_interop::adapter);
            let image = gpu_interop::shared_image(W, H);
            let handles = (adapter, image.clone());
            let (frame, pixels) = (image.clone(), image);
            (
                handles,
                ui::viewport_surface(move || pixels.pixels())
                    .gpu_frame(move || frame.frame())
                    .pixelated(true)
                    .style(style().size(W, H)),
            )
        });
        Surface {
            screen,
            adapter,
            image,
        }
    }

    /// Run `engine` on a thread of its own with the image's link, and
    /// answer what it asks on this (the UI) thread, settling the App in
    /// between, until the engine's thread ends.
    fn run_engine(&self, engine: impl FnOnce(ShareLink, Ui) + Send + 'static) {
        let (ask, asked) = mpsc::channel::<Ask>();
        let (answer, answered) = mpsc::channel();
        let link = self.image.link();
        let thread = std::thread::spawn(move || engine(link, Ui { ask, answered }));
        let start = Instant::now();
        loop {
            self.screen.run_until_idle();
            match asked.recv_timeout(Duration::from_millis(5)) {
                Ok(ask) => {
                    ask(self);
                    answer.send(()).expect("the engine's thread waits");
                }
                Err(mpsc::RecvTimeoutError::Timeout) => assert!(
                    start.elapsed() < Duration::from_secs(120),
                    "the engine's thread does not finish"
                ),
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        if let Err(panic) = thread.join() {
            std::panic::resume_unwind(panic);
        }
    }

    /// Settle (the engine's last publish woke the UI thread before it
    /// asked), and what the surface shows of frame `number` of the
    /// pattern.
    fn frame_verdict(&self, number: u64) -> pattern::Verdict {
        self.screen.run_until_idle();
        let shot = self.screen.screenshot();
        pattern::check(
            &shot.pixels,
            shot.width as usize,
            (0, 0),
            (W, H),
            number,
            TOLERANCE,
        )
    }
}

/// The solid picture the engine keeps on the CPU for a window that cannot
/// show its frames.
fn solid() -> Pixels {
    Pixels::new(
        SIDE,
        SIDE,
        Arc::from([0xd0, 0x60, 0x20, 0xff].repeat((SIDE * SIDE) as usize)),
    )
}

/// A GPU surface that can import; why not when there is none here (the
/// test is skipped).
fn gpu_surface() -> Result<Surface, String> {
    if let Some(reason) = gpu_unavailable() {
        return Err(reason);
    }
    let surface = Surface::mount(
        RenderOptions::from_env()
            .window_size(W as f64, (H + 40) as f64)
            .theme(ui::theme(kivact::style::ThemeMode::Dark))
            .reduced_motion()
            .gpu()
            .mirror(MirrorMode::Required),
    );
    if surface.adapter.get_untracked().is_none() {
        return Err("the window's GPU device cannot import shared textures".into());
    }
    Ok(surface)
}

/// The newest `ShareMode::Gpu` request, and its adapter (a request of
/// `Pixels` may come first, while the window's device is not ready).
fn gpu_request(link: &ShareLink) -> (ShareRequest, GpuAdapter) {
    loop {
        let request = link.wait_request(UI_WAIT).expect("the image asks");
        if let ShareMode::Gpu(adapter) = &request.mode {
            let adapter = adapter.clone();
            return (request, adapter);
        }
    }
}

/// The image's `Gpu` request, served by the engine: the solid `Pixels`
/// published for a window that cannot import, and its pool provided,
/// imported and ready.
fn serve(link: &ShareLink) -> (ShareRequest, Producer) {
    let (request, adapter) = gpu_request(link);
    assert_eq!((request.width, request.height), (W, H));
    link.publish_pixels(solid());
    let mut producer = Producer::new(&adapter, Default::default(), W, H)
        .expect("the engine starts on the adapter");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    // SAFETY: the test engine keeps the contract of `SharedTextures`.
    unsafe { link.provide(&request, shared) }.expect("the newest request takes the pool");
    link.wait_ready(&request, UI_WAIT)
        .expect("the pool is imported");
    (request, producer)
}

/// The surface shows the engine's frames, a frame at a time, at the frame's
/// own size: it fits the surface at scale 1, so the pattern is exact (the
/// `Pixels` are 16 px).
fn shows_the_gpu_frame(surface: Surface) {
    surface.run_engine(|link, ui| {
        let (request, mut producer) = serve(&link);
        for number in 0..4 {
            let frame = producer.draw(number).expect("the engine draws");
            link.publish(&request, frame.slot, frame.value);
            ui.ask(move |surface| {
                assert_eq!(surface.image.status(), ShareStatus::Ready);
                assert_eq!(
                    surface.frame_verdict(number),
                    pattern::Verdict::Ok,
                    "frame {number}"
                );
            });
        }
    });
    assert!(
        surface.screen.errors().is_empty(),
        "{:?}",
        surface.screen.errors()
    );
}

#[test]
fn the_surface_shows_the_gpu_frame() {
    match gpu_surface() {
        Ok(surface) => shows_the_gpu_frame(surface),
        Err(reason) => eprintln!("SKIPPED the_surface_shows_the_gpu_frame: {reason}"),
    }
}
