//! Role: Drawable export on the GPU: shared textures, generations and device loss.
//!
//! The window makes the textures of a drawable's GPU export. The test's
//! consumer is the UI thread: it imports each `ShareUpdate::Textures` with
//! `gpu_interop::import` (which keeps the import until it is dropped) and
//! reads the frames that follow. GPU tests skip, saying why, when the window
//! has no GPU device that exports shared textures (as `gpu_interop.rs`).
#![allow(
    unsafe_code,
    reason = "importing shared textures is an unsafe fn: the test consumer keeps its contract"
)]
#![allow(
    clippy::print_stderr,
    reason = "report unavailable optional hardware explicitly"
)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use kivact::draw::{Draw, ImageOptions, Pixels};
use kivact::geometry::Rect;
use kivact::gpu_interop::{self, ExternalImage, GpuAdapter, SharedTextures};
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions, Screen, Screenshot, gpu_unavailable};
use kivact::view::{Drawable, DrawableFrame, DrawableShare, ShareError, ShareUpdate, viewport};

use super::paint::{BLUE, RED, distance, pixel};
use super::support::mount_with;

/// The exported drawable's size at the start (logical px, scale 1).
const W: f64 = 40.0;
const H: f64 = 20.0;
/// The longest a test waits for an update of the export.
const WAIT: Duration = Duration::from_secs(10);

/// What the UI thread keeps of the export: the CPU frame, and the imported
/// textures with the GPU frame to show.
#[derive(Default)]
struct Shown {
    cpu: Option<Pixels>,
    image: Option<ExternalImage>,
    frame: Option<(u32, u64)>,
    generation: u64,
}

/// The mounted App: a viewport that exports a drawable (its size and colour
/// are signals), a canvas showing the CPU and the GPU frames, and a clock
/// whose colour changes with `ticks`.
struct App {
    screen: Screen,
    card: Drawable,
    adapter: Value<Option<GpuAdapter>>,
    size: Signal<f64>,
    color: Signal<u32>,
    ticks: Signal<u32>,
    shown: Rc<RefCell<Shown>>,
    version: Signal<u64>,
}

/// The exported content: a box of the drawable's size and colour.
fn content(size: Signal<f64>, color: Signal<u32>) -> Node {
    node().style(move || style().size(size.get(), H).background(rgb(color.get())))
}

/// Draw the CPU frame at the left and the GPU frame at x 60.
fn draw_shown(d: &mut Draw, shown: &Shown) {
    if let Some(pixels) = &shown.cpu {
        d.pixels(pixels, Rect::new(0.0, 0.0, W, H), ImageOptions::default());
    }
    if let (Some(image), Some((slot, value))) = (&shown.image, shown.frame) {
        let frame = image.frame(slot, value);
        d.gpu_frame(&frame, Rect::new(60.0, 0.0, W, H), ImageOptions::default());
    }
}

impl App {
    #[track_caller]
    fn mount() -> App {
        let shown = Rc::new(RefCell::new(Shown::default()));
        let paint = shown.clone();
        let options = RenderOptions::from_env().gpu().mirror(MirrorMode::Required);
        let (screen, (card, adapter, size, color, ticks, version)) =
            mount_with(options, move || {
                let card = drawable();
                let adapter = memo(gpu_interop::adapter);
                let size = signal(W);
                let color = signal(RED);
                let ticks = signal(0u32);
                let version = signal(0u64);
                let view = column((
                    viewport(Pixels::new(1, 1, vec![0, 0, 0, 0]))
                        .drawable(card, move || content(size, color))
                        .style(style().size(120, 80)),
                    canvas(move |d, _| {
                        version.get();
                        draw_shown(d, &paint.borrow());
                    })
                    .test_id("shown")
                    .style(style().size(120, 20)),
                    node().test_id("clock").style(move || {
                        let color = if ticks.get() % 2 == 0 { 0x00ff00 } else { BLUE };
                        style().size(10, 10).background(rgb(color))
                    }),
                ));
                ((card, adapter, size, color, ticks, version), view)
            });
        App {
            screen,
            card,
            adapter,
            size,
            color,
            ticks,
            shown,
            version,
        }
    }

    /// Redraw the canvas after the shown frames changed.
    fn repaint(&self) {
        self.version.set(self.version.get_untracked() + 1);
        self.screen.run_until_idle();
    }
}

