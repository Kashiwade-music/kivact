//! Role: A presenter's GPU renderer and its life cycle (feature `gpu`).
//!
//! The renderer of the presenter's one window ([`State`]): off (CPU only);
//! started in the background (a native window): a worker thread creates
//! the device, the window's thread makes the window's surface and a worker
//! configures its swapchain (~0.2 s the first time on some drivers), then
//! frames go to the window's GPU surface; started in the foreground
//! ([`Gpu::enable`], a test window): the device is made on the calling
//! thread and is ready at once, and the window's first frame makes its
//! surface; failed: CPU for the rest of the session. Until ready, frames
//! are drawn on the CPU, so no step stalls the window's thread.
//!
//! A native window starts the GPU as it is created, not once CPU frames
//! grow costly. The worker's device is taken on the window's thread
//! (winit hands window handles out only there), which makes the window's
//! surface; the first swapchain is configured on a worker too. Each
//! finished step wakes the shell, which moves the start along before the
//! host sees the wake; once the GPU can draw the window, a frame is asked
//! for and drawn on the GPU at once, so the switch does not wait for
//! input. While the swapchain is configured the window keeps its last CPU
//! frame and new frames are not shown (once a flip-model swapchain
//! exists, a frame presented through GDI may not show) but drawn again
//! later. The host never knows these steps. A test window that draws on
//! the GPU creates its device on its own thread as it opens
//! ([`Gpu::enable`]).
//!
//! Renderer changes are noted ([`Notes`]) as they happen. A surface the
//! platform destroys (Android, in the background) is dropped with it; the
//! window's next frame makes a new one.
//!
//! Failure policy ([`Gpu::failed`], with `arvane-render`'s surface
//! policy): a surface error, from a frame or from making or configuring
//! the swapchain, drops the window's surface and the next frame makes a
//! new one; [`SURFACE_FAILURES`] of them in a row on one device move the
//! window to the CPU for the rest of the session. A device failure (device
//! loss, out of memory, a validation error) drops the device and its
//! surface; a background start then starts a new device once per session,
//! the frames in between drawn on the CPU. A second device failure, a
//! failed restart or a foreground device's failure leaves the presenter on
//! the CPU.

#[cfg(feature = "testing")]
use crate::error::{self, Failure};
use crate::presentation::{Notes, RendererChange};
use arvane_render::{GpuContext, GpuEnv, GpuSurface, Presentation, RenderError};
use kivact_types::Size;
#[cfg(feature = "testing")]
use kivact_types::render::CpuFrame;
use kivact_types::render::RenderScene;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Device restarts allowed per session (background starts only).
const RESTARTS: u8 = 1;

/// Bound DLL shutdown without waiting forever for a graphics driver.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);

/// GPU surface failures in a row, on one device, after which the window is
/// drawn on the CPU.
pub(crate) const SURFACE_FAILURES: u8 = 2;

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
    /// `None` once joined (by `poll` or `shutdown`, which then drop it).
    run: Option<Running<T>>,
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
            run: Some(Running { result, thread }),
        })
    }
}

impl<T> Worker<T> {
    /// The worker's result, once it sent one (the thread is joined then).
    fn poll(mut self) -> Polled<T> {
        let received = self
            .run
            .as_ref()
            .expect("invariant: only `poll` and `shutdown` join a worker, and both consume it")
            .result
            .try_recv();
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
        if let Some(Running { thread, .. }) = self.run.take()
            && let Err(payload) = thread.join()
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(payload);
        }
    }

    /// Wait (bounded, the window kept responsive) for the thread to end and
    /// join it; a thread still running is left to `Drop`, which detaches it.
    fn shutdown(mut self, window: &mut dyn arvane_shell::ShellWindow) {
        let finished = self.run.as_ref().is_none_or(|running| {
            window.wait_until(&mut || running.thread.is_finished(), SHUTDOWN_WAIT)
        });
        if finished {
            // Queued surfaces and devices are dropped here, on the UI thread.
            self.join();
        }
    }
}

