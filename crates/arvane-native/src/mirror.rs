//! Role: Real windows for tests (feature `test`).
//!
//! A test process gets one display: a UI thread started on first use that
//! owns the only winit event loop the process may ever create (winit
//! refuses a second one, even after the first is dropped). Tests keep their
//! host model, engine and renderer on their own thread and hand finished
//! frames and accessibility trees to their [`MirrorWindow`]; the UI thread
//! shows them in a real window that assistive technology (UI Automation)
//! can read.
//!
//! Invariants:
//! - Presentation is latest-wins: a window holds at most one pending frame,
//!   title and size, and at most one wake-up of the UI thread is in flight
//!   per window, so a busy UI thread coalesces frames instead of queueing
//!   them. A test changes its window's accessibility tree on its own thread
//!   ([`MirrorWindow::accessibility`]); the UI thread hands the platform the
//!   nodes changed since it last did, so changes coalesce too and none is
//!   lost.
//! - Windows open inactive (they never take the keyboard focus), are sized
//!   in physical pixels to the frames (independent of the monitor's scale)
//!   and are tiled over the primary monitor.
//! - A closed window returns to a pool and is reused by the next test,
//!   unless a GPU surface presented to it ([`MirrorWindow::gpu_presents`]):
//!   such a window shows the test's GPU frames instead of the frames
//!   handed to it, and is destroyed when closed.
//! - OS input and assistive technology's action requests on test windows
//!   are ignored: tests inject input into their host in-process.
//! - Where no window can exist (no display server, macOS, where winit needs
//!   the main thread) [`TestDisplay::global`] fails and callers render
//!   headless.
//!
//! The UI thread's event loop is built `with_any_thread` (Windows, X11,
//! Wayland), since it is not the process's main thread.
//!
//! Limits: a test binary that opens test windows cannot also make a
//! [`crate::Shell`] (nor run an application on one): winit allows one event
//! loop per process, and this display holds it.

use crate::a11y::native::NativeAccessibility;
use arvane_shell::{Accessibility, PlatformTree};
use arvane_shell::{ShellError, ShellWaker};
use kivact_types::Transform;
use kivact_types::render::{SemanticsNode, SemanticsTree};
use kivact_types::{Point, Rect, Role, SemanticsId};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, mpsc};
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

/// How long a test waits for the UI thread before giving up.
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a test waits for its window. One UI thread opens every
/// window in turn, so a test started alongside many others waits behind
/// their opens as well as its own; on a loaded machine that queue alone
/// can take longer than [`REPLY_TIMEOUT`].
const OPEN_TIMEOUT: Duration = Duration::from_secs(120);
/// Background of the part of a window no frame covers.
const BLANK: u32 = 0x0080_8080;

/// Options of [`TestDisplay::open`].
///
/// The default is an untitled 800 by 600 window at scale 1.
#[derive(Clone, Debug)]
pub struct MirrorWindowOptions {
    /// Window title.
    pub title: String,
    /// Content width in physical pixels (the frame width).
    pub width: u32,
    /// Content height in physical pixels (the frame height).
    pub height: u32,
    /// Physical pixels per logical pixel of the frames: accessibility
    /// bounds (logical) are mapped to the window's pixels with it.
    pub scale: f64,
}

impl Default for MirrorWindowOptions {
    fn default() -> Self {
        Self {
            title: String::new(),
            width: 800,
            height: 600,
            scale: 1.0,
        }
    }
}

/// Counters of a [`TestDisplay`] since the process started.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MirrorStats {
    /// Native windows created.
    pub created: u64,
    /// Pooled windows handed to a new test.
    pub reused: u64,
    /// Frames shown (after coalescing).
    pub presented: u64,
}

#[derive(Default)]
struct Counters {
    created: AtomicU64,
    reused: AtomicU64,
    presented: AtomicU64,
}

// xtask-allow(state-fields): Mailbox of independent pending window commands; optional payloads and notifications are consumed together without conditional presence invariants.
/// What a test window shows next; the UI thread takes it all at once.
#[derive(Default)]
struct MirrorBatch {
    frame: Option<Frame>,
    /// The accessibility tree changed: hand the platform what it lacks.
    accessibility: bool,
    title: Option<String>,
    resize: Option<(u32, u32, f64)>,
    /// A GPU surface presents to the window from now on.
    gpu: bool,
}

