//! Role: UI-only callback state, separate from synchronous wndproc answers.
use super::native::Native;
use crate::embed::child::Shared;
use crate::embed::session::Session;
use arvane_shell::{ClickCounter, Mouse, ShellEvent};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub(super) struct State {
    pub(super) shared: Arc<Shared>,
    pub(super) native: Arc<Native>,
    pub(super) on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    pub(super) session: RefCell<Session>,
    pub(super) events: RefCell<VecDeque<ShellEvent>>,
    pub(super) busy: Cell<bool>,
    pub(super) commands_active: Cell<bool>,
    pub(super) closing: Cell<bool>,
    pub(super) resources_leaked: Cell<bool>,
    pub(super) opening: Cell<bool>,
    pub(super) original_proc: Cell<WNDPROC>,
    pub(super) cursor: Cell<kivact_types::Cursor>,
    pub(super) pending_surrogate: Cell<Option<u16>>,
    pub(super) mouse: Cell<Mouse>,
    pub(super) clicks: RefCell<ClickCounter>,
    pub(super) environment: RefCell<kivact_types::Environment>,
    pub(super) explicit_scale: Cell<bool>,
    pub(super) pixels: RefCell<Vec<u32>>,
    pub(super) scratch: RefCell<Vec<u32>>,
    pub(super) pixel_size: Cell<(u32, u32)>,
    pub(super) cpu_shown: Cell<bool>,
    pub(super) services: RefCell<crate::services::Services>,
    pub(super) input: crate::native_input::state::NativeInput,
    pub(super) a11y: RefCell<Option<super::bridge::Bridge>>,
    pub(super) target: Arc<super::target::Target>,
    pub(super) started: Instant,
}
