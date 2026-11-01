//! Role: What every module uses: test windows, scenes and pixels.

use arvane::test_display::{VirtualWindow, VirtualWindowOptions, test_font};
use arvane::{Engine, Presented, Window as _};
use kivact_types::protocol::{DrawCommand, Paint, Scene};
use kivact_types::{Rect, Rgba, Size};

/// 60 Hz.
pub(crate) const PERIOD: u64 = 16_666_667;
/// The background a frame is composited over.
pub(crate) const WHITE: u32 = 0x00ff_ffff;
pub(crate) const RED: u32 = 0x00ff_0000;
pub(crate) const BLUE: u32 = 0x0000_00ff;

/// The options of a CPU test window of `width` × `height` logical pixels at
/// scale 1, a real window where one can open (else headless), titled
/// after the test.
pub(crate) fn options(width: f32, height: f32) -> VirtualWindowOptions {
    VirtualWindowOptions {
        title: std::thread::current()
            .name()
            .unwrap_or("arvane e2e")
            .to_owned(),
        size: Size::new(width, height),
        frame_period_ns: PERIOD,
        ..Default::default()
    }
}

/// A CPU test window of `width` × `height` logical pixels (see
/// [`options`]).
pub(crate) fn open(width: f32, height: f32) -> VirtualWindow {
    VirtualWindow::open(options(width, height)).expect("a test window opens")
}

/// Text that sees the bundled test font only.
pub(crate) fn engine() -> Engine {
    Engine::with_fonts_only(&[test_font()]).expect("the test font is usable")
}

pub(crate) fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(x, y, width, height)
}

/// An opaque color from 0x00RRGGBB.
pub(crate) fn color(rgb: u32) -> Rgba {
    color_alpha(rgb, 1.0)
}

/// A straight-alpha color from 0x00RRGGBB and an alpha.
pub(crate) fn color_alpha(rgb: u32, a: f32) -> Rgba {
    let channel = |shift: u32| ((rgb >> shift) & 0xff) as f32 / 255.0;
    Rgba::new(channel(16), channel(8), channel(0), a)
}

pub(crate) fn fill(x: f32, y: f32, width: f32, height: f32, rgb: u32) -> DrawCommand {
    DrawCommand::Rect {
        rect: rect(x, y, width, height),
        fill: Paint::Solid(color(rgb)),
    }
}

pub(crate) fn scene(commands: Vec<DrawCommand>) -> Scene {
    Scene { commands }
}

/// Present `commands` as the window plans its frame.
pub(crate) fn present(window: &mut VirtualWindow, commands: Vec<DrawCommand>) -> Presented {
    let plan = window.frame_plan();
    window
        .present(&scene(commands), plan)
        .expect("the window can present")
}

/// Present `commands` and require the frame to be shown.
#[track_caller]
pub(crate) fn show(window: &mut VirtualWindow, commands: Vec<DrawCommand>) {
    let presented = present(window, commands);
    assert!(
        matches!(presented, Presented::Shown { .. }),
        "the frame is shown: {presented:?}"
    );
}

/// The last frame's 0x00RRGGBB pixel at physical `(x, y)`.
#[track_caller]
pub(crate) fn pixel(window: &VirtualWindow, x: u32, y: u32) -> u32 {
    let (width, height, pixels) = window.frame().expect("a frame was presented");
    assert!(x < width && y < height, "({x}, {y}) is in the frame");
    pixels[(y * width + x) as usize]
}

