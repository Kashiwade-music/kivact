//! Role: Starts, runs and closes an embedded X11 UI thread.

use super::conn::Native;
use super::event::Input;
use super::keyboard::Keyboard;
use super::notifier::{MAX_DIMENSION, Notifier};
use super::poll::Waiter;
use super::window::Window;
use crate::embed::child::{Backend as ChildBackend, Command, Handle, Shared};
use crate::embed::session::Session;
use arvane_shell::{
    ChildOptions, MakeHandler, ShellChild, ShellError, ShellEvent, ShellWaker, ShellWindow,
};
use std::sync::atomic::Ordering;
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use x11rb::connection::Connection;

impl ChildBackend for Arc<Notifier> {
    const MAX_DIMENSION: u32 = MAX_DIMENSION;
    fn wake(&self) {
        super::poll::signal(&self.wake);
    }
    fn close(&self, shared: &Shared) {
        // `request_close` already ran; the wake makes the UI loop observe it.
        if shared.alive.load(Ordering::Acquire) {
            self.wake();
        }
        let thread = self.thread.lock().expect(LOCK_POISONED).take();
        if let Some(thread) = thread {
            // The callback must return before its own event loop can close.
            if thread.thread().id() == std::thread::current().id() {
                crate::detached::DETACHED.store(true, Ordering::Release);
                crate::retain_module();
                return;
            }
            if crate::wait::until(&mut || thread.is_finished(), crate::wait::SHUTDOWN_WAIT) {
                if let Err(payload) = thread.join() {
                    shared.keep_panic(payload);
                }
            } else {
                crate::detached::DETACHED.store(true, Ordering::Release);
                crate::retain_module();
                tracing::warn!(target: "kivact", "embedded X11 UI thread did not stop within two seconds; detaching");
            }
        }
    }
    fn collect_panic(&self, shared: &Shared) {
        let mut slot = self.thread.lock().expect(LOCK_POISONED);
        if slot.as_ref().is_some_and(JoinHandle::is_finished)
            && let Some(thread) = slot.take()
            && let Err(payload) = thread.join()
        {
            shared.keep_panic(payload);
        }
    }
}
const LOCK_POISONED: &str = "embedded child lock poisoned: a panic already happened";

/// Marks the UI thread dead when it ends, by return or by unwind, so the
/// host sees a stopped child before it joins the thread.
struct UiThreadExit(Arc<Shared>);
impl Drop for UiThreadExit {
    fn drop(&mut self) {
        self.0.alive.store(false, Ordering::Release);
    }
}
pub(super) fn waker(backend: &Arc<Notifier>) -> ShellWaker {
    // Async wakers must not retain the eventfd after the UI thread closes.
    let weak = Arc::downgrade(backend);
    ShellWaker::new(move || {
        if let Some(backend) = weak.upgrade() {
            backend.wake();
        }
    })
}

pub(in crate::embed) fn open(
    parent: kivact_types::protocol::ParentWindow,
    options: ChildOptions,
    make: MakeHandler,
) -> Result<Box<dyn ShellChild>, ShellError> {
    let shared = Arc::new(Shared::new(0, 0, 1.0, options.panic.clone()));
    let backend = Arc::new(Notifier {
        wake: super::poll::eventfd()?,
        thread: std::sync::Mutex::new(None),
    });
    let ui_backend = backend.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    let ui_shared = shared.clone();
    let thread = std::thread::Builder::new()
        .name("arvane-embed".into())
        .spawn(move || {
            let exit = UiThreadExit(ui_shared.clone());
            let result = run(
                parent,
                options,
                make,
                ui_shared.clone(),
                ui_backend,
                &sender,
            );
            drop(exit);
            if let Err(error) = result {
                ui_shared.fail(error.clone());
                let _ = sender.try_send(Err(error));
            }
        })
        .map_err(super::error::platform)?;
    *backend.thread.lock().expect(LOCK_POISONED) = Some(thread);
    let child = Handle { shared, backend };
    match receiver.recv() {
        Ok(Ok(())) => Ok(Box::new(child)),
        Ok(Err(error)) => Err(error),
        Err(_) => {
            // The UI thread ended without answering: it failed (see below) or
            // it panicked, and its payload is kept for the host to resume.
            let thread = child.backend.thread.lock().expect(LOCK_POISONED).take();
            if let Some(thread) = thread
                && let Err(payload) = thread.join()
            {
                child.shared.keep_panic(payload);
                return Err(ShellError::new("the embedded App panicked while opening"));
            }
            Err(child
                .shared
                .failure()
                .unwrap_or_else(|| super::error::platform("UI thread stopped during startup")))
        }
    }
}

fn run(
    parent: kivact_types::protocol::ParentWindow,
    options: ChildOptions,
    make: MakeHandler,
    shared: Arc<Shared>,
    backend: Arc<Notifier>,
    ready: &mpsc::SyncSender<Result<(), ShellError>>,
) -> Result<(), ShellError> {
    let kivact_types::protocol::ParentWindow::X11(parent) = parent else {
        return Err(ShellError::unsupported("X11 requires an X11 parent"));
    };
    let (native, width, height, scale) =
        Native::create_unmapped(parent.get(), options.width, options.height, options.scale)?;
    let keyboard = Keyboard::new(&native.connection)?;
    let waiter = Waiter::new()?;
    let mut window = Window::new(
        native,
        shared.clone(),
        backend.clone(),
        (width, height),
        scale,
        options.on_resize_request,
        waker(&backend),
    )?;
    let mut session = Session::new(shared.clone());
    session.start(make, &mut window, waker(&backend));
    window.opening = false;
    let result = if let Some(error) = shared.failure() {
        let _ = ready.send(Err(error.clone()));
        Err(error)
    } else if let Err(error) = window.native.show() {
        Err(error)
    } else if ready.send(Ok(())).is_err() {
        Ok(())
    } else {
        event_loop(&mut session, &mut window, Input::new(keyboard), waiter)
    };
    if let Err(error) = &result {
        shared.fail(error.clone());
    }
    shared.request_close();
    if session.close(&mut window) {
        drop(window);
    } else {
        std::mem::forget(window);
    }
    shared.alive.store(false, Ordering::Release);
    result
}