#[derive(Clone)]
struct Frame {
    width: u32,
    height: u32,
    pixels: Arc<[u32]>,
}

#[derive(Default)]
enum MirrorPending {
    #[default]
    Idle,
    Scheduled(MirrorBatch),
}
type Slot = Arc<Mutex<MirrorPending>>;

fn lock(slot: &Mutex<MirrorPending>) -> MutexGuard<'_, MirrorPending> {
    slot.lock()
        .expect("test display lock poisoned: a panic already happened")
}

enum Command {
    Open {
        id: u64,
        options: MirrorWindowOptions,
        slot: Slot,
        reply: mpsc::Sender<Result<PlatformTree, ShellError>>,
    },
    /// Run a closure with a test window's native window, on the UI thread
    /// (the only thread winit hands window handles out on).
    WithWindow {
        id: u64,
        run: Box<dyn FnOnce(arvane_shell::GpuTarget) + Send>,
    },
    Flush {
        id: u64,
    },
    Close {
        id: u64,
    },
    Sync {
        reply: mpsc::Sender<()>,
    },
}

/// The process-wide test display: the UI thread and its event loop.
pub struct TestDisplay {
    proxy: Mutex<EventLoopProxy<Command>>,
    next: AtomicU64,
    counters: Arc<Counters>,
}

#[expect(
    clippy::disallowed_types,
    reason = "process-wide native test display initialized once"
)]
static DISPLAY: std::sync::OnceLock<Result<TestDisplay, ShellError>> = std::sync::OnceLock::new();

impl TestDisplay {
    /// The display, its UI thread started on first use. Fails where no
    /// window can exist; the failure is remembered for the process.
    pub fn global() -> Result<&'static TestDisplay, ShellError> {
        DISPLAY
            .get_or_init(start)
            .as_ref()
            .map_err(ShellError::clone)
    }

    /// Open a window (a pooled one when available) without activating it.
    pub fn open(&'static self, options: MirrorWindowOptions) -> Result<MirrorWindow, ShellError> {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let slot = Slot::default();
        let (reply, answer) = mpsc::channel();
        self.send(Command::Open {
            id,
            options,
            slot: slot.clone(),
            reply,
        })?;
        let tree = answer
            .recv_timeout(OPEN_TIMEOUT)
            .map_err(|_| ShellError::new("the test UI thread did not open a window"))??;
        let changed = {
            let slot = slot.clone();
            move || schedule(self, id, &slot, |pending| pending.accessibility = true)
        };
        Ok(MirrorWindow {
            display: self,
            id,
            slot,
            accessibility: Accessibility::of_window(tree, changed),
        })
    }

    /// Counters since the process started.
    pub fn stats(&self) -> MirrorStats {
        MirrorStats {
            created: self.counters.created.load(Ordering::Relaxed),
            reused: self.counters.reused.load(Ordering::Relaxed),
            presented: self.counters.presented.load(Ordering::Relaxed),
        }
    }

    fn send(&self, command: Command) -> Result<(), ShellError> {
        self.proxy
            .lock()
            .expect("test display lock poisoned: a panic already happened")
            .send_event(command)
            .map_err(|_| ShellError::new("the test UI thread has stopped"))
    }
}

/// Start the UI thread and wait until its event loop exists.
fn start() -> Result<TestDisplay, ShellError> {
    let counters = Arc::new(Counters::default());
    let (ready, started) = mpsc::channel();
    let shared = counters.clone();
    std::thread::Builder::new()
        .name("arvane-test-display".into())
        .spawn(move || {
            let event_loop = match event_loop() {
                Ok(event_loop) => event_loop,
                Err(error) => {
                    let _ = ready.send(Err(error));
                    return;
                }
            };
            let _ = ready.send(Ok(event_loop.create_proxy()));
            let mut display = Display {
                windows: HashMap::new(),
                tests: HashMap::new(),
                idle: Vec::new(),
                placed: 0,
                counters: shared,
            };
            let _ = event_loop.run_app(&mut display);
        })
        .map_err(|e| ShellError::new(e.to_string()))?;
    let proxy = started
        .recv()
        .map_err(|_| ShellError::new("the test UI thread stopped while starting"))??;
    Ok(TestDisplay {
        proxy: Mutex::new(proxy),
        next: AtomicU64::new(1),
        counters,
    })
}

