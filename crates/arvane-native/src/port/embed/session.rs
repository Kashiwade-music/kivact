//! Role: Own the embedded UI lifecycle, handler and module-retention sentinel.
use super::child::Shared;
use super::keys::KeyLedger;
use super::resize::{self, ResizeWait};
use arvane_shell::{
    EventTarget, InjectedInput, MakeHandler, ShellControl, ShellError, ShellEvent, ShellHandler,
    ShellLoop, ShellWaker, ShellWindow,
};
use kivact_types::{Capabilities, KeyCode, RequestId, WindowId, WindowRequest};
use std::cell::RefCell;
use std::collections::VecDeque;
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
    /// The synchronous verdict of the native key being delivered.
    keys: RefCell<KeyLedger>,
    /// The one size request waiting for the host's confirmation.
    resize: RefCell<ResizeWait>,
    /// Deferred answers waiting for the handler to return.
    answers: RefCell<VecDeque<ShellEvent>>,
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
            keys: RefCell::new(KeyLedger::default()),
            resize: RefCell::new(ResizeWait::Idle),
            answers: RefCell::new(VecDeque::new()),
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
        let control = match &*self.state.borrow() {
            ChildLifecycle::Opening(HandlerSlot::Running { control, .. })
            | ChildLifecycle::Live(HandlerSlot::Running { control, .. }) => control.deadline(),
            _ => None,
        };
        let wait = resize::deadline(&self.resize);
        match (control, wait) {
            (Some(control), Some(wait)) => Some(control.min(wait)),
            (control, wait) => control.or(wait),
        }
    }
    /// The synchronous verdict of the native key being delivered.
    pub(in crate::port) fn keys(&self) -> &RefCell<KeyLedger> {
        &self.keys
    }
    /// The engine reported that nobody handled the press of `code`.
    pub(in crate::port) fn unhandled_key(&self, code: KeyCode) {
        self.keys.borrow_mut().unhandled(code);
    }
    /// Answer a resize request whose host size arrived or whose wait ended.
    pub(in crate::port) fn settle_resize(&self, now_ns: u64) {
        resize::settle(&self.resize, &self.shared, &self.answers, now_ns);
    }
    /// Ask the host for a physical size and say what it did (see [`resize::negotiate`]).
    #[allow(
        clippy::too_many_arguments,
        reason = "The host callback, request identity and wake are the negotiation contract; a bundle would hide their ordering."
    )]
    pub(in crate::port) fn request_size(
        &self,
        size: kivact_types::Size,
        scale: f64,
        maximum: u32,
        callback: Option<&(dyn Fn(u32, u32) -> kivact_types::ResizeAnswer + Send + Sync)>,
        request: RequestId,
        now_ns: u64,
        wake: impl FnOnce(),
    ) -> arvane_shell::SizeReply {
        resize::negotiate(
            size,
            scale,
            maximum,
            callback,
            &self.shared,
            request,
            now_ns,
            &self.resize,
            &self.answers,
            wake,
        )
    }
    /// Take the deferred answers queued for the handler's caller.
    pub(in crate::port) fn take_answers(&self) -> Vec<ShellEvent> {
        self.answers.borrow_mut().drain(..).collect()
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
        if let HandlerSlot::Running { handler, .. } = slot {
            if self.shared.panicked() {
                self.forget(handler);
                return false;
            }
            // Held in a `ManuallyDrop`: if `close` panics, the handler is forgotten rather than
            // dropped during the unwind.
            let mut handler = std::mem::ManuallyDrop::new(handler);
            let mut lp = EmbedLoop {
                window,
                waker: self.waker.clone(),
                injected: Vec::new(),
            };
            handler.close(&mut lp);
            drop(std::mem::ManuallyDrop::into_inner(handler));
            self.retention.retain();
        }
        true
    }
    /// Forget the App after a panic that reached the platform boundary and
    /// could not run the normal shutdown (a window that is being destroyed
    /// cannot run a queued command). The handler is never dropped or closed.
    #[cfg(windows)]
    pub(in crate::port) fn leak(&self) {
        let previous = self.state.replace(ChildLifecycle::Leaked);
        if let ChildLifecycle::Opening(HandlerSlot::Running { handler, .. })
        | ChildLifecycle::Live(HandlerSlot::Running { handler, .. }) = previous
        {
            self.forget(handler);
        }
        crate::module::retain_module();
    }
    /// Keep a handler that must never be dropped or closed again, and its module loaded.
    fn forget(&self, handler: Box<dyn ShellHandler>) {
        self.state.replace(ChildLifecycle::Leaked);
        crate::module::retain_module();
        std::mem::forget(handler);
    }
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