/// The App, or None (the test skipped, saying why) when no GPU device here
/// exports shared textures.
fn gpu_app(test: &str) -> Option<App> {
    if let Some(reason) = gpu_unavailable() {
        eprintln!("SKIPPED {test}: {reason}");
        return None;
    }
    let app = App::mount();
    if app.adapter.get_untracked().is_none() {
        eprintln!("SKIPPED {test}: the window's GPU device cannot export shared textures");
        return None;
    }
    Some(app)
}

/// The window's adapter, for `request_gpu` (checked by `gpu_app`).
fn adapter(app: &App) -> GpuAdapter {
    app.adapter
        .get_untracked()
        .expect("the App has a GPU adapter")
}

/// One update of the export after `after`: new textures first, then a frame.
enum Update {
    Textures {
        generation: u64,
        textures: SharedTextures,
    },
    Frame(DrawableFrame),
}

/// The next update of the export, driving the screen between the waits.
#[track_caller]
fn next_update(screen: &Screen, share: &DrawableShare, after: u64) -> Update {
    let start = Instant::now();
    loop {
        screen.run_until_idle();
        match share.wait(after, Duration::from_millis(5)) {
            Ok(ShareUpdate::Textures(textures)) => {
                return Update::Textures {
                    generation: textures.generation,
                    textures: textures.textures,
                };
            }
            Ok(ShareUpdate::Frame(frame)) => return Update::Frame(frame),
            Err(ShareError::Timeout) => {}
            Err(error) => panic!("the export failed: {error}"),
        }
        assert!(start.elapsed() < WAIT, "no update after {after}");
    }
}

/// Import the window's textures on this (the consumer's) thread.
fn import(textures: SharedTextures) -> ExternalImage {
    // SAFETY: the window keeps the contract of `SharedTextures` and refuses
    // textures it cannot import before it reads their handles.
    unsafe { gpu_interop::import(textures) }
}

/// Serve the GPU export: import each textures update and return the first
/// GPU frame after `after`, which is shown on the canvas.
#[track_caller]
fn serve_gpu(app: &App, share: &DrawableShare, after: u64) -> DrawableFrame {
    loop {
        match next_update(&app.screen, share, after) {
            Update::Textures {
                generation,
                textures,
            } => {
                let mut shown = app.shown.borrow_mut();
                shown.image = Some(import(textures));
                shown.generation = generation;
            }
            Update::Frame(frame) => {
                let DrawableFrame::Gpu { slot, value, .. } = frame else {
                    panic!("a GPU export gave {frame:?}");
                };
                app.shown.borrow_mut().frame = Some((slot, value));
                app.repaint();
                return frame;
            }
        }
    }
}

/// The first CPU frame after `after`.
#[track_caller]
fn next_pixels(screen: &Screen, share: &DrawableShare, after: u64) -> DrawableFrame {
    loop {
        match next_update(screen, share, after) {
            Update::Frame(frame) => return frame,
            Update::Textures { .. } => panic!("a pixel export gave textures"),
        }
    }
}

/// The window's frame ticks advance over `millis` of the virtual clock.
#[track_caller]
fn presents_over(screen: &Screen, millis: u64) {
    let before = screen.frame_ticks();
    screen.advance(Duration::from_millis(millis));
    screen.run_until_idle();
    assert!(
        screen.frame_ticks() > before,
        "the window keeps presenting over {millis} ms"
    );
}

/// The colour of the clock at its centre, after a change of `ticks`.
fn clock_colour(screen: &Screen) -> u32 {
    let rect = screen
        .get_by_test_id("clock")
        .rect()
        .expect("the clock is laid out");
    let shot: Screenshot = screen.screenshot();
    pixel(&shot, (rect.x + 5.0) as u32, (rect.y + 5.0) as u32)
}

/// The GPU frame shows the same picture as the CPU frame (tier 2: the same
/// texels at scale 1, within 2/255 per channel).
#[track_caller]
fn assert_halves_match(screen: &Screen) {
    let rect = screen
        .get_by_test_id("shown")
        .rect()
        .expect("the canvas is laid out");
    let shot = screen.screenshot();
    let (x, y) = (rect.x as u32, rect.y as u32);
    let cpu = shot.crop(x, y, W as u32, H as u32);
    let gpu = shot.crop(x + 60, y, W as u32, H as u32);
    assert_eq!(pixel(&cpu, 20, 10), RED, "the CPU frame is drawn");
    for py in 0..H as u32 {
        for px in 0..W as u32 {
            let (a, b) = (pixel(&cpu, px, py), pixel(&gpu, px, py));
            assert!(
                distance(a, b) <= 2,
                "pixel ({px}, {py}): cpu {a:06x}, gpu {b:06x}"
            );
        }
    }
}

