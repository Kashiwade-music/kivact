//! Role: External engine textures imported and drawn through kivact::gpu_interop.
//!
//! The engine is `gpu-interop-producer` (a device of its own on the
//! window's adapter, its frames a known picture with the frame number in
//! it), as in Arvane's contract tests (`crates/arvane/tests/e2e/external.rs`).
//! The low-level tests import its pool themselves and publish its frames
//! from another thread (`ExternalImage::publisher`); the `shared_image`
//! tests run the engine on a thread of its own with a `ShareLink`, as an
//! App would, and the test's thread (the UI thread) answers what it asks
//! to check. Each GPU test runs once per graphics API, Direct3D 12
//! (Windows) and Vulkan, and is skipped, saying why, when the window
//! cannot have a GPU device of that API that imports.
#![allow(
    unsafe_code,
    reason = "importing shared textures is an unsafe fn: the test engine keeps its contract"
)]
#![allow(
    clippy::print_stderr,
    reason = "report unavailable optional hardware explicitly"
)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use gpu_interop_producer::{Producer, pattern};
use kivact::view::canvas;
use kivact::ErrorKind;
use kivact::draw::ImageOptions;
use kivact::geometry::Rect;
use kivact::gpu_interop::{self, AdapterMatch, ExternalImage, FramePublisher, GpuAdapter, GpuBackend, GpuPreference, ImportStatus, ShareError, ShareLink, ShareMode, ShareRequest, ShareStatus, SharedHandle, SharedImage, SharedTextureFormat, SharedTextures};
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions, Screen, gpu_unavailable};

use super::support::mount_with;

/// The engine's frame size: the canvas shows a frame twice.
const W: u32 = 256;
const H: u32 = 128;
/// Frames shown (each checked in both places).
const FRAMES: u64 = 12;

/// The places of the canvas: nearest on the left, bilinear on the right.
const PLACES: [(u32, bool); 2] = [(0, true), (W, false)];

/// An App whose viewport imports the textures in `textures` while `go`
/// is true (`show`: the import's owner) and draws the frames published
/// for the import (`ExternalImage::latest_frame`) in both places.
struct Viewport {
    screen: Screen,
    adapter: Value<Option<GpuAdapter>>,
    go: Signal<bool>,
    textures: Rc<RefCell<Option<SharedTextures>>>,
    image: Rc<RefCell<Option<ExternalImage>>>,
}

impl Viewport {
    #[track_caller]
    fn mount(options: RenderOptions) -> Viewport {
        let textures = Rc::new(RefCell::new(None::<SharedTextures>));
        let image = Rc::new(RefCell::new(None::<ExternalImage>));
        let (pool, handed) = (textures.clone(), image.clone());
        let (screen, (adapter, go)) = mount_with(options, move || {
            let adapter = memo(gpu_interop::adapter);
            let go = signal(false);
            let view = show(go, move || {
                let shared = pool.borrow_mut().take().expect("textures to import");
                // SAFETY: the test engine keeps the contract of
                // `SharedTextures` (or the window refuses the textures
                // before it reads them).
                let imported = unsafe { gpu_interop::import(shared) };
                *handed.borrow_mut() = Some(imported.clone());
                canvas(move |d, _| {
                    if let Some(f) = imported.latest_frame() {
                        for (x, pixelated) in PLACES {
                            let options = ImageOptions {
                                pixelated,
                                ..ImageOptions::default()
                            };
                            let rect = Rect::new(x as f64, 0.0, W as f64, H as f64);
                            d.gpu_frame(&f, rect, options);
                        }
                    }
                })
                .style(style().size(2 * W, H))
            });
            ((adapter, go), view)
        });
        Viewport {
            screen,
            adapter,
            go,
            textures,
            image,
        }
    }

    /// Import `textures` (a new owner) and settle.
    fn import(&self, textures: SharedTextures) -> ExternalImage {
        *self.textures.borrow_mut() = Some(textures);
        self.go.set(true);
        self.screen.run_until_idle();
        self.image.borrow().clone().expect("the viewport imported")
    }

    /// Dispose of the import's owner and settle.
    fn dispose(&self) {
        self.go.set(false);
        self.screen.run_until_idle();
    }

    /// The publisher of the import.
    fn publisher(&self) -> FramePublisher {
        self.image
            .borrow()
            .as_ref()
            .expect("the viewport imported")
            .publisher()
    }

    /// Publish a frame from another thread, as the engine's thread would.
    fn show(&self, slot: u32, value: u64) {
        let publisher = self.publisher();
        std::thread::spawn(move || publisher.publish(slot, value))
            .join()
            .expect("the publishing thread ends");
    }
}

