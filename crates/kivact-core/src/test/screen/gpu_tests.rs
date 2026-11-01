//! Role: GPU rendering verification tests.
//!
//! The GPU renderer on a real window (feature `gpu`): a representative
//! screen presented through the window's GPU surface (read back from the
//! texture copied to the window) matches the deterministic CPU frame within
//! a tolerance, as the GPU and CPU rasterizers anti-alias edges and glyphs
//! and approximate blurs slightly differently. Consumers select the GPU with
//! public `RenderOptions::gpu` and verify `Screen::renderer`; these internal
//! helpers additionally inspect the shell surface directly.
//!
//! The test is skipped, with a message, only where no hardware GPU adapter
//! exists; any other GPU failure fails it.

use std::panic::Location;

use crate::control::image;
use crate::draw::{
    BlendMode, Filter, Shadow, Shape, Transform, conic_gradient, linear_gradient, radial_gradient,
};
use kivact_engine::testing::{Renderer, gpu_unavailable};
use kivact_style::asset::{Asset, ImageFit};
use kivact_style::{BorderStyle, Direction, Position, rgb};

use super::model::Screen;
use super::results::Screenshot;
use super::support::button_box;
use crate::control::canvas;
use crate::draw::{Point, Rect};
use crate::prelude::*;
use crate::test::{RenderOptions, WindowMode};
use kivact_style::{Align, Color, Justify, pct, rgba};

/// A 2 × 2 RGBA PNG (stretched and sampled bilinearly) and a two-color
/// SVG with a translucent half (rasterized at its drawn size).
static PNG: &[u8] =
    include_bytes!("../../../../../dev/tests/kivact-public-api/tests/fixtures/rgba.png");
static SVG: &[u8] =
    include_bytes!("../../../../../dev/tests/kivact-public-api/tests/fixtures/shapes.svg");

/// Text, rounded and bordered boxes with blurred shadows, a gradient, a
/// translucent layer, a clipped overflow and images (stretched, faded):
/// the paint kinds of a typical screen.
fn representative() -> impl IntoView {
    let card = |title: &'static str, color: u32| {
        column((
            text(title).style(style().font_size(18.0).color(rgb(0x1a1a2e))),
            text("The quick brown fox jumps over the lazy dog 0123456789")
                .style(style().font_size(12.0).color(rgb(0x444444))),
        ))
        .style(
            style()
                .padding(12.0)
                .gap(6.0)
                .radius(10.0)
                .border(1.5, rgb(color))
                .background(rgb(0xf7f7fb))
                .shadow(Shadow::new(0.0, 3.0, 12.0, rgb(0x303050)).spread(1.0)),
        )
    };
    column((
        card("Inbox", 0x3b82f6),
        card("Archive", 0x10b981),
        row((
            node().style(
                style()
                    .width(120.0)
                    .height(48.0)
                    .radius(24.0)
                    .background(linear_gradient(90).colors([rgb(0xff6b6b), rgb(0x4d96ff)])),
            ),
            node().style(
                style()
                    .width(80.0)
                    .height(48.0)
                    .background(rgb(0x222222))
                    .opacity(0.5),
            ),
            column((text("clipped overflow text that runs past its box"),)).style(
                style()
                    .width(90.0)
                    .height(20.0)
                    .clip(true)
                    .background(rgb(0xffe8a3)),
            ),
        ))
        .style(style().gap(10.0)),
        row((
            image(Asset::from(PNG))
                .fit(ImageFit::Fill)
                .style(style().width(48.0).height(36.0)),
            image(Asset::from(PNG))
                .fit(ImageFit::Fill)
                .style(style().width(48.0).height(36.0).opacity(0.6)),
            image(Asset::from(SVG)).style(style().width(64.0).height(32.0)),
        ))
        .style(style().gap(10.0)),
        button_box("Submit"),
    ))
    .style(style().gap(12.0).padding(16.0).background(rgb(0xffffff)))
}

