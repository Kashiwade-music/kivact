//! Role: A presenter's GPU renderer and its life cycle (feature `gpu`).
//!
//! States: off (CPU only) → starting (a worker thread creates the device)
//! → configuring (the window's thread made the window's surface; a worker
//! configures its swapchain, which takes ~0.2 s the first time on some
//! drivers) → ready (frames go to the windows' GPU surfaces) → failed (CPU
//! for the rest of the session). Until ready, frames are drawn on the CPU,
//! so neither step stalls the window's thread. A device created on the
//! calling thread ([`Gpu::enable`]) is ready at once and configures
//! surfaces with their first frame.
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
//! Failure policy (with `arvane-render`'s surface policy): a surface that
//! stays lost is dropped and made again by the next frame; a window whose
//! surface fails [`SURFACE_FAILURES`] frames in a row (it cannot be made,
//! the adapter cannot present to it, no usable format, lost again after
//! reconfiguring) is drawn on the CPU for the rest of the session; a device
//! failure (device loss, out of memory, a validation error) drops the
//! device and every surface and starts a new device once per session, the
//! frames in between drawn on the CPU; a second device failure, or a
//! failed restart, leaves the presenter on the CPU.

#[cfg(feature = "testing")]
use crate::error::{self, Error};
use crate::presentation::{Notes, RendererChange};
use arvane_render::{GpuContext, GpuEnv, GpuSurface, Presentation, RenderError};
#[cfg(feature = "testing")]
use kivact_types::CpuFrame;
use kivact_types::{Scene, Size};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Device restarts allowed per session.
const RESTARTS: u8 = 1;

/// Bound DLL shutdown without waiting forever for a graphics driver.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);

struct Worker<T> {
    result: Option<mpsc::Receiver<T>>,
    thread: Option<JoinHandle<()>>,
    name: &'static str,
}

impl<T> Worker<T> {
    fn poll(&mut self) -> Result<T, mpsc::TryRecvError> {
        let result = self
            .result
            .as_ref()
            .expect("invariant: active worker has a result channel")
            .try_recv();
        if !matches!(result, Err(mpsc::TryRecvError::Empty)) {
            self.join();
        }
        result
    }

    fn join(&mut self) {
        if let Some(thread) = self.thread.take()
            && let Err(payload) = thread.join()
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(payload);
        }
    }

    fn shutdown(&mut self, window: &mut dyn arvane_shell::ShellWindow) {
        if let Some(thread) = self.thread.as_ref()
            && window.wait_until(&mut || thread.is_finished(), SHUTDOWN_WAIT)
        {
            self.join();
            // Queued surfaces/devices are dropped here, on the UI thread.
            drop(self.result.take());
        }
    }
}

impl<T> Drop for Worker<T> {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            if thread.is_finished() {
                let _ = thread.join();
                return;
            }
            crate::shutdown::DETACHED.store(true, std::sync::atomic::Ordering::Release);
            tracing::warn!(target: "kivact", worker = self.name,
                "GPU worker still running when its window closed; its thread is detached");
            // Keep a late result alive rather than destroying its native surface
            // on the worker. A timed-out driver is an explicitly documented leak.
            if let Some(result) = self.result.take() {
                std::mem::forget(result);
            }
        }
    }
}

/// GPU surface failures in a row after which a window is drawn on the CPU.
pub(crate) const SURFACE_FAILURES: u8 = 2;

/// A device and how long the worker took to create it.
type Started = Result<(GpuContext, Duration), RenderError>;
/// A device, the window's configured surface, and how long configuring took.
type Configured = (GpuContext, Result<(GpuSurface, Duration), RenderError>);

enum State {
    Off,
    Starting(Worker<Started>),
    /// The device is ready; no window presented since.
    Waiting {
        context: GpuContext,
        init: Duration,
    },
    Configuring {
        worker: Worker<Configured>,
        key: u64,
        init: Duration,
    },
    Ready(GpuContext),
    Failed,
}

/// Makes a window's surface on the window's thread.
pub(crate) type MakeSurface<'a> = &'a dyn Fn(&GpuContext) -> Result<GpuSurface, RenderError>;

