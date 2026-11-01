//! Role: Share embedded child commands and implement the host handle contract.
//!
//! Invariants: snapshots describe applied values, never queued requests. Host
//! sizes advance a generation so stale accepted negotiations cannot override
//! corrections. Backend wake only schedules work and never invokes App code.
//! Stopping cancels new App work immediately. A panic of the App is kept in
//! the child's `PanicSlot` and stops the child like a failure; the host's next
//! call resumes it. Alive becomes false when the
//! native window is destroyed (WM_DESTROY on Windows; on X11 once the UI
//! thread has destroyed its window after DestroyNotify/BadWindow or a close).
//! Cleanup of the App may still be deferred after that while an App callback
//! is on the stack.
use arvane_shell::{PanicSlot, ShellChild, ShellError};
use std::any::Any;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub(super) enum Command {
    SetSize {
        width: u32,
        height: u32,
        generation: Option<u64>,
    },
    SetScale(f32),
}
pub(super) struct Shared {
    pub(super) commands: Mutex<VecDeque<Command>>,
    size: AtomicU64,
    scale: AtomicU32,
    size_generation: AtomicU64,
    failure: Mutex<Option<ShellError>>,
    failed: AtomicBool,
    pub(super) alive: AtomicBool,
    stopping: AtomicBool,
    panic: PanicSlot,
}
impl Shared {
    pub(super) fn new(width: u32, height: u32, scale: f32, panic: PanicSlot) -> Self {
        Self {
            commands: Mutex::new(VecDeque::new()),
            size: AtomicU64::new(pack(width, height)),
            scale: AtomicU32::new(scale.to_bits()),
            size_generation: AtomicU64::new(0),
            failure: Mutex::new(None),
            failed: AtomicBool::new(false),
            alive: AtomicBool::new(true),
            stopping: AtomicBool::new(false),
            panic,
        }
    }
    /// Keep the payload of a panic that reached a platform boundary; the
    /// child stops and the host resumes it (see [`PanicSlot`]).
    pub(super) fn keep_panic(&self, payload: Box<dyn Any + Send>) {
        self.panic.put(payload);
    }
    pub(super) fn panicked(&self) -> bool {
        self.panic.is_set()
    }
    pub(super) fn request_close(&self) {
        self.stopping.store(true, Ordering::Release);
    }
    pub(super) fn stopped(&self) -> bool {
        self.stopping.load(Ordering::Acquire)
            || !self.alive.load(Ordering::Acquire)
            || self.failed()
            || self.panicked()
    }
    /// Queues a host change for the UI thread. Unnegotiated sizes and scales
    /// coalesce so that the newest host value wins; negotiated sizes keep their
    /// generation check and are never dropped here.
    pub(super) fn queue(&self, command: Command) {
        if self.stopped() {
            return;
        }
        let mut commands = self
            .commands
            .lock()
            .expect("embedded child lock poisoned: a panic already happened");
        match &command {
            Command::SetSize {
                generation: None, ..
            } => commands.retain(|queued| {
                !matches!(
                    queued,
                    Command::SetSize {
                        generation: None,
                        ..
                    }
                )
            }),
            Command::SetScale(_) => {
                commands.retain(|queued| !matches!(queued, Command::SetScale(_)));
            }
            Command::SetSize { .. } => {}
        }
        commands.push_back(command);
    }
    pub(super) fn current(&self, generation: Option<u64>) -> bool {
        generation.is_none_or(|generation| generation == self.generation())
    }
    pub(super) fn generation(&self) -> u64 {
        self.size_generation.load(Ordering::Acquire)
    }
    pub(super) fn size(&self) -> (u32, u32) {
        unpack(self.size.load(Ordering::Acquire))
    }
    pub(super) fn scale(&self) -> f32 {
        f32::from_bits(self.scale.load(Ordering::Acquire))
    }
    pub(super) fn set_scale_value(&self, scale: f32) {
        self.scale.store(scale.to_bits(), Ordering::Release);
    }
    pub(super) fn update(&self, width: u32, height: u32, scale: f32) {
        self.size.store(pack(width, height), Ordering::Release);
        self.set_scale_value(scale);
    }
    pub(super) fn fail(&self, error: ShellError) {
        let mut failure = self
            .failure
            .lock()
            .expect("embedded child lock poisoned: a panic already happened");
        if failure.is_none() {
            *failure = Some(error);
            self.failed.store(true, Ordering::Release);
        }
    }
    pub(super) fn failure(&self) -> Option<ShellError> {
        self.failure
            .lock()
            .expect("embedded child lock poisoned: a panic already happened")
            .clone()
    }
    pub(super) fn failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }
}
/// OS side of an embedded child; `Handle` implements `ShellChild` once on top.
///
/// - `MAX_DIMENSION`: the largest physical dimension the native window
///   accepts; setters ignore larger values with a warning.
/// - `wake`: schedule the App UI thread to drain `Shared::commands`. Callable
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
        self.shared.size_generation.fetch_add(1, Ordering::AcqRel);
        self.shared.queue(Command::SetSize {
            width,
            height,
            generation: None,
        });
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
fn pack(width: u32, height: u32) -> u64 {
    (u64::from(width) << 32) | u64::from(height)
}
fn unpack(value: u64) -> (u32, u32) {
    ((value >> 32) as u32, value as u32)
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
        let commands = shared.commands.lock().unwrap();
        commands
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
        assert!(!failed.failed());
        failed.fail(ShellError::new("first"));
        failed.fail(ShellError::new("second"));
        assert!(failed.failed() && failed.stopped());
        failed.queue(size(1, None));
        assert!(queued(&failed).is_empty());
    }
}
