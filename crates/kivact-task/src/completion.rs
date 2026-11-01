//! Role: This module provides one-shot futures for host results.

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

/// UI-thread one-shot slot used for host results (focus, platform effects):
/// the host side calls [`Completion::complete`], the future side awaits the
/// slot. Clones share the slot.
pub struct Completion<T>(Rc<RefCell<(Option<T>, Option<Waker>)>>);

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
        Completion(Rc::new(RefCell::new((None, None))))
    }

    /// Fill the slot and wake the awaiting future.
    pub fn complete(&self, v: T) {
        let w = {
            let mut s = self.0.borrow_mut();
            s.0 = Some(v);
            s.1.take()
        };
        if let Some(w) = w {
            w.wake();
        }
    }
}

impl<T> Future for Completion<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut s = self.0.borrow_mut();
        match s.0.take() {
            Some(v) => Poll::Ready(v),
            None => {
                s.1 = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
