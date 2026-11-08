//! Role: Drawable export on the GPU: shared textures, generations and device loss.
//!
//! The window makes the textures of a drawable's GPU export. The consumer is
//! the test's own device, as an app's is (design §10.5): each
//! `ShareUpdate::Textures` is imported with `kivact_gpu_share::import::Import`
//! (`Import::new` tells the window the textures are ready), and the frames are
//! drawn or read back on that device. The documented loop (design §10.5) runs
//! on a thread of its own. The tests advance the virtual clock while they wait,
//! so the export's retries run (design §10.5, step 6). GPU tests skip, saying
//! why, when the window has no GPU device that exports shared textures (as
//! `gpu_interop.rs`).
#![allow(
    unsafe_code,
    reason = "importing shared textures is an unsafe fn: the test consumer keeps its contract"
)]
#![allow(
    clippy::print_stderr,
    reason = "report unavailable optional hardware explicitly"
)]

use std::collections::VecDeque;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use kivact::draw::Pixels;
use kivact::gpu_interop::{self, GpuAdapter, GpuBackend, SharedTextures};
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions, Screen, Screenshot, gpu_unavailable};
use kivact::view::{Drawable, DrawableFrame, DrawableShare, ShareError, ShareUpdate, viewport};
use kivact_gpu_share::import::Import;
use kivact_gpu_share::wgpu::{read_back, request_device};

use super::paint::{BLUE, RED, distance, pixel};
use super::support::mount_with;

/// The exported drawable's size at the start (logical px, scale 1).
const W: f64 = 40.0;
const H: f64 = 20.0;
/// The longest a test waits (real time) for an update of the export.
const WAIT: Duration = Duration::from_secs(10);
/// The virtual time each wait step moves: a skipped export is retried at its
/// deadline, which only runs when the clock moves.
const STEP: Duration = Duration::from_millis(16);

/// The mounted App: a viewport that exports a drawable (its size and colour
/// are signals), and a clock whose colour changes with `ticks`.
struct App {
    screen: Screen,
    card: Drawable,
    adapter: Value<Option<GpuAdapter>>,
    size: Signal<f64>,
    color: Signal<u32>,
    ticks: Signal<u32>,
}

/// The exported content: a box of the drawable's size and colour.
fn content(size: Signal<f64>, color: Signal<u32>) -> Node {
    node().style(move || style().size(size.get(), H).background(rgb(color.get())))
}

