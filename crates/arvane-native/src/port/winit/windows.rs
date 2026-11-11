//! Role: The open windows of the native loop, by app id and by winit id.

use crate::port::winit::window::WinitWindow;
use arvane_shell::{InjectedInput, WindowInput};
use kivact_types::{RequestId, WindowId};
use std::collections::{BTreeMap, HashMap, VecDeque};
use winit::window::WindowId as WinitWindowId;

/// One open window and the input state the loop keeps for it.
pub(super) struct NativeWindow {
    pub(super) window: WinitWindow,
    pub(super) input: WindowInput,
    /// Injected input batches and the request each finishes.
    pub(super) injected: VecDeque<(Vec<InjectedInput>, RequestId)>,
    /// The parent of a modal window: closing it re-enables that parent.
    #[cfg(windows)]
    pub(super) modal_parent: Option<WindowId>,
}

/// The open windows of one running loop.
pub(super) struct WindowTable {
    by_id: BTreeMap<WindowId, NativeWindow>,
    ids: HashMap<WinitWindowId, WindowId>,
    // xtask-allow(state-fields): independent: filled when the handler opens a window during a callback and emptied by the runner once it returns; placement is fixed when the table is made, so windows wait to be shown on platforms with and without placement alike
    pending_show: Vec<WindowId>,
    // xtask-allow(state-fields): option: whether the platform honours explicit window placement (false on Wayland)
    placement: bool,
}

impl WindowTable {
    /// An empty table; `placement` is whether the platform honours explicit
    /// window placement (false on Wayland).
    pub(super) fn new(placement: bool) -> Self {
        Self {
            by_id: BTreeMap::new(),
            ids: HashMap::new(),
            pending_show: Vec::new(),
            placement,
        }
    }

    /// Add a window and its winit id.
    pub(super) fn insert(&mut self, id: WindowId, winit_id: WinitWindowId, window: NativeWindow) {
        self.ids.insert(winit_id, id);
        self.by_id.insert(id, window);
    }

    /// The window with `id`, if it is open.
    pub(super) fn get(&self, id: WindowId) -> Option<&NativeWindow> {
        self.by_id.get(&id)
    }

    /// The window with `id`, if it is open.
    pub(super) fn get_mut(&mut self, id: WindowId) -> Option<&mut NativeWindow> {
        self.by_id.get_mut(&id)
    }

    /// The app id of the window with `winit_id`.
    pub(super) fn by_winit(&self, winit_id: WinitWindowId) -> Option<WindowId> {
        self.ids.get(&winit_id).copied()
    }

    /// Remove a window and its winit id.
    pub(super) fn remove(&mut self, id: WindowId) -> Option<NativeWindow> {
        let window = self.by_id.remove(&id)?;
        self.ids.remove(&window.window.window.id());
        Some(window)
    }

    /// The open windows, ascending by id.
    pub(super) fn ids(&self) -> Vec<WindowId> {
        self.by_id.keys().copied().collect()
    }

    /// Show `id` when the current callback returns.
    pub(super) fn push_pending_show(&mut self, id: WindowId) {
        self.pending_show.push(id);
    }

    /// Take the windows to show once the current callback returned.
    pub(super) fn take_pending_show(&mut self) -> Vec<WindowId> {
        std::mem::take(&mut self.pending_show)
    }

    /// Whether the platform honours explicit window placement.
    pub(super) fn placement(&self) -> bool {
        self.placement
    }

    /// The first queued injection batch of the lowest open window id.
    pub(super) fn next_injected(&mut self) -> Option<(WindowId, Vec<InjectedInput>, RequestId)> {
        for (id, window) in &mut self.by_id {
            if let Some((inputs, request)) = window.injected.pop_front() {
                return Some((*id, inputs, request));
            }
        }
        None
    }
}
