//! Role: Starts, runs and closes an embedded X11 UI thread.
use super::conn::Native;
use super::event::Input;
use super::keyboard::Keyboard;
use super::notifier::{MAX_DIMENSION, Notifier, UiThread};
use super::poll::Waiter;
use super::window::{ChildRedraw, Window};
use crate::port::embed::child::{ChildPort, Command, Handle, Shared};
use crate::port::embed::keys::Verdict;
use crate::port::embed::session::Session;
use arvane_shell::{
    ChildHooks, ChildStop, EmbedOptions, MakeHandler, NoticeSink, ShellChild, ShellError,
    ShellEvent, ShellWaker, ShellWindow,
};
use std::sync::{Arc, mpsc};
use x11rb::connection::Connection;

impl ChildPort for Arc<Notifier> {
    const MAX_DIMENSION: u32 = MAX_DIMENSION;
    fn wake(&self) {
        self.request(arvane_shell::WakeReasons::CLIENT);
    }
    fn close(&self, shared: &Shared) {
        // `request_close` already ran; the wake makes the UI loop observe it.
        if shared.alive() {
            self.wake();
        }
        let mut slot = self.thread.lock().expect(LOCK_POISONED);
        let thread = match std::mem::replace(&mut *slot, UiThread::Joined) {
            UiThread::Running(thread) => Some(thread),
            // Nothing was spawned, or the handle was already taken.
            unspawned @ (UiThread::Unspawned | UiThread::Joined) => {
                *slot = unspawned;
                None
            }
        };
        drop(slot);
        if let Some(thread) = thread {
            // The callback must return before its own event loop can close.
            if thread.thread().id() == std::thread::current().id() {
                kivact_types::process::DetachedThreads::note();
                crate::retain_module();
                return;
            }
            if crate::wait::until(&mut || thread.is_finished(), crate::wait::SHUTDOWN_WAIT) {
                if let Err(payload) = thread.join() {
                    shared.keep_panic(payload);
                }
            } else {
                kivact_types::process::DetachedThreads::note();
                crate::retain_module();
                tracing::warn!(target: "kivact", wait = ?crate::wait::SHUTDOWN_WAIT, "embedded X11 UI thread did not stop in time; detaching");
            }
        }
    }
    fn collect_panic(&self, shared: &Shared) {
        let mut slot = self.thread.lock().expect(LOCK_POISONED);
        let finished = match &*slot {
            UiThread::Running(thread) => thread.is_finished(),
            UiThread::Unspawned | UiThread::Joined => false,
        };
        if finished {
            let UiThread::Running(thread) = std::mem::replace(&mut *slot, UiThread::Joined) else {
                unreachable!("just saw a running UI thread");
            };
            if let Err(payload) = thread.join() {
                shared.keep_panic(payload);
            }
        }
    }
}
const LOCK_POISONED: &str = "embedded child lock poisoned: a panic already happened";

/// Marks the UI thread dead when it ends, by return or by unwind, so the
/// host sees a stopped child before it joins the thread.
struct UiThreadExit(Arc<Shared>);
impl Drop for UiThreadExit {
    fn drop(&mut self) {
        self.0.window_gone();
    }
}
pub(super) fn waker(backend: &Arc<Notifier>) -> ShellWaker {
    // Async wakers must not retain the eventfd after the UI thread closes.
    let weak = Arc::downgrade(backend);
    ShellWaker::new(move |reasons| {
        if let Some(backend) = weak.upgrade() {
            backend.request(reasons);
        }
    })
}

