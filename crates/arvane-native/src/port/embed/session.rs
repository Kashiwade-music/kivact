//! Role: Own the embedded UI lifecycle, handler and module-retention sentinel.
use super::child::Shared;
use super::keys::KeyLedger;
use super::resize::{self, ResizeWait};
use arvane_shell::{
    ChildStop, EventTarget, InjectedInput, MakeHandler, ShellControl, ShellError, ShellEvent,
    ShellHandler, ShellLoop, ShellWaker, ShellWindow,
};
use kivact_types::{Capabilities, KeyCode, RequestId, WindowId, WindowRequest};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

/// The embedded child's life on its UI thread (AR-15): whether App code
/// may run, whether its handler is lent to a callback, and what is left
/// to clean up once it stopped.
pub(in crate::port) enum Child {
    /// The window is being made; the handler is made at start.
    Opening { make: MakeHandler },
    /// The App runs. Its handler is in its slot or lent to a callback.
    Live {
        handler: HandlerSlot,
        control: ShellControl,
    },
    /// The App stopped (`cause`, as `Shared` records it); the handler is
    /// still to be closed, and the native resources released, once.
    Stopped { cause: ChildStop, cleanup: Cleanup },
    /// A callback panicked: the handler was forgotten (never dropped)
    /// and the native resources are left as the panic left them.
    Leaked,
}

/// Where a live child's handler is.
pub(in crate::port) enum HandlerSlot {
    /// In the slot, between callbacks.
    Idle(Box<dyn ShellHandler>),
    /// Lent to the callback running now (`Lent` gives it back).
    Calling,
}

/// What a stopped child still has to release.
pub(in crate::port) enum Cleanup {
    /// The handler's `close` and the native resources (exactly once).
    /// `None` when the handler was never made.
    Pending(Option<Box<dyn ShellHandler>>),
    /// Released.
    Done,
}

impl Child {
    pub(in crate::port) fn opening(&self) -> bool {
        matches!(self, Self::Opening { .. })
    }
    #[cfg(windows)]
    pub(in crate::port) fn live(&self) -> bool {
        matches!(self, Self::Live { .. })
    }
    /// Whether App code may run now: the handler is idle in its slot.
    pub(in crate::port) fn accepts_callbacks(&self) -> bool {
        matches!(
            self,
            Self::Live {
                handler: HandlerSlot::Idle(_),
                ..
            }
        )
    }
    /// Whether the handler or the native resources still wait for `dispose`.
    #[cfg(any(windows, test))]
    pub(in crate::port) fn needs_cleanup(&self) -> bool {
        matches!(
            self,
            Self::Opening { .. }
                | Self::Live { .. }
                | Self::Stopped {
                    cleanup: Cleanup::Pending(_),
                    ..
                }
        )
    }
    /// Make the handler: `Opening` becomes `Live`. `false` in any other state.
    pub(in crate::port) fn start(&mut self) -> bool {
        match std::mem::replace(self, Self::Leaked) {
            Self::Opening { make } => {
                *self = Self::Live {
                    handler: HandlerSlot::Idle(make()),
                    control: ShellControl::Wait,
                };
                true
            }
            other => {
                *self = other;
                false
            }
        }
    }
    /// Lend the idle handler to one callback; `None` when App code may not run.
    pub(in crate::port) fn lend(&mut self) -> Option<Box<dyn ShellHandler>> {
        if !self.accepts_callbacks() {
            return None;
        }
        let Self::Live { handler, .. } = self else {
            return None;
        };
        match std::mem::replace(handler, HandlerSlot::Calling) {
            HandlerSlot::Idle(handler) => Some(handler),
            HandlerSlot::Calling => None,
        }
    }
    /// Take back the lent handler: live again, or stopped with the shared
    /// stop the callback recorded (`stop`).
    pub(in crate::port) fn give_back(
        &mut self,
        handler: Box<dyn ShellHandler>,
        stop: Option<ChildStop>,
    ) {
        match std::mem::replace(self, Self::Leaked) {
            Self::Live {
                handler: HandlerSlot::Calling,
                control,
            } => {
                *self = match stop {
                    None => Self::Live {
                        handler: HandlerSlot::Idle(handler),
                        control,
                    },
                    Some(cause) => Self::Stopped {
                        cause,
                        cleanup: Cleanup::Pending(Some(handler)),
                    },
                };
            }
            other => {
                *self = other;
                debug_assert!(false, "invariant: a lent handler returns to a live child");
                std::mem::forget(handler);
            }
        }
    }
    /// Forget a handler after its panic: `Leaked`, and the handler is never dropped.
    pub(in crate::port) fn leak(&mut self, handler: Box<dyn ShellHandler>) {
        *self = Self::Leaked;
        std::mem::forget(handler);
    }
    /// The shared stop reached a child that no callback holds. A lent handler
    /// is left alone: its callback records the stop when it gives the handler back.
    pub(in crate::port) fn stop_idle(&mut self, cause: ChildStop) {
        match std::mem::replace(self, Self::Leaked) {
            Self::Opening { .. } => {
                *self = Self::Stopped {
                    cause,
                    cleanup: Cleanup::Pending(None),
                };
            }
            Self::Live {
                handler: HandlerSlot::Idle(handler),
                ..
            } => {
                *self = Self::Stopped {
                    cause,
                    cleanup: Cleanup::Pending(Some(handler)),
                };
            }
            other => *self = other,
        }
    }
    pub(in crate::port) fn set_control(&mut self, next: ShellControl) {
        if let Self::Live { control, .. } = self {
            *control = next;
        }
    }
    /// Hand out the cleanup once: `Some(handler)` (the handler, or `None` when
    /// it was never made). `None` when nothing is left, or when a callback runs.
    pub(in crate::port) fn take_cleanup(&mut self) -> Option<Option<Box<dyn ShellHandler>>> {
        match std::mem::replace(self, Self::Leaked) {
            Self::Opening { .. } => {
                *self = Self::Stopped {
                    cause: ChildStop::Closed,
                    cleanup: Cleanup::Done,
                };
                Some(None)
            }
            Self::Live {
                handler: HandlerSlot::Idle(handler),
                ..
            } => {
                *self = Self::Stopped {
                    cause: ChildStop::Closed,
                    cleanup: Cleanup::Done,
                };
                Some(Some(handler))
            }
            Self::Stopped {
                cause,
                cleanup: Cleanup::Pending(handler),
            } => {
                *self = Self::Stopped {
                    cause,
                    cleanup: Cleanup::Done,
                };
                Some(handler)
            }
            other => {
                let calling = matches!(
                    other,
                    Self::Live {
                        handler: HandlerSlot::Calling,
                        ..
                    }
                );
                *self = other;
                debug_assert!(
                    !calling,
                    "invariant: disposal is deferred while a callback runs"
                );
                None
            }
        }
    }
    /// The handler a panic must forget: the idle one, or the one awaiting
    /// cleanup. A lent handler stays with its callback.
    #[cfg(windows)]
    pub(in crate::port) fn take_idle(&mut self) -> Option<Box<dyn ShellHandler>> {
        if matches!(
            self,
            Self::Live {
                handler: HandlerSlot::Calling,
                ..
            }
        ) {
            return None;
        }
        match std::mem::replace(self, Self::Leaked) {
            Self::Live {
                handler: HandlerSlot::Idle(handler),
                ..
            } => Some(handler),
            Self::Stopped {
                cleanup: Cleanup::Pending(handler),
                ..
            } => handler,
            _ => None,
        }
    }
}