fn gpu_options(backend: GpuBackend) -> RenderOptions {
    RenderOptions::from_env()
        .window_size((2 * W) as f64, (H + 40) as f64)
        .gpu()
        .mirror(MirrorMode::Required)
        .gpu_preference(GpuPreference {
            backend: Some(backend),
            ..GpuPreference::default()
        })
}

/// A GPU viewport of `backend` that can import, and its adapter; why not
/// when there is none here (the test is skipped).
fn gpu_viewport(backend: GpuBackend) -> Result<(Viewport, GpuAdapter), String> {
    if let Some(reason) = gpu_unavailable() {
        return Err(reason);
    }
    let viewport = Viewport::mount(gpu_options(backend));
    let Some(adapter) = viewport.adapter.get_untracked() else {
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
    Ok((viewport, adapter))
}

fn imports_draw_and_drop(backend: GpuBackend) -> Result<(), String> {
    let (viewport, adapter) = gpu_viewport(backend)?;
    let screen = &viewport.screen;
    // The engine makes its device on the adapter the window reports.
    let mut producer =
        Producer::new(&adapter, Default::default(), W, H).expect("the engine starts on it");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    let image = viewport.import(shared);
    assert_eq!(image.status(), ImportStatus::Ready);
    // A frame knows the size of the textures it was imported with.
    let sized = image.frame(0, 1);
    assert_eq!((sized.width(), sized.height()), (W, H));
    // The engine writes its textures only once the import is ready.
    let mut last = None;
    for number in 0..FRAMES {
        let frame = producer.draw(number).expect("the engine draws");
        viewport.show(frame.slot, frame.value);
        let shot = screen.screenshot();
        for (x, pixelated) in PLACES {
            let tolerance = if pixelated { 0 } else { 1 };
            assert_eq!(
                pattern::check(
                    &shot.pixels,
                    shot.width as usize,
                    (x, 0),
                    (W, H),
                    number,
                    tolerance
                ),
                pattern::Verdict::Ok,
                "frame {number} at x {x}, {}",
                if pixelated { "nearest" } else { "bilinear" }
            );
        }
        last = Some(frame);
    }
    let last = last.expect("frames were shown");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    // Disposing the owner drops the image, whatever handles are left: the
    // window releases its last frame, and the publisher is closed.
    let publisher = viewport.publisher();
    assert!(!publisher.is_closed());
    viewport.dispose();
    assert!(publisher.is_closed(), "the import's owner is gone");
    assert!(
        producer.wait_release(last.value, Duration::from_secs(5)),
        "the dropped image releases its last frame (release {})",
        producer.release_value()
    );
    assert_eq!(image.status(), ImportStatus::Dropped, "the owner is gone");
    // Textures of another adapter are refused, as a platform failure.
    let mut other = Producer::new(&adapter, Default::default(), W, H).expect("the engine starts");
    let mut shared = other.shared_textures().expect("the engine shares its pool");
    shared.adapter = AdapterMatch::Luid([0xa5; 8]);
    let refused = viewport.import(shared);
    match refused.status() {
        ImportStatus::Failed(error) => {
            assert_eq!(*error.kind(), ErrorKind::Platform, "{error}");
            assert!(error.location().is_some(), "located at the import");
        }
        status => panic!("textures of another adapter: {status:?}"),
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    Ok(())
}

/// On the window's adapter (as the App asked: the graphics API), an
/// engine's import is ready and the frames another thread publishes
/// (`ExternalImage::publisher`) draw in a canvas, nearest and bilinear,
/// frame after frame; disposing the import's owner closes the publisher
/// and releases the last frame; textures of another adapter fail as
/// `Platform`.
#[cfg(windows)]
#[test]
fn imports_draw_and_drop_on_dx12() {
    if let Err(reason) = imports_draw_and_drop(GpuBackend::Dx12) {
        eprintln!("SKIPPED imports_draw_and_drop_on_dx12: {reason}");
    }
}

#[test]
fn imports_draw_and_drop_on_vulkan() {
    if let Err(reason) = imports_draw_and_drop(GpuBackend::Vulkan) {
        eprintln!("SKIPPED imports_draw_and_drop_on_vulkan: {reason}");
    }
}

/// Textures whose handles are an open file's: a window that draws on the
/// CPU refuses them before it reads a handle (it closes them).
fn unread_textures() -> SharedTextures {
    let handle = || {
        let file = std::fs::File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
            .expect("the manifest opens");
        #[cfg(windows)]
        let handle = SharedHandle::from(std::os::windows::io::OwnedHandle::from(file));
        #[cfg(target_os = "linux")]
        let handle = SharedHandle::from(std::os::fd::OwnedFd::from(file));
        handle
    };
    SharedTextures {
        width: W,
        height: H,
        format: SharedTextureFormat::Rgba8UnormPremultiplied,
        backend: GpuBackend::Vulkan,
        adapter: AdapterMatch::Luid([0; 8]),
        storage: false,
        textures: vec![handle()],
        ready: handle(),
        release: handle(),
    }
}

/// The size `shared_image` asks after `set_size` in the tests.
const W2: u32 = 224;
const H2: u32 = 96;

/// The longest the engine's thread waits for the UI thread.
const UI_WAIT: Duration = Duration::from_secs(10);

/// What the engine's thread asks the test's (UI) thread to do.
enum Ask {
    /// Check that the window shows frame `number` of a pool of the size.
    Shows {
        number: u64,
        width: u32,
        height: u32,
    },
    /// Ask the image for another size.
    Resize(u32, u32),
    /// Dispose of the image's owner.
    Dispose,
}

/// The engine thread's way to the test's thread: each ask waits for its
/// answer.
struct Ui {
    ask: mpsc::Sender<Ask>,
    answered: mpsc::Receiver<()>,
}

impl Ui {
    fn ask(&self, ask: Ask) {
        self.ask.send(ask).expect("the UI thread listens");
        self.answered.recv().expect("the UI thread answers");
    }
}

/// An App whose viewport has a `shared_image` of `W` × `H` while `go` is
/// true (`show`: the image's owner), drawing its frame at its own size,
/// or else its pixels.
struct SharedViewport {
    screen: Screen,
    adapter: Value<Option<GpuAdapter>>,
    go: Signal<bool>,
    image: SharedImage,
}

impl SharedViewport {
    #[track_caller]
    fn mount(options: RenderOptions) -> SharedViewport {
        let image = Rc::new(RefCell::new(None::<SharedImage>));
        let handed = image.clone();
        let (screen, (adapter, go)) = mount_with(options, move || {
            let adapter = memo(gpu_interop::adapter);
            let go = signal(true);
            let view = show(go, move || {
                let shared = gpu_interop::shared_image(W, H);
                *handed.borrow_mut() = Some(shared.clone());
                canvas(move |d, _| {
                    let options = ImageOptions {
                        pixelated: true,
                        ..ImageOptions::default()
                    };
                    if let Some(f) = shared.frame() {
                        let rect = Rect::new(0.0, 0.0, f.width() as f64, f.height() as f64);
                        d.gpu_frame(&f, rect, options);
                    } else if let Some(p) = shared.pixels() {
                        let rect = Rect::new(0.0, 0.0, p.width as f64, p.height as f64);
                        d.pixels(&p, rect, options);
                    }
                })
                .style(style().size(2 * W, H))
            });
            ((adapter, go), view)
        });
        let image = image.borrow().clone().expect("the viewport made its image");
        SharedViewport {
            screen,
            adapter,
            go,
            image,
        }
    }

    /// Run `engine` on a thread of its own with the image's link, and
    /// answer what it asks on this (the UI) thread, settling the App in
    /// between, until the engine's thread ends.
    fn run_engine(&self, engine: impl FnOnce(ShareLink, Ui) + Send + 'static) {
        let (ask, asked) = mpsc::channel();
        let (answer, answered) = mpsc::channel();
        let link = self.image.link();
        let thread = std::thread::spawn(move || engine(link, Ui { ask, answered }));
        let start = Instant::now();
        loop {
            self.screen.run_until_idle();
            match asked.recv_timeout(Duration::from_millis(5)) {
                Ok(request) => {
                    self.answer(request);
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

    fn answer(&self, ask: Ask) {
        let screen = &self.screen;
        match ask {
            Ask::Shows {
                number,
                width,
                height,
            } => {
                // The publish woke the UI thread before the ask was sent.
                screen.run_until_idle();
                assert_eq!(self.image.status(), ShareStatus::Ready);
                let frame = self.image.frame().expect("a frame of the ready import");
                assert_eq!((frame.width(), frame.height()), (width, height));
                let shot = screen.screenshot();
                assert_eq!(
                    pattern::check(
                        &shot.pixels,
                        shot.width as usize,
                        (0, 0),
                        (width, height),
                        number,
                        0
                    ),
                    pattern::Verdict::Ok,
                    "frame {number} of {width} x {height}"
                );
            }
            Ask::Resize(width, height) => {
                self.image.set_size(width, height);
                screen.run_until_idle();
                assert_eq!(self.image.status(), ShareStatus::Waiting);
                assert_eq!(self.image.frame(), None, "the old import is dropped");
            }
            Ask::Dispose => {
                self.go.set(false);
                screen.run_until_idle();
                assert_eq!(
                    self.image.status(),
                    ShareStatus::Closed,
                    "the image's owner is gone"
                );
            }
        }
    }
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

/// How an engine starts: on the adapter, at a size.
type Start = fn(&GpuAdapter, u32, u32) -> Result<Producer, String>;

/// The engine on the graphics API itself.
const NATIVE: Start =
    |adapter, width, height| Producer::new(adapter, Default::default(), width, height);

/// Serve `request` with a new engine (`start`): provide its pool and wait
/// until it is ready.
fn serve(link: &ShareLink, request: &ShareRequest, adapter: &GpuAdapter, start: Start) -> Producer {
    let mut producer =
        start(adapter, request.width, request.height).expect("the engine starts on the adapter");
    let shared = producer
        .shared_textures()
        .expect("the engine shares its pool");
    // SAFETY: the test engine keeps the contract of `SharedTextures`.
    unsafe { link.provide(request, shared) }.expect("the newest request takes the pool");
    link.wait_ready(request, UI_WAIT)
        .expect("the pool is imported");
    producer
}

/// Draw and publish `FRAMES` frames, each checked by the UI thread; the
/// value of the last.
fn publish_frames(
    link: &ShareLink,
    request: &ShareRequest,
    producer: &mut Producer,
    ui: &Ui,
) -> u64 {
    let mut last = 0;
    for number in 0..FRAMES {
        let frame = producer.draw(number).expect("the engine draws");
        link.publish(request, frame.slot, frame.value);
        ui.ask(Ask::Shows {
            number,
            width: request.width,
            height: request.height,
        });
        last = frame.value;
    }
    last
}

fn shared_image_round(backend: GpuBackend, start: Start) -> Result<(), String> {
    if let Some(reason) = gpu_unavailable() {
        return Err(reason);
    }
    let viewport = SharedViewport::mount(gpu_options(backend));
    match viewport.adapter.get_untracked() {
        Some(adapter) if adapter.backend == backend => {}
        Some(adapter) => {
            return Err(format!(
                "the window's device is {:?} (WGPU_BACKEND?)",
                adapter.backend
            ));
        }
        None => {
            return Err(format!(
                "the {backend:?} device cannot import shared textures"
            ));
        }
    }
    viewport.run_engine(move |link, ui| {
        // The engine takes the request the image made for the adapter.
        let (request, adapter) = gpu_request(&link);
        assert_eq!((request.width, request.height), (W, H));
        let mut producer = serve(&link, &request, &adapter, start);
        let last = publish_frames(&link, &request, &mut producer, &ui);
        // Another size: a new request, the old one stale.
        ui.ask(Ask::Resize(W2, H2));
        // SAFETY: refused before a handle is read (the request is stale).
        let late = unsafe { link.provide(&request, unread_textures()) };
        assert_eq!(late, Err(ShareError::Stale));
        assert_eq!(link.wait_ready(&request, UI_WAIT), Err(ShareError::Stale));
        link.publish(&request, 0, last + 1);
        assert!(
            producer.wait_release(last, Duration::from_secs(5)),
            "the old import was dropped: its last frame is released (release {})",
            producer.release_value()
        );
        drop(producer);
        let (request, adapter) = gpu_request(&link);
        assert_eq!((request.width, request.height), (W2, H2));
        let mut producer = serve(&link, &request, &adapter, start);
        let last = publish_frames(&link, &request, &mut producer, &ui);
        // The owner goes: the link closes, its waits return at once, and
        // the window releases the last frame.
        assert!(!link.is_closed());
        ui.ask(Ask::Dispose);
        assert!(link.is_closed());
        let start = Instant::now();
        assert_eq!(link.wait_request(UI_WAIT), None);
        assert_eq!(link.wait_ready(&request, UI_WAIT), Err(ShareError::Closed));
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "the waits return at once"
        );
        link.publish(&request, 0, last + 1);
        assert!(
            producer.wait_release(last, Duration::from_secs(5)),
            "the dropped image releases its last frame (release {})",
            producer.release_value()
        );
    });
    assert!(
        viewport.screen.errors().is_empty(),
        "{:?}",
        viewport.screen.errors()
    );
    Ok(())
}

/// `shared_image` end to end: the engine's thread takes the image's `Gpu`
/// request, provides a pool, waits until it is ready and publishes
/// frames the canvas draws; `set_size` makes a new request (the old one
/// stale, its import dropped) served by a pool of the new size; disposing
/// the owner closes the link and releases the last frame.
#[cfg(windows)]
#[test]
fn shared_image_follows_requests_on_dx12() {
    if let Err(reason) = shared_image_round(GpuBackend::Dx12, NATIVE) {
        eprintln!("SKIPPED shared_image_follows_requests_on_dx12: {reason}");
    }
}

#[test]
fn shared_image_follows_requests_on_vulkan() {
    if let Err(reason) = shared_image_round(GpuBackend::Vulkan, NATIVE) {
        eprintln!("SKIPPED shared_image_follows_requests_on_vulkan: {reason}");
    }
}
