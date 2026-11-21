//! Role: Presenter contract tests.
//!
//! What presenting reports (`Presented`) and the frame policy behind it:
//! frames shown, skipped while the window is hidden, dropped while it has
//! no surface, lost surfaces counted to an error (a skipped frame ends the
//! row); surfaces destroyed and
//! made again; the renderer a window is forced to.

use arvane::{DriverOptions, Handler, Presented, Renderer, RendererChange, SteppedShell, Windows};
use arvane_shell::{
    EventTarget, ShellControl, ShellError, ShellEvent, ShellWaker, Step, StepOutcome,
};
use arvane_virtual::{GpuMode, VirtualBackend, VirtualOptions};
use kivact_types::gpu::GpuStart;
use kivact_types::{FailureKind, GpuPreference, Size, WindowId, WindowRequest};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::support::{PERIOD, RED, commands_composition, fill, open, pixel, present, show};

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
    window.script().fail_presents(2);
    lost(present(&mut window, frame()));
    lost(present(&mut window, frame()));
    show(&mut window, frame());
    window.script().fail_presents(3);
    lost(present(&mut window, frame()));
    lost(present(&mut window, frame()));
    let plan = window.with(move |w| w.frame_plan());
    let error = window
        .with(move |w| {
            let composition = crate::support::commands_composition(frame(), plan.size);
            w.present(&composition, plan)
        })
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
    let now = window.platform_probe().now_ns();
    window.script().fail_presents(2);
    lost(present(&mut window, frame()));
    lost(present(&mut window, frame()));
    window.script().set_occluded(true);
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
    window.script().set_occluded(false);
    window.script().fail_presents(2);
    lost(present(&mut window, frame()));
    lost(present(&mut window, frame()));
    // The losses started the backoff again.
    window.script().set_occluded(true);
    assert_eq!(
        skipped(present(&mut window, frame())),
        now + PERIOD,
        "a loss restarts the backoff"
    );
}

/// The frames presented so far: each window with the renderer that drew it.
type Frames = Rc<RefCell<Vec<(WindowId, Renderer)>>>;
/// The renderer changes so far, with the window they were noted for.
type Changes = Rc<RefCell<Vec<(WindowId, RendererChange)>>>;

/// Opens `extra` at start, then presents one frame to every open window on
/// each wake, recording what drew it.
struct Presenting {
    extra: Vec<WindowRequest>,
    frames: Frames,
}

impl Handler for Presenting {
    fn start(
        &mut self,
        windows: &mut Windows<'_>,
        _waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        for request in self.extra.drain(..) {
            windows.open(request)?;
        }
        Ok(ShellControl::Wait)
    }

    fn event(
        &mut self,
        _target: EventTarget,
        event: ShellEvent,
        windows: &mut Windows<'_>,
    ) -> Result<ShellControl, ShellError> {
        if matches!(event, ShellEvent::Wake(_)) {
            for id in windows.ids() {
                let Some(mut window) = windows.get(id) else {
                    continue;
                };
                let plan = window.frame_plan();
                let composition =
                    commands_composition(vec![fill(0.0, 0.0, 32.0, 32.0, RED)], plan.size);
                if let Ok(Presented::Shown { renderer, .. }) = window.present(&composition, plan) {
                    self.frames.borrow_mut().push((id, renderer));
                }
            }
        }
        Ok(ShellControl::Wait)
    }