#[cfg(target_vendor = "apple")]
fn event_loop() -> Result<EventLoop<Command>, ShellError> {
    Err(ShellError::new(
        "test windows need winit's event loop off the main thread, which macOS forbids",
    ))
}

#[cfg(not(target_vendor = "apple"))]
fn event_loop() -> Result<EventLoop<Command>, ShellError> {
    let mut builder = EventLoop::<Command>::with_user_event();
    #[cfg(target_os = "windows")]
    winit::platform::windows::EventLoopBuilderExtWindows::with_any_thread(&mut builder, true);
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        winit::platform::x11::EventLoopBuilderExtX11::with_any_thread(&mut builder, true);
        winit::platform::wayland::EventLoopBuilderExtWayland::with_any_thread(&mut builder, true);
    }
    builder.build().map_err(|e| ShellError::new(e.to_string()))
}

/// A test's real window (`Send`); it returns to the pool when dropped.
pub struct MirrorWindow {
    display: &'static TestDisplay,
    id: u64,
    slot: Slot,
    accessibility: Accessibility,
}

impl MirrorWindow {
    /// Show a `width` × `height` frame of 0x00RRGGBB pixels. Asynchronous;
    /// a frame not shown yet is replaced by the next one.
    pub fn present(&self, width: u32, height: u32, pixels: Arc<[u32]>) {
        if pixels.len() != width as usize * height as usize {
            return;
        }
        self.schedule(|pending| {
            pending.frame = Some(Frame {
                width,
                height,
                pixels,
            })
        });
    }

    /// The window's accessibility tree: changes are validated here, on the
    /// calling thread, and handed to the window's native accessibility
    /// asynchronously.
    pub fn accessibility(&self) -> &Accessibility {
        &self.accessibility
    }

    /// Change the title.
    pub fn set_title(&self, title: &str) {
        let title = title.to_owned();
        self.schedule(|pending| pending.title = Some(title));
    }

    /// Resize the content to `width` × `height` physical pixels, mapping
    /// accessibility bounds with `scale` from now on.
    pub fn resize(&self, width: u32, height: u32, scale: f64) {
        self.schedule(|pending| pending.resize = Some((width, height, scale)));
    }

    /// Wait until the UI thread has shown everything handed to it before.
    pub fn sync(&self) {
        let (reply, done) = mpsc::channel();
        if self.display.send(Command::Sync { reply }).is_ok() {
            let _ = done.recv_timeout(REPLY_TIMEOUT);
        }
    }

    /// Run `run` with the native window as a GPU surface's target, on the
    /// test display's thread (the only thread winit hands window handles
    /// out on); asynchronously, and never for a closed window.
    pub fn with_gpu_target(
        &self,
        run: Box<dyn FnOnce(arvane_shell::GpuTarget) + Send>,
    ) -> Result<(), ShellError> {
        self.display.send(Command::WithWindow { id: self.id, run })
    }

    /// A GPU surface presents to the window from now on: the display stops
    /// drawing the frames handed to it.
    pub fn gpu_presents(&self) {
        self.schedule(|pending| pending.gpu = true);
    }

    fn schedule(&self, change: impl FnOnce(&mut MirrorBatch)) {
        schedule(self.display, self.id, &self.slot, change);
    }
}

/// Change what test window `id` shows next, queueing its flush unless one
/// is queued already.
fn schedule(display: &TestDisplay, id: u64, slot: &Slot, change: impl FnOnce(&mut MirrorBatch)) {
    let mut pending = lock(slot);
    let schedule = matches!(*pending, MirrorPending::Idle);
    if schedule {
        *pending = MirrorPending::Scheduled(MirrorBatch::default());
    }
    if let MirrorPending::Scheduled(batch) = &mut *pending {
        change(batch);
    }
    drop(pending);
    if schedule {
        let _ = display.send(Command::Flush { id });
    }
}

impl Drop for MirrorWindow {
    fn drop(&mut self) {
        let _ = self.display.send(Command::Close { id: self.id });
    }
}

/// What a native window of the display shows.
enum Use {
    /// In the pool, showing the idle tree.
    Pooled,
    /// A test window's, drawn here on the CPU; `frame` is the last frame it was given.
    Cpu { slot: Slot, frame: Option<Frame> },
    /// A test window's; a GPU surface presents to it, so its test's close destroys it.
    Gpu { slot: Slot },
}

/// A native window on the UI thread.
struct Entry {
    window: Arc<Window>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    accessibility: NativeAccessibility,
    usage: Use,
}