// xtask-allow(state): legacy: child, keys, resize, answers; injected_input normalizes DevTools input apart from the platform's input and pairs with no other field
pub(in crate::port) struct Session {
    child: RefCell<Child>,
    shared: Arc<Shared>,
    waker: ShellWaker,
    /// The synchronous verdict of the native key being delivered.
    keys: RefCell<KeyLedger>,
    /// The one size request waiting for the host's confirmation.
    resize: RefCell<ResizeWait>,
    /// Deferred answers waiting for the handler to return.
    answers: RefCell<VecDeque<ShellEvent>>,
    /// The input normalizer of injected input (DevTools); the platform's own input has its own.
    injected_input: RefCell<(arvane_shell::WindowInput, arvane_shell::pipeline::ImeDriver)>,
    retention: ModuleRetention,
}

/// The embedded editor's loop: one window, no more.
pub(in crate::port) struct EmbedLoop<'a> {
    window: &'a mut dyn ShellWindow,
    waker: ShellWaker,
    /// The inject requests answered with `Injected` after the callback returns.
    injected: Vec<(Vec<InjectedInput>, RequestId)>,
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

    fn inject(&mut self, _: WindowId, inputs: Vec<InjectedInput>, request: RequestId) {
        self.injected.push((inputs, request));
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
/// A handler lent to one callback; dropping it gives the handler back, so
/// unwinding restores the child too. A panicked handler stays in its slot
/// until `dispose` forgets it (the platform boundary records the panic first).
struct Lent<'a> {
    session: &'a Session,
    handler: Option<Box<dyn ShellHandler>>,
}
impl Lent<'_> {
    fn handler(&mut self) -> &mut dyn ShellHandler {
        self.handler
            .as_deref_mut()
            .expect("invariant: lent until dropped")
    }
}
impl Drop for Lent<'_> {
    fn drop(&mut self) {
        let Some(handler) = self.handler.take() else {
            return;
        };
        let stop = self.session.shared.stop();
        self.session.child.borrow_mut().give_back(handler, stop);
    }
}
impl Session {
    pub(in crate::port) fn new(shared: Arc<Shared>, waker: ShellWaker, make: MakeHandler) -> Self {
        Self {
            child: RefCell::new(Child::Opening { make }),
            shared,
            waker,
            keys: RefCell::new(KeyLedger::default()),
            resize: RefCell::new(ResizeWait::Idle),
            answers: RefCell::new(VecDeque::new()),
            injected_input: RefCell::new((
                arvane_shell::WindowInput::new(),
                arvane_shell::pipeline::ImeDriver::default(),
            )),
            retention: ModuleRetention,
        }
    }
    pub(in crate::port) fn waker(&self) -> ShellWaker {
        self.waker.clone()
    }
    pub(in crate::port) fn opening(&self) -> bool {
        self.child.borrow().opening()
    }
    #[cfg(windows)]
    pub(in crate::port) fn live(&self) -> bool {
        self.child.borrow().live()
    }
    #[cfg(windows)]
    pub(in crate::port) fn needs_cleanup(&self) -> bool {
        self.child.borrow().needs_cleanup()
    }
    fn callback(&self) -> Option<Lent<'_>> {
        let handler = self.child.borrow_mut().lend()?;
        Some(Lent {
            session: self,
            handler: Some(handler),
        })
    }
    /// Start the App once: only a session that is still opening starts it.
    pub(in crate::port) fn start(&self, window: &mut dyn ShellWindow) {
        if self.shared.stopped() || !self.child.borrow_mut().start() {
            return;
        }
        let Some(mut lent) = self.callback() else {
            return;
        };
        let mut lp = EmbedLoop {
            window,
            waker: self.waker.clone(),
            injected: Vec::new(),
        };
        let result = if self.shared.stopped() {
            Ok(ShellControl::Wait)
        } else {
            lent.handler().start(&mut lp, self.waker.clone())
        };
        self.deliver_injected(&mut lent, &mut lp);
        self.accept(result);
    }
    pub(in crate::port) fn deliver(&self, event: ShellEvent, window: &mut dyn ShellWindow) {
        if self.shared.stopped() {
            return;
        }
        let Some(mut lent) = self.callback() else {
            return;
        };
        let target = EventTarget::of(&event, WindowId::MAIN);
        let mut lp = EmbedLoop {
            window,
            waker: self.waker.clone(),
            injected: Vec::new(),
        };
        let result = lent.handler().event(target, event, &mut lp);
        self.deliver_injected(&mut lent, &mut lp);
        self.accept(result);
    }
    /// Deliver the injected input the callback queued, then the `Injected`
    /// event of each request.
    fn deliver_injected(&self, lent: &mut Lent<'_>, lp: &mut EmbedLoop<'_>) {
        while !lp.injected.is_empty() && !self.shared.stopped() {
            for (inputs, request) in std::mem::take(&mut lp.injected) {
                for event in self.injected_events(lp, inputs) {
                    if !self.deliver_one(lent, lp, event) {
                        return;
                    }
                }
                if !self.deliver_one(lent, lp, ShellEvent::Injected { request }) {
                    return;
                }
            }
        }
    }

    /// The events `inputs` make, normalized like platform input.
    fn injected_events(
        &self,
        lp: &mut EmbedLoop<'_>,
        inputs: Vec<InjectedInput>,
    ) -> Vec<ShellEvent> {
        let now = lp.window.now_ns();
        let pointer = lp.window.environment().pointer;
        let double_click = (pointer.double_click_ns, pointer.double_click_distance);
        let mut state = self.injected_input.borrow_mut();
        let (input, ime) = &mut *state;
        let mut events = Vec::new();
        for injected in inputs {
            match injected {
                InjectedInput::Accessibility { node, request } => {
                    events.extend(lp.window.request_accessibility_action(node, request));
                }
                other => events.extend(input.normalize(other, now, double_click, ime)),
            }
        }
        events
    }

    /// Deliver one event to the lent handler; `false` once it stopped.
    fn deliver_one(&self, lent: &mut Lent<'_>, lp: &mut EmbedLoop<'_>, event: ShellEvent) -> bool {
        let result = lent
            .handler()
            .event(EventTarget::of(&event, WindowId::MAIN), event, lp);
        self.accept(result);
        !self.shared.stopped()
    }
    fn accept(&self, result: Result<ShellControl, ShellError>) {
        match result {
            Err(error) => self.shared.fail(error),
            Ok(_) if self.shared.stopped() => {}
            Ok(ShellControl::Exit) => self.shared.fail(ShellError::new(
                "the App asked to exit; an embedded App cannot exit by itself",
            )),
            Ok(next) => self.child.borrow_mut().set_control(next),
        }
    }
    pub(in crate::port) fn deadline(&self) -> Option<u64> {
        let control = match &*self.child.borrow() {
            Child::Live {
                handler: HandlerSlot::Idle(_),
                control,
            } => control.deadline(),
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
    /// The shared stop reached a child that no callback holds (the host's
    /// drop, the window's destruction, the UI thread's end).
    pub(in crate::port) fn stopped_outside_callback(&self) {
        if let Some(cause) = self.shared.stop() {
            self.child.borrow_mut().stop_idle(cause);
        }
    }
    /// Close the handler and release the child's resources, once. Returns
    /// `true` when the caller releases its native resources now.
    pub(in crate::port) fn dispose(&self, window: &mut dyn ShellWindow) -> bool {
        self.shared.request_close();
        let Some(cleanup) = self.child.borrow_mut().take_cleanup() else {
            return false;
        };
        let Some(handler) = cleanup else {
            return true;
        };
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
        true
    }
    /// Forget the App after a panic that reached the platform boundary and
    /// could not run the normal shutdown (a window that is being destroyed
    /// cannot run a queued command). The handler is never dropped or closed.
    #[cfg(windows)]
    pub(in crate::port) fn leak(&self) {
        let idle = self.child.borrow_mut().take_idle();
        match idle {
            Some(handler) => self.forget(handler),
            None => crate::module::retain_module(),
        }
    }
    /// Keep a handler that must never be dropped or closed again, and its module loaded.
    fn forget(&self, handler: Box<dyn ShellHandler>) {
        self.child.borrow_mut().leak(handler);
        crate::module::retain_module();
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

#[cfg(test)]
mod tests {
    //! Role: Tests of the embedded child's lifecycle transitions (AR-15, TV-09).
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    /// A handler that counts its closes (see `dispose`).
    struct Counted(Rc<Cell<u32>>);
    impl ShellHandler for Counted {
        fn start(
            &mut self,
            _: &mut dyn ShellLoop,
            _: ShellWaker,
        ) -> Result<ShellControl, ShellError> {
            Ok(ShellControl::Wait)
        }
        fn event(
            &mut self,
            _: arvane_shell::EventTarget,
            _: ShellEvent,
            _: &mut dyn ShellLoop,
        ) -> Result<ShellControl, ShellError> {
            Ok(ShellControl::Wait)
        }
        fn close(&mut self, _: &mut dyn ShellLoop) {
            self.0.set(self.0.get() + 1);
        }
    }

    fn opening() -> Child {
        Child::Opening {
            make: Box::new(|| Box::new(Counted(Rc::new(Cell::new(0))))),
        }
    }

    #[test]
    fn a_child_stopped_before_start_cleans_up_natively_once() {
        let mut child = opening();
        child.stop_idle(ChildStop::Closed);
        assert!(matches!(child.take_cleanup(), Some(None)));
        assert!(child.take_cleanup().is_none());
    }

    #[test]
    fn an_error_in_a_callback_keeps_the_handler_for_one_close() {
        let mut child = opening();
        assert!(child.start());
        let handler = child.lend().expect("a started child lends its handler");
        child.give_back(handler, Some(ChildStop::Failed(ShellError::new("failed"))));
        assert!(!child.accepts_callbacks());
        assert!(child.needs_cleanup());
        assert!(matches!(child.take_cleanup(), Some(Some(_))));
        assert!(child.take_cleanup().is_none());
    }

    #[test]
    fn exit_from_a_callback_stops_and_closes_once() {
        let mut child = opening();
        assert!(child.start());
        let handler = child.lend().expect("a started child lends its handler");
        child.give_back(handler, Some(ChildStop::Closed));
        assert!(matches!(child.take_cleanup(), Some(Some(_))));
        assert!(child.take_cleanup().is_none());
    }

    #[test]
    fn a_host_side_close_of_an_idle_child_closes_once() {
        let mut child = opening();
        assert!(child.start());
        child.stop_idle(ChildStop::Closed);
        assert!(matches!(child.take_cleanup(), Some(Some(_))));
        assert!(child.take_cleanup().is_none());
    }

    #[test]
    fn a_panic_in_a_callback_leaks_the_handler() {
        let mut child = opening();
        assert!(child.start());
        let handler = child.lend().expect("a started child lends its handler");
        child.leak(handler);
        assert!(child.take_cleanup().is_none());
        assert!(!child.needs_cleanup());
    }

    #[test]
    fn no_callback_starts_after_the_window_is_gone() {
        let mut child = opening();
        assert!(child.start());
        child.stop_idle(ChildStop::Closed);
        assert!(child.lend().is_none());
    }
}