    fn close(&mut self, _windows: &mut Windows<'_>) {}
}

/// A stepped driver whose GPU device starts with its first window, with
/// the main window and `extra` windows.
struct Gpu {
    shell: SteppedShell,
    frames: Frames,
    changes: Changes,
}

fn gpu_driver(extra: Vec<WindowRequest>) -> Gpu {
    let frames: Frames = Rc::default();
    let changes: Changes = Rc::default();
    let observed = Rc::clone(&changes);
    let observer: arvane::Observer = Rc::new(RefCell::new(
        move |window: WindowId, change: &RendererChange| {
            observed.borrow_mut().push((window, change.clone()));
        },
    ));
    let backend = VirtualBackend::new(VirtualOptions {
        main: WindowRequest::main(
            "gpu driver",
            Size {
                width: 32.0,
                height: 32.0,
            },
        ),
        frame_period_ns: PERIOD,
        gpu: GpuMode::Required,
        ..Default::default()
    });
    let handler = Presenting {
        extra,
        frames: Rc::clone(&frames),
    };
    let options = DriverOptions {
        gpu: GpuPreference {
            start: GpuStart::AtLaunch,
            ..GpuPreference::default()
        },
        observer: Some(observer),
    };
    let shell = arvane::start_stepped(Box::new(backend), options, handler)
        .expect("a stepped driver starts");
    Gpu {
        shell,
        frames,
        changes,
    }
}

/// A window of the same size as the main window, with id `2`.
fn second_window() -> WindowRequest {
    WindowRequest {
        id: WindowId::new(2).expect("a non-zero id"),
        parent: Some(WindowId::MAIN),
        ..WindowRequest::main(
            "second",
            Size {
                width: 32.0,
                height: 32.0,
            },
        )
    }
}

/// Step `shell` until `done` holds, waiting for the GPU's worker threads.
#[track_caller]
fn step_until(shells: &mut [&mut SteppedShell], done: impl Fn() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "the GPU renders within the time limit"
        );
        for shell in shells.iter_mut() {
            match shell.step(Step::Wake) {
                StepOutcome::Idle { .. } => {}
                outcome => panic!("the test shell remains live: {outcome:?}"),
            }
        }
        std::thread::park_timeout(Duration::from_millis(5));
    }
}

fn gpu_notes(changes: &Changes) -> usize {
    changes
        .borrow()
        .iter()
        .filter(|(_, change)| matches!(change, RendererChange::Gpu { .. }))
        .count()
}

fn drew_on(frames: &Frames, window: WindowId, renderer: Renderer) -> bool {
    frames
        .borrow()
        .iter()
        .any(|(id, drawn)| *id == window && *drawn == renderer)
}

#[test]
fn two_windows_of_one_driver_share_one_gpu_device() {
    if let Some(reason) = arvane::gpu_unavailable() {
        eprintln!("SKIPPED two_windows_of_one_driver_share_one_gpu_device: {reason}");
        return;
    }
    let mut driver = gpu_driver(vec![second_window()]);
    let frames = Rc::clone(&driver.frames);
    step_until(&mut [&mut driver.shell], || {
        drew_on(&frames, WindowId::MAIN, Renderer::Gpu)
            && drew_on(&frames, WindowId::new(2).expect("two"), Renderer::Gpu)
    });
    assert_eq!(
        gpu_notes(&driver.changes),
        1,
        "the driver's device is noted once for both windows"
    );
}

#[test]
fn two_drivers_have_a_device_each_and_closing_one_leaves_the_other_drawing() {
    if let Some(reason) = arvane::gpu_unavailable() {
        eprintln!(
            "SKIPPED two_drivers_have_a_device_each_and_closing_one_leaves_the_other_drawing: {reason}"
        );
        return;
    }
    let mut first = gpu_driver(Vec::new());
    let mut second = gpu_driver(Vec::new());
    let (first_frames, second_frames) = (Rc::clone(&first.frames), Rc::clone(&second.frames));
    step_until(&mut [&mut first.shell, &mut second.shell], || {
        drew_on(&first_frames, WindowId::MAIN, Renderer::Gpu)
            && drew_on(&second_frames, WindowId::MAIN, Renderer::Gpu)
    });
    assert_eq!(gpu_notes(&first.changes), 1, "the first driver's device");
    assert_eq!(gpu_notes(&second.changes), 1, "the second driver's device");
    drop(first);
    second_frames.borrow_mut().clear();
    step_until(&mut [&mut second.shell], || {
        drew_on(&second_frames, WindowId::MAIN, Renderer::Gpu)
    });
}