/// The element paint of CSS on one screen: gradient layers and tiles,
/// per-side, dashed and elliptical borders, inset and stacked shadows,
/// group opacity, color filters, blur and drop shadow filters, a backdrop
/// blur, a blend mode, a gradient mask, a rounded clip and a clip path.
fn effects() -> impl IntoView {
    let at = |x: i32, y: i32, w: i32, h: i32| {
        style()
            .position(Position::Absolute)
            .left(x)
            .top(y)
            .size(w, h)
    };
    let stripes = linear_gradient(90)
        .repeating()
        .stop(rgb(0x1e293b), 0)
        .stop(rgb(0x1e293b), 3)
        .stop(rgb(0xf8fafc), 3)
        .stop(rgb(0xf8fafc), 8);
    let line = rgba(0x334155, 0.5);
    node()
        .child((
            node().style(
                at(10, 10, 120, 90).radius(12).background((
                    linear_gradient(180)
                        .stop(line, 1)
                        .stop(Color::TRANSPARENT, 1)
                        .size(16, 16),
                    linear_gradient(90)
                        .stop(line, 1)
                        .stop(Color::TRANSPARENT, 1)
                        .size(16, 16),
                    radial_gradient()
                        .radii(pct(80), pct(60))
                        .at(pct(20), 0)
                        .colors([rgba(0x2563eb, 0.4), Color::TRANSPARENT]),
                    rgb(0xf1f5f9),
                )),
            ),
            node().style(
                at(150, 10, 120, 40)
                    .radius((pct(50), pct(50)))
                    .background(conic_gradient(-60).colors([
                        rgb(0xff4040),
                        rgb(0xffd040),
                        rgb(0x40ff60),
                        rgb(0x4060ff),
                        rgb(0xff4040),
                    ]))
                    .shadow(Shadow::new(0, 6, 14, rgba(0x000000, 0.5)).spread(-4))
                    .shadow(Shadow::new(0, 1, 0, rgba(0xffffff, 0.6)).inset()),
            ),
            node().style(
                at(150, 60, 120, 40)
                    .radius_corners(16, 0, 16, 0)
                    .border_left(4, rgb(0xdc2626))
                    .border_top(2, rgb(0x2563eb))
                    .background(rgb(0xffffff)),
            ),
            node().style(
                at(290, 10, 80, 40)
                    .border(2, rgb(0x0f172a))
                    .border_style(BorderStyle::Dashed)
                    .radius(8),
            ),
            node()
                .child((
                    node().style(at(0, 0, 40, 30).background(rgb(0x2563eb))),
                    node().style(at(20, 10, 40, 30).background(rgb(0xdc2626))),
                ))
                .style(at(290, 60, 60, 40).opacity(0.6)),
            node().style(at(10, 120, 460, 60).background(stripes)),
            node().style(
                at(30, 125, 120, 50)
                    .radius(10)
                    .backdrop_filter(Filter::new().blur(4).saturate(1.6))
                    .background(rgba(0xffffff, 0.3)),
            ),
            column((
                text("Filtered").style(style().font_size(14.0).color(rgb(0xdc2626))),
                image(Asset::from(PNG))
                    .fit(ImageFit::Fill)
                    .style(style().width(30.0).height(20.0)),
            ))
            .style(
                at(170, 125, 90, 50)
                    .background(rgb(0xfde68a))
                    .filter(Filter::new().grayscale(1).brightness(1.1)),
            ),
            node().style(
                at(280, 130, 40, 40)
                    .radius(pct(50))
                    .background(rgb(0x2563eb))
                    .filter(Filter::new().drop_shadow(0, 3, 6, rgba(0x000000, 0.6))),
            ),
            node().style(
                at(340, 130, 40, 40)
                    .background(rgb(0xffffff))
                    .mix_blend_mode(BlendMode::Difference),
            ),
            node().style(
                at(400, 130, 50, 40)
                    .background(rgb(0x16a34a))
                    .filter(Filter::new().blur(3)),
            ),
            node()
                .child(
                    image(Asset::from(SVG))
                        .fit(ImageFit::Fill)
                        .style(style().size(100, 60)),
                )
                .style(at(10, 200, 100, 60).radius(20).clip(true)),
            node().style(
                at(130, 200, 120, 60)
                    .background(linear_gradient(135).colors([rgb(0x7c3aed), rgb(0x22d3ee)]))
                    .mask(linear_gradient(180).colors([Color::BLACK, Color::TRANSPARENT])),
            ),
            node().style(
                at(270, 200, 120, 50).background(rgb(0x0ea5e9)).clip_path(
                    Shape::polygon()
                        .point(0, 0)
                        .point(pct(100) - 18, 0)
                        .point(pct(100), 18)
                        .point(pct(100), pct(100))
                        .point(0, pct(100)),
                ),
            ),
        ))
        .style(style().size(480, 360).background(rgb(0xffffff)))
}

