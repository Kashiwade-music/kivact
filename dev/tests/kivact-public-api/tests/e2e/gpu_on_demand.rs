//! Role: The GPU device a window starts when it needs it (`GpuStart::OnDemand`).
//!
//! A window that may draw on the GPU starts on the CPU and starts its device
//! only when something needs it: a user shader drawn (the CPU draws its
//! fallback meanwhile), or the App reading the window's adapter, as a
//! `gpu_interop::shared_image` does. A window on the CPU only (the test
//! default, `ARVANE_RENDERER=cpu`) never starts one. The tests run where the
//! window can have a GPU device and are skipped, saying why, elsewhere
//! (`gpu_unavailable`). Under `ARVANE_RENDERER=cpu` the on-demand tests check
//! that no device starts instead. The shared image's tests are in
//! `gpu_on_demand_shared`.
#![allow(
    clippy::print_stderr,
    reason = "report unavailable optional hardware explicitly"
)]

use std::time::Duration;

use super::paint::{BLUE, GREEN, RED, at, expect};
use super::support::mount_with;
use kivact::draw::{Paint, PaintShader};
use kivact::prelude::*;
use kivact::style::rgb;
use kivact::test::{MirrorMode, RenderOptions, Renderer, gpu_unavailable};

/// A paint shader that returns blue (its fallback in the tests is red).
const BLUE_PAINT: &str = "fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(0.0, 0.0, 1.0, 1.0);\n}\n";

/// The real time a device that must not start is given to start.
pub(crate) const GRACE: Duration = Duration::from_millis(300);

/// Whether the GPU lane can run here; says why when it cannot.
pub(crate) fn gpu_lane(name: &str) -> bool {
    match gpu_unavailable() {
        Some(reason) => {
            eprintln!("SKIPPED {name}: {reason}");
            false
        }
        None => true,
    }
}

/// `ARVANE_RENDERER=cpu`: every window draws on the CPU only.
pub(crate) fn cpu_forced() -> bool {
    std::env::var("ARVANE_RENDERER").as_deref() == Ok("cpu")
}

/// A window that may draw on the GPU, its device started when needed.
pub(crate) fn on_demand() -> RenderOptions {
    RenderOptions::from_env()
        .window_size(200.0, 100.0)
        .gpu_on_demand()
        .mirror(MirrorMode::Required)
}

/// A real window on the CPU only (the test default).
pub(crate) fn cpu_only() -> RenderOptions {
    RenderOptions::default()
        .window_size(200.0, 100.0)
        .mirror(MirrorMode::Required)
}

/// A green box, and beside it, while `shaded`, a box painted by `shader`
/// (blue on the GPU, its red fallback on the CPU).
fn scene(shaded: Signal<bool>, shader: PaintShader) -> impl IntoView {
    node()
        .child((
            node().style(at(0, 0, 100, 100).background(rgb(GREEN))),
            show(shaded, move || {
                node().style(
                    at(100, 0, 100, 100).background(Paint::from(shader.paint().fallback(rgb(RED)))),
                )
            }),
        ))
        .style(style().size(200, 100).background(rgb(0xffffff)))
}

/// A shader drawn in a window started with `options` is drawn by its
/// fallback, and no device starts, given the time to.
#[track_caller]
fn shader_stays_on_cpu(options: RenderOptions) {
    let shader = PaintShader::wgsl(BLUE_PAINT).expect("a valid paint shader");
    let (screen, shaded) = mount_with(options, move || {
        let shaded = signal(false);
        (shaded, scene(shaded, shader))
    });
    shaded.set(true);
    screen.run_until_idle();
    std::thread::sleep(GRACE);
    screen.run_until_idle();
    assert_eq!(
        screen.renderer(),
        Renderer::Cpu,
        "a window on the CPU only starts no device"
    );
    expect(
        &screen.screenshot(),
        150,
        50,
        RED,
        2,
        "the CPU draws the fallback",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A window waits on the CPU until it draws a user shader; the frame that
/// draws it first starts the device (drawn by the fallback meanwhile), and
/// the GPU then draws the shader.
#[test]
fn a_shader_starts_the_gpu_on_demand() {
    if !gpu_lane("a_shader_starts_the_gpu_on_demand") {
        return;
    }
    if cpu_forced() {
        return shader_stays_on_cpu(on_demand());
    }
    let shader = PaintShader::wgsl(BLUE_PAINT).expect("a valid paint shader");
    let (screen, shaded) = mount_with(on_demand(), move || {
        let shaded = signal(false);
        (shaded, scene(shaded, shader))
    });
    // Nothing needs the GPU, and an idle window's cheap CPU frames do not
    // start the device.
    screen.advance(Duration::from_millis(500));
    assert_eq!(
        screen.renderer(),
        Renderer::Cpu,
        "no device before the window needs one"
    );
    expect(&screen.screenshot(), 50, 50, GREEN, 2, "the CPU's frame");
    shaded.set(true);
    screen.run_until_idle();
    match screen.renderer() {
        Renderer::GpuStarting => expect(
            &screen.screenshot(),
            150,
            50,
            RED,
            2,
            "the CPU draws the fallback while the device starts",
        ),
        Renderer::Gpu => {}
        other => panic!("a drawn shader starts the device, not {other:?}"),
    }
    screen.wait_for_gpu();
    assert_eq!(screen.renderer(), Renderer::Gpu);
    let shot = screen.screenshot();
    expect(&shot, 150, 50, BLUE, 2, "the GPU draws the shader");
    expect(&shot, 50, 50, GREEN, 2, "and the rest of the window");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A window on the CPU only (as with `ARVANE_RENDERER=cpu`) draws a user
/// shader by its fallback and never starts a device.
#[test]
fn a_cpu_window_never_starts_the_gpu_for_a_shader() {
    if !gpu_lane("a_cpu_window_never_starts_the_gpu_for_a_shader") {
        return;
    }
    shader_stays_on_cpu(cpu_only());
}