/// Tier 2 (solid fill, the same texels at scale 1): the GPU export of the
/// drawable, imported and drawn, matches its CPU export pixel for pixel.
#[test]
fn gpu_export_matches_cpu_export() {
    let Some(app) = gpu_app("gpu_export_matches_cpu_export") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    share.request_pixels();
    let cpu = next_pixels(&app.screen, &share, 0);
    let DrawableFrame::Pixels { pixels, .. } = cpu else {
        panic!("a pixel export gave {cpu:?}");
    };
    app.shown.borrow_mut().cpu = Some(pixels);
    app.repaint();
    share.request_gpu(adapter(&app));
    serve_gpu(&app, &share, 0);
    app.screen.run_until_idle();
    assert_halves_match(&app.screen);
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
}

/// Two windows of stalled consumers: first the textures are never imported,
/// then imported and kept unreleased. In both, ten content changes leave the
/// window presenting, and another node's change (the clock) shows.
#[test]
fn stalled_consumer_does_not_stop_the_window() {
    let Some(app) = gpu_app("stalled_consumer_does_not_stop_the_window") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    share.request_gpu(adapter(&app));
    // Not imported: the textures are dropped unread (the consumer stalls).
    let Update::Textures { .. } = next_update(&app.screen, &share, 0) else {
        panic!("the export asks for its textures first");
    };
    for i in 0..10u32 {
        app.color.set(RED + i);
        app.ticks.set(i);
        app.screen.run_until_idle();
        presents_over(&app.screen, 16);
        assert_eq!(clock_colour(&app.screen), clock_colour_for(i));
    }
    // Imported and kept unreleased: the same ten changes.
    share.request_gpu(adapter(&app));
    serve_gpu(&app, &share, 0);
    for i in 0..10u32 {
        app.color.set(RED + i);
        app.ticks.set(i + 1);
        app.screen.run_until_idle();
        presents_over(&app.screen, 16);
        assert_eq!(clock_colour(&app.screen), clock_colour_for(i + 1));
    }
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
}

/// The colour the clock takes for `ticks` (see `App::mount`).
fn clock_colour_for(ticks: u32) -> u32 {
    if ticks % 2 == 0 { 0x00ff00 } else { BLUE }
}

/// A resize makes a new generation: its textures come first, and the frames
/// of the old one never show as `latest`.
#[test]
fn export_generation_rejects_stale_frames() {
    let Some(app) = gpu_app("export_generation_rejects_stale_frames") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    share.request_gpu(adapter(&app));
    let first = serve_gpu(&app, &share, 0);
    let old = first.generation();
    app.size.set(60.0);
    app.screen.run_until_idle();
    let Update::Textures {
        generation,
        textures,
    } = next_update(&app.screen, &share, first.number())
    else {
        panic!("the new generation's textures come before any frame");
    };
    assert_ne!(generation, old, "the textures are of the new generation");
    app.shown.borrow_mut().image = Some(import(textures));
    let frame = serve_gpu(&app, &share, first.number());
    assert_eq!(frame.generation(), generation);
    let latest = share.latest();
    assert!(
        latest.is_none_or(|f| f.generation() != old),
        "no frame of the old generation is latest"
    );
}

/// The documented consumer loop is meant to run on a thread of its own, with
/// its own import on that thread. The facade's `gpu_interop::import` makes an
/// owner on the calling thread (UI only), and the `Import::new` of the
/// design is not reachable from the public test API.
#[test]
#[ignore = "pending: the facade's import is UI-thread only; the design's Import::new is not reachable from the public API"]
fn documented_consumer_loop_survives_resize() {
    let Some(app) = gpu_app("documented_consumer_loop_survives_resize") else {
        return;
    };
    let share = app.card.share();
    share.request_gpu(adapter(&app));
    let first = serve_gpu(&app, &share, 0);
    assert!(first.number() > 0);
}

/// The mode changes Pixels → Gpu → Pixels → stop: each mode delivers its
/// first frame, and after `stop` no frame comes.
#[test]
fn export_mode_switch_resends_static_content() {
    let Some(app) = gpu_app("export_mode_switch_resends_static_content") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    share.request_pixels();
    let pixels = next_pixels(&app.screen, &share, 0);
    share.request_gpu(adapter(&app));
    let gpu = serve_gpu(&app, &share, pixels.number());
    share.request_pixels();
    let again = next_pixels(&app.screen, &share, gpu.number());
    assert!(matches!(again, DrawableFrame::Pixels { .. }));
    share.stop();
    app.screen.run_until_idle();
    assert!(share.latest().is_none(), "stop drops the latest frame");
    assert!(
        matches!(
            share.wait(again.number(), Duration::from_millis(100)),
            Err(ShareError::Timeout)
        ),
        "no frame after stop"
    );
}

