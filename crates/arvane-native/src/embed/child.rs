//! Role: Share embedded child commands and implement the host handle contract.
//!
//! Invariants: snapshots describe applied values, never queued requests. Host
//! sizes advance a generation so stale accepted negotiations cannot override
//! corrections. Backend wake only schedules work and never invokes App code.
//! Stopping cancels new App work immediately. A panic of the App is kept in
//! the child's `PanicSlot` and stops the child like a failure; the host's next
//! call resumes it. A panic is sticky: resuming its payload does not restart
//! the child. The native window is `Gone` when it is destroyed (Win32
//! `WM_DESTROY`) or the X11 UI thread ended. Cleanup of the App may still be
//! deferred after that while an App callback is on the stack.
use arvane_shell::{PanicSlot, ShellChild, ShellError};
use std::any::Any;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

pub(super) enum Command {
    SetSize {
        width: u32,
        height: u32,
        generation: Option<u64>,
    },
    SetScale(f32),
}
pub(super) struct Shared {
    inner: Mutex<Inner>,
    panic: PanicSlot,
}
struct Inner {
    state: ChildState,
    applied: Applied,
    /// Advanced by every host size (`queue_host_size`); negotiated sizes carry the generation they
    /// were accepted at.
    generation: u64,
    commands: VecDeque<Command>,
}
/// The physical size and scale the child applied last, read together.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Applied {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) scale: f32,
}
enum ChildState {
    /// The App runs.
    Running,
    /// No more App work starts.
    Stopped { cause: Stop, window: NativeWindow },
}
enum Stop {
    /// Closed by the host's drop, the backend or the native window's destruction.
    Closed,
    /// The first failure.
    Failed(ShellError),
    /// The App panicked (the payload is in the `PanicSlot`); sticky even after the host resumed it.
    /// `failure` is a failure that came first.
    Panicked { failure: Option<ShellError> },
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum NativeWindow {
    /// The native window and its UI loop still exist.
    Live,
    /// Win32: the window was destroyed (`WM_DESTROY`); X11: the UI thread ended.
    Gone,
}
impl Shared {
    pub(super) fn new(width: u32, height: u32, scale: f32, panic: PanicSlot) -> Self {
        Self {
            inner: Mutex::new(Inner {
                state: ChildState::Running,
                applied: Applied {
                    width,
                    height,
                    scale,
                },
                generation: 0,
                commands: VecDeque::new(),
            }),
            panic,
        }
    }
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .expect("embedded child lock poisoned: a panic already happened")
    }
    /// Keep the payload of a panic that reached a platform boundary; the
    /// child stops and the host resumes it (see [`PanicSlot`]).
    pub(super) fn keep_panic(&self, payload: Box<dyn Any + Send>) {
        self.panic.put(payload);
        let mut inner = self.lock();
        inner.state = match std::mem::replace(&mut inner.state, ChildState::Running) {
            ChildState::Running => ChildState::Stopped {
                cause: Stop::Panicked { failure: None },
                window: NativeWindow::Live,
            },
            ChildState::Stopped { cause, window } => ChildState::Stopped {
                cause: match cause {
                    Stop::Closed => Stop::Panicked { failure: None },
                    Stop::Failed(error) => Stop::Panicked {
                        failure: Some(error),
                    },
                    panicked @ Stop::Panicked { .. } => panicked,
                },
                window,
            },
        };
    }
    pub(super) fn panicked(&self) -> bool {
        matches!(
            self.lock().state,
            ChildState::Stopped {
                cause: Stop::Panicked { .. },
                ..
            }
        )
    }
    pub(super) fn request_close(&self) {
        let mut inner = self.lock();
        if matches!(inner.state, ChildState::Running) {
            inner.state = ChildState::Stopped {
                cause: Stop::Closed,
                window: NativeWindow::Live,
            };
        }
    }
    /// The native window is gone (Win32 `WM_DESTROY`; X11: the UI thread ended).
    pub(super) fn window_gone(&self) {
        let mut inner = self.lock();
        inner.state = match std::mem::replace(&mut inner.state, ChildState::Running) {
            ChildState::Running => ChildState::Stopped {
                cause: Stop::Closed,
                window: NativeWindow::Gone,
            },
            ChildState::Stopped { cause, .. } => ChildState::Stopped {
                cause,
                window: NativeWindow::Gone,
            },
        };
    }
    /// Record a failure and stop; the first failure wins.
    pub(super) fn fail(&self, error: ShellError) {
        let mut inner = self.lock();
        inner.state = match std::mem::replace(&mut inner.state, ChildState::Running) {
            ChildState::Running => ChildState::Stopped {
                cause: Stop::Failed(error),
                window: NativeWindow::Live,
            },
            ChildState::Stopped {
                cause: Stop::Closed,
                window,
            } => ChildState::Stopped {
                cause: Stop::Failed(error),
                window,
            },
            ChildState::Stopped {
                cause: Stop::Panicked { failure: None },
                window,
            } => ChildState::Stopped {
                cause: Stop::Panicked {
                    failure: Some(error),
                },
                window,
            },
            stopped @ ChildState::Stopped { .. } => stopped,
        };
    }
    pub(super) fn failure(&self) -> Option<ShellError> {
        match &self.lock().state {
            ChildState::Stopped {
                cause:
                    Stop::Failed(error)
                    | Stop::Panicked {
                        failure: Some(error),
                    },
                ..
            } => Some(error.clone()),
            _ => None,
        }
    }
    pub(super) fn stopped(&self) -> bool {
        !matches!(self.lock().state, ChildState::Running)
    }
    /// Whether the native window and its UI loop still exist.
    pub(super) fn alive(&self) -> bool {
        matches!(
            self.lock().state,
            ChildState::Running
                | ChildState::Stopped {
                    window: NativeWindow::Live,
                    ..
                }
        )
    }
    /// Queues a host change for the UI thread. Unnegotiated sizes and scales
    /// coalesce so that the newest host value wins; negotiated sizes keep their
    /// generation check and are never dropped here.
    pub(super) fn queue(&self, command: Command) {
        let mut inner = self.lock();
        if matches!(inner.state, ChildState::Running) {
            inner.push(command);
        }
    }
    /// Queues a size the host chose: it advances the generation, so earlier negotiated sizes go
    /// stale.
    pub(super) fn queue_host_size(&self, width: u32, height: u32) {
        let mut inner = self.lock();
        if matches!(inner.state, ChildState::Running) {
            inner.generation += 1;
            inner.push(Command::SetSize {
                width,
                height,
                generation: None,
            });
        }
    }
    /// The next queued command (the Win32 UI loop re-checks its state between commands).
    #[cfg(windows)]
    pub(super) fn pop_command(&self) -> Option<Command> {
        self.lock().commands.pop_front()
    }
    /// Drain every queued command at once (the X11 UI loop).
    #[cfg(target_os = "linux")]
    pub(super) fn take_commands(&self) -> Vec<Command> {
        self.lock().commands.drain(..).collect()
    }
    pub(super) fn current(&self, generation: Option<u64>) -> bool {
        generation.is_none_or(|generation| generation == self.generation())
    }
    pub(super) fn generation(&self) -> u64 {
        self.lock().generation
    }
    /// The applied size and scale, read together.
    pub(super) fn applied(&self) -> Applied {
        self.lock().applied
    }
    pub(super) fn size(&self) -> (u32, u32) {
        let applied = self.applied();
        (applied.width, applied.height)
    }
    pub(super) fn scale(&self) -> f32 {
        self.applied().scale
    }
    /// Record the applied size; whether it changed.
    #[cfg(windows)]
    pub(super) fn set_size(&self, width: u32, height: u32) -> bool {
        let mut inner = self.lock();
        let changed = (inner.applied.width, inner.applied.height) != (width, height);
        inner.applied.width = width;
        inner.applied.height = height;
        changed
    }
    #[cfg(windows)]
    pub(super) fn set_scale(&self, scale: f32) {
        self.lock().applied.scale = scale;
    }
    /// Record the applied size and scale together (the X11 UI loop).
    #[cfg(target_os = "linux")]
    pub(super) fn update(&self, width: u32, height: u32, scale: f32) {
        self.lock().applied = Applied {
            width,
            height,
            scale,
        };
    }
}
impl Inner {
    fn push(&mut self, command: Command) {
        match &command {
            Command::SetSize {
                generation: None, ..
            } => self.commands.retain(|queued| {
                !matches!(
                    queued,
                    Command::SetSize {
                        generation: None,
                        ..
                    }
                )
            }),
            Command::SetScale(_) => {
                self.commands
                    .retain(|queued| !matches!(queued, Command::SetScale(_)));
            }
            Command::SetSize { .. } => {}
        }
        self.commands.push_back(command);
    }
}
/// OS side of an embedded child; `Handle` implements `ShellChild` once on top.
///
/// - `MAX_DIMENSION`: the largest physical dimension the native window
///   accepts; setters ignore larger values with a warning.
/// - `wake`: schedule the App UI thread to drain the host command queue. Callable
///   from any thread; never runs App code and never blocks.
/// - `close`: called once from `Handle::drop`, after `Shared::request_close`.
///   Off the UI thread, wait for cleanup at most `SHUTDOWN_WAIT`, then detach
///   the thread and retain the module. On the UI thread (inside a callback),
///   return at once, let the outer loop finish cleanup and retain the module.
pub(super) trait Backend: Send + Sync + 'static {
    const MAX_DIMENSION: u32;
    fn wake(&self);
    fn close(&self, shared: &Shared);
    /// Move the panic of a UI thread that ended into `shared` (a backend
    /// whose App runs on its own thread; the Win32 backend has none).
    fn collect_panic(&self, shared: &Shared);
}
pub(super) struct Handle<B: Backend> {
    pub(super) shared: Arc<Shared>,
    pub(super) backend: B,
}
impl<B: Backend> ShellChild for Handle<B> {
    fn set_size(&self, width: u32, height: u32) {
        self.backend.collect_panic(&self.shared);
        if self.shared.stopped() {
            return;
        }
        if width == 0 || height == 0 || width > B::MAX_DIMENSION || height > B::MAX_DIMENSION {
            tracing::warn!(target: "kivact", width, height, "ignored embedded child size outside native dimensions");
            return;
        }
        self.shared.queue_host_size(width, height);
        self.backend.wake();
    }
    fn size(&self) -> (u32, u32) {
        self.backend.collect_panic(&self.shared);
        self.shared.size()
    }
    fn set_scale(&self, scale: f32) {
        self.backend.collect_panic(&self.shared);
        if self.shared.stopped() {
            return;
        }
        if !scale.is_finite() || scale <= 0.0 {
            tracing::warn!(target: "kivact", scale, "ignored embedded child scale that is not positive and finite");
            return;
        }
        self.shared.queue(Command::SetScale(scale));
        self.backend.wake();
    }
    fn scale(&self) -> f32 {
        self.backend.collect_panic(&self.shared);
        self.shared.scale()
    }
    fn failure(&self) -> Option<ShellError> {
        self.backend.collect_panic(&self.shared);
        self.shared.failure()
    }
}
impl<B: Backend> Drop for Handle<B> {
    fn drop(&mut self) {
        self.shared.request_close();
        self.backend.close(&self.shared);
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for host command coalescing and stopped-queue refusal.
    use super::*;

    fn size(width: u32, generation: Option<u64>) -> Command {
        Command::SetSize {
            width,
            height: 10,
            generation,
        }
    }
    /// Sizes as `(width, generation)`; a scale `s` is listed as `(1000 + s, None)`.
    fn queued(shared: &Shared) -> Vec<(u32, Option<u64>)> {
        let inner = shared.lock();
        inner
            .commands
            .iter()
            .map(|command| match command {
                Command::SetSize {
                    width, generation, ..
                } => (*width, *generation),
                Command::SetScale(scale) => (1000 + *scale as u32, None),
            })
            .collect()
    }

    #[test]
    fn newest_unnegotiated_size_wins_and_negotiated_sizes_stay() {
        let shared = Shared::new(1, 1, 1.0, PanicSlot::default());
        shared.queue(size(1, None));
        shared.queue(size(2, Some(7)));
        shared.queue(Command::SetScale(2.0));
        shared.queue(size(3, None));
        shared.queue(size(4, Some(8)));
        assert_eq!(
            queued(&shared),
            [(2, Some(7)), (1002, None), (3, None), (4, Some(8))]
        );
    }

    #[test]
    fn newest_scale_wins_and_keeps_order_of_the_rest() {
        let shared = Shared::new(1, 1, 1.0, PanicSlot::default());
        shared.queue(Command::SetScale(1.0));
        shared.queue(size(5, None));
        shared.queue(Command::SetScale(3.0));
        assert_eq!(queued(&shared), [(5, None), (1003, None)]);
    }

    #[test]
    fn stopped_queue_refuses_commands() {
        let shared = Shared::new(1, 1, 1.0, PanicSlot::default());
        shared.queue(size(1, None));
        shared.request_close();
        shared.queue(size(2, None));
        shared.queue(Command::SetScale(2.0));
        assert_eq!(queued(&shared), [(1, None)]);

        let failed = Shared::new(1, 1, 1.0, PanicSlot::default());
        assert!(failed.failure().is_none());
        failed.fail(ShellError::new("first"));
        failed.fail(ShellError::new("second"));
        assert!(failed.failure().is_some() && failed.stopped());
        failed.queue(size(1, None));
        assert!(queued(&failed).is_empty());
    }
}
