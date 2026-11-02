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

pub(super) struct Session {
    handler: Option<Box<dyn ShellHandler>>,
    pub(super) control: ShellControl,
    shared: Arc<Shared>,
}
impl Session {
    pub(super) fn new(shared: Arc<Shared>) -> Self {
        Self {
            handler: None,
            control: ShellControl::Wait,
            shared,
        }
    }
    pub(super) fn start(
        &mut self,
        make: MakeHandler,
        window: &mut dyn ShellWindow,
        waker: ShellWaker,
    ) {
        if self.shared.stopped() {
            return;
        }
        self.handler = Some(make());
        let result = if self.shared.stopped() {
            Ok(ShellControl::Wait)
        } else {
            self.handler
                .as_mut()
                .expect("invariant: handler was constructed")
                .start(window, waker)
        };
        self.accept(result);
    }
    pub(super) fn deliver(&mut self, event: ShellEvent, window: &mut dyn ShellWindow) {
        if self.shared.stopped() {
            return;
        }
        let Some(handler) = self.handler.as_mut() else {
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
            Ok(control) => self.control = control,
        }
    }
    /// Returns false when the backend must forget its remaining native resources.
    pub(super) fn close(&mut self, window: &mut dyn ShellWindow) -> bool {
        self.shared.request_close();
        let Some(mut handler) = self.handler.take() else {
            return true;
        };
        if self.shared.panicked() {
            // After a panic no more App code runs: not even its close.
            // Forgotten owners can bypass every worker's shutdown/drop guard.
            crate::module::retain_module();
            std::mem::forget(handler);
            return false;
        }
        handler.close(window);
        drop(handler);
        true
    }
}
pub(super) fn negotiate(
    size: kivact_types::Size,
    scale: f32,
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