fn closed(window: &Window) -> bool {
    window.shared.stopped() || !window.native.alive.load(Ordering::Acquire)
}

fn event_loop(
    session: &mut Session,
    window: &mut Window,
    mut input: Input,
    waiter: Waiter,
) -> Result<(), ShellError> {
    let mut pending = Vec::new();
    let id = window.native.id.get();
    loop {
        if closed(window) {
            return Ok(());
        }
        // Drain XCB's internal queue before poll: replies can already have moved
        // events off the socket, leaving its fd unreadable despite pending input.
        let mut events = std::mem::take(&mut pending);
        while let Some(event) = window
            .native
            .connection
            .poll_for_event()
            .map_err(super::error::platform)?
        {
            events.push(event);
        }
        let mut events = events.into_iter().peekable();
        while let Some(event) = events.next() {
            // Legacy X autorepeat is a release/press pair with matching time.
            // Keep the key held so the following press is marked as a repeat.
            if let x11rb::protocol::Event::KeyRelease(release) = &event
                && matches!(events.peek(), Some(x11rb::protocol::Event::KeyPress(press))
                    if press.detail == release.detail && press.time == release.time)
            {
                continue;
            }
            if let x11rb::protocol::Event::Error(e) = &event {
                window.ime.borrow_mut().x_error(e);
            }
            // An unchecked request on our window (focus) can fail after the host
            // destroyed the parent; that is a close, not an error.
            if matches!(&event, x11rb::protocol::Event::Error(e) if super::error::gone(e, id)) {
                window.native.alive.store(false, Ordering::Release);
                window.shared.request_close();
                return Ok(());
            }
            let destroyed = matches!(&event, x11rb::protocol::Event::DestroyNotify(e)
                if e.window == id);
            if destroyed {
                window.native.alive.store(false, Ordering::Release);
                window.shared.request_close();
                // Parent destruction is an external close. Delivering it as
                // CloseRequested would turn the App's ordinary close response
                // into an erroneous App-requested exit in the shared session.
                return Ok(());
            }
            for event in input.translate(event, window)? {
                session.deliver(event, window);
                if closed(window) {
                    return Ok(());
                }
            }
        }
        if closed(window) {
            return Ok(());
        }
        let woke = super::poll::drain(&window.backend.wake)?;
        let commands: Vec<_> = window
            .shared
            .commands
            .lock()
            .expect(LOCK_POISONED)
            .drain(..)
            .collect();
        for command in commands {
            if closed(window) {
                return Ok(());
            }
            match command {
                Command::SetSize {
                    width,
                    height,
                    generation,
                } => {
                    if window.shared.current(generation) {
                        window.resize(width, height)?;
                    }
                }
                Command::SetScale(scale) => {
                    // `Handle::set_scale` rejects other values before queuing.
                    debug_assert!(scale.is_finite() && scale > 0.0);
                    window.scale = scale;
                    window.resized(window.width, window.height);
                    session.deliver(ShellEvent::Resized, window);
                }
            }
        }
        if closed(window) {
            return Ok(());
        }
        let actions = window.accessibility.tree.drain_actions();
        for action in actions {
            session.deliver(action, window);
            if closed(window) {
                return Ok(());
            }
        }
        let due = session
            .control
            .next_wake_ns
            .is_some_and(|deadline| deadline <= window.now_ns());
        if due || woke {
            session.deliver(ShellEvent::Wake, window);
            if closed(window) {
                return Ok(());
            }
        }
        if window.redraw.replace(false) {
            session.deliver(ShellEvent::Redraw, window);
        }
        if closed(window) {
            return Ok(());
        }
        let ime_events = input.drain_ime(window);
        for event in ime_events {
            session.deliver(event, window);
            if closed(window) {
                return Ok(());
            }
        }
        window
            .accessibility
            .publish(&window.native, window.width, window.height, window.scale)?;
        if closed(window) {
            return Ok(());
        }
        if window.redraw.get() {
            continue;
        }
        // Checked X requests made by commands or callbacks may have queued
        // events while consuming the socket. Never poll past that queued work.
        if let Some(event) = window
            .native
            .connection
            .poll_for_event()
            .map_err(super::error::platform)?
        {
            pending.push(event);
            continue;
        }
        // A close or failure from any thread ends delivery before waiting.
        if closed(window) {
            return Ok(());
        }
        let remaining = session
            .control
            .next_wake_ns
            .map(|deadline| deadline.saturating_sub(window.now_ns()));
        // A lost XIM acknowledgement is given up at its own deadline.
        let remaining = [remaining, window.ime.borrow().remaining_ns()]
            .into_iter()
            .flatten()
            .min();
        waiter.wait(&*window.native.connection, &window.backend.wake, remaining)?;
    }
}
