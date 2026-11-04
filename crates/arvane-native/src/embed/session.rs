//! Role: Own the embedded handler and share failure, shutdown and negotiation.
//!
//! Invariants: a stop request, the first failure or a panic stops event delivery. Close consumes the
//! handler exactly once before native resources are released. Callbacks are
//! not caught here: panic payloads are kept in the child's `PanicSlot`, and after one the handler is
//! forgotten, never run again, and the backend forgets all remaining resources.
use super::child::{Command, Shared};
use arvane_shell::{
    MakeHandler, ShellControl, ShellError, ShellEvent, ShellHandler, ShellWaker, ShellWindow,
};
use std::sync::Arc;

enum HandlerState {
    /// `start` has not built the handler.
    Unstarted,
    /// The handler runs; `control` is what it last returned.
    Running {
        handler: Box<dyn ShellHandler>,
        control: ShellControl,
    },
    /// `close` consumed the handler.
    Closed,
}
pub(super) struct Session {
    state: HandlerState,
    shared: Arc<Shared>,
    retention: ModuleRetention,
}
impl Session {
    pub(super) fn new(shared: Arc<Shared>) -> Self {
        Self {
            state: HandlerState::Unstarted,
            shared,
            retention: ModuleRetention,
        }
    }
    pub(super) fn start(
        &mut self,
        make: MakeHandler,
        window: &mut dyn ShellWindow,
        waker: ShellWaker,
    ) {
        debug_assert!(matches!(self.state, HandlerState::Unstarted));
        if self.shared.stopped() {
            return;
        }
        self.state = HandlerState::Running {
            handler: make(),
            control: ShellControl::Wait,
        };
        let HandlerState::Running { handler, .. } = &mut self.state else {
            unreachable!("invariant: the handler was just installed");
        };
        let result = if self.shared.stopped() {
            Ok(ShellControl::Wait)
        } else {
            handler.start(window, waker)
        };
        self.accept(result);
    }
    pub(super) fn deliver(&mut self, event: ShellEvent, window: &mut dyn ShellWindow) {
        if self.shared.stopped() {
            return;
        }
        let HandlerState::Running { handler, .. } = &mut self.state else {
            return;
        };
        let result = handler.event(event, window);
        self.accept(result);
    }
    fn accept(&mut self, result: Result<ShellControl, ShellError>) {
        // Closing cancels successful control output, but must not hide a real
        // callback error that occurred before the callback returned.
        match result {
            Err(error) => self.shared.fail(error),
            Ok(_) if self.shared.stopped() => {}
            Ok(ShellControl::Exit) => self.shared.fail(ShellError::new(
                "the App asked to exit; an embedded App cannot exit by itself",
            )),
            Ok(next) => {
                if let HandlerState::Running { control, .. } = &mut self.state {
                    *control = next;
                }
            }
        }
    }
    /// The wake deadline the running handler asked for.
    pub(super) fn deadline(&self) -> Option<u64> {
        match &self.state {
            HandlerState::Running { control, .. } => control.deadline(),
            HandlerState::Unstarted | HandlerState::Closed => None,
        }
    }
    /// Returns false when the backend must forget its remaining native resources.
    pub(super) fn close(&mut self, window: &mut dyn ShellWindow) -> bool {
        self.shared.request_close();
        match std::mem::replace(&mut self.state, HandlerState::Closed) {
            HandlerState::Unstarted | HandlerState::Closed => true,
            HandlerState::Running { mut handler, .. } => {
                if self.shared.panicked() {
                    // After a panic no more App code runs: not even its close.
                    // Forgotten owners can bypass every worker's shutdown/drop guard.
                    crate::module::retain_module();
                    std::mem::forget(handler);
                    return false;
                }
                handler.close(window);
                drop(handler);
                self.retention.retain();
                true
            }
        }
    }
}
pub(super) fn negotiate(
    size: kivact_types::Size,
    scale: f64,
    maximum: u32,
    callback: Option<&(dyn Fn(u32, u32) -> bool + Send + Sync)>,
    shared: &Shared,
    wake: impl FnOnce(),
) -> Result<(), ShellError> {
    if shared.stopped() {
        return Ok(());
    }
    let (width, height) = super::geometry::physical_size(size, scale, maximum)?;
    let callback =
        callback.ok_or_else(|| ShellError::unsupported("the host has no resize callback"))?;
    let generation = shared.generation();
    let accepted = callback(width, height);
    if shared.stopped() {
        return Ok(());
    }
    if !accepted {
        return Err(ShellError::new("the host refused the size"));
    }
    shared.queue(Command::SetSize {
        width,
        height,
        generation: Some(generation),
    });
    wake();
    Ok(())
}

// Fields drop in declaration order: Session releases its handler and presenter first.
struct ModuleRetention;
impl ModuleRetention {
    fn retain(&self) {
        if kivact_types::process::DetachedThreads::any() {
            crate::module::retain_module();
        }
    }
}
impl Drop for ModuleRetention {
    fn drop(&mut self) {
        self.retain();
    }
}