/// After `request_gpu`, no GPU frame comes before the import; the window
/// keeps presenting meanwhile. After the import a frame of the generation
/// comes within 200 ms of the virtual clock, and so does one after a resize
/// once its new textures are imported.
#[test]
fn gpu_frames_start_after_import() {
    let Some(app) = gpu_app("gpu_frames_start_after_import") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    share.request_gpu(adapter(&app));
    let Update::Textures {
        generation,
        textures,
    } = next_update(&app.screen, &share, 0)
    else {
        panic!("the export asks for its textures first");
    };
    app.screen.advance(Duration::from_millis(200));
    app.screen.run_until_idle();
    assert!(share.latest().is_none(), "no GPU frame before the import");
    presents_over(&app.screen, 200);
    app.shown.borrow_mut().image = Some(import(textures));
    app.shown.borrow_mut().generation = generation;
    let frame = frame_within(&app.screen, &share, 0, 200).expect("a frame within 200 ms");
    assert_eq!(frame.generation(), generation);
    app.size.set(60.0);
    app.screen.run_until_idle();
    let Update::Textures {
        generation: resized,
        textures,
    } = next_update(&app.screen, &share, frame.number())
    else {
        panic!("the new textures come first after a resize");
    };
    app.shown.borrow_mut().image = Some(import(textures));
    let after = frame_within(&app.screen, &share, frame.number(), 200)
        .expect("a frame of the new textures within 200 ms");
    assert_eq!(after.generation(), resized);
    assert_ne!(after.generation(), generation);
}

/// Polls `share.wait` while the virtual clock moves in 16 ms steps, for at
/// most `millis` of it, and returns the first GPU frame after `after`.
fn frame_within(
    screen: &Screen,
    share: &DrawableShare,
    after: u64,
    millis: u64,
) -> Option<DrawableFrame> {
    let mut elapsed = 0;
    loop {
        screen.run_until_idle();
        if let Ok(ShareUpdate::Frame(frame)) = share.wait(after, Duration::from_millis(2)) {
            return Some(frame);
        }
        if elapsed >= millis {
            return None;
        }
        screen.advance(Duration::from_millis(16));
        elapsed += 16;
    }
}

/// A device loss during the export: the window keeps presenting, the export
/// asks new textures of a new generation, and their frames arrive once they
/// are imported.
#[test]
fn gpu_export_recovers_after_device_loss() {
    let Some(app) = gpu_app("gpu_export_recovers_after_device_loss") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    share.request_gpu(adapter(&app));
    let first = serve_gpu(&app, &share, 0);
    let old = first.generation();
    app.screen.lose_gpu_device();
    presents_over(&app.screen, 100);
    let frame = serve_gpu(&app, &share, first.number());
    assert_ne!(
        frame.generation(),
        old,
        "the export starts a new generation"
    );
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
}

/// The export retries while its last frame was skipped, and stops when the
/// content returns to what was last sent. The retry needs a slot that a
/// consumer holds unreleased and releases on demand; the public API has no
/// such release, so the retry states cannot be driven from a test.
#[test]
#[ignore = "pending: the public API cannot hold a shared slot unreleased and release it on demand, so the retry cannot be driven"]
fn retry_stops_when_content_returns() {
    let Some(app) = gpu_app("retry_stops_when_content_returns") else {
        return;
    };
    let share = app.card.share();
    share.request_gpu(adapter(&app));
    let _ = serve_gpu(&app, &share, 0);
}

/// A slot that the consumer has not released: the content changes to C and
/// then the release comes, and the latest content C arrives within 200 ms
/// without other UI changes. The release is not a call of the public API
/// (the import releases with its own handling), so this cannot be scripted.
#[test]
#[ignore = "pending: no public release call for a held slot; the release timing cannot be scripted"]
fn released_slot_delivers_latest_without_other_updates() {
    let Some(app) = gpu_app("released_slot_delivers_latest_without_other_updates") else {
        return;
    };
    let share = app.card.share();
    share.request_gpu(adapter(&app));
    let _ = serve_gpu(&app, &share, 0);
}
