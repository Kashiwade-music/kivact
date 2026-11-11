//! Role: Main-thread event loop and window life cycle.
//!
//! The window outlives its surface; every callback runs on the creator thread.
//! The CPU surface (softbuffer) is created by the first CPU presentation, so
//! a window the GPU presents to never has one.
//!
//! What the runner adds to winit's events:
//! - Life cycle: winit's `suspended`, `resumed` and `memory_warning` are
//!   [`ShellEvent::Suspended`], [`ShellEvent::Resumed`] and
//!   [`ShellEvent::MemoryWarning`]. On Android a suspension destroys the
//!   window's surface: the CPU surface is dropped and
//!   [`ShellEvent::SurfaceDestroyed`] follows; the resumption gives the
//!   next surface generation ([`ShellEvent::SurfaceCreated`]). Desktops
//!   resume once, at start (the window is created then), and keep their
//!   first surface.
//! - Buttons and wheels: winit reports them without a position, so when
//!   the message's own position (Windows' subclass) differs from the last
//!   move, a mouse `Move` to it is delivered first. Every mouse event
//!   carries the last position and the held buttons
//!   ([`arvane_shell::Mouse`]); every press and release is counted
//!   ([`arvane_shell::ClickCounter`]).
//! - Wheels: a notch scrolls the system's lines (characters with Shift)
//!   of 20 logical px each, Shift turning a vertical notch sideways; a
//!   "page" setting (more than 100 lines) scrolls the window's height.
//!   Pixel deltas pass as they are, in logical px.
//! - The environment ([`crate::port::winit::window::WinitWindow::environment`]): a theme change, a
//!   move to a monitor of another identity (`Moved` or a scale change; the
//!   refresh period is read again first), a reduced motion, mouse settings
//!   or locale change (`WM_SETTINGCHANGE`) and a locale changed while the
//!   window had no focus (read again at `Focused(true)`) each arrive as one
//!   [`ShellEvent::EnvironmentChanged`]. winit tells of no locale change.
//! - Contacts: winit's touches become [`ShellEvent::Pointer`]s
//!   (`contacts.rs`); losing the focus cancels every contact in progress
//!   and, with a mouse button held, the mouse's gesture (one `Cancel`).
//!
//! Sizes, minimum sizes and IME areas are `kivact-types` logical sizes and
//! rectangles. A panic in a callback propagates out of `run`: winit carries it
//! across its window procedure (Win32, macOS) and resumes it after the loop,
//! and the X11 and Wayland dispatch frames are all Rust.
//!
//! Limits:
//! - A precision touchpad scrolls by winit's wheel lines: no pixel deltas
//!   and no inertia (DirectManipulation).
//! - Logoff and shutdown (`WM_QUERYENDSESSION`, `WM_ENDSESSION`) are not
//!   received, so no `CloseRequested` arrives then.
//! - winit 0.30 reports no safe area on any platform (it stays zero), nor
//!   the area a software keyboard covers.

use crate::port::winit::open::create_window;
use crate::port::winit::shell_loop::NativeLoop;
use crate::port::winit::window::Surface;
use crate::port::winit::window_events;
use crate::port::winit::windows::{NativeWindow, WindowTable};
use crate::present::CpuSurface;
use arvane_shell::pipeline::RawDrop;
use arvane_shell::{
    Backend, EventTarget, InjectedInput, ShellControl, ShellError, ShellEvent, ShellHandler,
    ShellWaker, ShellWindow, WindowInput,
};
use kivact_types::{RequestId, WindowId, WindowRequest};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId as WinitWindowId;

/// Whether the platform destroys a window's surface when it suspends the
/// application (Android), so the window has none until it resumes.
const SUSPENSION_DESTROYS_SURFACES: bool = cfg!(target_os = "android");

