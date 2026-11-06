//! Role: Host-clock timers (`sleep`, `timeout`).
//!
//! In tests the clock is virtual and only moves with `advance`.

use std::future::Future;
use std::panic::Location;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::task::{Context, Poll};
use std::time::Duration;

use crate::Runtime;
use crate::{ErrorKind, UiError, raise};

enum SleepState {
    Unregistered { deadline: u64 },
    Registered { id: super::state::TimerId },
    Fired,
}

/// Future returned by [`sleep`].
pub(crate) struct Sleep {
    rt: Weak<Runtime>,
    state: SleepState,
    cancel: Rc<super::state::CancelQueue>,
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if matches!(self.state, SleepState::Fired) {
            return Poll::Ready(());
        }
        let Some(rt) = self.rt.upgrade() else {
            self.state = SleepState::Fired;
            return Poll::Ready(());
        };
        let mut scheduler = super::TaskState::of(&rt).time.borrow_mut();
        let key = match self.state {
            SleepState::Unregistered { deadline } => {
                if rt.now_ns() >= deadline {
                    self.state = SleepState::Fired;
                    return Poll::Ready(());
                }
                scheduler.next_id += 1;
                let id = scheduler.next_id;
                self.state = SleepState::Registered { id };
                (deadline, id)
            }
            SleepState::Registered { id } => {
                let key = scheduler
                    .timers
                    .keys()
                    .find(|(_, timer)| *timer == id)
                    .copied();
                let Some(key) = key else {
                    self.state = SleepState::Fired;
                    return Poll::Ready(());
                };
                if rt.now_ns() >= key.0 {
                    let fired = scheduler.timers.remove(&key);
                    self.state = SleepState::Fired;
                    drop(scheduler);
                    drop(fired);
                    return Poll::Ready(());
                }
                key
            }
            SleepState::Fired => return Poll::Ready(()),
        };
        let old = scheduler.timers.insert(key, cx.waker().clone());
        drop(scheduler);
        drop(old);
        rt.request_turn();
        Poll::Pending
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        if let SleepState::Registered { id } = self.state {
            self.cancel.push(id);
        }
    }
}

pub(crate) fn sleep_ns(rt: &Rc<Runtime>, ns: u64) -> Sleep {
    Sleep {
        rt: Rc::downgrade(rt),
        state: SleepState::Unregistered {
            deadline: rt.now_ns().saturating_add(ns),
        },
        cancel: super::TaskState::of(rt).time.borrow().cancel.clone(),
    }
}

/// Wait for `d` on the host clock (virtual in tests), counted from this call.
/// Must be created in a running App's frame; otherwise raises a located `WrongApp`.
#[track_caller]
pub fn sleep(d: Duration) -> impl Future<Output = ()> {
    let ns = d.as_nanos().min(u64::MAX as u128) as u64;
    sleep_ns(&timer_runtime("sleep", Location::caller()), ns)
}

fn timer_runtime(api: &'static str, at: &'static Location<'static>) -> Rc<Runtime> {
    Runtime::current().unwrap_or_else(|| {
        raise(
            ErrorKind::WrongApp,
            format!("`{api}` needs a running App"),
            at,
        )
    })
}

struct Timeout<F> {
    fut: Pin<Box<F>>,
    sleep: Pin<Box<Sleep>>,
}

impl<F: Future> Future for Timeout<F> {
    type Output = Result<F::Output, UiError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(v) = self.fut.as_mut().poll(cx) {
            return Poll::Ready(Ok(v));
        }
        match self.sleep.as_mut().poll(cx) {
            Poll::Ready(()) => {
                Poll::Ready(Err(UiError::new(ErrorKind::Timeout, "timed out", None)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Run `fut` with a deadline `d` after this call; `Err(Timeout)` if it passes first.
/// Must be created in a running App's frame; otherwise raises a located `WrongApp`.
#[track_caller]
pub fn timeout<F: Future>(d: Duration, fut: F) -> impl Future<Output = Result<F::Output, UiError>> {
    let ns = d.as_nanos().min(u64::MAX as u128) as u64;
    let sleep = sleep_ns(&timer_runtime("timeout", Location::caller()), ns);
    Timeout {
        fut: Box::pin(fut),
        sleep: Box::pin(sleep),
    }
}
