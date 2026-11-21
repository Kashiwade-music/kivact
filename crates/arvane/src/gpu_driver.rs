//! Role: The driver's one GPU device and its start, restart and failure.
//!
//! A driver (one App's event loop) holds one device, shared by every
//! window's context ([`ReadyDevice`]); another driver in the same process
//! (an embedded editor) has its own. The device starts once per driver: with
//! the first window when the host asks for it (`GpuStart::AtLaunch`), else
//! when it is needed (`GpuStart::OnDemand`: the App's demand, a user shader,
//! or CPU frames that keep missing half the display period). A worker thread
//! creates it (`GpuDevice::new` blocks). Until it is ready, frames are drawn
//! on the CPU, so no step stalls a window's thread.
//!
//! Each window of the driver then makes its own surface on its thread and a
//! worker configures the swapchain (`presenter/gpu.rs`). Each finished step
//! wakes the shell, which moves the start along before the host sees the wake.
//!
//! Failure policy: a device failure (device loss, out of memory, a validation
//! error) drops the device; a new one starts once per driver, the frames in
//! between drawn on the CPU. A second device failure, or a failed start,
//! leaves the driver on the CPU for the rest of the session. A surface
//! failure belongs to its window ([`Fallback::Surface`]).

use crate::env::RendererStart;
use crate::presentation::{GpuStartCause, Notes, RendererChange};
use arvane_compose::{GpuDevice, GpuEnv};
use arvane_render::RenderError;
use arvane_shell::{ShellWaker, ShellWindow, WakeReasons};
use std::rc::Rc;
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Device restarts allowed per driver (background starts only).
const RESTARTS: u8 = 1;

/// Consecutive CPU frames over half a display period that start the device
/// (on demand).
const SLOW_FRAMES: u8 = 8;

/// Bound DLL shutdown without waiting forever for a graphics driver.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);

/// Wakes the presenting thread (any thread may call it).
pub(crate) type Wake = Arc<dyn Fn() + Send + Sync>;

/// A device and how long the worker took to create it.
type Started = Result<(GpuDevice, Duration), RenderError>;

/// The driver's ready device and the wake of its worker threads: what a
/// window draws with.
pub(crate) struct ReadyDevice {
    /// The device, shared by the driver's windows.
    pub(crate) gpu: Rc<GpuDevice>,
    /// Wakes the window's thread when a worker of the device finishes.
    pub(crate) wake: Wake,
}

/// What a failed GPU step means for the window's frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fallback {
    /// The window's surface failed and was dropped; the next frame makes a
    /// new one.
    Surface,
    /// The GPU does not draw this frame: the device failed (restarting or
    /// not), or the window's surface kept failing.
    Cpu,
    /// Not a device or surface failure (the scene, GPU limits, shared
    /// textures): the caller handles the error.
    Other,
}

/// A worker thread not joined yet, and the channel of its one result.
struct Running<T> {
    result: mpsc::Receiver<T>,
    thread: JoinHandle<()>,
}

/// A worker thread of the GPU start. `poll` and `shutdown` consume it, so
/// a worker is never polled after it was joined.
pub(crate) struct Worker<T> {
    name: &'static str,
    /// Joined after `poll` or `shutdown` consumes the result.
    run: WorkerState<T>,
}

enum WorkerState<T> {
    Running(Running<T>),
    Joined,
}

/// What polling a worker found.
pub(crate) enum Polled<T> {
    /// Still working.
    Working(Worker<T>),
    /// It sent its result and ended.
    Sent(T),
    /// It ended without a result.
    Stopped,
}

impl<T: Send + 'static> Worker<T> {
    /// Run `work` on a new thread named `name`, calling `wake` once it sent
    /// its result; `None` when the system cannot start a thread.
    pub(crate) fn spawn(
        name: &'static str,
        wake: Wake,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Option<Self> {
        let (sender, result) = mpsc::sync_channel(1);
        let thread = std::thread::Builder::new()
            .name(name.into())
            .spawn(move || {
                let _ = sender.send(work());
                wake();
            })
            .ok()?;
        Some(Self {
            name,
            run: WorkerState::Running(Running { result, thread }),
        })
    }
}