/// Create the native window backend.
pub fn backend(main: WindowRequest) -> Result<Box<dyn Backend>, ShellError> {
    NativeBackend::new(main).map(|value| Box::new(value) as Box<dyn Backend>)
}
/// Main-thread window loop. Construct and run on the application's main thread.
struct NativeBackend {
    event_loop: EventLoop<()>,
    main: WindowRequest,
}
impl NativeBackend {
    /// Create the native event loop, rejecting invalid geometry and OS failures.
    fn new(main: WindowRequest) -> Result<Self, ShellError> {
        if !main.size.width.is_finite()
            || !main.size.height.is_finite()
            || main.size.width <= 0.0
            || main.size.height <= 0.0
        {
            return Err(ShellError::new(
                "initial window size must be positive and finite",
            ));
        }
        let event_loop = EventLoop::with_user_event()
            .build()
            .map_err(|e| ShellError::new(e.to_string()))?;
        Ok(Self { event_loop, main })
    }
}
/// Where the runner is in the loop's life; the window table lives in the
/// phases that own windows.
enum LoopPhase {
    /// `resumed` has not created the main window yet.
    Starting,
    /// The handler receives the windows' events.
    Running(WindowTable),
    /// The handler asked to exit (`failure` None) or failed: it gets no more
    /// events; `close` follows the loop.
    Stopped {
        table: WindowTable,
        failure: Option<ShellError>,
    },
    /// The main window could not be created: no handler started.
    Unopened(ShellError),
}
struct Runner<'a> {
    handler: &'a mut dyn ShellHandler,
    main: WindowRequest,
    phase: LoopPhase,
    started: Instant,
    waker: ShellWaker,
    // xtask-allow(state-fields): independent: bits OR-ed in by the ShellWaker on any thread and swapped to zero by `user_event`; wakes arrive in every LoopPhase (before the main window, while running, after a stop) and are drained in each, so every pair with phase occurs
    /// Wake reasons accumulated until the event loop receives the notification.
    wake_reasons: Arc<AtomicU8>,
}
impl Runner<'_> {
    /// Stop delivering events: the loop ends after this callback and `close` follows it.
    fn stop(&mut self, failure: Option<ShellError>) {
        self.phase = match std::mem::replace(&mut self.phase, LoopPhase::Starting) {
            LoopPhase::Running(table) => LoopPhase::Stopped { table, failure },
            other => other,
        };
    }

    /// The open windows, ascending by id.
    fn ids(&self) -> Vec<WindowId> {
        match &self.phase {
            LoopPhase::Running(table) => table.ids(),
            _ => Vec::new(),
        }
    }

    /// Run `f` on an open window.
    fn with_window<R>(
        &mut self,
        id: WindowId,
        f: impl FnOnce(&mut NativeWindow) -> R,
    ) -> Option<R> {
        match &mut self.phase {
            LoopPhase::Running(table) => table.get_mut(id).map(f),
            _ => None,
        }
    }

    /// Whether the window is still open.
    fn has_window(&self, id: WindowId) -> bool {
        match &self.phase {
            LoopPhase::Running(table) => table.get(id).is_some(),
            _ => false,
        }
    }

    /// Deliver one callback to the handler; `None` is its `start`. An event for
    /// a window that an earlier callback closed is dropped: no event follows
    /// `close_window`.
    fn dispatch(&mut self, el: &ActiveEventLoop, target: EventTarget, event: Option<ShellEvent>) {
        let result = {
            let LoopPhase::Running(table) = &mut self.phase else {
                return;
            };
            if let EventTarget::Window(id) = target
                && table.get(id).is_none()
            {
                return;
            }
            let mut lp = NativeLoop {
                el,
                table,
                started: self.started,
                waker: &self.waker,
            };
            let result = match event {
                Some(event) => self.handler.event(target, event, &mut lp),
                None => self.handler.start(&mut lp, self.waker.clone()),
            };
            for id in lp.table.take_pending_show() {
                if let Some(native) = lp.table.get(id) {
                    native.window.window.set_visible(true);
                }
            }
            result
        };
        match result {
            Ok(ShellControl::Exit) => {
                self.stop(None);
                el.exit();
            }
            Ok(ShellControl::Wait) => el.set_control_flow(ControlFlow::Wait),
            Ok(ShellControl::WakeAt(ns)) => {
                match self.started.checked_add(Duration::from_nanos(ns)) {
                    Some(deadline) => el.set_control_flow(ControlFlow::WaitUntil(deadline)),
                    None => {
                        self.stop(Some(ShellError::new(
                            "shell wake deadline exceeds the platform clock range",
                        )));
                        el.exit();
                    }
                }
            }
            Err(error) => {
                self.stop(Some(error));
                el.exit();
            }
        }
    }

    /// Deliver one callback and then the input queued during it.
    fn callback(&mut self, el: &ActiveEventLoop, target: EventTarget, event: Option<ShellEvent>) {
        self.dispatch(el, target, event);
        self.drain_injected(el);
    }

    /// Deliver injected input queued during the callback that just returned.
    /// It goes through the window's `WindowInput` like platform input and is
    /// followed by `ShellEvent::Injected` once all of it was delivered.
    fn drain_injected(&mut self, el: &ActiveEventLoop) {
        loop {
            let Some((id, inputs, request)) = self.next_injected() else {
                return;
            };
            let mut closed = false;
            for input in inputs {
                let Some(events) = self.injected_events(id, input) else {
                    closed = true;
                    break;
                };
                for event in events {
                    self.dispatch(el, EventTarget::Window(id), Some(event));
                }
                if !self.has_window(id) {
                    closed = true;
                    break;
                }
            }
            if closed || !self.has_window(id) {
                continue;
            }
            self.dispatch(
                el,
                EventTarget::Window(id),
                Some(ShellEvent::Injected { request }),
            );
        }
    }

    /// The first injected batch of the lowest open window id.
    fn next_injected(&mut self) -> Option<(WindowId, Vec<InjectedInput>, RequestId)> {
        match &mut self.phase {
            LoopPhase::Running(table) => table.next_injected(),
            _ => None,
        }
    }

    /// The events one injected input makes, or `None` when its window closed.
    fn injected_events(&mut self, id: WindowId, input: InjectedInput) -> Option<Vec<ShellEvent>> {
        let LoopPhase::Running(table) = &mut self.phase else {
            return None;
        };
        let native = table.get_mut(id)?;
        let now = native.window.now_ns();
        let double_click = native.window.double_click();
        let events = match input {
            InjectedInput::Accessibility { node, request } => native
                .window
                .accessibility
                .request_action(node, request)
                .into_iter()
                .collect(),
            input => {
                let mut ime = native.window.input.ime.borrow_mut();
                native.input.normalize(input, now, double_click, &mut ime)
            }
        };
        Some(events)
    }
}
impl ApplicationHandler<()> for Runner<'_> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        match &self.phase {
            LoopPhase::Running(_) => {
                for id in self.ids() {
                    let created = self
                        .with_window(id, |native| {
                            if let Surface::Destroyed { last } = &native.window.surface {
                                native.window.surface = Surface::Live {
                                    generation: last.saturating_add(1),
                                    cpu: CpuSurface::Released,
                                };
                                true
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false);
                    if created {
                        self.callback(
                            el,
                            EventTarget::Window(id),
                            Some(ShellEvent::SurfaceCreated),
                        );
                    }
                }
                self.callback(el, EventTarget::Loop, Some(ShellEvent::Resumed));
            }
            LoopPhase::Starting => self.start_loop(el),
            LoopPhase::Stopped { .. } | LoopPhase::Unopened(_) => {}
        }
    }

    fn suspended(&mut self, el: &ActiveEventLoop) {
        self.callback(el, EventTarget::Loop, Some(ShellEvent::Suspended));
        // Android destroys the native window's surface with the suspension
        // (winit asks applications to drop theirs); iOS keeps it.
        if SUSPENSION_DESTROYS_SURFACES {
            for id in self.ids() {
                let destroyed = self
                    .with_window(id, |native| {
                        if let Surface::Live { generation, .. } = &native.window.surface {
                            native.window.surface = Surface::Destroyed { last: *generation };
                            true
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false);
                if destroyed {
                    self.callback(
                        el,
                        EventTarget::Window(id),
                        Some(ShellEvent::SurfaceDestroyed),
                    );
                }
            }
        }
    }

    fn memory_warning(&mut self, el: &ActiveEventLoop) {
        self.callback(el, EventTarget::Loop, Some(ShellEvent::MemoryWarning));
    }

    fn user_event(&mut self, el: &ActiveEventLoop, _: ()) {
        let reasons = crate::wake::decode(self.wake_reasons.swap(0, Ordering::AcqRel));
        if reasons.is_empty() {
            return;
        }
        // A file drag delivers its held position and, after every dropped
        // file, the drop itself.
        for id in self.ids() {
            let events = self
                .with_window(id, |native| {
                    let Some(position) = native.window.input.file_pointer.take() else {
                        return Vec::new();
                    };
                    let now = native.window.now_ns();
                    let double_click = native.window.double_click();
                    native
                        .input
                        .dnd_mut()
                        .input(RawDrop::Move(position))
                        .into_iter()
                        .map(|event| native.input.platform(event, now, double_click))
                        .collect()
                })
                .unwrap_or_default();
            for event in events {
                self.callback(el, EventTarget::Window(id), Some(event));
            }
        }
        // A wake for the environment (the Linux settings portal) carries the new
        // preference into every window before the change is announced.
        for id in self.ids() {
            let events = self
                .with_window(id, |native| {
                    if reasons.contains(arvane_shell::WakeReasons::ENVIRONMENT) {
                        native
                            .window
                            .input
                            .reduced_motion
                            .set(crate::native_input::reduced_motion());
                    }
                    let changed = native.window.input.env_changed.take().is_some();
                    if changed || reasons.contains(arvane_shell::WakeReasons::ENVIRONMENT) {
                        vec![ShellEvent::EnvironmentChanged]
                    } else {
                        Vec::new()
                    }
                })
                .unwrap_or_default();
            for event in events {
                self.callback(el, EventTarget::Window(id), Some(event));
            }
        }
        for id in self.ids() {
            let actions = self
                .with_window(id, |native| native.window.accessibility.drain_actions())
                .unwrap_or_default();
            for action in actions {
                self.callback(el, EventTarget::Window(id), Some(action));
            }
        }
        self.callback(el, EventTarget::Loop, Some(ShellEvent::Wake(reasons)));
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        for id in self.ids() {
            let events = self
                .with_window(id, window_events::deliver_drop)
                .unwrap_or_default();
            for event in events {
                self.callback(el, EventTarget::Window(id), Some(event));
            }
        }
    }

    fn new_events(&mut self, el: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::ResumeTimeReached { .. }) {
            self.callback(
                el,
                EventTarget::Loop,
                Some(ShellEvent::Wake(arvane_shell::WakeReasons::CLIENT)),
            );
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, id: WinitWindowId, event: WindowEvent) {
        let Some(window) = (match &self.phase {
            LoopPhase::Running(table) => table.by_winit(id),
            _ => None,
        }) else {
            return;
        };
        let events = self
            .with_window(window, |native| window_events::window_events(native, event))
            .unwrap_or_default();
        for event in events {
            self.callback(el, EventTarget::Window(window), Some(event));
        }
    }

    fn exiting(&mut self, el: &ActiveEventLoop) {
        let table = match &mut self.phase {
            LoopPhase::Running(table) => table,
            LoopPhase::Stopped { table, .. } => table,
            LoopPhase::Starting | LoopPhase::Unopened(_) => return,
        };
        let mut lp = NativeLoop {
            el,
            table,
            started: self.started,
            waker: &self.waker,
        };
        self.handler.close(&mut lp);
    }
}
impl Runner<'_> {
    /// Create the main window, start the handler and run the loop.
    fn start_loop(&mut self, el: &ActiveEventLoop) {
        let mut table = WindowTable::new(crate::display::placement_supported(el));
        match create_window(el, &self.main, None, &self.waker, self.started) {
            Ok(window) => {
                let winit_id = window.window.id();
                table.insert(
                    self.main.id,
                    winit_id,
                    NativeWindow {
                        window,
                        input: WindowInput::new(),
                        injected: VecDeque::new(),
                        #[cfg(windows)]
                        modal_parent: None,
                    },
                );
                self.phase = LoopPhase::Running(table);
                self.callback(el, EventTarget::Loop, None);
                self.with_window(self.main.id, |native| {
                    native.window.window.set_visible(true);
                });
            }
            Err(error) => {
                self.phase = LoopPhase::Unopened(error);
                el.exit();
            }
        }
    }
}

