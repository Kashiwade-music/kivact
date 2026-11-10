//! Role: A presenter's GPU renderer and its life cycle (`arvane-render`'s feature `gpu`).
//!
//! The renderer of the presenter's one window ([`State`]): off (CPU only);
//! started in the background with the window: a worker thread creates
//! the device, the window's thread makes the window's surface and a worker
//! configures its swapchain (~0.2 s the first time on some drivers), then
//! frames go to the window's GPU surface; failed: CPU for the rest of the
//! session. Until ready, frames are drawn on the CPU, so no step stalls the
//! window's thread.
//!
//! A native window starts the GPU as it is created when the host asks for
//! it (`GpuStart::AtLaunch`), else when the window needs it (the first
//! demand, or CPU frames that keep missing half the display period:
//! `GpuStart::OnDemand`, the default), not once CPU frames grow costly.
//! The worker's device is taken on the window's thread
//! (winit hands window handles out only there), which makes the window's
//! surface; the first swapchain is configured on a worker too. Each
//! finished step wakes the shell, which moves the start along before the
//! host sees the wake; once the GPU can draw the window, a frame is asked
//! for and drawn on the GPU at once, so the switch does not wait for
//! input. While the swapchain is configured the window keeps its last CPU
//! frame and new frames are not shown (once a flip-model swapchain
//! exists, a frame presented through GDI may not show) but drawn again
//! later. The host never knows these steps.
//!
//! Renderer changes are noted ([`Notes`]) as they happen. A surface the
//! platform destroys (Android, in the background) is dropped with it; the
//! window's next frame makes a new one.
//!
//! Failure policy ([`Gpu::failed`], with `arvane-render`'s surface
//! policy): a surface error, from a frame or from making or configuring
//! the swapchain, drops the window's surface and the next frame makes a
//! new one; the frame policy counts consecutive losses and moves the
//! window to the CPU for the rest of the session. A device failure (device
//! loss, out of memory, a validation error) drops the device and its
//! surface; a new device is started once per session, the frames in
//! between drawn on the CPU. A second device failure or a failed restart
//! leaves the presenter on the CPU.

use crate::ShellWaker;
use crate::env::RendererStart;
use crate::presentation::{GpuStartCause, Notes, RendererChange};
use arvane_compose::{GpuContext, GpuEnv, GpuSurface, Presentation};
use arvane_render::RenderError;
use kivact_types::render::{Composition, CpuFrame, FrameReport};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Device restarts allowed per session (background starts only).
const RESTARTS: u8 = 1;

/// Consecutive CPU frames over half a display period that start the device
/// (on demand).
const SLOW_FRAMES: u8 = 8;

/// Bound DLL shutdown without waiting forever for a graphics driver.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);

/// Wakes the presenting thread (any thread may call it).
pub(crate) type Wake = Arc<dyn Fn() + Send + Sync>;

/// Makes the window's surface on the window's thread.
pub(crate) type MakeSurface<'a> = &'a dyn Fn(&GpuContext) -> Result<GpuSurface, RenderError>;

/// A device and how long the worker took to create it.
type Started = Result<(Box<GpuContext>, Duration), RenderError>;
/// A device, the window's configured surface, and how long configuring took.
type Configured = (Box<GpuContext>, Result<(GpuSurface, Duration), RenderError>);

/// A worker thread not joined yet, and the channel of its one result.
struct Running<T> {
    result: mpsc::Receiver<T>,
    thread: JoinHandle<()>,
}

/// A worker thread of the GPU start. `poll` and `shutdown` consume it, so
/// a worker is never polled after it was joined.
struct Worker<T> {
    name: &'static str,
    /// Joined after `poll` or `shutdown` consumes the result.
    run: WorkerState<T>,
}

enum WorkerState<T> {
    Running(Running<T>),
    Joined,
}