pub(in crate::port) fn open(
    parent: kivact_types::ParentWindow,
    options: EmbedOptions,
    hooks: ChildHooks,
    make: MakeHandler,
) -> Result<Box<dyn ShellChild>, ShellError> {
    let shared = Arc::new(Shared::new(
        0,
        0,
        1.0,
        options.visibility,
        hooks.panic.clone(),
        NoticeSink::new(hooks.on_notice.clone()),
    ));
    let backend = Arc::new(Notifier {
        wake: super::poll::eventfd()?,
        reasons: std::sync::atomic::AtomicU8::new(0),
        thread: std::sync::Mutex::new(UiThread::Unspawned),
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
                hooks,
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
    *backend.thread.lock().expect(LOCK_POISONED) = UiThread::Running(thread);
    let child = Handle { shared, backend };
    match receiver.recv() {
        Ok(Ok(())) => Ok(Box::new(child)),
        Ok(Err(error)) => Err(error),
        Err(_) => {
            // The UI thread ended without answering: it failed (see below) or
            // it panicked, and its message is kept for the host to read.
            let mut slot = child.backend.thread.lock().expect(LOCK_POISONED);
            let thread = match std::mem::replace(&mut *slot, UiThread::Joined) {
                UiThread::Running(thread) => Some(thread),
                _ => None,
            };
            drop(slot);
            if let Some(thread) = thread
                && let Err(payload) = thread.join()
            {
                child.shared.keep_panic(payload);
                return Err(ShellError::new("the embedded App panicked while opening"));
            }
            Err(child
                .shared
                .stop()
                .and_then(|stop| match stop {
                    ChildStop::Failed(error) => Some(error),
                    ChildStop::Closed | ChildStop::Panicked => None,
                })
                .unwrap_or_else(|| super::error::platform("UI thread stopped during startup")))
        }
    }
}

fn run(
    parent: kivact_types::ParentWindow,
    options: EmbedOptions,
    hooks: ChildHooks,
    make: MakeHandler,
    shared: Arc<Shared>,
    backend: Arc<Notifier>,
    ready: &mpsc::SyncSender<Result<(), ShellError>>,
) -> Result<(), ShellError> {
    let kivact_types::ParentWindow::X11(parent) = parent else {
        return Err(ShellError::unsupported("X11 requires an X11 parent"));
    };
    let (native, width, height, scale) = Native::create_unmapped(
        parent.get(),
        options.size.width,
        options.size.height,
        options.scale,
        options.visibility,
    )?;
    let keyboard = Keyboard::new(&native.connection)?;
    let waiter = Waiter::new()?;
    let mut window = Window::new(
        native,
        shared.clone(),
        backend.clone(),
        (width, height),
        scale,
        options.visibility,
        options.unhandled_keys,
        hooks.on_resize_request,
        waker(&backend),
    )?;
    let session = window.session.clone();
    let _watch = crate::port::linux_settings::watch(session.waker());
    session.start(make, &mut window);
    session.opened();
    let result = if let Some(ChildStop::Failed(error)) = shared.stop() {
        let _ = ready.send(Err(error.clone()));
        Err(error)
    } else if shared.panicked() {
        Err(ShellError::new("the embedded App panicked while opening"))
    } else if options.visibility == kivact_types::embed::Visibility::Shown
        && let Err(error) = window.native.show()
    {
        Err(error)
    } else if ready.send(Ok(())).is_err() {
        Ok(())
    } else {
        event_loop(&session, &mut window, Input::new(keyboard), waiter)
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
    shared.window_gone();
    result
}

fn closed(window: &Window) -> bool {
    window.shared.stopped() || !window.native.exists.is_alive()
}

/// Deliver the wake of this turn: the reasons taken with the commands, a due
/// session deadline, and a changed environment (announced before the wake).
/// Returns whether the window closed meanwhile.
fn deliver_wake(
    session: &Session,
    window: &mut Window,
    mut reasons: arvane_shell::WakeReasons,
) -> bool {
    let due = session
        .deadline()
        .is_some_and(|deadline| deadline <= window.now_ns());
    if due {
        reasons.insert(arvane_shell::WakeReasons::CLIENT);
    }
    if reasons.is_empty() {
        return false;
    }
    if reasons.contains(arvane_shell::WakeReasons::ENVIRONMENT) {
        session.deliver(ShellEvent::EnvironmentChanged, window);
        if closed(window) {
            return true;
        }
    }
    session.deliver(ShellEvent::Wake(reasons), window);
    closed(window)
}

fn event_loop(
    session: &Session,
    window: &mut Window,
    mut input: Input,
    waiter: Waiter,
) -> Result<(), ShellError> {
    let mut pending = Vec::new();
    loop {
        if closed(window) {
            return Ok(());
        }
        deliver_native_events(session, window, &mut input, &mut pending)?;
        if closed(window) {
            return Ok(());
        }
        // Take wake reasons together with the eventfd drain: `request` signals
        // only when the reasons go from empty to set, so reasons left set after
        // the drain would swallow the signal of a later command.
        let reasons = deliver_commands(session, window, &mut input)?;
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
        if deliver_wake(session, window, reasons) {
            return Ok(());
        }
        if window.redraw.replace(ChildRedraw::Clean) == ChildRedraw::Needed {
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
        if window.redraw.get() == ChildRedraw::Needed {
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
        window.dnd.expire(&window.native)?;
        let remaining = session
            .deadline()
            .map(|deadline| deadline.saturating_sub(window.now_ns()));
        // A lost XIM acknowledgement and a stalled file drop are given up at
        // their own deadlines.
        let remaining = [
            remaining,
            window.ime.borrow().remaining_ns(),
            window.dnd.remaining_ns(),
        ]
        .into_iter()
        .flatten()
        .min();
        waiter.wait(&*window.native.connection, &window.backend.wake, remaining)?;
    }
}

/// Drain queued native events, preserving legacy autorepeat pairs.
fn deliver_native_events(
    session: &Session,
    window: &mut Window,
    input: &mut Input,
    pending: &mut Vec<x11rb::protocol::Event>,
) -> Result<(), ShellError> {
    let id = window.native.id.get();
    // Drain XCB's internal queue before poll: replies can already have moved
    // events off the socket, leaving its fd unreadable despite pending input.
    let mut events = std::mem::take(pending);
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
            window.native.exists.end();
            window.shared.request_close();
            return Ok(());
        }
        let destroyed = matches!(&event, x11rb::protocol::Event::DestroyNotify(e)
                if e.window == id);
        if destroyed {
            window.native.exists.end();
            window.shared.request_close();
            // Parent destruction is an external close. Delivering it as
            // CloseRequested would turn the App's ordinary close response
            // into an erroneous App-requested exit in the shared session.
            return Ok(());
        }
        let translated = input.translate(event, window)?;
        let native_key = input.take_native_key();
        let key_at = native_key.as_ref().and_then(|_| {
            translated
                .iter()
                .rposition(|event| matches!(event, ShellEvent::Key(_)))
        });
        for (index, translated) in translated.into_iter().enumerate() {
            if let (ShellEvent::Key(key), Some((down, original)), Some(at)) =
                (&translated, &native_key, key_at)
                && index == at
            {
                let code = key.code;
                deliver_native_key(session, window, translated, code, *down, original)?;
            } else {
                session.deliver(translated, window);
            }
            if closed(window) {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// Deliver one native key and give it back to the parent when nobody handled it.
fn deliver_native_key(
    session: &Session,
    window: &mut Window,
    event: ShellEvent,
    code: kivact_types::KeyCode,
    down: bool,
    original: &x11rb::protocol::xproto::KeyPressEvent,
) -> Result<(), ShellError> {
    if window.unhandled_keys != kivact_types::UnhandledKeys::ToParent {
        session.deliver(event, window);
        return Ok(());
    }
    if !down {
        session.deliver(event, window);
        if session.keys().borrow_mut().take_forwarded(code) {
            send_key(window, original, false)?;
        }
        return Ok(());
    }
    session.keys().borrow_mut().open(code);
    session.deliver(event, window);
    session.deliver(ShellEvent::Wake(arvane_shell::WakeReasons::CLIENT), window);
    if session.keys().borrow_mut().close() == Verdict::Unhandled {
        session.keys().borrow_mut().forward(code);
        send_key(window, original, true)?;
    }
    Ok(())
}

/// Send a synthetic key event to the host's parent window.
fn send_key(
    window: &Window,
    key: &x11rb::protocol::xproto::KeyPressEvent,
    down: bool,
) -> Result<(), ShellError> {
    use x11rb::protocol::xproto::EventMask;
    let event = x11rb::protocol::xproto::KeyPressEvent {
        event: window.native.parent,
        child: 0,
        ..key.clone()
    };
    let mask = if down {
        EventMask::KEY_PRESS
    } else {
        EventMask::KEY_RELEASE
    };
    window
        .native
        .connection
        .send_event(false, window.native.parent, mask, event)
        .map_err(super::error::platform)?;
    window
        .native
        .connection
        .flush()
        .map_err(super::error::platform)
}

/// Apply host commands before accessibility, wakes, and redraw delivery.
/// Returns the wake reasons requested up to the eventfd drain.
fn deliver_commands(
    session: &Session,
    window: &mut Window,
    input: &mut Input,
) -> Result<arvane_shell::WakeReasons, ShellError> {
    let _ = super::poll::drain(&window.backend.wake)?;
    let reasons = window.backend.take();
    for command in window.shared.take_commands() {
        if closed(window) {
            return Ok(reasons);
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
            Command::SetVisibility(visibility) => {
                for event in visibility_events(window, input, visibility)? {
                    session.deliver(event, window);
                    if closed(window) {
                        return Ok(reasons);
                    }
                }
            }
        }
    }
    session.settle_resize(window.now_ns());
    for answer in session.take_answers() {
        session.deliver(answer, window);
        if closed(window) {
            return Ok(reasons);
        }
    }
    Ok(reasons)
}

/// Map or unmap the child and return the events that report it.
fn visibility_events(
    window: &mut Window,
    input: &mut Input,
    visibility: kivact_types::embed::Visibility,
) -> Result<Vec<ShellEvent>, ShellError> {
    let mut events = Vec::new();
    match visibility {
        kivact_types::embed::Visibility::Hidden => {
            window.native.hide()?;
            events.extend(input.blur_events());
        }
        kivact_types::embed::Visibility::Shown => {
            window.native.show()?;
            window.redraw.set(ChildRedraw::Needed);
        }
    }
    if let Some(event) = window.set_surface(visibility) {
        events.push(event);
    }
    if visibility == kivact_types::embed::Visibility::Shown {
        events.push(ShellEvent::Redraw);
    }
    Ok(events)
}
