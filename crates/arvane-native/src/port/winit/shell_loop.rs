//! Role: The native loop as a shell handler sees it during one callback.

use crate::port::winit::open::create_window;
use crate::port::winit::windows::{NativeWindow, WindowTable};
use arvane_shell::{InjectedInput, ShellError, ShellLoop, ShellWaker, ShellWindow, WindowInput};
#[cfg(windows)]
use kivact_types::WindowMode;
use kivact_types::{Capabilities, RequestId, WindowId, WindowRequest};
use std::collections::VecDeque;
use std::time::Instant;
use winit::event_loop::ActiveEventLoop;

/// The native loop borrowed for one handler callback.
pub(super) struct NativeLoop<'a> {
    pub(super) el: &'a ActiveEventLoop,
    pub(super) table: &'a mut WindowTable,
    pub(super) started: Instant,
    pub(super) waker: &'a ShellWaker,
}

impl ShellLoop for NativeLoop<'_> {
    fn window(&mut self, id: WindowId) -> Option<&mut dyn ShellWindow> {
        self.table
            .get_mut(id)
            .map(|native| &mut native.window as &mut dyn ShellWindow)
    }

    fn windows(&self) -> Vec<WindowId> {
        self.table.ids()
    }

    fn open_window(&mut self, request: WindowRequest) -> Result<(), ShellError> {
        let parent = request.parent.and_then(|id| self.table.get(id));
        let window = create_window(
            self.el,
            &request,
            parent.map(|native| &native.window),
            self.waker,
            self.started,
        )?;
        let winit_id = window.window.id();
        #[cfg(windows)]
        let modal_parent = (request.mode == WindowMode::Modal)
            .then_some(request.parent)
            .flatten();
        self.table.insert(
            request.id,
            winit_id,
            NativeWindow {
                window,
                input: WindowInput::new(),
                injected: VecDeque::new(),
                #[cfg(windows)]
                modal_parent,
            },
        );
        self.table.push_pending_show(request.id);
        Ok(())
    }

    fn close_window(&mut self, id: WindowId) {
        let Some(window) = self.table.remove(id) else {
            return;
        };
        #[cfg(windows)]
        if let Some(parent) = window.modal_parent {
            use winit::platform::windows::WindowExtWindows;
            if let Some(parent) = self.table.get(parent) {
                parent.window.window.set_enable(true);
            }
        }
        drop(window);
    }

    fn inject(&mut self, id: WindowId, inputs: Vec<InjectedInput>, request: RequestId) {
        if let Some(native) = self.table.get_mut(id) {
            native.injected.push_back((inputs, request));
        }
    }

    fn now_ns(&self) -> u64 {
        self.started.elapsed().as_nanos().min(u64::MAX as u128) as u64
    }

    fn waker(&self) -> ShellWaker {
        self.waker.clone()
    }

    fn capabilities(&self) -> Capabilities {
        match self.table.get(WindowId::MAIN) {
            Some(main) => Capabilities {
                multi_window: true,
                window_placement: self.table.placement(),
                ..main.window.capabilities()
            },
            None => Capabilities {
                multi_window: true,
                window_placement: self.table.placement(),
                ..Capabilities::default()
            },
        }
    }
}