impl App {
    #[track_caller]
    fn mount() -> App {
        let options = RenderOptions::from_env().gpu().mirror(MirrorMode::Required);
        let (screen, (card, adapter, size, color, ticks)) = mount_with(options, move || {
            let card = drawable();
            let adapter = memo(gpu_interop::adapter);
            let size = signal(W);
            let color = signal(RED);
            let ticks = signal(0u32);
            let view = column((
                viewport(Pixels::new(1, 1, vec![0, 0, 0, 0]))
                    .drawable(card, move || content(size, color))
                    .style(style().size(120, 80)),
                node().test_id("clock").style(move || {
                    let color = clock_colour_for(ticks.get());
                    style().size(10, 10).background(rgb(color))
                }),
            ));
            ((card, adapter, size, color, ticks), view)
        });
        App {
            screen,
            card,
            adapter,
            size,
            color,
            ticks,
        }
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

/// The next update of the export, moving the virtual clock between the waits.
#[track_caller]
fn next_update(screen: &Screen, share: &DrawableShare, after: u64) -> Update {
    let start = Instant::now();
    loop {
        screen.advance(STEP);
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

/// The colour the clock takes for `ticks` (see `App::mount`).
fn clock_colour_for(ticks: u32) -> u32 {
    if ticks % 2 == 0 { 0x00ff00 } else { BLUE }
}

/// The colour the export shows after the test changes the content to a third
/// colour (the first is `RED`, the second `BLUE`).
const GREEN: u32 = 0x00ff00;

/// Copies a texture onto the whole target, sampled at the texel centres.
const COPY_WGSL: &str = "
struct Out {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};
@vertex
fn vs(@builtin(vertex_index) index: u32) -> Out {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: Out;
    out.pos = vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(corner.x, 1.0 - corner.y);
    return out;
}
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var nearest: sampler;
@fragment
fn fs(in: Out) -> @location(0) vec4<f32> {
    return textureSampleLevel(source, nearest, in.uv, 0.0);
}
";

/// The consumer of an export that a test runs itself, as an app does (design
/// §10.5): a wgpu device on the window's adapter and the pool imported with
/// `Import::new`. A frame can be held (acquired and not released); the
/// oldest held frame is released first, as the release fence requires.
struct Consumer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    import: Option<Import>,
    /// The frames held, oldest first: their slot and value.
    held: VecDeque<(usize, u64)>,
}

impl Consumer {
    /// A device on `adapter` (the window's), opened as an app opens one.
    #[track_caller]
    fn open(adapter: &GpuAdapter) -> Consumer {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: match adapter.backend {
                GpuBackend::Dx12 => wgpu::Backends::DX12,
                GpuBackend::Vulkan => wgpu::Backends::VULKAN,
            },
            flags: wgpu::InstanceFlags::empty().with_env(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let (_, device, queue) =
            request_device(&instance, adapter, &wgpu::DeviceDescriptor::default())
                .expect("the consumer's device opens on the window's adapter");
        Consumer {
            device,
            queue,
            import: None,
            held: VecDeque::new(),
        }
    }

    /// Import the textures of a generation (`Import::new` signals their
    /// preparation to the window).
    #[track_caller]
    fn import(&mut self, textures: SharedTextures) {
        let import = Import::new(&self.device, &self.queue, textures)
            .unwrap_or_else(|error| panic!("the textures import: {error}"));
        self.import = Some(import);
        self.held.clear();
    }

    /// The pool's width in px (after `import`).
    fn width(&self) -> u32 {
        self.import
            .as_ref()
            .expect("the textures are imported")
            .size()
            .0
    }

    /// Acquire frame `value` of `slot` and keep it: the window cannot write
    /// that slot again until it is released.
    #[track_caller]
    fn hold(&mut self, slot: u32, value: u64) {
        let import = self.import.as_mut().expect("the textures are imported");
        assert!(
            import.wait_ready(value, WAIT),
            "frame {value} becomes ready"
        );
        import
            .acquire(slot as usize, value)
            .unwrap_or_else(|error| panic!("the slot is acquired: {error}"));
        self.held.push_back((slot as usize, value));
    }

    /// Release the oldest held frame and signal the release fence to its value.
    #[track_caller]
    fn release_oldest(&mut self) {
        let (slot, value) = self.held.pop_front().expect("a frame is held");
        let import = self.import.as_mut().expect("the textures are imported");
        import
            .release(slot)
            .unwrap_or_else(|error| panic!("the slot is released: {error}"));
        import
            .signal(value)
            .unwrap_or_else(|error| panic!("the release fence signals: {error}"));
    }

    /// Acquire, then give back at once: the documented consumer's drawing.
    #[track_caller]
    fn draw(&mut self, slot: u32, value: u64) {
        let import = self.import.as_mut().expect("the textures are imported");
        assert!(
            import.wait_ready(value, WAIT),
            "frame {value} becomes ready"
        );
        import
            .acquire(slot as usize, value)
            .unwrap_or_else(|error| panic!("the slot is acquired: {error}"));
        import
            .release(slot as usize)
            .unwrap_or_else(|error| panic!("the slot is released: {error}"));
        import
            .signal(value)
            .unwrap_or_else(|error| panic!("the release fence signals: {error}"));
    }

    /// Frame `value` of `slot` as straight-alpha RGBA8. It is acquired, sampled
    /// by a render pass of this device into a texture that is read back, and
    /// released after the read back (which waited for the sampling).
    #[track_caller]
    fn read(&mut self, slot: u32, value: u64) -> Vec<u8> {
        let Consumer {
            device,
            queue,
            import,
            ..
        } = self;
        let import = import.as_mut().expect("the textures are imported");
        assert!(
            import.wait_ready(value, WAIT),
            "frame {value} becomes ready"
        );
        import
            .acquire(slot as usize, value)
            .unwrap_or_else(|error| panic!("the slot is acquired: {error}"));
        let (width, height) = import.size();
        let source =
            import.textures()[slot as usize].create_view(&wgpu::TextureViewDescriptor::default());
        let target = sample(device, queue, &source, (width, height));
        let bytes = read_back(device, queue, &target, WAIT)
            .unwrap_or_else(|error| panic!("the frame reads back: {error}"));
        import
            .release(slot as usize)
            .unwrap_or_else(|error| panic!("the slot is released: {error}"));
        import
            .signal(value)
            .unwrap_or_else(|error| panic!("the release fence signals: {error}"));
        bytes
    }
}

/// The pipeline that copies a texture onto a whole target (`COPY_WGSL`).
fn copy_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("the frame read back"),
        source: wgpu::ShaderSource::Wgsl(COPY_WGSL.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("the frame read back"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

/// A new `size` texture of `device` with the copy of `source` on it, drawn on
/// `queue` (readable with `read_back`).
fn sample(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &wgpu::TextureView,
    (width, height): (u32, u32),
) -> wgpu::Texture {
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("the frame read back"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let pipeline = copy_pipeline(device);
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..wgpu::SamplerDescriptor::default()
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("the frame read back"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("the frame read back"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("the frame read back"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
    }
    queue.submit([encoder.finish()]);
    target
}

/// The consumer loop of design §10.5, as written, on a thread of its own: each
/// new set of textures is imported, and the width of each frame's pool is
/// sent to the UI thread.
fn documented_consumer(share: DrawableShare, adapter: GpuAdapter, widths: mpsc::Sender<u32>) {
    let mut consumer = Consumer::open(&adapter);
    let mut seen = 0;
    loop {
        match share.wait(seen, Duration::from_millis(100)) {
            Ok(ShareUpdate::Textures(update)) => consumer.import(update.textures),
            Ok(ShareUpdate::Frame(frame)) => {
                seen = frame.number();
                let DrawableFrame::Gpu { slot, value, .. } = frame else {
                    panic!("a GPU export gave {frame:?}");
                };
                consumer.draw(slot, value);
                if widths.send(consumer.width()).is_err() {
                    return;
                }
            }
            Err(ShareError::Timeout) => continue,
            Err(_) => return,
        }
    }
}

/// Drive the screen until the consumer thread reports a frame `want` px wide.
#[track_caller]
fn expect_width(screen: &Screen, widths: &mpsc::Receiver<u32>, want: u32) {
    let start = Instant::now();
    loop {
        screen.advance(STEP);
        match widths.recv_timeout(Duration::from_millis(5)) {
            Ok(width) if width == want => return,
            Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => panic!("the consumer thread ended"),
        }
        assert!(start.elapsed() < WAIT, "no frame {want} px wide");
    }
}

/// The next GPU frame after `after`. The textures that come before it are
/// imported by `consumer`; the frame itself is neither drawn nor held.
#[track_caller]
fn next_gpu_frame(
    screen: &Screen,
    share: &DrawableShare,
    consumer: &mut Consumer,
    after: u64,
) -> DrawableFrame {
    loop {
        match next_update(screen, share, after) {
            Update::Textures { textures, .. } => consumer.import(textures),
            Update::Frame(frame) => {
                assert!(
                    matches!(frame, DrawableFrame::Gpu { .. }),
                    "a GPU export gave {frame:?}"
                );
                return frame;
            }
        }
    }
}

/// The next GPU frame after `after`, drawn and given back by `consumer` (the
/// documented consumer's drawing).
#[track_caller]
fn draw_next(
    screen: &Screen,
    share: &DrawableShare,
    consumer: &mut Consumer,
    after: u64,
) -> DrawableFrame {
    let frame = next_gpu_frame(screen, share, consumer, after);
    let DrawableFrame::Gpu { slot, value, .. } = frame else {
        unreachable!("next_gpu_frame returns GPU frames");
    };
    consumer.draw(slot, value);
    frame
}

/// The next GPU frame after `after`, held by `consumer` (the window cannot
/// write its slot until the consumer releases it). Returns the frame's number.
#[track_caller]
fn hold_next(screen: &Screen, share: &DrawableShare, consumer: &mut Consumer, after: u64) -> u64 {
    let frame = next_gpu_frame(screen, share, consumer, after);
    let DrawableFrame::Gpu { slot, value, .. } = frame else {
        unreachable!("next_gpu_frame returns GPU frames");
    };
    consumer.hold(slot, value);
    frame.number()
}

/// The next CPU frame after `after`.
#[track_caller]
fn next_pixels(screen: &Screen, share: &DrawableShare, after: u64) -> DrawableFrame {
    match next_update(screen, share, after) {
        Update::Frame(frame) => frame,
        Update::Textures { .. } => panic!("a pixel export gave textures"),
    }
}

/// Polls `share.wait` while the virtual clock moves in 16 ms steps, for at
/// most `millis` of it, and returns the first frame after `after`.
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
        screen.advance(STEP);
        elapsed += 16;
    }
}

/// The colour of the clock at its centre, in the window's last frame.
fn clock_colour(screen: &Screen) -> u32 {
    let rect = screen
        .get_by_test_id("clock")
        .rect()
        .expect("the clock is laid out");
    let shot: Screenshot = screen.screenshot();
    pixel(&shot, (rect.x + 5.0) as u32, (rect.y + 5.0) as u32)
}

/// Changes the clock to `ticks` and checks that the window shows it: the
/// clock's colour in the window's frame is the change's, so the window
/// presented after the change, whatever the export is doing.
#[track_caller]
fn shows_clock(app: &App, ticks: u32) {
    app.ticks.set(ticks);
    app.screen.run_until_idle();
    assert_eq!(
        clock_colour(&app.screen),
        clock_colour_for(ticks),
        "the window shows the clock at {ticks} ticks"
    );
}

/// Tier 2 (solid fill, the same texels at scale 1): the GPU export of the
/// drawable, imported and read back, matches its CPU export pixel for pixel.
#[test]
fn gpu_export_matches_cpu_export() {
    let Some(app) = gpu_app("gpu_export_matches_cpu_export") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    share.request_pixels();
    let DrawableFrame::Pixels { pixels: cpu, .. } = next_pixels(&app.screen, &share, 0) else {
        panic!("a pixel export gave no pixels");
    };
    share.request_gpu(adapter(&app));
    let mut consumer = Consumer::open(&adapter(&app));
    let frame = next_gpu_frame(&app.screen, &share, &mut consumer, 0);
    let DrawableFrame::Gpu { slot, value, .. } = frame else {
        unreachable!("next_gpu_frame returns GPU frames");
    };
    let gpu = consumer.read(slot, value);
    let bytes: &[u8] = &cpu.rgba;
    assert_eq!(gpu.len(), bytes.len(), "the frames have the same size");
    let colour = |px: &[u8]| (u32::from(px[0]) << 16) | (u32::from(px[1]) << 8) | u32::from(px[2]);
    for y in 0..H as u32 {
        for x in 0..W as u32 {
            let i = ((y * W as u32 + x) * 4) as usize;
            let (a, b) = (colour(&bytes[i..]), colour(&gpu[i..]));
            assert!(
                distance(a, b) <= 2,
                "pixel ({x}, {y}): cpu {a:06x}, gpu {b:06x}"
            );
        }
    }
    assert_eq!(colour(&gpu), RED, "the GPU frame is drawn");
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
}

/// The window keeps presenting while the consumer stalls: each of ten content
/// changes is followed by a clock change, which must show, and by 16 ms of
/// virtual time for the export's retries.
#[track_caller]
fn keeps_presenting(app: &App, from: u32) {
    for i in 0..10u32 {
        app.color.set(RED + from + i);
        app.screen.advance(STEP);
        shows_clock(app, from + i + 1);
    }
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
    keeps_presenting(&app, 0);
    // Imported and kept unreleased: one frame is held, the rest are not drawn.
    share.request_gpu(adapter(&app));
    let mut consumer = Consumer::open(&adapter(&app));
    hold_next(&app.screen, &share, &mut consumer, 0);
    keeps_presenting(&app, 10);
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
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
    let mut consumer = Consumer::open(&adapter(&app));
    let first = next_gpu_frame(&app.screen, &share, &mut consumer, 0);
    let old = first.generation();
    let DrawableFrame::Gpu { slot, value, .. } = first else {
        unreachable!("next_gpu_frame returns GPU frames");
    };
    consumer.draw(slot, value);
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
    consumer.import(textures);
    let frame = next_gpu_frame(&app.screen, &share, &mut consumer, first.number());
    assert_eq!(frame.generation(), generation);
    let latest = share.latest();
    assert!(
        latest.is_none_or(|f| f.generation() != old),
        "no frame of the old generation is latest"
    );
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
}

/// The documented consumer loop (design §10.5), run as written on a thread of
/// its own with its own import: the first frame arrives, and after the
/// drawable is resized twice, a frame of each new size arrives.
#[test]
fn documented_consumer_loop_survives_resize() {
    let Some(app) = gpu_app("documented_consumer_loop_survives_resize") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    let (widths, received) = mpsc::channel();
    let (thread_share, thread_adapter) = (share.clone(), adapter(&app));
    std::thread::spawn(move || documented_consumer(thread_share, thread_adapter, widths));
    share.request_gpu(adapter(&app));
    expect_width(&app.screen, &received, W as u32);
    app.size.set(60.0);
    app.screen.run_until_idle();
    expect_width(&app.screen, &received, 60);
    app.size.set(30.0);
    app.screen.run_until_idle();
    expect_width(&app.screen, &received, 30);
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
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
    let mut consumer = Consumer::open(&adapter(&app));
    let gpu = draw_next(&app.screen, &share, &mut consumer, pixels.number());
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
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
}

/// After `request_gpu`, no GPU frame comes before the import; the window keeps
/// presenting meanwhile. After the import a frame of the generation comes
/// within 200 ms of the virtual clock, and so does one after a resize once its
/// new textures are imported.
#[test]
fn gpu_frames_start_after_import() {
    let Some(app) = gpu_app("gpu_frames_start_after_import") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    let mut consumer = Consumer::open(&adapter(&app));
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
    shows_clock(&app, 1);
    consumer.import(textures);
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
    consumer.import(textures);
    let after = frame_within(&app.screen, &share, frame.number(), 200)
        .expect("a frame of the new textures within 200 ms");
    assert_eq!(after.generation(), resized);
    assert_ne!(after.generation(), generation);
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
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
    let mut consumer = Consumer::open(&adapter(&app));
    let first = draw_next(&app.screen, &share, &mut consumer, 0);
    let old = first.generation();
    app.screen.lose_gpu_device();
    shows_clock(&app, 1);
    let frame = next_gpu_frame(&app.screen, &share, &mut consumer, first.number());
    assert_ne!(
        frame.generation(),
        old,
        "the export starts a new generation"
    );
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
}

/// Whether the window's last frame is one with no damage. A presentation of
/// a skipped export is such a frame (design §10.5, step 6: the export alone
/// is composed), so after a frame that changed the clock (which has damage),
/// this is true exactly when a retry presented since.
fn presented_retry(screen: &Screen) -> bool {
    screen.frame_stats().damage_pixels == 0
}

/// Three frames are held (A, B, A: the last one sent is A), so every slot of
/// the pool is held and unreleased. Content C then cannot be sent: the export
/// retries while C waits. Back to A, the content last sent, the retry stops
/// and no presentation is asked for (design §10.5, step 6).
#[test]
fn retry_stops_when_content_returns() {
    let Some(app) = gpu_app("retry_stops_when_content_returns") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    share.request_gpu(adapter(&app));
    let mut consumer = Consumer::open(&adapter(&app));
    let mut after = hold_next(&app.screen, &share, &mut consumer, 0);
    app.color.set(BLUE);
    app.screen.run_until_idle();
    after = hold_next(&app.screen, &share, &mut consumer, after);
    app.color.set(RED);
    app.screen.run_until_idle();
    after = hold_next(&app.screen, &share, &mut consumer, after);
    app.color.set(GREEN);
    shows_clock(&app, 1);
    assert!(
        !presented_retry(&app.screen),
        "the clock's change is a frame with damage"
    );
    app.screen.advance(Duration::from_secs(1));
    assert!(
        presented_retry(&app.screen),
        "the export retries while C waits for a slot"
    );
    assert!(
        matches!(
            share.wait(after, Duration::from_millis(10)),
            Err(ShareError::Timeout)
        ),
        "no slot is free, so no frame of C comes"
    );
    app.color.set(RED);
    shows_clock(&app, 2);
    assert!(
        !presented_retry(&app.screen),
        "the clock's change is a frame with damage"
    );
    app.screen.advance(Duration::from_secs(1));
    assert!(
        !presented_retry(&app.screen),
        "back at A, the retry stops: no presentation is asked for"
    );
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
}

/// The pool is full of held frames (as in `retry_stops_when_content_returns`)
/// and the content changes to C once. While nothing is released the retry
/// interval grows to 100 ms; the release of the oldest frame then brings the
/// latest content C within 200 ms, with no other UI change.
///
/// Not asserted yet: the §17.2 bound of the retries (2 to 20 presentations in
/// one second). `Screen` offers no presentation count (`frame_ticks` counts
/// motion ticks); only the last frame's stats are visible.
#[test]
#[ignore = "pending: kivact-test has no presentation counter; the §17.2 retry bound (2 to 20 presentations a second) cannot be measured"]
fn released_slot_delivers_latest_without_other_updates() {
    let Some(app) = gpu_app("released_slot_delivers_latest_without_other_updates") else {
        return;
    };
    let share = app.card.share();
    share.set_scale(1.0);
    share.request_gpu(adapter(&app));
    let mut consumer = Consumer::open(&adapter(&app));
    let mut after = hold_next(&app.screen, &share, &mut consumer, 0);
    app.color.set(BLUE);
    app.screen.run_until_idle();
    after = hold_next(&app.screen, &share, &mut consumer, after);
    app.color.set(RED);
    app.screen.run_until_idle();
    after = hold_next(&app.screen, &share, &mut consumer, after);
    app.color.set(GREEN);
    shows_clock(&app, 1);
    app.screen.advance(Duration::from_secs(1));
    assert!(
        matches!(
            share.wait(after, Duration::from_millis(10)),
            Err(ShareError::Timeout)
        ),
        "nothing is released, so no frame of C yet"
    );
    assert!(
        presented_retry(&app.screen),
        "the export retries while C waits for a slot"
    );
    consumer.release_oldest();
    let frame = frame_within(&app.screen, &share, after, 200)
        .expect("the latest content C arrives within 200 ms of the release");
    assert!(frame.number() > after, "a new frame of C");
    assert!(app.screen.errors().is_empty(), "{:?}", app.screen.errors());
}