impl<T> Worker<T> {
    /// The worker's result, once it sent one (the thread is joined then).
    pub(crate) fn poll(mut self) -> Polled<T> {
        let WorkerState::Running(running) = &self.run else {
            return Polled::Stopped;
        };
        let received = running.result.try_recv();
        match received {
            Err(mpsc::TryRecvError::Empty) => Polled::Working(self),
            Ok(value) => {
                self.join();
                Polled::Sent(value)
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.join();
                Polled::Stopped
            }
        }
    }

    /// Join the ended thread, resuming its panic (unless one is already
    /// unwinding). A result still queued is dropped here, on this thread.
    fn join(&mut self) {
        if let WorkerState::Running(Running { thread, .. }) =
            std::mem::replace(&mut self.run, WorkerState::Joined)
            && let Err(payload) = thread.join()
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(payload);
        }
    }

    /// Wait (bounded, the window kept responsive) for the thread to end and
    /// join it; a thread still running is left to `Drop`, which detaches it.
    pub(crate) fn shutdown(mut self, window: &mut dyn ShellWindow) {
        let finished = match &self.run {
            WorkerState::Joined => true,
            WorkerState::Running(running) => window
                .surface()
                .wait_until(&mut || running.thread.is_finished(), SHUTDOWN_WAIT),
        };
        if finished {
            // Queued surfaces and devices are dropped here, on the UI thread.
            self.join();
        }
    }
}

impl<T> Drop for Worker<T> {
    fn drop(&mut self) {
        let WorkerState::Running(Running { result, thread }) =
            std::mem::replace(&mut self.run, WorkerState::Joined)
        else {
            return;
        };
        if thread.is_finished() {
            let _ = thread.join();
            return;
        }
        kivact_types::process::DetachedThreads::note();
        tracing::warn!(target: "kivact", worker = self.name,
            "GPU worker still running when its window closed; its thread is detached");
        // Keep a late result alive rather than destroying its native surface
        // on the worker. A timed-out driver is an explicitly documented leak.
        std::mem::forget(result);
    }
}

/// The driver's GPU device life (see the module documentation).
pub(crate) struct GpuDriver {
    /// How devices are made.
    env: GpuEnv,
    state: State,
}

/// The driver's GPU state.
enum State {
    /// Not begun yet ([`GpuDriver::begin`]).
    New,
    /// No device, ever: the host or `ARVANE_RENDERER` chose the CPU.
    CpuOnly,
    /// No device yet (`GpuStart::OnDemand`): one starts on the App's demand
    /// ([`GpuDriver::demand`]), for a drawn user shader, or after
    /// [`SLOW_FRAMES`] CPU frames in a row over half the display period;
    /// `waker` wakes the driver's thread for its start steps.
    Deferred { slow: u8, waker: ShellWaker },
    /// Started on worker threads.
    Background(Background),
    /// CPU for the rest of the session.
    Failed,
    /// The driver shut down.
    ShutDown,
}

/// A GPU started on worker threads.
struct Background {
    /// Wakes the driver's thread when a worker finishes.
    wake: Wake,
    /// Device restarts left this session.
    restarts: u8,
    step: Step,
}

/// Where a background start is.
enum Step {
    /// A worker creates the device.
    Starting(Worker<Started>),
    /// The device is ready. `announced`: a window of this device has drawn
    /// on it, and its renderer change was noted.
    Ready {
        gpu: Rc<GpuDevice>,
        init: Duration,
        announced: bool,
    },
}

impl GpuDriver {
    /// A driver whose device is made as `env` says; nothing starts until
    /// [`GpuDriver::begin`].
    pub(crate) fn new(env: GpuEnv) -> Self {
        Self {
            env,
            state: State::New,
        }
    }

