//! Role: What differs between a window run, an embedded run and a test run.

use std::rc::Rc;

/// How a launch treats closing, DevTools and test timing. Built once by
/// `App::run`, `App::embed` and kivact-test.
pub struct LaunchPolicy {
    pub(crate) close: ClosePolicy,
    /// Whether the DevTools agent may start.
    pub(crate) devtools: bool,
    pub(crate) test: Option<TestRun>,
}

/// Who may end the run.
pub(crate) enum ClosePolicy {
    /// `AppHandle::close` and the main window's close end it.
    Accept,
    /// The embedder ends it by dropping `Embedded`; `AppHandle::close` is
    /// reported as unsupported. The signal tells that the host is closing.
    Embedded(kivact_engine::contract::CloseSignal),
}

/// A kivact-test run: its turn timing reporter and the test's name (in debug
/// builds the trace file is named after it).
pub(crate) struct TestRun {
    pub(crate) timing: Rc<dyn Fn(u64)>,
    #[cfg(debug_assertions)]
    pub(crate) name: String,
}

impl LaunchPolicy {
    pub(crate) fn window() -> Self {
        Self {
            close: ClosePolicy::Accept,
            devtools: true,
            test: None,
        }
    }
    pub(crate) fn embedded(closing: kivact_engine::contract::CloseSignal) -> Self {
        Self {
            close: ClosePolicy::Embedded(closing),
            devtools: true,
            test: None,
        }
    }
    /// A kivact-test run.
    pub fn test(name: String, timing: Rc<dyn Fn(u64)>, devtools: bool) -> Self {
        // Only debug builds write the trace, which is named after the test.
        #[cfg(not(debug_assertions))]
        let _ = name;
        Self {
            close: ClosePolicy::Accept,
            devtools,
            test: Some(TestRun {
                timing,
                #[cfg(debug_assertions)]
                name,
            }),
        }
    }
    /// A window run without a test harness (kivact-test's own launch tests).
    pub fn plain_window() -> Self {
        Self::window()
    }
    pub(crate) fn is_embedded(&self) -> bool {
        matches!(self.close, ClosePolicy::Embedded(_))
    }
    /// Whether the embedded host asked to close (never for other runs).
    pub(crate) fn closing(&self) -> bool {
        matches!(&self.close, ClosePolicy::Embedded(closing) if closing.is_set())
    }
}