impl<T> Drop for Worker<T> {
    fn drop(&mut self) {
        let Some(Running { result, thread }) = self.run.take() else {
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

/// A ready device and the window's surface on it.
struct Device {
    /// The window's surface; the window's next GPU frame makes it when `None`.
    /// Boxed: a surface is large.
    surface: Option<Box<GpuSurface>>,
    /// The window's surface failures in a row on this device.
    failures: u8,
    /// Boxed: a context is large.
    context: Box<GpuContext>,
}

impl Device {
    fn new(context: Box<GpuContext>) -> Self {
        Self {
            surface: None,
            failures: 0,
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
    /// A worker configures the window's swapchain; `stale` once the
    /// platform destroyed the window's surface meanwhile.
    Configuring {
        worker: Worker<Configured>,
        init: Duration,
        stale: bool,
    },
    /// Frames go to the GPU.
    Ready(Device),
}

/// The GPU renderer's life cycle (see the module documentation).
enum State {
    /// Not started: frames are drawn on the CPU.
    Off,
    /// Started on worker threads.
    Background(Background),
    /// Made on the window's thread ([`Gpu::enable`]): ready at once, never
    /// restarted.
    #[cfg_attr(
        not(feature = "testing"),
        expect(
            dead_code,
            reason = "only a test window makes the GPU in the foreground"
        )
    )]
    Foreground(Device),
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
            State::Foreground(device)
            | State::Background(Background {
                step: Step::Ready(device),
                ..
            }) => device.context.release_caches(),
            State::Background(Background {
                step: Step::Waiting { context, .. },
                ..
            }) => context.release_caches(),
            // A device still on a worker has drawn no window frame yet.
            State::Off | State::Failed | State::Background(_) => {}
        }
    }

    fn device(&self) -> Option<&Device> {
        match &self.state {
            State::Foreground(device)
            | State::Background(Background {
                step: Step::Ready(device),
                ..
            }) => Some(device),
            _ => None,
        }
    }

    fn device_mut(&mut self) -> Option<&mut Device> {
        match &mut self.state {
            State::Foreground(device)
            | State::Background(Background {
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

    /// Create the device on a worker thread; frames are drawn on the CPU
    /// until it and the window's surface are ready. `wake` is called when
    /// a worker finishes.
    pub(crate) fn start(&mut self, wake: Wake, notes: &mut Notes) {
        if !matches!(self.state, State::Off) {
            return;
        }
        self.state = match start_device(self.env, &wake) {
            Some(worker) => State::Background(Background {
                wake,
                restarts: RESTARTS,
                step: Step::Starting(worker),
            }),
            None => cpu_session(notes, "the GPU worker thread cannot start"),
        };
    }

    /// Create the device now, on the calling thread (test windows,
    /// offscreen frames); the window's first frame makes its surface.
    #[cfg(feature = "testing")]
    pub(crate) fn enable(&mut self, notes: &mut Notes) -> Result<(), Failure> {
        assert!(
            matches!(self.state, State::Off),
            "invariant: a test window enables the GPU once, on a new presenter"
        );
        let since = Instant::now();
        let context = Box::new(GpuContext::new(&self.env).map_err(error::render)?);
        notes.note(ready_note(&context, since.elapsed(), None));
        self.state = State::Foreground(Device::new(context));
        Ok(())
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
        // The device the worker made.
        let step = match step {
            Step::Starting(worker) => match worker.poll() {
                Polled::Working(worker) => Step::Starting(worker),
                Polled::Sent(Ok((context, init))) => Step::Waiting { context, init },
                Polled::Sent(Err(error)) => {
                    self.state = cpu_session(notes, format!("GPU unavailable: {error}"));
                    return;
                }
                Polled::Stopped => {
                    self.state = cpu_session(notes, "GPU initialization stopped");
                    return;
                }
            },
            step => step,
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
                stale,
            } => match worker.poll() {
                Polled::Working(worker) => Step::Configuring {
                    worker,
                    init,
                    stale,
                },
                // The platform destroyed the window's surface meanwhile:
                // whatever configuring did, the window's next frame makes a
                // new one (a device failure stays recorded in the context
                // and fails that frame).
                Polled::Sent((context, _)) if stale => Step::Waiting { context, init },
                Polled::Sent((context, Ok((surface, configure)))) => {
                    notes.note(ready_note(&context, init, Some(configure)));
                    Step::Ready(Device {
                        surface: Some(Box::new(surface)),
                        failures: 0,
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
            RenderError::Surface(_) => self.surface_failed(error, notes),
            RenderError::Backend(_) | RenderError::NoAdapter(_) => {
                self.device_failed(error, notes);
                Fallback::Cpu
            }
            _ => Fallback::Other,
        }
    }

    fn surface_failed(&mut self, error: &RenderError, notes: &mut Notes) -> Fallback {
        let device = self
            .device_mut()
            .expect("invariant: a surface fails only on a ready device");
        device.surface = None;
        device.failures = device.failures.saturating_add(1);
        if device.failures < SURFACE_FAILURES {
            return Fallback::Surface;
        }
        self.state = cpu_session(
            notes,
            format!("GPU surface keeps failing, CPU for the session: {error}"),
        );
        Fallback::Cpu
    }

    /// The device failed: drop it, and start a new one once per session
    /// (background starts only).
    fn device_failed(&mut self, error: &RenderError, notes: &mut Notes) {
        let state = std::mem::replace(&mut self.state, State::Off);
        self.state = match state {
            State::Background(Background { wake, restarts, .. }) if restarts > 0 => {
                notes.note(cpu(format!("GPU device failed, restarting: {error}")));
                match start_device(self.env, &wake) {
                    Some(worker) => State::Background(Background {
                        wake,
                        restarts: restarts - 1,
                        step: Step::Starting(worker),
                    }),
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
            State::Foreground(device)
            | State::Background(Background {
                step: Step::Ready(device),
                ..
            }) => device.surface = None,
            State::Background(Background {
                step: Step::Configuring { stale, .. },
                ..
            }) => *stale = true,
            _ => {}
        }
    }

    /// Present to the window on the GPU, its surface made by `make` when it
    /// has none; `None` when the GPU is not ready (draw on the CPU).
    pub(crate) fn present(
        &mut self,
        make: MakeSurface<'_>,
        scene: &RenderScene,
        size: Size,
        scale: f64,
        physical: (u32, u32),
    ) -> Option<Result<(Presentation, Duration), RenderError>> {
        let Device {
            surface,
            failures,
            context,
        } = self.device_mut()?;
        let surface = match window_surface(context, surface, make) {
            Ok(surface) => surface,
            Err(error) => return Some(Err(error)),
        };
        let presented = context.present(surface, scene, size, scale, physical);
        if presented.is_ok() {
            *failures = 0;
        }
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
        let Device {
            surface,
            failures,
            context,
        } = self.device_mut()?;
        let surface = match window_surface(context, surface, make) {
            Ok(surface) => surface,
            Err(error) => return Some(Err(error)),
        };
        let presented = context.present_pixels(surface, pixels, physical);
        if presented.is_ok() {
            *failures = 0;
        }
        Some(presented)
    }

    /// The adapter of the device drawing the window's frames, when it can
    /// import shared textures.
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn adapter(&self) -> Option<kivact_types::GpuAdapter> {
        self.device()?.context.interop_adapter()
    }

    /// The context of the device drawing the window's frames, when ready.
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn context(&mut self) -> Option<&mut GpuContext> {
        self.device_mut().map(|device| &mut *device.context)
    }

    /// The external frames the last frames did not draw (each once).
    pub(crate) fn take_missed(&mut self) -> Vec<kivact_types::ExternalFrame> {
        self.device_mut()
            .map_or_else(Vec::new, |device| device.context.take_missed_external())
    }

    /// Render offscreen and read back; `None` when the GPU is not ready.
    #[cfg(feature = "testing")]
    pub(crate) fn render(
        &mut self,
        scene: &RenderScene,
        size: Size,
        scale: f64,
    ) -> Option<Result<CpuFrame, RenderError>> {
        Some(self.device_mut()?.context.render(scene, size, scale))
    }

    /// Read back the last frame presented to the window.
    #[cfg(feature = "testing")]
    pub(crate) fn read_back(&self) -> Option<Result<CpuFrame, RenderError>> {
        let device = self.device()?;
        Some(device.context.read_back(device.surface.as_ref()?))
    }
}

/// The window's surface, made by `make` when there is none.
fn window_surface<'a>(
    context: &GpuContext,
    surface: &'a mut Option<Box<GpuSurface>>,
    make: MakeSurface<'_>,
) -> Result<&'a mut GpuSurface, RenderError> {
    if surface.is_none() {
        *surface = Some(Box::new(make(context)?));
    }
    Ok(surface
        .as_mut()
        .expect("invariant: the surface was just made"))
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
        stale: false,
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
