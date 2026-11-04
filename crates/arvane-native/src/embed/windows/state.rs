//! Role: UI-only callback state, separate from synchronous wndproc answers.
use super::native::Native;
use crate::embed::child::Shared;
use crate::embed::session::Session;
use arvane_shell::{ClickCounter, Mouse, PlatformTree, ShellEvent};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;
use windows_sys::Win32::UI::WindowsAndMessaging::WNDPROC;

/// Where the child is in its life. It is read on every message, including reentrant ones, so it
/// lives in a `Cell`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    /// `open` has not returned: events queue, nothing drains.
    Opening,
    /// The App runs and events drain.
    Live,
    /// The handler was closed and the child's resources released; the HWND may still exist.
    Disposed,
    /// After a panic: the handler and the State were forgotten (module retained); nothing is released.
    Leaked,
}

/// Who decides the child's scale. The change is one-way: once the host sets a scale, the system's
/// DPI no longer does (`ShellChild::set_scale` has no way back).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScaleSource {
    /// The system DPI of the window.
    System,
    /// The host chose the scale (at creation or by `set_scale`).
    Host,
}

/// The last CPU frame the window shows.
pub(super) struct CpuFrame {
    pub(super) pixels: Vec<u32>,
    pub(super) width: NonZeroU32,
    pub(super) height: NonZeroU32,
}

/// Holds a reentrancy flag for a scope; clears it on drop, also on unwind (the window procedure can
/// then stop the child after it caught a panic).
pub(super) struct Hold<'a>(&'a Cell<bool>);
impl<'a> Hold<'a> {
    /// `None` when the flag is already held.
    pub(super) fn enter(flag: &'a Cell<bool>) -> Option<Self> {
        (!flag.replace(true)).then_some(Self(flag))
    }
}
impl Drop for Hold<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

pub(super) struct State {
    pub(super) shared: Arc<Shared>,
    pub(super) native: Arc<Native>,
    pub(super) on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    pub(super) session: RefCell<Session>,
    pub(super) events: RefCell<VecDeque<ShellEvent>>,
    pub(super) phase: Cell<Phase>,
    /// App code (a handler callback) is on the stack.
    pub(super) in_callback: Cell<bool>,
    /// The host command queue is being drained.
    pub(super) draining: Cell<bool>,
    pub(super) original_proc: Cell<WNDPROC>,
    pub(super) cursor: Cell<kivact_types::Cursor>,
    pub(super) pending_surrogate: Cell<Option<u16>>,
    pub(super) mouse: Cell<Mouse>,
    pub(super) clicks: RefCell<ClickCounter>,
    pub(super) environment: RefCell<kivact_types::Environment>,
    pub(super) scale_source: Cell<ScaleSource>,
    pub(super) frame: RefCell<Option<CpuFrame>>,
    /// The buffer the next frame is drawn into (the last frame's allocation, reused).
    pub(super) scratch: RefCell<Vec<u32>>,
    pub(super) services: RefCell<crate::services::Services>,
    pub(super) input: crate::native_input::state::NativeInput,
    /// The child's accepted semantics, for its whole life.
    pub(super) a11y: PlatformTree,
    /// The UIA adapter, until the child is disposed.
    pub(super) uia: RefCell<Option<accesskit_windows::Adapter>>,
    pub(super) target: Arc<super::target::Target>,
    pub(super) started: Instant,
}
