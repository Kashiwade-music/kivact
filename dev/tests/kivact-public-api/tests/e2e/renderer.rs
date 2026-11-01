//! Role: Public selection of the test frame's rasterizer.
use kivact::prelude::*;
use kivact::test::{RenderOptions, WindowMode, render_with};

fn screen(gpu: bool) -> kivact::test::Screen {
    render_with(
        RenderOptions {
            width: 160.,
            height: 80.,
            gpu,
            window: WindowMode::Real,
            ..Default::default()
        },
        || node().style(style().size(80, 40).background(rgb(0x00cc88))),
    )
}

#[test]
fn public_cpu_option_renders_the_requested_pixels() {
    let screen = screen(false);
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0x00cc88));
    assert!(screen.errors().is_empty());
}

/// WSLg may have no hardware adapter, as in the existing GPU contract suite.
#[cfg(feature = "gpu")]
#[test]
#[allow(
    clippy::print_stderr,
    reason = "report unavailable optional hardware explicitly"
)]
fn public_gpu_option_uses_a_real_surface_and_reads_its_pixels() {
    if let Some(reason) = kivact::test::gpu_unavailable() {
        eprintln!("SKIPPED public_gpu_option_uses_a_real_surface_and_reads_its_pixels: {reason}");
        return;
    }
    let screen = screen(true);
    assert_eq!(screen.screenshot().pixel(20, 20), Some(0x00cc88));
    assert!(screen.errors().is_empty());
}
