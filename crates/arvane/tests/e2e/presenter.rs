//! Role: Presenter contract tests.
//!
//! What presenting reports (`Presented`) and the frame policy behind it:
//! frames shown, skipped while the window is hidden, dropped while it has
//! no surface, lost surfaces counted to an error (a skipped frame ends the
//! row); surfaces destroyed and
//! made again; the renderer a window is forced to.

use arvane::{FailureKind, Presented, Renderer, Window as _};

use crate::support::{PERIOD, RED, fill, open, pixel, present, show};

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
fn a_lost_surface_is_made_again_and_the_third_loss_in_a_row_is_an_error() {
    let mut window = open(32.0, 32.0);
    let frame = || vec![fill(0.0, 0.0, 32.0, 32.0, RED)];
    let lost = |presented: Presented| match presented {
        Presented::SurfaceLost(error) => assert_eq!(error.kind, FailureKind::Surface),
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
    assert_eq!(error.kind, FailureKind::Surface);
}

#[test]
fn a_skipped_frame_ends_a_row_of_losses_and_a_loss_restarts_the_backoff() {
    let mut window = open(32.0, 32.0);
    let frame = || vec![fill(0.0, 0.0, 32.0, 32.0, RED)];
    let lost = |presented: Presented| match presented {
        Presented::SurfaceLost(error) => assert_eq!(error.kind, FailureKind::Surface),
        other => panic!("not lost: {other:?}"),
    };
    let skipped = |presented: Presented| match presented {
        Presented::Skipped { retry_at_ns } => retry_at_ns,
        other => panic!("not skipped: {other:?}"),
    };
    let now = window.now_ns();
    window.fail_presents(2);
    lost(present(&mut window, frame()));
    lost(present(&mut window, frame()));
    window.set_occluded(true);
    assert_eq!(
        skipped(present(&mut window, frame())),
        now + PERIOD,
        "the first skip waits a period"
    );
    assert_eq!(
        skipped(present(&mut window, frame())),
        now + 2 * PERIOD,
        "the second waits two"
    );
    // The skipped frames ended the row: two more losses are no error.
    window.set_occluded(false);
    window.fail_presents(2);
    lost(present(&mut window, frame()));
    lost(present(&mut window, frame()));
    // The losses started the backoff again.
    window.set_occluded(true);
    assert_eq!(
        skipped(present(&mut window, frame())),
        now + PERIOD,
        "a loss restarts the backoff"
    );
}