/// Element transforms: rotated text (unhinted outlines), a skewed button
/// with its counter-skewed label (hinted again), a rotated image, a world
/// scaled 1.5× holding text, an SVG (rasterized for the zoom) and a
/// rounded, shadowed card, and boxes ordered by z-index.
fn transforms() -> impl IntoView {
    let at = |x: i32, y: i32, w: i32, h: i32| {
        style()
            .position(Position::Absolute)
            .left(x)
            .top(y)
            .size(w, h)
    };
    node()
        .child((
            text("Rotated text, 30 degrees").style(
                at(20, 60, 220, 24)
                    .font_size(16.0)
                    .color(rgb(0x111827))
                    .rotate(30),
            ),
            node()
                .child(
                    text("PLAY").style(
                        style()
                            .font_size(20.0)
                            .color(rgb(0xffffff))
                            .transform(Transform::new().skew_x(14)),
                    ),
                )
                .style(
                    at(260, 30, 140, 44)
                        .direction(Direction::Row)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .background(linear_gradient(90).colors([rgb(0x2563eb), rgb(0x7c3aed)]))
                        .shadow(Shadow::new(0, 4, 10, rgba(0x000000, 0.4)))
                        .transform(Transform::new().skew_x(-14)),
                ),
            image(Asset::from(PNG))
                .fit(ImageFit::Fill)
                .style(at(400, 110, 48, 36).rotate(-20)),
            node()
                .child((
                    text("Zoomed world").style(style().font_size(12.0).color(rgb(0x111827))),
                    image(Asset::from(SVG)).style(style().width(64.0).height(32.0)),
                    node().style(
                        style()
                            .size(80, 30)
                            .radius(8)
                            .background(rgb(0xffffff))
                            .border(1, rgb(0xcbd5e1))
                            .shadow(Shadow::new(0, 2, 6, rgba(0x000000, 0.3))),
                    ),
                ))
                .style(
                    at(20, 130, 160, 110)
                        .gap(6)
                        .padding(6)
                        .background(rgb(0xf1f5f9))
                        .transform_origin(0, 0)
                        .scale(1.5),
                ),
            node().style(at(300, 220, 80, 60).background(rgb(0xef4444)).z_index(2)),
            node().style(at(340, 240, 80, 60).background(rgb(0x22c55e))),
            node().style(at(320, 260, 80, 60).background(rgb(0x3b82f6)).z_index(-1)),
        ))
        .style(style().size(480, 360).background(rgb(0xffffff)))
}

#[track_caller]
fn shot(gpu: bool) -> (Screenshot, Option<Renderer>) {
    shot_of(gpu, || representative().into_view())
}

#[track_caller]
fn shot_of(gpu: bool, scene: fn() -> crate::view::View) -> (Screenshot, Option<Renderer>) {
    let screen = Screen::start(
        RenderOptions {
            width: 480.0,
            height: 360.0,
            window: WindowMode::Real,
            ..RenderOptions::default()
        },
        gpu,
        Vec::new(),
        Box::new(scene),
        Location::caller(),
    );
    let shot = screen.screenshot();
    let renderer = screen.with_host(|host| host.renderer());
    (shot, renderer)
}

