//! Role: UI-only callback state, separate from synchronous wndproc answers.
use super::native::Native;
use crate::port::embed::child::Shared;
use crate::port::embed::session::Session;
use arvane_shell::{ClickCounter, ModifierState, Mouse, PlatformTree, ShellEvent};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use windows_sys::Win32::UI::WindowsAndMessaging::WNDPROC;

/// Who decides the child's scale. The change is one-way: once the host sets a scale, the system's
/// DPI no longer does (`ShellChild::set_scale` has no way back).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScaleSource {
    /// The system DPI of the window.
    System,
    /// The host chose the scale (at creation or by `set_scale`).
    Host,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Held {
    #[default]
    Free,
    Held,
}
#[derive(Default)]
pub(super) struct ReentryGuard(Cell<Held>);
#[derive(Default)]
pub(super) struct DrainGuard(Cell<Held>);
pub(super) struct Hold<'a>(&'a Cell<Held>);
impl ReentryGuard {
    pub(super) fn enter(&self) -> Option<Hold<'_>> {
        Hold::enter(&self.0)
    }
    pub(super) fn get(&self) -> bool {
        self.0.get() == Held::Held
    }
}
impl DrainGuard {
    pub(super) fn enter(&self) -> Option<Hold<'_>> {
        Hold::enter(&self.0)
    }
}
impl<'a> Hold<'a> {
    fn enter(flag: &'a Cell<Held>) -> Option<Self> {
        (flag.replace(Held::Held) == Held::Free).then(|| Self(flag))
    }
}
impl Drop for Hold<'_> {
    fn drop(&mut self) {
        self.0.set(Held::Free);
    }
}

#[derive(Default)]
pub(super) struct Utf16Decoder {
    // xtask-allow(state-fields): option: the unpaired high surrogate waiting for its pair; absent when none waits
    pending: Option<u16>,
    // xtask-allow(state-fields): option: the incomplete character waiting for its second unit; absent when none waits
    tail: Option<char>,
}
impl Utf16Decoder {
    pub(super) fn push(&mut self, unit: u16) -> Option<char> {
        if (0xd800..=0xdbff).contains(&unit) {
            return self
                .pending
                .replace(unit)
                .map(|_| char::REPLACEMENT_CHARACTER);
        }
        let text = match self.pending.take() {
            Some(previous) => String::from_utf16_lossy(&[previous, unit]),
            None => String::from_utf16_lossy(&[unit]),
        };
        let mut chars = text.chars();
        let first = chars.next();
        self.tail = chars.next();
        first
    }
    pub(super) fn take_tail(&mut self) -> Option<char> {
        self.tail.take()
    }
}

pub(super) enum Uia {
    Inactive,
    Active(accesskit_windows::Adapter),
}
impl Uia {
    pub(super) fn as_mut(&mut self) -> Option<&mut accesskit_windows::Adapter> {
        match self {
            Self::Inactive => None,
            Self::Active(adapter) => Some(adapter),
        }
    }
}

pub(super) struct State {
    pub(super) shared: Arc<Shared>,
    pub(super) native: Arc<Native>,
    pub(super) on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    pub(super) session: Session,
    pub(super) events: RefCell<VecDeque<ShellEvent>>,
    /// App code (a handler callback) is on the stack.
    pub(super) in_callback: ReentryGuard,
    /// The host command queue is being drained.
    pub(super) draining: DrainGuard,
    pub(super) original_proc: Cell<WNDPROC>,
    pub(super) cursor: Cell<kivact_types::Cursor>,
    pub(super) decoder: RefCell<Utf16Decoder>,
    pub(super) mouse: Cell<Mouse>,
    pub(super) modifiers: Cell<ModifierState>,
    pub(super) clicks: RefCell<ClickCounter>,
    pub(super) environment: RefCell<kivact_types::Environment>,
    pub(super) scale_source: Cell<ScaleSource>,
    pub(super) cpu:
        RefCell<crate::present::CpuSurface<Arc<super::target::Target>, Arc<super::target::Target>>>,
    pub(super) services: RefCell<crate::services::Services>,
    pub(super) input: crate::native_input::state::NativeInput,
    /// The child's accepted semantics, for its whole life.
    pub(super) a11y: PlatformTree,
    /// The UIA adapter, until the child is disposed.
    pub(super) uia: RefCell<Uia>,
    pub(super) target: Arc<super::target::Target>,
    pub(super) started: Instant,
}