/// What polling a worker found.
enum Polled<T> {
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
    fn spawn(
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
    fn poll(mut self) -> Polled<T> {
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
    fn shutdown(mut self, window: &mut dyn arvane_shell::ShellWindow) {
        let finished = match &self.run {
            WorkerState::Joined => true,
            WorkerState::Running(running) => {
                window.wait_until(&mut || running.thread.is_finished(), SHUTDOWN_WAIT)
            }
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

enum DeviceSurface {
    Pending,
    Live(Box<GpuSurface>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfigureTarget {
    Current,
    Destroyed,
}

/// A ready device and the window's surface on it.
struct Device {
    /// The next GPU frame creates a pending surface.
    /// Boxed: a surface is large.
    surface: DeviceSurface,
    /// Boxed: a context is large.
    context: Box<GpuContext>,
}

impl Device {
    fn new(context: Box<GpuContext>) -> Self {
        Self {
            surface: DeviceSurface::Pending,
            context,
        }
    }
}

/// A GPU started on worker threads (a native window).
struct Background {
    /// Wakes the window's thread when a worker finishes.
    wake: Wake,
    /// Device restarts left this session.
    restarts: u8,
    step: Step,
}

/// Where a background start is.
enum Step {
    /// A worker creates the device.
    Starting(Worker<Started>),
    /// The device is ready; the window's surface is made with its next
    /// frame or wake.
    Waiting {
        context: Box<GpuContext>,
        init: Duration,
    },
    /// A worker configures the swapchain for the current or destroyed surface.
    Configuring {
        worker: Worker<Configured>,
        init: Duration,
        surface: ConfigureTarget,
    },
    /// Frames go to the GPU.
    Ready(Device),
}

/// The GPU renderer's life cycle (see the module documentation).
enum State {
    /// No device, and none starts: the window draws on the CPU only (the
    /// host or `ARVANE_RENDERER` chose it), or its presenter shut down. A new
    /// presenter is here until [`Gpu::begin`].
    Off,
    /// No device yet (`GpuStart::OnDemand`): one starts on the App's demand
    /// ([`Gpu::demand`]), for a drawn user shader, or after [`SLOW_FRAMES`]
    /// CPU frames in a row over half the display period
    /// ([`Gpu::note_cpu_frame`]); `waker` wakes the window's thread for its
    /// start steps.
    Deferred { slow: u8, waker: ShellWaker },
    /// Started on worker threads.
    Background(Background),
    /// CPU for the rest of the session.
    Failed,
}

/// What a failed GPU step means for the window's frame ([`Gpu::failed`]).
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

/// The GPU renderer of a presenter's window.
pub(crate) struct Gpu {
    /// How devices are made.
    env: GpuEnv,
    state: State,
}

impl Gpu {
    pub(crate) fn new(env: GpuEnv) -> Self {
        Self {
            env,
            state: State::Off,
        }
    }

    /// Stop the workers that need the native window before it is dropped.
    pub(crate) fn shutdown(&mut self, window: &mut dyn arvane_shell::ShellWindow) {
        if let State::Background(background) = std::mem::replace(&mut self.state, State::Off) {
            match background.step {
                Step::Starting(worker) => worker.shutdown(window),
                Step::Configuring { worker, .. } => worker.shutdown(window),
                Step::Waiting { .. } | Step::Ready(_) => {}
            }
        }
    }

    /// Release rebuildable validation metadata under memory pressure.
    pub(crate) fn release_caches(&mut self) {
        match &mut self.state {
            State::Background(Background {
                step: Step::Ready(device),
                ..
            }) => device.context.release_caches(),
            State::Background(Background {
                step: Step::Waiting { context, .. },
                ..
            }) => context.release_caches(),
            // A device still on a worker has drawn no window frame yet.
            State::Off | State::Deferred { .. } | State::Failed | State::Background(_) => {}
        }
    }

    fn device(&self) -> Option<&Device> {
        match &self.state {
            State::Background(Background {
                step: Step::Ready(device),
                ..
            }) => Some(device),
            _ => None,
        }
    }

    fn device_mut(&mut self) -> Option<&mut Device> {
        match &mut self.state {
            State::Background(Background {
                step: Step::Ready(device),
                ..
            }) => Some(device),
            _ => None,
        }
    }

    /// Whether frames go to the GPU now.
    pub(crate) fn ready(&self) -> bool {
        self.device().is_some()
    }

    /// Whether a worker is configuring the window's swapchain.
    pub(crate) fn configuring(&self) -> bool {
        matches!(
            &self.state,
            State::Background(Background {
                step: Step::Configuring { .. },
                ..
            })
        )
    }

    /// The renderer the window starts with, as `start` says (once, when the
    /// window opens): the CPU only, a device started on a worker now, or a
    /// device that waits until the window needs it. Notes
    /// [`RendererChange::Start`].
    pub(crate) fn begin(&mut self, start: RendererStart, waker: ShellWaker, notes: &mut Notes) {
        debug_assert!(
            matches!(self.state, State::Off),
            "invariant: begin runs once, on a new presenter"
        );
        notes.note(RendererChange::Start {
            gpu: start == RendererStart::GpuAtLaunch,
        });
        match start {
            RendererStart::Cpu => {}
            RendererStart::GpuAtLaunch => self.spawn(waker, notes),
            RendererStart::GpuOnDemand => self.state = State::Deferred { slow: 0, waker },
        }
    }

    /// Whether the device waits for a reason to start (`GpuStart::OnDemand`).
    pub(crate) fn waits(&self) -> bool {
        matches!(self.state, State::Deferred { .. })
    }

    /// Start the device the window waited for, for `cause`. Nothing in any
    /// other state: a window on the CPU only never starts one.
    pub(crate) fn demand(&mut self, cause: GpuStartCause, notes: &mut Notes) {
        let State::Deferred { waker, .. } = &self.state else {
            return;
        };
        let waker = waker.clone();
        notes.note(RendererChange::GpuStarting { cause });
        self.spawn(waker, notes);
    }

    /// A CPU frame took `spent` of the `period` the display gives a frame:
    /// a device that waits starts after [`SLOW_FRAMES`] frames in a row over
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

    /// Create the device on a worker thread; frames are drawn on the CPU
    /// until it and the window's surface are ready. `waker` wakes the
    /// window's thread when a worker finishes. The CPU for the rest of the
    /// session when no worker can start.
    fn spawn(&mut self, waker: ShellWaker, notes: &mut Notes) {
        let wake: Wake = Arc::new(move || waker.wake(crate::WakeReasons::GPU_STEP));
        self.state = match start_device(self.env, &wake) {
            Some(worker) => State::Background(Background {
                wake,
                restarts: RESTARTS,
                step: Step::Starting(worker),
            }),
            None => cpu_session(notes, "the GPU worker thread cannot start"),
        };
    }

    /// Move a background start along for the window (`make` makes its
    /// surface, `physical` is its size): take the worker's device, make the
    /// window's surface and hand it to a worker to configure, take the
    /// configured surface. Nothing for any other state.
    pub(crate) fn advance(
        &mut self,
        notes: &mut Notes,
        make: MakeSurface<'_>,
        physical: (u32, u32),
    ) {
        let state = std::mem::replace(&mut self.state, State::Off);
        let State::Background(Background {
            wake,
            restarts,
            step,
        }) = state
        else {
            self.state = state;
            return;
        };
        let step = match poll_start(step, notes) {
            Ok(step) => step,
            Err(state) => {
                self.state = state;
                return;
            }
        };
        let step = match step {
            // The window's surface, made here, configured on a worker.
            Step::Waiting { context, init } => match make(&context) {
                Ok(surface) => match configure(&wake, context, init, surface, physical) {
                    Some(step) => step,
                    None => {
                        self.state = cpu_session(notes, "the GPU worker thread cannot start");
                        return;
                    }
                },
                Err(error) => {
                    let note = ready_note(&context, init, None);
                    self.state = State::Background(Background {
                        wake,
                        restarts,
                        step: Step::Ready(Device::new(context)),
                    });
                    self.unconfigured(&error, note, notes);
                    return;
                }
            },
            Step::Configuring {
                worker,
                init,
                surface,
            } => match worker.poll() {
                Polled::Working(worker) => Step::Configuring {
                    worker,
                    init,
                    surface,
                },
                // The platform destroyed the window's surface meanwhile:
                // whatever configuring did, the window's next frame makes a
                // new one (a device failure stays recorded in the context
                // and fails that frame).
                Polled::Sent((context, _)) if surface == ConfigureTarget::Destroyed => {
                    Step::Waiting { context, init }
                }
                Polled::Sent((context, Ok((surface, configure)))) => {
                    notes.note(ready_note(&context, init, Some(configure)));
                    Step::Ready(Device {
                        surface: DeviceSurface::Live(Box::new(surface)),
                        context,
                    })
                }
                Polled::Sent((context, Err(error))) => {
                    let note = ready_note(&context, init, None);
                    self.state = State::Background(Background {
                        wake,
                        restarts,
                        step: Step::Ready(Device::new(context)),
                    });
                    self.unconfigured(&error, note, notes);
                    return;
                }
                Polled::Stopped => {
                    self.state = cpu_session(notes, "GPU surface configuration stopped");
                    return;
                }
            },
            step @ (Step::Starting(_) | Step::Ready(_)) => step,
        };
        self.state = State::Background(Background {
            wake,
            restarts,
            step,
        });
    }

    /// The window's swapchain could not be made or configured (`error`;
    /// the device is ready without a surface now): classify it as a
    /// frame's failure would be, and note the GPU (`note`) when it keeps
    /// the device.
    fn unconfigured(&mut self, error: &RenderError, note: RendererChange, notes: &mut Notes) {
        if self.failed(error, notes) == Fallback::Surface {
            notes.note(note);
        }
    }

    /// Apply the failure `error` of a GPU step of the window (a frame, an
    /// offscreen render, an import, making or configuring its swapchain)
    /// and say what it means for the window's frame (see the module's
    /// failure policy).
    pub(crate) fn failed(&mut self, error: &RenderError, notes: &mut Notes) -> Fallback {
        match error {
            RenderError::Surface(_) => {
                self.drop_surface();
                Fallback::Surface
            }
            RenderError::Backend(_) | RenderError::NoAdapter(_) => {
                self.device_failed(error, notes);
                Fallback::Cpu
            }
            _ => Fallback::Other,
        }
    }

    pub(crate) fn fallback_to_cpu(&mut self, notes: &mut Notes, error: &dyn std::fmt::Display) {
        self.state = cpu_session(
            notes,
            format!("GPU surface keeps failing, CPU for the session: {error}"),
        );
    }

    /// The device failed: drop it, and start a new one once per session
    /// (background starts only).
    fn device_failed(&mut self, error: &RenderError, notes: &mut Notes) {
        let state = std::mem::replace(&mut self.state, State::Off);
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

    /// The platform destroyed the window's surface: drop its GPU surface; a
    /// swapchain a worker is configuring for it is dropped when the worker
    /// hands it back (the device is kept).
    pub(crate) fn drop_surface(&mut self) {
        match &mut self.state {
            State::Background(Background {
                step: Step::Ready(device),
                ..
            }) => device.surface = DeviceSurface::Pending,
            State::Background(Background {
                step: Step::Configuring { surface, .. },
                ..
            }) => *surface = ConfigureTarget::Destroyed,
            _ => {}
        }
    }

    /// Present to the window on the GPU, its surface made by `make` when it
    /// has none; `None` when the GPU is not ready (draw on the CPU).
    pub(crate) fn present(
        &mut self,
        make: MakeSurface<'_>,
        composition: &Composition,
        scale: f64,
        physical: (u32, u32),
    ) -> Option<Result<(Presentation, Duration, FrameReport), RenderError>> {
        let Device { surface, context } = self.device_mut()?;
        let surface = match window_surface(context, surface, make) {
            Ok(surface) => surface,
            Err(error) => return Some(Err(error)),
        };
        let presented = context.present(surface, composition, scale, physical);
        Some(presented)
    }

    /// Present `pixels` (0x00RRGGBB rows of the window's size, drawn on the
    /// CPU) through the window's GPU surface; `None` when the GPU is not
    /// ready.
    pub(crate) fn present_pixels(
        &mut self,
        make: MakeSurface<'_>,
        pixels: &[u32],
        physical: (u32, u32),
    ) -> Option<Result<(Presentation, Duration), RenderError>> {
        let Device { surface, context } = self.device_mut()?;
        let surface = match window_surface(context, surface, make) {
            Ok(surface) => surface,
            Err(error) => return Some(Err(error)),
        };
        let presented = context.present_pixels(surface, pixels, physical);
        Some(presented)
    }

    /// The adapter of the device drawing the window's frames, when it can
    /// import shared textures.
    pub(crate) fn adapter(&self) -> Option<kivact_types::GpuAdapter> {
        self.device()?.context.interop_adapter()
    }

    /// The context of the device drawing the window's frames, when ready.
    pub(crate) fn context(&mut self) -> Option<&mut GpuContext> {
        self.device_mut().map(|device| &mut *device.context)
    }

    /// Read back the last frame presented to the window.
    pub(crate) fn read_back(&self) -> Option<Result<CpuFrame, RenderError>> {
        let device = self.device()?;
        match &device.surface {
            DeviceSurface::Pending => None,
            DeviceSurface::Live(surface) => Some(device.context.read_back(surface)),
        }
    }
}

/// The window's surface, made by `make` when there is none.
fn window_surface<'a>(
    context: &GpuContext,
    surface: &'a mut DeviceSurface,
    make: MakeSurface<'_>,
) -> Result<&'a mut GpuSurface, RenderError> {
    if matches!(surface, DeviceSurface::Pending) {
        *surface = DeviceSurface::Live(Box::new(make(context)?));
    }
    match surface {
        DeviceSurface::Live(surface) => Ok(surface),
        DeviceSurface::Pending => unreachable!("invariant: the surface was just made"),
    }
}

/// A worker creating a device as `env` says; `None` when it cannot start.
fn start_device(env: GpuEnv, wake: &Wake) -> Option<Worker<Started>> {
    Worker::spawn("arvane-gpu-init", Arc::clone(wake), move || {
        let since = Instant::now();
        GpuContext::new(&env).map(|context| (Box::new(context), since.elapsed()))
    })
}

/// The step that configures `surface`'s swapchain for `physical` on a
/// worker; `None` when the worker cannot start.
fn configure(
    wake: &Wake,
    context: Box<GpuContext>,
    init: Duration,
    mut surface: GpuSurface,
    (width, height): (u32, u32),
) -> Option<Step> {
    let worker = Worker::spawn("arvane-gpu-configure", Arc::clone(wake), move || {
        let since = Instant::now();
        let configured = context.configure(&mut surface, width, height);
        let configured = configured.map(|()| (surface, since.elapsed()));
        (context, configured)
    })?;
    Some(Step::Configuring {
        worker,
        init,
        surface: ConfigureTarget::Current,
    })
}

/// The CPU for the rest of the session, noted with `reason`.
fn cpu_session(notes: &mut Notes, reason: impl Into<String>) -> State {
    notes.note(cpu(reason.into()));
    State::Failed
}

fn ready_note(context: &GpuContext, init: Duration, configure: Option<Duration>) -> RendererChange {
    RendererChange::Gpu {
        adapter: context.describe(),
        init,
        configure,
        interop: context.interop_adapter(),
    }
}

fn cpu(reason: String) -> RendererChange {
    RendererChange::Cpu { reason }
}

/// Poll device initialization before beginning surface configuration.
fn poll_start(step: Step, notes: &mut Notes) -> Result<Step, State> {
    Ok(match step {
        Step::Starting(worker) => match worker.poll() {
            Polled::Working(worker) => Step::Starting(worker),
            Polled::Sent(Ok((context, init))) => Step::Waiting { context, init },
            Polled::Sent(Err(error)) => {
                return Err(cpu_session(notes, format!("GPU unavailable: {error}")));
            }
            Polled::Stopped => {
                return Err(cpu_session(notes, "GPU initialization stopped"));
            }
        },
        step => step,
    })
}
