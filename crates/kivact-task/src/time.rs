//! Role: Host-clock timers: `sleep`, `timeout` and the App's store of pending timers.
//!
//! In tests the clock is virtual and only moves with `advance`.

use std::collections::BTreeMap;
use std::future::Future;
use std::panic::Location;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use kivact_reactive::Runtime;
use kivact_reactive::{ErrorKind, UiError, raise};

/// Future returned by [`sleep`].
pub(crate) struct Sleep {
    rt: Weak<Runtime>,
    duration: u64,
    deadline: Option<u64>,
    id: Option<(u64, u64)>,
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let Some(rt) = self.rt.upgrade() else {
            return Poll::Ready(());
        };
        let now = rt.now_ns();
        let mut t = super::TaskState::of(&rt).time.borrow_mut();
        let dur = self.duration;
        let deadline = *self.deadline.get_or_insert(now.saturating_add(dur));
        if now >= deadline {
            if let Some(k) = self.id.take() {
                t.timers.remove(&k);
            }
            return Poll::Ready(());
        }
        let key = match self.id {
            Some(k) => k,
            None => {
                t.next_id += 1;
                let k = (deadline, t.next_id);
                self.id = Some(k);
                k
            }
        };
        t.timers.insert(key, cx.waker().clone());
        drop(t);
        rt.request_turn();
        Poll::Pending
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        if let (Some(k), Some(rt)) = (self.id.take(), self.rt.upgrade())
            && let Ok(mut t) = super::TaskState::of(&rt).time.try_borrow_mut()
        {
            t.timers.remove(&k);
        }
    }
}

pub(crate) fn sleep_ns(rt: &Rc<Runtime>, ns: u64) -> Sleep {
    Sleep {
        rt: Rc::downgrade(rt),
        duration: ns,
        deadline: None,
        id: None,
    }
}

/// Wait for `d` on the host clock (virtual in tests).
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

/// Run `fut` with a deadline; `Err(Timeout)` if `d` passes first.
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

/// The pending host-clock timers of an App, by deadline and arrival.
#[derive(Default)]
pub(crate) struct TimeState {
    pub(crate) timers: BTreeMap<(u64, u64), Waker>,
    pub(crate) next_id: u64,
}

impl TimeState {
    pub(crate) fn next_deadline(&self) -> Option<u64> {
        self.timers.keys().next().map(|k| k.0)
    }

    pub(crate) fn len(&self) -> usize {
        self.timers.len()
    }

    /// Remove and return wakers of timers due at `now` (ns).
    pub(crate) fn take_due(&mut self, now: u64) -> Vec<Waker> {
        let due: Vec<(u64, u64)> = self.timers.range(..(now + 1, 0)).map(|(k, _)| *k).collect();
        due.into_iter()
            .filter_map(|k| self.timers.remove(&k))
            .collect()
    }
}