/// The GPU renderer of an engine and its windows' surfaces.
pub(crate) struct Gpu {
    /// How devices are made.
    env: GpuEnv,
    state: State,
    /// Surfaces by window key.
    surfaces: Vec<(u64, GpuSurface)>,
    /// The device was started in the background: a failed device is
    /// restarted the same way.
    background: bool,
    /// Wakes the window's thread when a background step finishes.
    wake: Option<Wake>,
    restarts: u8,
    /// GPU surface failures in a row, by window key.
    surface_failures: Vec<(u64, u8)>,
    /// Windows drawn on the CPU after their surfaces kept failing.
    cpu_windows: Vec<u64>,
    /// The window whose surface the platform destroyed while a worker
    /// configured it.
    stale: Option<u64>,
    clear_metadata: bool,
}

/// Wakes the presenting thread (any thread may call it).
pub(crate) type Wake = std::sync::Arc<dyn Fn() + Send + Sync>;

impl Gpu {
    pub(crate) fn shutdown(&mut self, window: &mut dyn arvane_shell::ShellWindow) {
        let mut state = std::mem::replace(&mut self.state, State::Off);
        match &mut state {
            State::Starting(worker) => worker.shutdown(window),
            State::Configuring { worker, .. } => worker.shutdown(window),
            _ => {}
        }
        self.surfaces.clear();
        self.wake = None;
        drop(state);
    }

    pub(crate) fn release_caches(&mut self) {
        self.clear_metadata = true;
        self.clear_ready_metadata();
    }

    fn clear_ready_metadata(&mut self) {
        if self.clear_metadata {
            match &mut self.state {
                State::Ready(context) | State::Waiting { context, .. } => {
                    context.release_caches();
                    self.clear_metadata = false;
                }
                _ => {}
            }
        }
    }

    pub(crate) fn new(env: GpuEnv) -> Self {
        Self {
            env,
            state: State::Off,
            surfaces: Vec::new(),
            background: false,
            wake: None,
            restarts: RESTARTS,
            surface_failures: Vec::new(),
            cpu_windows: Vec::new(),
            stale: None,
            clear_metadata: false,
        }
    }

    /// Whether frames go to the GPU now.
    pub(crate) fn ready(&self) -> bool {
        matches!(self.state, State::Ready(_))
    }

    /// Whether a worker is configuring the window `key`'s swapchain.
    pub(crate) fn configuring(&self, key: u64) -> bool {
        matches!(self.state, State::Configuring { key: k, .. } if k == key)
    }

    /// Create the device on a worker thread; frames are drawn on the CPU
    /// until it and the presenting window's surface are ready.
    /// `wake` is called when the device or a surface is ready.
    pub(crate) fn start(&mut self, wake: Wake) {
        if !matches!(self.state, State::Off) {
            return;
        }
        self.background = true;
        self.wake = Some(wake);
        self.spawn();
    }

    fn spawn(&mut self) {
        let (sender, result) = mpsc::sync_channel(1);
        let wake = self.wake.clone();
        let env = self.env;
        let spawned = std::thread::Builder::new()
            .name("arvane-gpu-init".into())
            .spawn(move || {
                let since = Instant::now();
                let _ = sender.send(GpuContext::new(&env).map(|c| (c, since.elapsed())));
                if let Some(wake) = wake {
                    wake();
                }
            });
        self.state = match spawned {
            Ok(thread) => State::Starting(Worker {
                result: Some(result),
                thread: Some(thread),
                name: "arvane-gpu-init",
            }),
            Err(_) => State::Failed,
        };
    }

    /// Create the device now, on the calling thread (tests, offscreen
    /// frames).
    #[cfg(feature = "testing")]
    pub(crate) fn enable(&mut self, notes: &mut Notes) -> Result<(), Error> {
        if self.ready() {
            return Ok(());
        }
        let since = Instant::now();
        let context = GpuContext::new(&self.env).map_err(error::render)?;
        notes.note(ready_note(&context, since.elapsed(), None));
        self.surfaces.clear();
        self.state = State::Ready(context);
        Ok(())
    }