    /// The renderer the driver starts with, as `start` says (once, when the
    /// driver begins): the CPU only, a device started on a worker now, or a
    /// device that waits until it is needed. Notes [`RendererChange::Start`].
    pub(crate) fn begin(&mut self, start: RendererStart, waker: ShellWaker, notes: &mut Notes) {
        debug_assert!(
            matches!(self.state, State::New),
            "invariant: begin runs once, on a new driver"
        );
        notes.note(RendererChange::Start {
            gpu: start == RendererStart::GpuAtLaunch,
        });
        self.state = match start {
            RendererStart::Cpu => State::CpuOnly,
            RendererStart::GpuAtLaunch => self.spawn(waker, notes),
            RendererStart::GpuOnDemand => State::Deferred { slow: 0, waker },
        };
    }

    /// Whether the device waits for a reason to start (`GpuStart::OnDemand`).
    pub(crate) fn waits(&self) -> bool {
        matches!(self.state, State::Deferred { .. })
    }

    /// Start the device the driver waited for, for `cause`. Nothing in any
    /// other state: a driver on the CPU only never starts one.
    pub(crate) fn demand(&mut self, cause: GpuStartCause, notes: &mut Notes) {
        let State::Deferred { waker, .. } = &self.state else {
            return;
        };
        let waker = waker.clone();
        notes.note(RendererChange::GpuStarting { cause });
        self.state = self.spawn(waker, notes);
    }

    /// A CPU frame took `spent` of the `period` the display gives a frame: a
    /// device that waits starts after [`SLOW_FRAMES`] frames in a row over
    /// half of it.
    pub(crate) fn note_cpu_frame(&mut self, spent: Duration, period: Duration, notes: &mut Notes) {
        let State::Deferred { slow, .. } = &mut self.state else {
            return;
        };
        if spent <= period / 2 {
            *slow = 0;
            return;
        }
        // Below SLOW_FRAMES here: the device starts at SLOW_FRAMES.
        *slow += 1;
        if *slow >= SLOW_FRAMES {
            self.demand(GpuStartCause::SlowFrames, notes);
        }
    }

    /// The state that creates the device on a worker thread; `waker` wakes
    /// the driver's thread when the worker finishes. The CPU for the rest of
    /// the session when no worker can start.
    fn spawn(&self, waker: ShellWaker, notes: &mut Notes) -> State {
        let wake: Wake = Arc::new(move || waker.wake(WakeReasons::GPU_STEP));
        match start_device(self.env, &wake) {
            Some(worker) => State::Background(Background {
                wake,
                restarts: RESTARTS,
                step: Step::Starting(worker),
            }),
            None => cpu_session(notes, "the GPU worker thread cannot start"),
        }
    }

    /// Move the device's start along: once its worker has created the device,
    /// the device is ready for the driver's windows. A failed start leaves the
    /// driver on the CPU for the session. Nothing for any other state.
    pub(crate) fn advance(&mut self, notes: &mut Notes) {
        let state = std::mem::replace(&mut self.state, State::New);
        self.state = match state {
            State::Background(Background {
                wake,
                restarts,
                step: Step::Starting(worker),
            }) => match worker.poll() {
                Polled::Working(worker) => State::Background(Background {
                    wake,
                    restarts,
                    step: Step::Starting(worker),
                }),
                Polled::Sent(Ok((gpu, init))) => State::Background(Background {
                    wake,
                    restarts,
                    step: Step::Ready {
                        gpu: Rc::new(gpu),
                        init,
                        announced: false,
                    },
                }),
                Polled::Sent(Err(error)) => cpu_session(notes, format!("GPU unavailable: {error}")),
                Polled::Stopped => cpu_session(notes, "GPU initialization stopped"),
            },
            state => state,
        };
    }