struct Display {
    windows: HashMap<WindowId, Entry>,
    /// Test window id → its native window.
    tests: HashMap<u64, WindowId>,
    idle: Vec<WindowId>,
    /// Windows placed so far (the next tile).
    placed: u32,
    counters: Arc<Counters>,
}

impl Display {
    fn open(
        &mut self,
        el: &ActiveEventLoop,
        id: u64,
        options: MirrorWindowOptions,
        slot: Slot,
    ) -> Result<PlatformTree, ShellError> {
        let size = PhysicalSize::new(options.width.max(1), options.height.max(1));
        let native = match self.idle.pop() {
            Some(native) => {
                let entry = self
                    .windows
                    .get_mut(&native)
                    .expect("invariant: pooled windows are in the table");
                entry.window.set_title(&options.title);
                if entry.window.inner_size() != size {
                    let _ = entry.window.request_inner_size(size);
                }
                entry.accessibility.rescale(options.scale);
                // The idle tree stays shown until the test's first tree.
                entry.accessibility.forget();
                self.counters.reused.fetch_add(1, Ordering::Relaxed);
                native
            }
            None => {
                let entry = self.create(el, &options, size)?;
                let native = entry.window.id();
                self.windows.insert(native, entry);
                self.counters.created.fetch_add(1, Ordering::Relaxed);
                native
            }
        };
        let entry = self
            .windows
            .get_mut(&native)
            .expect("invariant: the window was just placed in the table");
        entry.usage = Use::Cpu { slot, frame: None };
        self.tests.insert(id, native);
        Ok(entry.accessibility.platform_tree())
    }

    fn create(
        &mut self,
        el: &ActiveEventLoop,
        options: &MirrorWindowOptions,
        size: PhysicalSize<u32>,
    ) -> Result<Entry, ShellError> {
        let position = self.tile(el, size);
        let window = Arc::new(
            el.create_window(
                Window::default_attributes()
                    .with_visible(false)
                    .with_active(false)
                    .with_title(&options.title)
                    .with_position(position)
                    .with_inner_size(size),
            )
            .map_err(|e| ShellError::new(e.to_string()))?,
        );
        // The adapter must exist before the window is shown.
        let accessibility =
            NativeAccessibility::with_scale(el, &window, ShellWaker::new(|_| {}), options.scale);
        let context =
            softbuffer::Context::new(window.clone()).map_err(|e| ShellError::new(e.to_string()))?;
        let surface = softbuffer::Surface::new(&context, window.clone())
            .map_err(|e| ShellError::new(e.to_string()))?;
        window.set_visible(true);
        Ok(Entry {
            window,
            surface,
            accessibility,
            usage: Use::Pooled,
        })
    }

    /// The next free tile of the primary monitor for a window of `size`.
    fn tile(&mut self, el: &ActiveEventLoop, size: PhysicalSize<u32>) -> PhysicalPosition<i32> {
        const GAP: u32 = 24;
        let screen = el
            .primary_monitor()
            .map_or(PhysicalSize::new(1920, 1080), |m| m.size());
        let cell = (size.width + GAP, size.height + 2 * GAP);
        let columns = (screen.width.saturating_sub(GAP) / cell.0).max(1);
        let rows = (screen.height.saturating_sub(2 * GAP) / cell.1).max(1);
        let tile = self.placed % (columns * rows);
        self.placed = self.placed.wrapping_add(1);
        let (column, row) = (tile % columns, tile / columns);
        PhysicalPosition::new((GAP + column * cell.0) as i32, (GAP + row * cell.1) as i32)
    }

