//! Role: Answer resize requests now or after the host confirms the size.

use super::child::{Command, Shared};
use arvane_shell::{ShellError, ShellEvent, SizeReply};
use kivact_types::{Failure, FailureKind, PlatformResult, ResizeAnswer, Size};
use std::cell::RefCell;
use std::collections::VecDeque;

/// How long a `Pending` request waits for the host's size before it times out.
pub(in crate::port) const PENDING_RESIZE_WAIT_NS: u64 = 2_000_000_000;

/// The one deferred size request of a UI thread.
pub(in crate::port) enum ResizeWait {
    /// No request waits for the host.
    Idle,
    /// The host decides later and confirms with `Embedded::set_size`.
    Awaiting {
        /// The effect request answered by `ShellEvent::PlatformAnswer`.
        request: kivact_types::RequestId,
        /// The physical size the App asked for.
        size: (u32, u32),
        /// The host-size generation at the callback; a newer one answers it.
        generation: u64,
        /// When the wait ends, in the UI thread's monotonic nanoseconds.
        deadline_ns: u64,
    },
}

/// Ask the host for a physical size and say what it did.
///
/// The callback runs on the UI thread and may call `Embedded::set_size`, which
/// advances the host generation; a `Pending` request then resolves at once.
#[allow(
    clippy::too_many_arguments,
    reason = "one resize exchange needs the host state, its deadline and its answer queue together"
)]
pub(in crate::port) fn negotiate(
    size: Size,
    scale: f64,
    maximum: u32,
    callback: Option<&(dyn Fn(u32, u32) -> ResizeAnswer + Send + Sync)>,
    shared: &Shared,
    request: kivact_types::RequestId,
    now_ns: u64,
    wait: &RefCell<ResizeWait>,
    answers: &RefCell<VecDeque<ShellEvent>>,
    wake: impl FnOnce(),
) -> SizeReply {
    if shared.stopped() {
        return SizeReply::Now(PlatformResult::Done);
    }
    let (width, height) = match super::geometry::physical_size(size, scale, maximum) {
        Ok(size) => size,
        Err(error) => return SizeReply::Now(platform(error)),
    };
    let Some(callback) = callback else {
        return SizeReply::Now(PlatformResult::Unsupported);
    };
    let generation = shared.generation();
    cancel(wait, answers);
    let answer = callback(width, height);
    if shared.stopped() {
        return SizeReply::Now(PlatformResult::Done);
    }
    match answer {
        ResizeAnswer::Refused => SizeReply::Now(failed("the host refused the size")),
        ResizeAnswer::Accepted => {
            shared.queue(Command::SetSize {
                width,
                height,
                generation: Some(generation),
            });
            wake();
            SizeReply::Now(PlatformResult::Done)
        }
        ResizeAnswer::Pending => {
            if let Some(host) = shared.host_size_since(generation) {
                return SizeReply::Now(compare(host, (width, height)));
            }
            *wait.borrow_mut() = ResizeWait::Awaiting {
                request,
                size: (width, height),
                generation,
                deadline_ns: now_ns.saturating_add(PENDING_RESIZE_WAIT_NS),
            };
            SizeReply::Later
        }
    }
}

/// Answer a waiting request when the host chose a size or the deadline passed.
pub(in crate::port) fn settle(
    wait: &RefCell<ResizeWait>,
    shared: &Shared,
    answers: &RefCell<VecDeque<ShellEvent>>,
    now_ns: u64,
) {
    let awaiting = {
        let slot = wait.borrow_mut();
        match &*slot {
            ResizeWait::Idle => return,
            ResizeWait::Awaiting {
                request,
                size,
                generation,
                deadline_ns,
            } => (*request, *size, *generation, *deadline_ns),
        }
    };
    let (request, size, generation, deadline_ns) = awaiting;
    let result = if let Some(host) = shared.host_size_since(generation) {
        compare(host, size)
    } else if now_ns >= deadline_ns {
        PlatformResult::TimedOut
    } else {
        return;
    };
    *wait.borrow_mut() = ResizeWait::Idle;
    answers
        .borrow_mut()
        .push_back(ShellEvent::PlatformAnswer { request, result });
}

/// When the waiting request ends, if one waits.
pub(in crate::port) fn deadline(wait: &RefCell<ResizeWait>) -> Option<u64> {
    match &*wait.borrow() {
        ResizeWait::Idle => None,
        ResizeWait::Awaiting { deadline_ns, .. } => Some(*deadline_ns),
    }
}

