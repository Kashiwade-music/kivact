//! Role: Own the embedded UI lifecycle, handler and module-retention sentinel.
use super::child::{Command, Shared};
use arvane_shell::{
    EventTarget, InjectedInput, MakeHandler, ShellControl, ShellError, ShellEvent, ShellHandler,
    ShellLoop, ShellWaker, ShellWindow,
};
use kivact_types::{Capabilities, RequestId, WindowId, WindowRequest};
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
    waker: ShellWaker,
    retention: ModuleRetention,
}

/// The embedded editor's loop: one window, no more.
pub(in crate::port) struct EmbedLoop<'a> {
    window: &'a mut dyn ShellWindow,
    waker: ShellWaker,
    /// The inject requests answered with `Injected` after the callback returns.
    injected: Vec<RequestId>,
}

impl ShellLoop for EmbedLoop<'_> {
    fn window(&mut self, id: WindowId) -> Option<&mut dyn ShellWindow> {
        if id == WindowId::MAIN {
            Some(&mut *self.window)
        } else {
            None
        }
    }

    fn windows(&self) -> Vec<WindowId> {
        vec![WindowId::MAIN]
    }

    fn open_window(&mut self, _: WindowRequest) -> Result<(), ShellError> {
        Err(ShellError::unsupported(
            "windows beyond the embedded editor",
        ))
    }

    fn close_window(&mut self, _: WindowId) {}

    fn inject(&mut self, _: WindowId, _: Vec<InjectedInput>, request: RequestId) {
        self.injected.push(request);
    }

    fn now_ns(&self) -> u64 {
        self.window.now_ns()
    }

    fn waker(&self) -> ShellWaker {
        self.waker.clone()
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            multi_window: false,
            window_placement: false,
            ..self.window.capabilities()
        }
    }
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
    pub(in crate::port) fn new(shared: Arc<Shared>, waker: ShellWaker) -> Self {
        Self {
            state: RefCell::new(ChildLifecycle::Opening(HandlerSlot::Unstarted)),
            shared,
            waker,
            retention: ModuleRetention,
        }
    }
    pub(in crate::port) fn waker(&self) -> ShellWaker {
        self.waker.clone()
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
    pub(in crate::port) fn start(&self, make: MakeHandler, window: &mut dyn ShellWindow) {
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
        let mut lp = EmbedLoop {
            window,
            waker: self.waker.clone(),
            injected: Vec::new(),
        };
        let result = {
            let HandlerSlot::Running { handler, .. } = &mut callback.slot else {
                unreachable!("invariant: handler just installed");
            };
            if self.shared.stopped() {
                Ok(ShellControl::Wait)
            } else {
                handler.start(&mut lp, self.waker.clone())
            }
        };
        self.deliver_injected(&mut callback.slot, &mut lp);
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
        let target = EventTarget::of(&event, WindowId::MAIN);
        let mut lp = EmbedLoop {
            window,
            waker: self.waker.clone(),
            injected: Vec::new(),
        };
        let result = handler.event(target, event, &mut lp);
        self.deliver_injected(&mut callback.slot, &mut lp);
        self.accept(&mut callback.slot, result);
    }
    /// Deliver the `Injected` events of the requests the callback queued.
    fn deliver_injected(&self, slot: &mut HandlerSlot, lp: &mut EmbedLoop<'_>) {
        while !lp.injected.is_empty() && !self.shared.stopped() {
            let requests = std::mem::take(&mut lp.injected);
            for request in requests {
                let HandlerSlot::Running { handler, .. } = slot else {
                    return;
                };
                let result = handler.event(
                    EventTarget::Window(WindowId::MAIN),
                    ShellEvent::Injected { request },
                    lp,
                );
                self.accept(slot, result);
                if self.shared.stopped() {
                    return;
                }
            }
        }
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
            let mut lp = EmbedLoop {
                window,
                waker: self.waker.clone(),
                injected: Vec::new(),
            };
            handler.close(&mut lp);
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