#[test]
fn gpu_frames_in_a_real_window_match_cpu_frames() {
    if let Some(reason) = gpu_unavailable() {
        eprintln!("SKIPPED gpu_frames_in_a_real_window_match_cpu_frames: {reason}");
        return;
    }
    let (cpu, cpu_renderer) = shot(false);
    let (gpu, gpu_renderer) = shot(true);
    assert_eq!(cpu_renderer, Some(Renderer::Cpu));
    assert_eq!(
        gpu_renderer,
        Some(Renderer::Gpu),
        "the frame went through the window's GPU surface"
    );
    assert_close(&cpu, &gpu, "gpu_smoke");
    let (cpu, _) = shot_of(false, || effects().into_view());
    let (gpu, gpu_renderer) = shot_of(true, || effects().into_view());
    assert_eq!(gpu_renderer, Some(Renderer::Gpu));
    assert_close(&cpu, &gpu, "gpu_effects");
    let (cpu, _) = shot_of(false, || transforms().into_view());
    let (gpu, gpu_renderer) = shot_of(true, || transforms().into_view());
    assert_eq!(gpu_renderer, Some(Renderer::Gpu));
    assert_close(&cpu, &gpu, "gpu_transforms");
    let (cpu, _) = shot_of(false, || canvas_vocabulary().into_view());
    let (gpu, gpu_renderer) = shot_of(true, || canvas_vocabulary().into_view());
    assert_eq!(gpu_renderer, Some(Renderer::Gpu));
    assert_close(&cpu, &gpu, "gpu_canvas");
}

/// A host transition is sampled before presenting through each real surface.
#[test]
fn gpu_host_state_transition_matches_the_cpu_intermediate_frame() {
    if let Some(reason) = gpu_unavailable() {
        eprintln!("SKIPPED gpu_host_state_transition_matches_the_cpu_intermediate_frame: {reason}");
        return;
    }
    let shot = |gpu| {
        let screen = Screen::start(
            RenderOptions {
                width: 180.0,
                height: 100.0,
                window: WindowMode::Real,
                ..RenderOptions::default()
            },
            gpu,
            Vec::new(),
            Box::new(|| {
                node()
                    .child(text("motion"))
                    .test_id("card")
                    .style(
                        style()
                            .size(100, 40)
                            .background(rgb(0xff0000))
                            .hover(style().background(rgb(0x0000ff)).translate(20, 0))
                            .transition(
                                kivact_style::TransitionProp::Paint,
                                crate::motion::Transition::tween(
                                    200,
                                    crate::motion::Easing::Linear,
                                ),
                            ),
                    )
                    .into_view()
            }),
            Location::caller(),
        );
        screen.get_by_test_id("card").hover();
        screen.advance(std::time::Duration::from_millis(80));
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
        let screenshot = screen.screenshot();
        let renderer = screen.with_host(|host| host.renderer());
        (screenshot, renderer)
    };
    let (cpu, _) = shot(false);
    let (gpu, renderer) = shot(true);
    assert_eq!(renderer, Some(Renderer::Gpu));
    assert_close(&cpu, &gpu, "host_motion");
}

fn canvas_vocabulary() -> impl IntoView {
    use crate::draw::{ImageOptions, Layer, Path, Pixels, TextBaseline, TextStyle};
    use crate::draw::{LineCap, Stroke};
    use kivact_style::TextAlign;
    canvas(|d, size| {
        d.layer(
            Layer::new().transform(Transform::new().translate(100, 20).rotate(30)),
            |d| {
                d.text(
                    "Canvas",
                    Point::new(0., 0.),
                    TextStyle {
                        color: Color::BLACK,
                        ..Default::default()
                    },
                );
                d.pixels(
                    &Pixels::new(
                        2,
                        1,
                        std::sync::Arc::from([255u8, 0, 0, 255, 0, 0, 255, 255]),
                    ),
                    Rect::new(0., 30., 60., 30.),
                    ImageOptions {
                        source: Some(Rect::new(1., 0., 1., 1.)),
                        pixelated: true,
                        opacity: 0.5,
                    },
                );
            },
        );
        d.layer(
            Layer::new()
                .area(Rect::new(200., 10., 60., 60.))
                .clip(&Shape::circle(30)),
            |d| {
                d.fill_rect(
                    size.to_rect(),
                    radial_gradient().colors([rgb(0x2563eb), Color::WHITE]),
                );
            },
        );
        d.layer(
            Layer::new().shadow(Shadow::new(0, 0, 6, rgb(0xff0000))),
            |d| {
                d.stroke_path(
                    &Path::new().arc(Point::new(60., 140.), 30, 135, 405, false),
                    &Stroke::new(3).cap(LineCap::Round).dash([6., 4.]),
                    rgb(0xff0000),
                );
            },
        );
        d.layer(
            Layer::new()
                .area(size.to_rect())
                .opacity(0.7)
                .filter(Filter::new().brightness(0.8))
                .blend(BlendMode::Screen)
                .mask(Color::WHITE),
            |d| {
                d.fill_shape(
                    Rect::new(120., 100., 100., 50.),
                    &Shape::ellipse(50, 25),
                    radial_gradient().colors([rgb(0xff0000), rgb(0x0000ff)]),
                );
            },
        );
        d.text(
            "Centered",
            Point::new(200., 220.),
            TextStyle {
                align: TextAlign::Center,
                baseline: TextBaseline::Alphabetic,
                color: Color::BLACK,
                ..Default::default()
            },
        );
    })
    .style(style().size(480, 360).background(Color::WHITE))
}