    /// Move a background start along: take the worker's device and, for
    /// the window `window` (its key, surface maker and physical size),
    /// hand its surface to a worker to configure; take the configured
    /// surface.
    pub(crate) fn advance(
        &mut self,
        notes: &mut Notes,
        window: Option<(u64, MakeSurface<'_>, (u32, u32))>,
    ) {
        if let State::Starting(worker) = &mut self.state {
            match worker.poll() {
                Err(mpsc::TryRecvError::Empty) => return,
                Ok(Ok((context, init))) => self.state = State::Waiting { context, init },
                Ok(Err(error)) => {
                    notes.note(cpu(format!("GPU unavailable: {error}")));
                    return self.fail();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    notes.note(cpu("GPU initialization stopped".into()));
                    return self.fail();
                }
            }
        }
        match std::mem::replace(&mut self.state, State::Off) {
            State::Waiting { context, init } => match window {
                Some((key, make, (width, height))) if !self.cpu_windows.contains(&key) => {
                    self.configure(context, key, make, width, height, init, notes);
                }
                // No window is presenting (on the GPU): keep the device
                // until one is.
                _ => self.state = State::Waiting { context, init },
            },
            State::Configuring {
                mut worker,
                key,
                init,
            } => match worker.poll() {
                Err(mpsc::TryRecvError::Empty) => {
                    self.state = State::Configuring { worker, key, init };
                }
                // The window's surface was destroyed while it was being
                // configured: its next frame makes a new one.
                Ok((context, Ok(_))) if self.stale.take() == Some(key) => {
                    self.state = State::Waiting { context, init };
                }
                Ok((context, Ok((surface, configure)))) => {
                    notes.note(ready_note(&context, init, Some(configure)));
                    self.surfaces = vec![(key, surface)];
                    self.state = State::Ready(context);
                }
                // The configuration ended without a surface: a destroyed
                // surface noted meanwhile has nothing left to drop (kept, it
                // would drop the window's next configured surface).
                Ok((context, Err(error))) => {
                    self.stale = None;
                    self.state = State::Ready(context);
                    self.device_failed(&error, notes);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.stale = None;
                    notes.note(cpu("GPU surface configuration stopped".into()));
                    self.fail();
                }
            },
            state => self.state = state,
        }
        self.clear_ready_metadata();
    }

    /// Make the window's surface here and configure it on a worker.
    #[expect(clippy::too_many_arguments, reason = "one hand-off step")]
    fn configure(
        &mut self,
        context: GpuContext,
        key: u64,
        make: MakeSurface<'_>,
        width: u32,
        height: u32,
        init: Duration,
        notes: &mut Notes,
    ) {
        let mut surface = match make(&context) {
            Ok(surface) => surface,
            Err(error) => {
                // The window cannot have a GPU surface: CPU frames.
                notes.note(cpu(format!("GPU cannot present: {error}")));
                self.fail();
                return;
            }
        };
        let (sender, result) = mpsc::sync_channel(1);
        let wake = self.wake.clone();
        let spawned = std::thread::Builder::new()
            .name("arvane-gpu-configure".into())
            .spawn(move || {
                let since = Instant::now();
                let configured = context.configure(&mut surface, width, height);
                let configured = configured.map(|()| (surface, since.elapsed()));
                let _ = sender.send((context, configured));
                if let Some(wake) = wake {
                    wake();
                }
            });
        self.state = match spawned {
            Ok(thread) => State::Configuring {
                worker: Worker {
                    result: Some(result),
                    thread: Some(thread),
                    name: "arvane-gpu-configure",
                },
                key,
                init,
            },
            Err(_) => State::Failed,
        };
    }

    fn fail(&mut self) {
        self.surfaces.clear();
        self.state = State::Failed;
    }

    /// The platform destroyed the window `key`'s surface: drop its GPU
    /// surface; a swapchain a worker is configuring for it is dropped
    /// when the worker hands it back (the device is kept).
    pub(crate) fn drop_surface(&mut self, key: u64) {
        self.surfaces.retain(|(k, _)| *k != key);
        if self.configuring(key) {
            self.stale = Some(key);
        }
    }

    /// Present to the window `key` on the GPU, its surface made by `make`
    /// the first time; `None` when the GPU is not ready or the window is
    /// drawn on the CPU (draw on the CPU).
    pub(crate) fn present(
        &mut self,
        key: u64,
        make: MakeSurface<'_>,
        scene: &Scene,
        size: Size,
        scale: f32,
        physical: (u32, u32),
    ) -> Option<Result<(Presentation, Duration), RenderError>> {
        let State::Ready(context) = &mut self.state else {
            return None;
        };
        if self.cpu_windows.contains(&key) {
            return None;
        }
        let index = match self.surfaces.iter().position(|(k, _)| *k == key) {
            Some(index) => index,
            None => match make(context) {
                Ok(surface) => {
                    self.surfaces.push((key, surface));
                    self.surfaces.len() - 1
                }
                Err(error) => return Some(Err(error)),
            },
        };
        let surface = &mut self.surfaces[index].1;
        let presented = context.present(surface, scene, size, scale, physical);
        if presented.is_ok() {
            self.surface_failures.retain(|(k, _)| *k != key);
        }
        Some(presented)
    }

    /// Present `pixels` (0x00RRGGBB rows of the window's size, drawn on
    /// the CPU) through the window `key`'s GPU surface; `None` when the GPU
    /// does not present that window.
    pub(crate) fn present_pixels(
        &mut self,
        key: u64,
        make: MakeSurface<'_>,
        pixels: &[u32],
        physical: (u32, u32),
    ) -> Option<Result<(Presentation, Duration), RenderError>> {
        let State::Ready(context) = &mut self.state else {
            return None;
        };
        if self.cpu_windows.contains(&key) {
            return None;
        }
        let index = match self.surfaces.iter().position(|(k, _)| *k == key) {
            Some(index) => index,
            None => match make(context) {
                Ok(surface) => {
                    self.surfaces.push((key, surface));
                    self.surfaces.len() - 1
                }
                Err(error) => return Some(Err(error)),
            },
        };
        Some(context.present_pixels(&mut self.surfaces[index].1, pixels, physical))
    }

    /// The adapter of the device drawing the window `key`'s frames, when
    /// it can import shared textures.
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn adapter(&self, key: u64) -> Option<kivact_types::GpuAdapter> {
        match &self.state {
            State::Ready(context) if !self.cpu_windows.contains(&key) => context.interop_adapter(),
            _ => None,
        }
    }

    /// The device's context, when it is ready (whatever window draws on
    /// the CPU).
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn ready_context(&mut self) -> Option<&mut GpuContext> {
        match &mut self.state {
            State::Ready(context) => Some(context),
            _ => None,
        }
    }

    /// The context drawing the window `key`'s frames, when the GPU is
    /// ready and the window is not drawn on the CPU.
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn context(&mut self, key: u64) -> Option<&mut GpuContext> {
        match &mut self.state {
            State::Ready(context) if !self.cpu_windows.contains(&key) => Some(context),
            _ => None,
        }
    }

    /// The external frames the last frames did not draw (each once).
    pub(crate) fn take_missed(&mut self) -> Vec<kivact_types::ExternalFrame> {
        match &mut self.state {
            State::Ready(context) => context.take_missed_external(),
            _ => Vec::new(),
        }
    }

    /// Render offscreen and read back; `None` when the GPU is not ready.
    #[cfg(feature = "testing")]
    pub(crate) fn render(
        &mut self,
        scene: &Scene,
        size: Size,
        scale: f32,
    ) -> Option<Result<CpuFrame, RenderError>> {
        match &mut self.state {
            State::Ready(context) => Some(context.render(scene, size, scale)),
            _ => None,
        }
    }

    /// Read back the last frame presented to the window `key`.
    #[cfg(feature = "testing")]
    pub(crate) fn read_back(&self, key: u64) -> Option<Result<CpuFrame, RenderError>> {
        let State::Ready(context) = &self.state else {
            return None;
        };
        let (_, surface) = self.surfaces.iter().find(|(k, _)| *k == key)?;
        Some(context.read_back(surface))
    }

    /// The window `key`'s surface failed (`error`): drop it, the next
    /// frame makes a new one. Returns whether the window is drawn on the
    /// CPU from now on (it failed [`SURFACE_FAILURES`] frames in a row).
    pub(crate) fn surface_failed(
        &mut self,
        key: u64,
        error: &RenderError,
        notes: &mut Notes,
    ) -> bool {
        self.surfaces.retain(|(k, _)| *k != key);
        let count = match self.surface_failures.iter_mut().find(|(k, _)| *k == key) {
            Some((_, count)) => {
                *count = count.saturating_add(1);
                *count
            }
            None => {
                self.surface_failures.push((key, 1));
                1
            }
        };
        if count < SURFACE_FAILURES {
            return false;
        }
        self.surface_failures.retain(|(k, _)| *k != key);
        self.cpu_windows.push(key);
        notes.note(cpu(format!(
            "GPU surface keeps failing, CPU for this window: {error}"
        )));
        true
    }

    /// The device failed: drop it, and start a new one once per session.
    pub(crate) fn device_failed(&mut self, error: &RenderError, notes: &mut Notes) {
        self.surfaces.clear();
        if self.background && self.restarts > 0 {
            self.restarts -= 1;
            notes.note(cpu(format!("GPU device failed, restarting: {error}")));
            self.spawn();
        } else {
            notes.note(cpu(format!(
                "GPU device failed, CPU for the session: {error}"
            )));
            self.fail();
        }
    }
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
