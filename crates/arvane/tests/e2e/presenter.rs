//! Role: Presenter contract tests.
//!
//! What presenting reports (`Presented`) and the frame policy behind it:
//! frames shown, skipped while the window is hidden, dropped while it has
//! no surface, lost surfaces counted to an error; surfaces destroyed and
//! made again; the renderer a window is forced to.

use arvane::test_display::{VirtualWindow, VirtualWindowOptions, gpu_unavailable};
use arvane::{ErrorKind, Presented, Renderer, Window as _};
use kivact_types::Size;

use crate::support::{BLUE, PERIOD, RED, fill, open, options, pixel, present, show};

#[test]
fn a_frame_shown_reports_its_renderer() {
    let mut window = open(32.0, 32.0);
    assert_eq!(window.renderer(), None, "nothing drawn yet");
    match present(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, RED)]) {
        Presented::Shown { renderer, .. } => assert_eq!(renderer, Renderer::Cpu),
        other => panic!("not shown: {other:?}"),
    }
    assert_eq!(window.renderer(), Some(Renderer::Cpu));
    assert_eq!(pixel(&window, 1, 1), RED);
}

#[test]
fn a_hidden_window_skips_frames_backing_off_to_sixteen_periods() {
    let mut window = open(32.0, 32.0);
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, BLUE)]);
    window.set_now(1_000);
    window.set_occluded(true);
    let mut waits = Vec::new();
    for _ in 0..7 {
        match present(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, RED)]) {
            Presented::Skipped { retry_at_ns } => waits.push((retry_at_ns - 1_000) / PERIOD),
            other => panic!("not skipped: {other:?}"),
        }
    }
    assert_eq!(waits, [1, 2, 4, 8, 16, 16, 16]);
    assert_eq!(pixel(&window, 1, 1), BLUE, "nothing new is shown");
    // Shown again, the window draws at once and the wait starts afresh.
    window.set_occluded(false);
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, RED)]);
    assert_eq!(pixel(&window, 1, 1), RED);
    window.set_occluded(true);
    assert_eq!(
        present(&mut window, Vec::new()),
        Presented::Skipped {
            retry_at_ns: 1_000 + PERIOD
        }
    );
}

#[test]
fn a_window_of_no_pixels_drops_frames_until_it_is_resized() {
    let mut window = open(32.0, 32.0);
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, BLUE)]);
    // Minimized: a window of no pixels draws nothing, on any renderer,
    // and is not drawn again until a resize asks for a frame.
    for size in [(0.0, 0.0), (0.0, 32.0), (32.0, 0.0)] {
        window.resize(Size {
            width: size.0,
            height: size.1,
        });
        match present(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, RED)]) {
            Presented::Dropped(error) => assert_eq!(error.kind(), ErrorKind::Content),
            other => panic!("not dropped at {size:?}: {other:?}"),
        }
        assert_eq!(pixel(&window, 1, 1), BLUE, "the picture stays");
    }
    window.resize(Size {
        width: 32.0,
        height: 32.0,
    });
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, RED)]);
    assert_eq!(pixel(&window, 1, 1), RED);
}

#[test]
fn without_a_surface_frames_are_dropped_until_a_new_one_draws_them() {
    let mut window = open(32.0, 32.0);
    assert_eq!(window.surface_generation(), Some(1));
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, BLUE)]);
    // Suspended on Android: the platform destroys the surface.
    window.destroy_surface();
    assert_eq!(window.surface_generation(), None);
    match present(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, RED)]) {
        Presented::Dropped(error) => assert_eq!(error.kind(), ErrorKind::Unavailable),
        other => panic!("not dropped: {other:?}"),
    }
    assert_eq!(pixel(&window, 1, 1), BLUE, "nothing new is shown");
    // Resumed: the next generation's surface draws the frame.
    window.create_surface();
    assert_eq!(window.surface_generation(), Some(2));
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, RED)]);
    assert_eq!(pixel(&window, 1, 1), RED);
    // Destroying and making it again counts the generations on.
    window.destroy_surface();
    window.destroy_surface();
    window.create_surface();
    window.create_surface();
    assert_eq!(window.surface_generation(), Some(3));
}

#[test]
fn a_lost_surface_is_made_again_and_the_third_loss_in_a_row_is_an_error() {
    let mut window = open(32.0, 32.0);
    let frame = || vec![fill(0.0, 0.0, 32.0, 32.0, RED)];
    let lost = |presented: Presented| match presented {
        Presented::SurfaceLost(error) => assert_eq!(error.kind(), ErrorKind::Surface),
        other => panic!("not lost: {other:?}"),
    };
    // Two losses, then a frame shown: the count starts again.
    window.fail_presents(2);
    lost(present(&mut window, frame()));
    lost(present(&mut window, frame()));
    show(&mut window, frame());
    window.fail_presents(3);
    lost(present(&mut window, frame()));
    lost(present(&mut window, frame()));
    let plan = window.frame_plan();
    let error = window
        .present(&crate::support::scene(frame()), plan)
        .expect_err("the third loss in a row");
    assert_eq!(error.kind(), ErrorKind::Surface);
}

#[test]
fn a_forced_renderer_overrides_the_preference_as_arvane_renderer_does() {
    // `ARVANE_RENDERER=cpu` on a window that prefers the GPU: no GPU is
    // started (this opens without one too) and frames are the CPU's.
    let mut window = VirtualWindow::open(VirtualWindowOptions {
        gpu: true,
        renderer: Some(Renderer::Cpu),
        ..options(32.0, 32.0)
    })
    .expect("a CPU window opens without a GPU");
    assert!(!window.capabilities().gpu);
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, RED)]);
    assert_eq!(window.renderer(), Some(Renderer::Cpu));
    // `ARVANE_RENDERER=gpu` on a window that prefers the CPU.
    if let Some(reason) = gpu_unavailable() {
        eprintln!("SKIPPED the forced GPU half of this test: {reason}");
        return;
    }
    let mut window = VirtualWindow::open(VirtualWindowOptions {
        gpu: false,
        renderer: Some(Renderer::Gpu),
        ..options(32.0, 32.0)
    })
    .expect("a GPU window opens");
    assert!(window.capabilities().gpu);
    show(&mut window, vec![fill(0.0, 0.0, 32.0, 32.0, RED)]);
    assert_eq!(window.renderer(), Some(Renderer::Gpu));
    assert_eq!(pixel(&window, 16, 16), RED);
}

#[test]
fn a_gpu_window_makes_its_surface_again_after_the_platform_destroyed_it() {
    if let Some(reason) = gpu_unavailable() {
        eprintln!(
            "SKIPPED a_gpu_window_makes_its_surface_again_after_the_platform_destroyed_it: {reason}"
        );
        return;
    }
    let mut window = VirtualWindow::open(VirtualWindowOptions {
        gpu: true,
        ..options(48.0, 32.0)
    })
    .expect("a GPU window opens");
    show(&mut window, vec![fill(0.0, 0.0, 48.0, 32.0, BLUE)]);
    assert_eq!(window.renderer(), Some(Renderer::Gpu));
    window.destroy_surface();
    assert!(matches!(
        present(&mut window, vec![fill(0.0, 0.0, 48.0, 32.0, RED)]),
        Presented::Dropped(_)
    ));
    window.create_surface();
    show(&mut window, vec![fill(0.0, 0.0, 48.0, 32.0, RED)]);
    assert_eq!(window.renderer(), Some(Renderer::Gpu));
    assert_eq!(pixel(&window, 24, 16), RED);
}