    /// The ready device, when there is one.
    pub(crate) fn device(&self) -> Option<ReadyDevice> {
        match &self.state {
            State::Background(Background {
                wake,
                step: Step::Ready { gpu, .. },
                ..
            }) => Some(ReadyDevice {
                gpu: Rc::clone(gpu),
                wake: Arc::clone(wake),
            }),
            _ => None,
        }
    }

    /// A window of the ready device is configured and draws on it (`configure`:
    /// how long configuring its swapchain took). The first window of a device
    /// notes the renderer change; the others of the same device note nothing.
    pub(crate) fn announce(&mut self, configure: Duration, notes: &mut Notes) {
        let State::Background(Background {
            step:
                Step::Ready {
                    gpu,
                    init,
                    announced,
                },
            ..
        }) = &mut self.state
        else {
            return;
        };
        if *announced {
            return;
        }
        *announced = true;
        notes.note(RendererChange::Gpu {
            adapter: gpu.describe(),
            init: *init,
            configure: Some(configure),
            interop: gpu.interop_adapter(),
        });
    }

    /// Apply the failure `error` of a GPU step of a window (a frame, an
    /// offscreen render, an import, making or configuring its swapchain)
    /// and say what it means for the window's frame. A device failure drops
    /// the device; the windows find out when they next look at it.
    pub(crate) fn failed(&mut self, error: &RenderError, notes: &mut Notes) -> Fallback {
        match error {
            RenderError::Surface(_) => Fallback::Surface,
            RenderError::Backend(_) | RenderError::NoAdapter(_) => {
                self.device_failed(error, notes);
                Fallback::Cpu
            }
            _ => Fallback::Other,
        }
    }

    /// The device failed: drop it, and start a new one once per driver
    /// (background starts only).
    fn device_failed(&mut self, error: &RenderError, notes: &mut Notes) {
        let state = std::mem::replace(&mut self.state, State::New);
        self.state = match state {
            State::Background(Background { wake, restarts, .. }) if restarts > 0 => {
                notes.note(cpu(format!("GPU device failed, restarting: {error}")));
                match start_device(self.env, &wake) {
                    Some(worker) => {
                        notes.note(RendererChange::GpuStarting {
                            cause: GpuStartCause::Restart,
                        });
                        State::Background(Background {
                            wake,
                            restarts: restarts - 1,
                            step: Step::Starting(worker),
                        })
                    }
                    None => cpu_session(notes, "the GPU worker thread cannot start"),
                }
            }
            _ => cpu_session(
                notes,
                format!("GPU device failed, CPU for the session: {error}"),
            ),
        };
    }

    /// The CPU for the rest of the session: a window's surface kept failing
    /// (see the failure policy).
    pub(crate) fn fallback_to_cpu(&mut self, notes: &mut Notes, error: &dyn std::fmt::Display) {
        self.state = cpu_session(
            notes,
            format!("GPU surface keeps failing, CPU for the session: {error}"),
        );
    }

    /// Stop the start's worker that needs the native `window`, before the
    /// window is dropped. The driver's device goes with its state.
    pub(crate) fn shutdown(&mut self, window: &mut dyn ShellWindow) {
        if let State::Background(Background {
            step: Step::Starting(worker),
            ..
        }) = std::mem::replace(&mut self.state, State::ShutDown)
        {
            worker.shutdown(window);
        }
    }
}

/// A worker creating a device as `env` says; `None` when it cannot start.
fn start_device(env: GpuEnv, wake: &Wake) -> Option<Worker<Started>> {
    Worker::spawn("arvane-gpu-init", Arc::clone(wake), move || {
        let since = Instant::now();
        GpuDevice::new(&env).map(|device| (device, since.elapsed()))
    })
}

/// The CPU for the rest of the session, noted with `reason`.
fn cpu_session(notes: &mut Notes, reason: impl Into<String>) -> State {
    notes.note(cpu(reason.into()));
    State::Failed
}

fn cpu(reason: String) -> RendererChange {
    RendererChange::Cpu { reason }
}
