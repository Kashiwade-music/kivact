//! Role: Adapt runtime frame loops to the public node presence contract.

use crate::view::node_ref::NodeRef;
use kivact_runtime::{FrameInfo, FrameRate, HandlerResult, Value};

/// Copy handle of a frame loop owned by the current setup owner.
#[derive(Clone, Copy)]
pub struct FrameLoop {
    inner: kivact_runtime::FrameLoop,
}

/// Create a stopped frame callback, released with the current setup owner.
/// Setup only; callback errors reach the nearest error handler.
#[track_caller]
pub fn frame_loop<R: HandlerResult>(f: impl Fn(FrameInfo) -> R + 'static) -> FrameLoop {
    FrameLoop {
        inner: kivact_runtime::frame_loop(f),
    }
}

impl FrameLoop {
    /// Run callbacks only while `node` is present, resuming the requested rate
    /// when it becomes present again. Entering, leaving and unbound nodes pause
    /// scheduling without changing the requested start/stop state. Setup only.
    #[track_caller]
    pub fn while_present(self, node: NodeRef) -> Self {
        let presence = node.presence();
        let gate = kivact_runtime::memo(move || presence.get() == kivact_types::Presence::Present);
        Self {
            inner: self.inner.with_presence_gate(gate),
        }
    }

    /// Start the loop or change its requested rate.
    #[track_caller]
    pub fn start(&self, rate: FrameRate) {
        self.inner.start(rate);
    }

    /// Stop callbacks until the next start, including across presence changes.
    #[track_caller]
    pub fn stop(&self) {
        self.inner.stop();
    }

    /// The requested rate, or `None` while stopped.
    #[track_caller]
    pub fn rate(&self) -> Value<Option<FrameRate>> {
        self.inner.rate()
    }
}
