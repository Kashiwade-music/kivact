//! Role: Own the embedded UI lifecycle, handler and module-retention sentinel.
use super::child::{Command, Shared};
use arvane_shell::{
    MakeHandler, ShellControl, ShellError, ShellEvent, ShellHandler, ShellWaker, ShellWindow,
};
use std::cell::RefCell;
use std::sync::Arc;

enum HandlerSlot {
    Unstarted,
    Running {
        handler: Box<dyn ShellHandler>,
        control: ShellControl,
    },
}
enum ChildLifecycle {
    Opening(HandlerSlot),
    Live(HandlerSlot),
    Disposed,
    Leaked,
}
pub(in crate::port) struct Session {
    state: RefCell<ChildLifecycle>,
    shared: Arc<Shared>,
    retention: ModuleRetention,
}
// Temporarily lend the handler to a callback while leaving its lifecycle observable on reentry.
struct Callback<'a> {
    session: &'a Session,
    slot: HandlerSlot,
}
impl Drop for Callback<'_> {
    fn drop(&mut self) {
        let slot = std::mem::replace(&mut self.slot, HandlerSlot::Unstarted);
        match &mut *self.session.state.borrow_mut() {
            ChildLifecycle::Opening(current) | ChildLifecycle::Live(current) => *current = slot,
            ChildLifecycle::Disposed | ChildLifecycle::Leaked => {
                unreachable!("invariant: callbacks defer disposal")
            }
        }
    }
}
impl Session {
    pub(in crate::port) fn new(shared: Arc<Shared>) -> Self {
        Self {
            state: RefCell::new(ChildLifecycle::Opening(HandlerSlot::Unstarted)),
            shared,
            retention: ModuleRetention,
        }
    }
    pub(in crate::port) fn opening(&self) -> bool {
        matches!(*self.state.borrow(), ChildLifecycle::Opening(_))
    }
    #[cfg(windows)]
    pub(in crate::port) fn live(&self) -> bool {
        matches!(*self.state.borrow(), ChildLifecycle::Live(_))
    }
    #[cfg(windows)]
    pub(in crate::port) fn active(&self) -> bool {
        matches!(
            *self.state.borrow(),
            ChildLifecycle::Opening(_) | ChildLifecycle::Live(_)
        )
    }
    pub(in crate::port) fn opened(&self) {
        let mut state = self.state.borrow_mut();
        if let ChildLifecycle::Opening(slot) =
            std::mem::replace(&mut *state, ChildLifecycle::Disposed)
        {
            *state = ChildLifecycle::Live(slot);
        } else {
            unreachable!("invariant: only opening sessions become live");
        }
    }
    fn callback(&self) -> Option<Callback<'_>> {
        match &mut *self.state.borrow_mut() {
            ChildLifecycle::Opening(slot) | ChildLifecycle::Live(slot) => Some(Callback {
                session: self,
                slot: std::mem::replace(slot, HandlerSlot::Unstarted),
            }),
            ChildLifecycle::Disposed | ChildLifecycle::Leaked => None,
        }
    }
    pub(in crate::port) fn start(
        &self,
        make: MakeHandler,
        window: &mut dyn ShellWindow,
        waker: ShellWaker,
    ) {
        if self.shared.stopped() {
            return;
        }
        let mut callback = self
            .callback()
            .expect("invariant: start owns an opening session");
        debug_assert!(matches!(callback.slot, HandlerSlot::Unstarted));
        callback.slot = HandlerSlot::Running {
            handler: make(),
            control: ShellControl::Wait,
        };
        let HandlerSlot::Running { handler, .. } = &mut callback.slot else {
            unreachable!("invariant: handler just installed");
        };
        let result = if self.shared.stopped() {
            Ok(ShellControl::Wait)
        } else {
            handler.start(window, waker)
        };
        self.accept(&mut callback.slot, result);
    }
    pub(in crate::port) fn deliver(&self, event: ShellEvent, window: &mut dyn ShellWindow) {
        if self.shared.stopped() {
            return;
        }
        let Some(mut callback) = self.callback() else {
            return;
        };
        let HandlerSlot::Running { handler, .. } = &mut callback.slot else {
            return;
        };
        let result = handler.event(event, window);
        self.accept(&mut callback.slot, result);
    }
    fn accept(&self, slot: &mut HandlerSlot, result: Result<ShellControl, ShellError>) {
        match result {
            Err(error) => self.shared.fail(error),
            Ok(_) if self.shared.stopped() => {}
            Ok(ShellControl::Exit) => self.shared.fail(ShellError::new(
                "the App asked to exit; an embedded App cannot exit by itself",
            )),
            Ok(next) => {
                if let HandlerSlot::Running { control, .. } = slot {
                    *control = next;
                }
            }
        }
    }
    pub(in crate::port) fn deadline(&self) -> Option<u64> {
        match &*self.state.borrow() {
            ChildLifecycle::Opening(HandlerSlot::Running { control, .. })
            | ChildLifecycle::Live(HandlerSlot::Running { control, .. }) => control.deadline(),
            _ => None,
        }
    }
    pub(in crate::port) fn close(&self, window: &mut dyn ShellWindow) -> bool {
        self.shared.request_close();
        let state = self.state.replace(ChildLifecycle::Disposed);
        let slot = match state {
            ChildLifecycle::Opening(slot) | ChildLifecycle::Live(slot) => slot,
            ChildLifecycle::Disposed => return true,
            ChildLifecycle::Leaked => {
                self.state.replace(ChildLifecycle::Leaked);
                return false;
            }
        };
        if let HandlerSlot::Running { mut handler, .. } = slot {
            if self.shared.panicked() {
                self.state.replace(ChildLifecycle::Leaked);
                crate::module::retain_module();
                std::mem::forget(handler);
                return false;
            }
            handler.close(window);
            drop(handler);
            self.retention.retain();
        }
        true
    }
}
pub(in crate::port) fn negotiate(
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

// The session's last field: it drops after the handler state, whose drop may detach threads.
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
