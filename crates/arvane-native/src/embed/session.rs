//! Role: Contain embedded callbacks and share failure, shutdown and negotiation.
//!
//! Invariants: a stop request or the first failure stops event delivery. Close consumes the handler
//! exactly once before native resources are released. Panic payloads are never
//! dropped; teardown failure tells the backend to forget all remaining resources.
use super::child::{Command, Shared};
use arvane_shell::{
    MakeHandler, ShellControl, ShellError, ShellEvent, ShellHandler, ShellWaker, ShellWindow,
};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};

pub(super) struct Session {
    handler: Option<Box<dyn ShellHandler>>,
    pub(super) control: ShellControl,
    shared: Arc<Shared>,
}
impl Session {
    pub(super) fn new(shared: Arc<Shared>) -> Self {
        Self {
            handler: None,
            control: ShellControl::default(),
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
        let result = contain(
            || {
                self.handler = Some(make());
                if self.shared.stopped() {
                    return Ok(ShellControl::default());
                }
                self.handler
                    .as_mut()
                    .expect("invariant: handler was constructed")
                    .start(window, waker)
            },
            "embedded App panicked while opening",
        );
        self.accept(result);
    }
    pub(super) fn deliver(&mut self, event: ShellEvent, window: &mut dyn ShellWindow) {
        if self.shared.stopped() {
            return;
        }
        let Some(handler) = self.handler.as_mut() else {
            return;
        };
        let result = contain(
            || handler.event(event, window),
            "embedded App callback panicked",
        );
        self.accept(result);
    }
    fn accept(&mut self, result: Result<Result<ShellControl, ShellError>, ShellError>) {
        // Closing cancels successful control output, but must not hide a real
        // callback error or panic that occurred before the callback returned.
        match result.and_then(|result| result) {
            Err(error) => self.shared.fail(error),
            Ok(_) if self.shared.stopped() => {}
            Ok(control) if !control.exit => self.control = control,
            Ok(_) => self.shared.fail(ShellError::new(
                "the App asked to exit; an embedded App cannot exit by itself",
            )),
        }
    }
    /// Returns false when the backend must forget its remaining native resources.
    pub(super) fn close(&mut self, window: &mut dyn ShellWindow) -> bool {
        self.shared.request_close();
        let Some(mut handler) = self.handler.take() else {
            return true;
        };
        if let Err(error) = contain(|| handler.close(window), "embedded App close panicked") {
            self.shared.fail(error);
            // Forgotten owners can bypass every worker's shutdown/drop guard.
            crate::module::retain_module();
            std::mem::forget(handler);
            tracing::warn!(target: "kivact", "embedded App close panicked; leaking remaining handler and native resources");
            return false;
        }
        if let Err(error) = contain(|| drop(handler), "embedded App destruction panicked") {
            self.shared.fail(error);
            crate::module::retain_module();
            tracing::warn!(target: "kivact", "embedded App destruction panicked; leaking remaining native resources");
            return false;
        }
        true
    }
}
pub(super) fn contain<T>(
    action: impl FnOnce() -> T,
    message: &'static str,
) -> Result<T, ShellError> {
    catch_unwind(AssertUnwindSafe(action)).map_err(|payload| {
        std::mem::forget(payload);
        ShellError::new(message)
    })
}
pub(super) fn negotiate(
    size: ui_types::Size,
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
