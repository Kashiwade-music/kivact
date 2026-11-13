//! Role: Child window options, host notices and the host event loop SPI.

use crate::{ShellError, ShellHandler};
use kivact_types::ResizeAnswer;
use kivact_types::embed::Visibility;
use std::any::Any;
use std::sync::{Arc, Mutex, MutexGuard};

/// A push notification an embedded child sends its host (see
/// `kivact::embed::Parent::on_notice`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EmbedNotice {
    /// The applied scale changed for a reason other than the host's
    /// `set_scale`: an operating-system scale change (Win32
    /// `WM_DPICHANGED_AFTERPARENT`). The host keeps its logical size by asking
    /// for `logical size * scale`. X11 has one scale (Xft), so it never sends
    /// this.
    ScaleChanged(f64),
    /// The child stopped: the host destroyed the parent, a platform failure
    /// stopped the App, or it panicked. Sent once, when the child's stop
    /// status leaves running; read the stop with `ShellChild::stop`.
    Stopped,
}

/// Delivers [`EmbedNotice`]s to the host's callback.
///
/// Clones share the callback. A sink without a callback drops every notice.
#[derive(Clone, Default)]
pub struct NoticeSink {
    callback: Option<Arc<dyn Fn(EmbedNotice) + Send + Sync>>,
}

impl NoticeSink {
    /// Send to `callback`, or nowhere when it is `None`.
    pub fn new(callback: Option<Arc<dyn Fn(EmbedNotice) + Send + Sync>>) -> Self {
        Self { callback }
    }

    /// Announce that the child stopped.
    pub fn stopped(&self) {
        self.send(EmbedNotice::Stopped);
    }

    /// Announce an operating-system scale change.
    pub fn scale_changed(&self, scale: f64) {
        self.send(EmbedNotice::ScaleChanged(scale));
    }

    fn send(&self, notice: EmbedNotice) {
        if let Some(callback) = &self.callback {
            callback(notice);
        }
    }
}

impl std::fmt::Debug for NoticeSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NoticeSink")
            .field(&self.callback.is_some())
            .finish()
    }
}

/// The panic of an embedded child's App, kept where it reached a platform
/// boundary as its message for the embedder to read.
///
/// Clones share the slot. It keeps the first payload's message; a later
/// payload is ignored. The payload itself is forgotten when it is put, so its
/// `Drop` never runs: a panic is a bug, and the embedder reads the message as
/// a value. Kivact never resumes the panic on the embedder's thread.
#[derive(Clone, Default)]
pub struct PanicSlot(Arc<Mutex<Option<String>>>);

impl PanicSlot {
    /// Keep the message of `payload` unless a message is already kept; the
    /// payload is forgotten either way. Returns whether this message was kept.
    pub fn put(&self, payload: Box<dyn Any + Send>) -> bool {
        let message = message_of(payload.as_ref());
        let kept = {
            let mut slot = self.lock();
            if slot.is_none() {
                *slot = Some(message);
                true
            } else {
                false
            }
        };
        // The payload's Drop may panic or wait; the message above is all
        // Kivact keeps, and a panic is reported as a value.
        std::mem::forget(payload);
        kept
    }

    /// The kept panic message, if any.
    pub fn message(&self) -> Option<String> {
        self.lock().clone()
    }

    /// Whether a message is kept.
    pub fn is_set(&self) -> bool {
        self.lock().is_some()
    }

    fn lock(&self) -> MutexGuard<'_, Option<String>> {
        self.0
            .lock()
            .expect("invariant: the panic slot lock is never held while panicking")
    }
}

/// The message of a panic payload: `&str` and `String` as they are, anything
/// else as a fixed sentence.
fn message_of(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "the embedded App panicked with a non-string payload".to_owned()
    }
}

impl std::fmt::Debug for PanicSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PanicSlot").field(&self.is_set()).finish()
    }
}

/// Why an embedded child stopped (`ShellChild::stop`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChildStop {
    /// The host destroyed the parent (or the child window) first. Not an error.
    Closed,
    /// A platform failure stopped the App and destroyed its window.
    Failed(ShellError),
    /// The App panicked (a bug); its message is in the child's [`PanicSlot`].
    Panicked,
}

/// Callback and panic state owned by the shell, separate from pure options.
#[derive(Clone, Default)]
pub struct ChildHooks {
    /// Ask the embedding host for a physical content size, and say whether it
    /// accepted now, decides later, or refused.
    pub on_resize_request: Option<Arc<dyn Fn(u32, u32) -> ResizeAnswer + Send + Sync>>,
    /// Receive notices from the child. Called on the thread that observes the
    /// change (normally the child's UI thread; X11 may report from the editor
    /// thread while it exits, and a close started by a handle drop may report
    /// from the dropping thread); it must not wait for the host's GUI thread
    /// and must not panic.
    pub on_notice: Option<Arc<dyn Fn(EmbedNotice) + Send + Sync>>,
    /// The existing shared panic message slot.
    pub panic: PanicSlot,
}

/// Builds the window's handler on its UI thread, once.
pub type MakeHandler = Box<dyn FnOnce() -> Box<dyn ShellHandler> + Send>;

/// A child in a host event loop. Drop closes it using the backend's thread rules.
pub trait ShellChild: Send + Sync {
    /// Resize in physical pixels.
    fn set_size(&self, width: u32, height: u32);
    /// Last applied physical size.
    fn size(&self) -> (u32, u32);
    /// Change scale, keeping the physical size.
    fn set_scale(&self, scale: f64);
    /// Last applied scale.
    fn scale(&self) -> f64;
    /// Show or hide the window. Queued; the getter returns the applied value.
    fn set_visibility(&self, visibility: Visibility);
    /// Last applied visibility (a queued change is not reflected yet).
    fn visibility(&self) -> Visibility;
    /// Why the window stopped, or `None` while running.
    fn stop(&self) -> Option<ChildStop>;
}

#[cfg(test)]
mod tests {
    //! Role: Tests of panic payload messages and notice delivery.
    use super::{EmbedNotice, NoticeSink, PanicSlot};
    use std::sync::{Arc, Mutex};

    struct Explosive(Arc<Mutex<bool>>);

    impl Drop for Explosive {
        fn drop(&mut self) {
            *self.0.lock().expect("test lock is not poisoned") = true;
        }
    }

    #[test]
    fn panic_payloads_become_messages_and_keep_the_first() {
        let slot = PanicSlot::default();
        assert!(slot.put(Box::new("first")));
        assert!(!slot.put(Box::new("second".to_owned())));
        assert_eq!(slot.message().as_deref(), Some("first"));
        assert!(slot.is_set());
    }

    #[test]
    fn a_payload_whose_drop_runs_code_is_forgotten() {
        let dropped = Arc::new(Mutex::new(false));
        let slot = PanicSlot::default();
        slot.put(Box::new(Explosive(dropped.clone())));
        assert!(
            !*dropped.lock().expect("test lock is not poisoned"),
            "the payload's Drop must not run"
        );
        assert_eq!(
            slot.message().as_deref(),
            Some("the embedded App panicked with a non-string payload")
        );
    }

    #[test]
    fn notices_reach_the_callback_in_order() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = {
            let seen = seen.clone();
            NoticeSink::new(Some(Arc::new(move |notice| {
                seen.lock().expect("test lock is not poisoned").push(notice);
            })))
        };
        sink.stopped();
        sink.scale_changed(2.0);
        assert_eq!(
            *seen.lock().expect("test lock is not poisoned"),
            [EmbedNotice::Stopped, EmbedNotice::ScaleChanged(2.0)]
        );
        NoticeSink::default().stopped();
    }
}