impl Backend for NativeBackend {
    fn run(self: Box<Self>, handler: &mut dyn ShellHandler) -> Result<(), ShellError> {
        let proxy = self.event_loop.create_proxy();
        let wake_reasons = Arc::new(AtomicU8::new(0));
        let pending = wake_reasons.clone();
        let waker = ShellWaker::new(move |reasons| {
            let bits = crate::wake::encode(reasons);
            if bits != 0 && pending.fetch_or(bits, Ordering::AcqRel) == 0 {
                let _ = proxy.send_event(());
            }
        });
        #[cfg(target_os = "linux")]
        let _watch = crate::port::linux_settings::watch(waker.clone());
        let main = self.main.clone();
        let mut app = Runner {
            handler,
            main,
            phase: LoopPhase::Starting,
            started: Instant::now(),
            waker,
            wake_reasons,
        };
        let result = self.event_loop.run_app(&mut app);
        let Runner { phase, .. } = app;
        let failure = match phase {
            LoopPhase::Starting | LoopPhase::Running(_) => None,
            LoopPhase::Stopped { table, failure } => {
                drop(table);
                failure
            }
            LoopPhase::Unopened(error) => Some(error),
        };
        result.map_err(|e| ShellError::new(e.to_string()))?;
        failure.map_or(Ok(()), Err)
    }
}
