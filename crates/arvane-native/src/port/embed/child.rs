//! Role: Share embedded child commands and implement the host handle contract.
//!
//! Invariants: snapshots describe applied values, never queued requests. Host
//! sizes advance a generation so stale accepted negotiations cannot override
//! corrections. ChildPort wake only schedules work and never invokes App code.
//! Stopping cancels new App work immediately. A panic of the App keeps its
//! message in the child's `PanicSlot` and stops the child like a failure; the
//! host reads it as `ChildStop::Panicked` and Kivact never resumes it. A
//! panic is sticky. The native window is `Gone` when it is destroyed (Win32
//! `WM_DESTROY`) or the X11 UI thread ended. Cleanup of the App may still be
//! deferred after that while an App callback is on the stack.
use arvane_shell::{ChildStop, NoticeSink, PanicSlot, ShellChild, ShellError};
use kivact_types::embed::Visibility;
use std::any::Any;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

#[allow(
    clippy::enum_variant_names,
    reason = "Every variant names the host operation it applies (size, scale, visibility); the shared verb keeps the queue readable."
)]
pub(in crate::port) enum Command {
    SetSize {
        width: u32,
        height: u32,
        generation: Option<u64>,
    },
    SetScale(f64),
    SetVisibility(Visibility),
}
pub(in crate::port) struct Shared {
    inner: Mutex<Inner>,
    panic: PanicSlot,
    /// Delivers the host's notices; the callback runs without a lock held.
    notices: NoticeSink,
}
struct Inner {
    state: ChildStatus,
    applied: Applied,
    /// The last size the host chose, with the generation that advanced for it.
    host: HostSize,
    commands: VecDeque<Command>,
}
/// The last host size and the identity that advances with it.
#[derive(Clone, Copy)]
struct HostSize {
    // xtask-allow(state-fields): identity: advanced by every host size; a negotiated size is accepted only at its generation
    generation: u64,
    // xtask-allow(state-fields): option: absent until the host first chooses a size
    size: Option<(u32, u32)>,
}
/// The physical size, scale and visibility the child applied last, read together.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::port) struct Applied {
    pub(in crate::port) width: u32,
    pub(in crate::port) height: u32,
    pub(in crate::port) scale: f64,
    pub(in crate::port) visibility: Visibility,
}
enum ChildStatus {
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
    /// The App panicked (the message is in the `PanicSlot`); sticky.
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
    pub(in crate::port) fn new(
        width: u32,
        height: u32,
        scale: f64,
        visibility: Visibility,
        panic: PanicSlot,
        notices: NoticeSink,
    ) -> Self {
        Self {
            inner: Mutex::new(Inner {
                state: ChildStatus::Running,
                applied: Applied {
                    width,
                    height,
                    scale,
                    visibility,
                },
                host: HostSize {
                    generation: 0,
                    size: None,
                },
                commands: VecDeque::new(),
            }),
            panic,
            notices,
        }
    }
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .expect("invariant: the embedded child lock is never held while panicking")
    }
    /// Keep the message of a panic that reached a platform boundary and stop
    /// the child; the host reads it as `ChildStop::Panicked` (see [`PanicSlot`]).
    pub(in crate::port) fn keep_panic(&self, payload: Box<dyn Any + Send>) {
        if self.panic.put(payload)
            && let Some(message) = self.panic.message()
        {
            tracing::error!(target: "kivact", message = %message, "the embedded App panicked; it stopped and will not run again");
        }
        let left_running = {
            let mut inner = self.lock();
            let left_running = matches!(inner.state, ChildStatus::Running);
            inner.state = match std::mem::replace(&mut inner.state, ChildStatus::Running) {
                ChildStatus::Running => ChildStatus::Stopped {
                    cause: Stop::Panicked { failure: None },
                    window: NativeWindow::Live,
                },
                ChildStatus::Stopped { cause, window } => ChildStatus::Stopped {
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
            left_running
        };
        if left_running {
            self.notices.stopped();
        }
    }
    pub(in crate::port) fn panicked(&self) -> bool {
        matches!(
            self.lock().state,
            ChildStatus::Stopped {
                cause: Stop::Panicked { .. },
                ..
            }
        )
    }
    pub(in crate::port) fn request_close(&self) {
        let left_running = {
            let mut inner = self.lock();
            if matches!(inner.state, ChildStatus::Running) {
                inner.state = ChildStatus::Stopped {
                    cause: Stop::Closed,
                    window: NativeWindow::Live,
                };
                true
            } else {
                false
            }
        };
        if left_running {
            self.notices.stopped();
        }
    }
    /// The native window is gone (Win32 `WM_DESTROY`; X11: the UI thread ended).
    pub(in crate::port) fn window_gone(&self) {
        let left_running = {
            let mut inner = self.lock();
            let left_running = matches!(inner.state, ChildStatus::Running);
            inner.state = match std::mem::replace(&mut inner.state, ChildStatus::Running) {
                ChildStatus::Running => ChildStatus::Stopped {
                    cause: Stop::Closed,
                    window: NativeWindow::Gone,
                },
                ChildStatus::Stopped { cause, .. } => ChildStatus::Stopped {
                    cause,
                    window: NativeWindow::Gone,
                },
            };
            left_running
        };
        if left_running {
            self.notices.stopped();
        }
    }
    /// Record a failure and stop; the first failure wins.
    pub(in crate::port) fn fail(&self, error: ShellError) {
        let left_running = {
            let mut inner = self.lock();
            let left_running = matches!(inner.state, ChildStatus::Running);
            inner.state = match std::mem::replace(&mut inner.state, ChildStatus::Running) {
                ChildStatus::Running => ChildStatus::Stopped {
                    cause: Stop::Failed(error),
                    window: NativeWindow::Live,
                },
                ChildStatus::Stopped {
                    cause: Stop::Closed,
                    window,
                } => ChildStatus::Stopped {
                    cause: Stop::Failed(error),
                    window,
                },
                ChildStatus::Stopped {
                    cause: Stop::Panicked { failure: None },
                    window,
                } => ChildStatus::Stopped {
                    cause: Stop::Panicked {
                        failure: Some(error),
                    },
                    window,
                },
                stopped @ ChildStatus::Stopped { .. } => stopped,
            };
            left_running
        };
        if left_running {
            self.notices.stopped();
        }
    }
    /// Why the child stopped, or `None` while it runs.
    pub(in crate::port) fn stop(&self) -> Option<ChildStop> {
        match &self.lock().state {
            ChildStatus::Running => None,
            ChildStatus::Stopped {
                cause: Stop::Closed,
                ..
            } => Some(ChildStop::Closed),
            ChildStatus::Stopped {
                cause: Stop::Failed(error),
                ..
            } => Some(ChildStop::Failed(error.clone())),
            ChildStatus::Stopped {
                cause: Stop::Panicked { .. },
                ..
            } => Some(ChildStop::Panicked),
        }
    }
    pub(in crate::port) fn stopped(&self) -> bool {
        !matches!(self.lock().state, ChildStatus::Running)
    }
    /// Whether the native window and its UI loop still exist.
    pub(in crate::port) fn alive(&self) -> bool {
        matches!(
            self.lock().state,
            ChildStatus::Running
                | ChildStatus::Stopped {
                    window: NativeWindow::Live,
                    ..
                }
        )
    }
    /// Queues a host change for the UI thread. Unnegotiated sizes and scales
    /// coalesce so that the newest host value wins; negotiated sizes keep their
    /// generation check and are never dropped here.
    pub(in crate::port) fn queue(&self, command: Command) {
        let mut inner = self.lock();
        if matches!(inner.state, ChildStatus::Running) {
            inner.push(command);
        }
    }
    /// Queues a size the host chose: it advances the generation, so earlier negotiated sizes go
    /// stale.
    pub(in crate::port) fn queue_host_size(&self, width: u32, height: u32) {
        let mut inner = self.lock();
        if matches!(inner.state, ChildStatus::Running) {
            inner.host.generation += 1;
            inner.host.size = Some((width, height));
            inner.push(Command::SetSize {
                width,
                height,
                generation: None,
            });
        }
    }
    /// The host size chosen since `generation`, if the generation advanced.
    pub(in crate::port) fn host_size_since(&self, generation: u64) -> Option<(u32, u32)> {
        let inner = self.lock();
        (inner.host.generation != generation)
            .then_some(inner.host.size)
            .flatten()
    }
    /// The next queued command (the Win32 UI loop re-checks its state between commands).
    #[cfg(windows)]
    pub(in crate::port) fn pop_command(&self) -> Option<Command> {
        self.lock().commands.pop_front()
    }
    /// Drain every queued command at once (the X11 UI loop).
    #[cfg(target_os = "linux")]
    pub(in crate::port) fn take_commands(&self) -> Vec<Command> {
        self.lock().commands.drain(..).collect()
    }
    pub(in crate::port) fn current(&self, generation: Option<u64>) -> bool {
        generation.is_none_or(|generation| generation == self.generation())
    }
    pub(in crate::port) fn generation(&self) -> u64 {
        self.lock().host.generation
    }
    /// The applied size and scale, read together.
    pub(in crate::port) fn applied(&self) -> Applied {
        self.lock().applied
    }
    pub(in crate::port) fn size(&self) -> (u32, u32) {
        let applied = self.applied();
        (applied.width, applied.height)
    }
    pub(in crate::port) fn scale(&self) -> f64 {
        self.applied().scale
    }
    /// Record the applied size; whether it changed.
    #[cfg(windows)]
    pub(in crate::port) fn set_size(&self, width: u32, height: u32) -> bool {
        let mut inner = self.lock();
        let changed = (inner.applied.width, inner.applied.height) != (width, height);
        inner.applied.width = width;
        inner.applied.height = height;
        changed
    }
    /// Record the applied scale; whether it changed.
    #[cfg(windows)]
    pub(in crate::port) fn set_scale(&self, scale: f64) -> bool {
        let mut inner = self.lock();
        let changed = inner.applied.scale != scale;
        inner.applied.scale = scale;
        changed
    }
    /// Record the applied visibility; whether it changed.
    #[cfg(windows)]
    pub(in crate::port) fn set_visibility(&self, visibility: Visibility) -> bool {
        let mut inner = self.lock();
        let changed = inner.applied.visibility != visibility;
        inner.applied.visibility = visibility;
        changed
    }
    /// Tell the host that the operating system changed the applied scale.
    #[cfg(windows)]
    pub(in crate::port) fn notice_scale_changed(&self, scale: f64) {
        self.notices.scale_changed(scale);
    }
    /// Record the applied size, scale and visibility together (the X11 UI loop).
    #[cfg(target_os = "linux")]
    pub(in crate::port) fn update(
        &self,
        width: u32,
        height: u32,
        scale: f64,
        visibility: Visibility,
    ) {
        self.lock().applied = Applied {
            width,
            height,
            scale,
            visibility,
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
            Command::SetVisibility(_) => {
                self.commands
                    .retain(|queued| !matches!(queued, Command::SetVisibility(_)));
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
pub(in crate::port) trait ChildPort: Send + Sync + 'static {
    const MAX_DIMENSION: u32;
    fn wake(&self);
    fn close(&self, shared: &Shared);
    /// Move the panic of a UI thread that ended into `shared` (a backend
    /// whose App runs on its own thread; the Win32 backend has none).
    fn collect_panic(&self, shared: &Shared);
}
pub(in crate::port) struct Handle<B: ChildPort> {
    pub(in crate::port) shared: Arc<Shared>,
    pub(in crate::port) backend: B,
}
impl<B: ChildPort> ShellChild for Handle<B> {
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
    fn set_scale(&self, scale: f64) {
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
    fn scale(&self) -> f64 {
        self.backend.collect_panic(&self.shared);
        self.shared.scale()
    }
    fn set_visibility(&self, visibility: Visibility) {
        self.backend.collect_panic(&self.shared);
        if self.shared.stopped() {
            return;
        }
        self.shared.queue(Command::SetVisibility(visibility));
        self.backend.wake();
    }
    fn visibility(&self) -> Visibility {
        self.backend.collect_panic(&self.shared);
        self.shared.applied().visibility
    }
    fn stop(&self) -> Option<ChildStop> {
        self.backend.collect_panic(&self.shared);
        self.shared.stop()
    }
}
impl<B: ChildPort> Drop for Handle<B> {
    fn drop(&mut self) {
        self.shared.request_close();
        self.backend.close(&self.shared);
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for host command coalescing, stop notices and panic messages.
    use super::*;
    use arvane_shell::EmbedNotice;
    use std::sync::Mutex;

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
            .filter_map(|command| match command {
                Command::SetSize {
                    width, generation, ..
                } => Some((*width, *generation)),
                Command::SetScale(scale) => Some((1000 + *scale as u32, None)),
                Command::SetVisibility(_) => None,
            })
            .collect()
    }

    fn notices() -> (Arc<Mutex<Vec<EmbedNotice>>>, NoticeSink) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = {
            let seen = seen.clone();
            NoticeSink::new(Some(Arc::new(move |notice| {
                seen.lock().expect("test lock is not poisoned").push(notice);
            })))
        };
        (seen, sink)
    }

    #[test]
    fn newest_unnegotiated_size_wins_and_negotiated_sizes_stay() {
        let shared = Shared::new(
            1,
            1,
            1.0,
            Visibility::Shown,
            PanicSlot::default(),
            NoticeSink::default(),
        );
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
        let shared = Shared::new(
            1,
            1,
            1.0,
            Visibility::Shown,
            PanicSlot::default(),
            NoticeSink::default(),
        );
        shared.queue(Command::SetScale(1.0));
        shared.queue(size(5, None));
        shared.queue(Command::SetScale(3.0));
        assert_eq!(queued(&shared), [(5, None), (1003, None)]);
    }

    #[test]
    fn newest_visibility_wins_and_keeps_order_of_the_rest() {
        let shared = Shared::new(
            1,
            1,
            1.0,
            Visibility::Shown,
            PanicSlot::default(),
            NoticeSink::default(),
        );
        shared.queue(Command::SetVisibility(Visibility::Hidden));
        shared.queue(size(5, None));
        shared.queue(Command::SetVisibility(Visibility::Shown));
        assert_eq!(queued(&shared), [(5, None)]);
    }

    #[test]
    fn stopped_queue_refuses_commands() {
        let shared = Shared::new(
            1,
            1,
            1.0,
            Visibility::Shown,
            PanicSlot::default(),
            NoticeSink::default(),
        );
        shared.queue(size(1, None));
        shared.request_close();
        shared.queue(size(2, None));
        shared.queue(Command::SetScale(2.0));
        shared.queue(Command::SetVisibility(Visibility::Hidden));
        assert_eq!(queued(&shared), [(1, None)]);

        let failed = Shared::new(
            1,
            1,
            1.0,
            Visibility::Shown,
            PanicSlot::default(),
            NoticeSink::default(),
        );
        assert!(failed.stop().is_none());
        failed.fail(ShellError::new("first"));
        failed.fail(ShellError::new("second"));
        assert!(matches!(failed.stop(), Some(ChildStop::Failed(_))) && failed.stopped());
        failed.queue(size(1, None));
        assert!(queued(&failed).is_empty());
    }

    #[test]
    fn the_first_stop_notifies_the_host_once() {
        let (seen, sink) = notices();
        let shared = Shared::new(1, 1, 1.0, Visibility::Shown, PanicSlot::default(), sink);
        assert_eq!(shared.stop(), None);
        shared.request_close();
        shared.request_close();
        shared.fail(ShellError::new("late"));
        assert_eq!(
            *seen.lock().expect("test lock is not poisoned"),
            [EmbedNotice::Stopped]
        );
        assert!(matches!(shared.stop(), Some(ChildStop::Failed(_))));
    }

    #[test]
    fn panics_keep_the_first_message_and_notify_once() {
        let (seen, sink) = notices();
        let shared = Shared::new(1, 1, 1.0, Visibility::Shown, PanicSlot::default(), sink);
        shared.keep_panic(Box::new("boom"));
        shared.keep_panic(Box::new("later"));
        assert_eq!(shared.stop(), Some(ChildStop::Panicked));
        assert_eq!(shared.panic.message().as_deref(), Some("boom"));
        assert_eq!(
            *seen.lock().expect("test lock is not poisoned"),
            [EmbedNotice::Stopped]
        );
    }
}
