//! Role: This module provides one-shot futures for host results.

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

/// UI-thread one-shot slot used for host results (focus, platform effects):
/// the host side calls [`Completion::complete`], the future side awaits the
/// slot. Clones share the slot. Complete it once; polling it after it
/// returned its value panics.
pub struct Completion<T>(Rc<RefCell<Slot<T>>>);

/// Where a [`Completion`] stands.
enum Slot<T> {
    /// Not completed; the waker of the awaiting future once it was polled.
    Waiting(Option<Waker>),
    /// Completed; the future has not taken the value yet.
    Ready(T),
    /// The future returned the value.
    Taken,
}

impl<T> Clone for Completion<T> {
    fn clone(&self) -> Self {
        Completion(self.0.clone())
    }
}

impl<T> Default for Completion<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Completion<T> {
    /// An empty slot.
    pub fn new() -> Self {
        Completion(Rc::new(RefCell::new(Slot::Waiting(None))))
    }

    /// Fill the slot and wake the awaiting future. A slot is completed once.
    pub fn complete(&self, v: T) {
        let previous = std::mem::replace(&mut *self.0.borrow_mut(), Slot::Ready(v));
        let Slot::Waiting(waker) = previous else {
            panic!("invariant: a Completion is completed once")
        };
        if let Some(w) = waker {
            w.wake();
        }
    }
}

impl<T> Future for Completion<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut slot = self.0.borrow_mut();
        match std::mem::replace(&mut *slot, Slot::Taken) {
            Slot::Ready(v) => Poll::Ready(v),
            Slot::Waiting(old) => {
                *slot = Slot::Waiting(Some(cx.waker().clone()));
                drop(slot);
                drop(old);
                Poll::Pending
            }
            Slot::Taken => panic!("`Completion` polled after it returned its value"),
        }
    }
}