pub(crate) fn channels(pixel: u32) -> [u8; 3] {
    [(pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8]
}

/// The largest channel difference of two pixels.
pub(crate) fn distance(a: u32, b: u32) -> u8 {
    let (a, b) = (channels(a), channels(b));
    (0..3).map(|i| a[i].abs_diff(b[i])).max().unwrap_or(0)
}

/// `actual` is `expected` within `tolerance` levels per channel.
#[track_caller]
pub(crate) fn assert_near(actual: u32, expected: u32, tolerance: u8, what: &str) {
    assert!(
        distance(actual, expected) <= tolerance,
        "{what}: {actual:06x} is not {expected:06x} within {tolerance}"
    );
}

/// A solid paint from 0x00RRGGBB.
pub(crate) fn solid(rgb: u32) -> Paint {
    Paint::Solid(color(rgb))
}

pub(crate) fn pt(x: f32, y: f32) -> kivact_types::Point {
    kivact_types::Point::new(x, y)
}

/// How the GPU's frame differs from the CPU's.
pub(crate) struct FrameDiff {
    /// Mean channel difference.
    pub(crate) mean: f64,
    /// Share of pixels off by more than 64.
    pub(crate) far: f64,
    /// Share of pixels within 8.
    pub(crate) close: f64,
}

impl FrameDiff {
    /// Within the tolerance of the GPU tests: the
    /// rasterizers anti-alias edges and glyphs and approximate blurs
    /// slightly differently.
    #[track_caller]
    pub(crate) fn assert_within_tolerance(&self, what: &str) {
        assert!(
            self.mean <= 1.0 && self.far <= 0.005 && self.close >= 0.99,
            "{what}: GPU vs CPU: mean {:.3}, {:.3}% off by more than 64, {:.2}% within 8",
            self.mean,
            self.far * 100.0,
            self.close * 100.0
        );
    }
}

/// The frames of `frames` (one scene per frame, in order) on a CPU and a
/// GPU test window of `width` × `height`, compared frame by frame. The
/// caller skips when `arvane::test_display::gpu_unavailable` says so.
pub(crate) fn gpu_against_cpu(
    (width, height): (f32, f32),
    frames: &[Vec<DrawCommand>],
) -> Vec<FrameDiff> {
    let run = |gpu: bool| {
        let mut window = VirtualWindow::open(VirtualWindowOptions {
            gpu,
            ..options(width, height)
        })
        .expect("a test window opens");
        let shots: Vec<Vec<u32>> = frames
            .iter()
            .map(|commands| {
                show(&mut window, commands.clone());
                window.frame().expect("a frame").2.to_vec()
            })
            .collect();
        let renderer = window.renderer();
        (shots, renderer)
    };
    let (cpu, cpu_renderer) = run(false);
    let (gpu, gpu_renderer) = run(true);
    assert_eq!(cpu_renderer, Some(arvane::Renderer::Cpu));
    assert_eq!(gpu_renderer, Some(arvane::Renderer::Gpu));
    cpu.iter()
        .zip(&gpu)
        .map(|(cpu, gpu)| {
            assert_eq!(cpu.len(), gpu.len());
            let diffs: Vec<u8> = cpu.iter().zip(gpu).map(|(a, b)| distance(*a, *b)).collect();
            let n = diffs.len() as f64;
            FrameDiff {
                mean: diffs.iter().map(|d| f64::from(*d)).sum::<f64>() / n,
                far: diffs.iter().filter(|d| **d > 64).count() as f64 / n,
                close: diffs.iter().filter(|d| **d <= 8).count() as f64 / n,
            }
        })
        .collect()
}

/// A pixel's mean channel.
pub(crate) fn gray(pixel: u32) -> u8 {
    let [r, g, b] = channels(pixel);
    ((u32::from(r) + u32::from(g) + u32::from(b)) / 3) as u8
}

/// The ink of text: the bounding box of the pixels darker than white,
/// and how many there are.
pub(crate) fn ink(window: &VirtualWindow) -> Option<((u32, u32, u32, u32), usize)> {
    let (width, height, pixels) = window.frame().expect("a frame");
    let mut bounds: Option<(u32, u32, u32, u32)> = None;
    let mut count = 0;
    for y in 0..height {
        for x in 0..width {
            if distance(pixels[(y * width + x) as usize], WHITE) > 32 {
                count += 1;
                bounds = Some(match bounds {
                    None => (x, y, x, y),
                    Some((l, t, r, b)) => (l.min(x), t.min(y), r.max(x), b.max(y)),
                });
            }
        }
    }
    bounds.map(|b| (b, count))
}

/// `text` shaped at `size` with the test font, its layout's origin at
/// `origin`, as glyph runs.
pub(crate) fn text_runs(text: &str, size: f32, origin: kivact_types::Point) -> Vec<DrawCommand> {
    let style = kivact_types::protocol::TextStyle {
        font_size: size,
        ..kivact_types::protocol::TextStyle::default()
    };
    let shaped = engine().shape(text, &style, None).expect("text shapes");
    shaped
        .runs
        .iter()
        .map(|run| DrawCommand::Glyphs {
            run: std::sync::Arc::clone(run),
            origin,
            color: None,
        })
        .collect()
}