/// Cancel a waiting request: its effect resolves as `Cancelled`.
fn cancel(wait: &RefCell<ResizeWait>, answers: &RefCell<VecDeque<ShellEvent>>) {
    let previous = std::mem::replace(&mut *wait.borrow_mut(), ResizeWait::Idle);
    if let ResizeWait::Awaiting { request, .. } = previous {
        answers.borrow_mut().push_back(ShellEvent::PlatformAnswer {
            request,
            result: PlatformResult::Cancelled,
        });
    }
}

/// The result of a host size compared with the requested one.
fn compare(host: (u32, u32), requested: (u32, u32)) -> PlatformResult {
    if host == requested {
        PlatformResult::Done
    } else {
        failed(format!("the host chose {} x {}", host.0, host.1))
    }
}

fn failed(message: impl Into<String>) -> PlatformResult {
    PlatformResult::Failed(Failure::new(FailureKind::Backend, message))
}

/// A platform error as the result the effect reports.
fn platform(error: ShellError) -> PlatformResult {
    match error {
        ShellError::Unsupported(_) => PlatformResult::Unsupported,
        ShellError::Failed(failure) => PlatformResult::Failed(failure),
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests of resize answers, host confirmation, cancellation and timeout.
    use super::{PENDING_RESIZE_WAIT_NS, ResizeWait, negotiate, settle};
    use crate::port::embed::child::Shared;
    use arvane_shell::{NoticeSink, PanicSlot, ShellEvent, SizeReply};
    use kivact_types::embed::Visibility;
    use kivact_types::{PlatformResult, RequestId, ResizeAnswer, Size};
    use std::cell::RefCell;
    use std::collections::VecDeque;

    fn shared() -> Shared {
        Shared::new(
            1,
            1,
            1.0,
            Visibility::Shown,
            PanicSlot::default(),
            NoticeSink::default(),
        )
    }

    fn size() -> Size {
        Size {
            width: 100.0,
            height: 50.0,
        }
    }

    fn run(
        answer: ResizeAnswer,
        shared: &Shared,
    ) -> (
        SizeReply,
        RefCell<ResizeWait>,
        RefCell<VecDeque<ShellEvent>>,
    ) {
        let wait = RefCell::new(ResizeWait::Idle);
        let answers = RefCell::new(VecDeque::new());
        let reply = negotiate(
            size(),
            1.0,
            1000,
            Some(&move |_, _| answer),
            shared,
            RequestId(1),
            0,
            &wait,
            &answers,
            || {},
        );
        (reply, wait, answers)
    }

    #[test]
    fn accepted_resizes_at_the_callback_generation() {
        let shared = shared();
        let (reply, _, _) = run(ResizeAnswer::Accepted, &shared);
        assert_eq!(reply, SizeReply::Now(PlatformResult::Done));
        assert_eq!(shared.generation(), 0);
    }

    #[test]
    fn refused_fails_with_a_platform_error() {
        let shared = shared();
        let (reply, _, _) = run(ResizeAnswer::Refused, &shared);
        assert!(matches!(reply, SizeReply::Now(PlatformResult::Failed(_))));
    }

    #[test]
    fn pending_resolves_when_the_host_confirms() {
        let shared = shared();
        let (reply, wait, answers) = run(ResizeAnswer::Pending, &shared);
        assert_eq!(reply, SizeReply::Later);
        shared.queue_host_size(100, 50);
        settle(&wait, &shared, &answers, 0);
        assert_eq!(
            answers.borrow_mut().pop_front(),
            Some(ShellEvent::PlatformAnswer {
                request: RequestId(1),
                result: PlatformResult::Done,
            })
        );
    }

    #[test]
    fn pending_times_out_and_a_new_request_cancels() {
        let shared = shared();
        let (_, wait, answers) = run(ResizeAnswer::Pending, &shared);
        settle(&wait, &shared, &answers, PENDING_RESIZE_WAIT_NS + 1);
        assert_eq!(
            answers.borrow_mut().pop_front(),
            Some(ShellEvent::PlatformAnswer {
                request: RequestId(1),
                result: PlatformResult::TimedOut,
            })
        );
        let wait = RefCell::new(ResizeWait::Idle);
        let first = RefCell::new(VecDeque::new());
        let _ = negotiate(
            size(),
            1.0,
            1000,
            Some(&|_, _| ResizeAnswer::Pending),
            &shared,
            RequestId(2),
            0,
            &wait,
            &first,
            || {},
        );
        let _ = negotiate(
            size(),
            1.0,
            1000,
            Some(&|_, _| ResizeAnswer::Pending),
            &shared,
            RequestId(3),
            0,
            &wait,
            &first,
            || {},
        );
        assert_eq!(
            first.borrow_mut().pop_front(),
            Some(ShellEvent::PlatformAnswer {
                request: RequestId(2),
                result: PlatformResult::Cancelled,
            })
        );
    }
}