/// The GPU frame `gpu` matches the CPU frame `cpu` within the tolerance of
/// the module doc (both kept as `<name>_cpu.png`/`<name>_gpu.png` in the
/// failure directory when not).
#[track_caller]
#[allow(
    clippy::print_stdout,
    reason = "the test reports how close the frames are, as the test functions do"
)]
fn assert_close(cpu: &Screenshot, gpu: &Screenshot, name: &str) {
    assert_eq!((gpu.width, gpu.height), (cpu.width, cpu.height));
    let channels = |p: u32| [(p >> 16) as u8, (p >> 8) as u8, p as u8];
    let diffs: Vec<u8> = cpu
        .pixels
        .iter()
        .zip(&gpu.pixels)
        .map(|(a, b)| {
            let (a, b) = (channels(*a), channels(*b));
            (0..3).map(|i| a[i].abs_diff(b[i])).max().unwrap_or(0)
        })
        .collect();
    let mean = diffs.iter().map(|d| f64::from(*d)).sum::<f64>() / diffs.len() as f64;
    let far = diffs.iter().filter(|d| **d > 64).count() as f64 / diffs.len() as f64;
    // The rasterizers' blurred shadows approximate the gaussian slightly
    // differently: up to 7 levels apart in their smooth falloff.
    let close = diffs.iter().filter(|d| **d <= 8).count() as f64 / diffs.len() as f64;
    println!(
        "{name}: GPU vs CPU frame: mean channel difference {mean:.3}, {:.3}% of pixels off by more than 64, {:.2}% within 8",
        far * 100.0,
        close * 100.0
    );
    let differs = mean > 1.0 || far > 0.005 || close < 0.99;
    // `KIVACT_GPU_SMOKE_SAVE=1` keeps both frames of a passing run too.
    if differs || std::env::var_os("KIVACT_GPU_SMOKE_SAVE").is_some() {
        let dir = super::failure_dir();
        let _ = std::fs::create_dir_all(&dir);
        let _ = cpu.save_png(dir.join(format!("{name}_cpu.png")));
        let _ = gpu.save_png(dir.join(format!("{name}_gpu.png")));
        assert!(
            !differs,
            "GPU frame differs from the CPU frame (frames in {})",
            dir.display()
        );
    }
}

/// A window drawn on the GPU follows a scale change (a move to a monitor
/// with another DPI): its surface takes the new device size and the frame
/// matches a CPU frame drawn at that scale.
#[test]
fn gpu_frames_follow_a_scale_change() {
    if let Some(reason) = gpu_unavailable() {
        eprintln!("SKIPPED gpu_frames_follow_a_scale_change: {reason}");
        return;
    }
    let start = |gpu: bool, scale: f32| {
        Screen::start(
            RenderOptions {
                width: 480.0,
                height: 360.0,
                scale,
                window: WindowMode::Real,
                ..RenderOptions::default()
            },
            gpu,
            Vec::new(),
            Box::new(|| representative().into_view()),
            Location::caller(),
        )
    };
    let gpu = start(true, 1.0);
    let _ = gpu.screenshot();
    for scale in [2.0, 1.25] {
        gpu.set_scale(scale);
        let shot = gpu.screenshot();
        assert_eq!(
            gpu.with_host(|host| host.renderer()),
            Some(Renderer::Gpu),
            "the frame went through the window's GPU surface"
        );
        let cpu = start(false, scale).screenshot();
        assert_close(&cpu, &shot, &format!("gpu_scale_{scale}"));
    }
}