    fn flush(&mut self, id: u64) {
        let Some(entry) = self.tests.get(&id).and_then(|w| self.windows.get_mut(w)) else {
            return;
        };
        let (Use::Cpu { slot, .. } | Use::Gpu { slot }) = &entry.usage else {
            return;
        };
        let MirrorPending::Scheduled(pending) = std::mem::take(&mut *lock(slot)) else {
            return;
        };
        if let Some(title) = pending.title {
            entry.window.set_title(&title);
        }
        if let Some((width, height, scale)) = pending.resize {
            let size = PhysicalSize::new(width.max(1), height.max(1));
            if entry.window.inner_size() != size {
                let _ = entry.window.request_inner_size(size);
            }
            entry.accessibility.rescale(scale);
        }
        if pending.accessibility {
            entry.accessibility.publish();
        }
        if pending.gpu
            && let Use::Cpu { slot, .. } = &entry.usage
        {
            let slot = slot.clone();
            entry.usage = Use::Gpu { slot };
        }
        // Assistive technology's requests on test windows are not input.
        let _ = entry.accessibility.drain_actions();
        if let Some(frame) = pending.frame {
            if let Use::Cpu { frame: shown, .. } = &mut entry.usage {
                *shown = Some(frame);
            }
            blit(entry);
            self.counters.presented.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn close(&mut self, id: u64) {
        let Some(native) = self.tests.remove(&id) else {
            return;
        };
        let Some(entry) = self.windows.get_mut(&native) else {
            return;
        };
        if matches!(entry.usage, Use::Gpu { .. }) {
            // Dropping the last handle destroys the native window.
            self.windows.remove(&native);
            return;
        }
        entry.usage = Use::Pooled;
        entry.window.set_title("(idle test window)");
        let _ = entry.accessibility.replace(&idle_tree());
        let _ = entry.accessibility.drain_actions();
        blit(entry);
        self.idle.push(native);
    }

    fn command(&mut self, el: &ActiveEventLoop, command: Command) {
        match command {
            Command::Open {
                id,
                options,
                slot,
                reply,
            } => {
                let _ = reply.send(self.open(el, id, options, slot));
            }
            Command::Flush { id } => self.flush(id),
            Command::WithWindow { id, run } => {
                if let Some(entry) = self.tests.get(&id).and_then(|w| self.windows.get(w)) {
                    run(entry.window.clone());
                }
            }
            Command::Close { id } => self.close(id),
            Command::Sync { reply } => {
                let _ = reply.send(());
            }
        }
    }
}

/// The tree of a pooled window: a window with nothing in it.
fn idle_tree() -> SemanticsTree {
    let root = SemanticsId(0);
    SemanticsTree {
        root,
        focus: None,
        nodes: vec![SemanticsNode {
            id: root,
            children: Vec::new(),
            bounds: Some(Rect::default()),
            scroll: Point::default(),
            transform: Transform::IDENTITY,
            semantics: kivact_types::Semantics {
                role: Some(Role::Generic),
                name: Some("idle test window".into()),
                description: Some(String::new()),
                value_text: None,
                disabled: false,
                hidden: false,
                focusable: false,
                checked: None,
                selected: Some(false),
                expanded: None,
                test_id: None,
                actions: Some(Vec::new()),
                ..Default::default()
            },
        }],
    }
}

/// Copy the window's frame to its surface; the part the frame does not
/// cover (a window larger than the frame) is blank.
fn blit(entry: &mut Entry) {
    let frame = match &entry.usage {
        Use::Gpu { .. } => return,
        Use::Cpu { frame, .. } => frame.as_ref(),
        Use::Pooled => None,
    };
    let size = entry.window.inner_size();
    let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
    else {
        return;
    };
    if entry.surface.resize(width, height).is_err() {
        return;
    }
    let Ok(mut buffer) = entry.surface.buffer_mut() else {
        return;
    };
    let (w, h) = (size.width as usize, size.height as usize);
    match frame {
        Some(frame) if (frame.width as usize, frame.height as usize) == (w, h) => {
            buffer.copy_from_slice(&frame.pixels);
        }
        Some(frame) => {
            buffer.fill(BLANK);
            let (fw, fh) = (frame.width as usize, frame.height as usize);
            let n = w.min(fw);
            for y in 0..h.min(fh) {
                buffer[y * w..y * w + n].copy_from_slice(&frame.pixels[y * fw..y * fw + n]);
            }
        }
        None => buffer.fill(BLANK),
    }
    let _ = buffer.present();
}

impl ApplicationHandler<Command> for Display {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        el.set_control_flow(ControlFlow::Wait);
    }

    fn user_event(&mut self, el: &ActiveEventLoop, command: Command) {
        // A display bug ends the test UI thread: winit resumes the panic after
        // its loop, and later `send` calls fail with "the test UI thread has
        // stopped".
        self.command(el, command);
    }

    fn window_event(&mut self, _: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(entry) = self.windows.get_mut(&id) else {
            return;
        };
        entry.accessibility.process_event(&entry.window, &event);
        if matches!(
            event,
            WindowEvent::RedrawRequested | WindowEvent::Resized(_)
        ) {
            blit(entry);
        }
    }
}
