//! Role: Implements embedded X11 window services and synchronous presentation.

use super::{
    conn::Native,
    notifier::{MAX_DIMENSION, Notifier},
};
use crate::embed::{
    child::{Backend as _, Shared},
    session::negotiate,
};
use arvane_shell::{ShellError, ShellWindow};
use kivact_types::{Capabilities, Cursor, Environment, Rect, Size};
use std::{
    cell::{Cell, RefCell},
    num::NonZeroU32,
    sync::Arc,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection,
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
};

pub(super) struct Window {
    pub(super) shared: Arc<Shared>,
    pub(super) backend: Arc<Notifier>,
    pub(super) opening: bool,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) scale: f32,
    on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    pub(super) redraw: Cell<bool>,
    started: Instant,
    generation: u64,
    cpu: Option<softbuffer::Surface<Arc<Native>, Arc<Native>>>,
    shown_cpu: bool,
    pub(super) accessibility: super::a11y::Bridge,
    services: super::services::Services,
    pub(super) ime: RefCell<super::ime::Ime>,
    pub(super) dnd: super::dnd::Dnd,
    cursors: super::cursor::Cursors,
    // The child is destroyed only after presentation and service resources.
    pub(super) native: Arc<Native>,
}

impl Window {
    pub(super) fn new(
        native: Arc<Native>,
        shared: Arc<Shared>,
        backend: Arc<Notifier>,
        size: (u32, u32),
        scale: f32,
        on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
        waker: arvane_shell::ShellWaker,
    ) -> Result<Self, ShellError> {
        let (width, height) = size;
        shared.update(width, height, scale);
        let ime = super::ime::Ime::new(&native);
        let dnd = super::dnd::Dnd::new(&native)?;
        Ok(Self {
            native,
            shared,
            backend,
            opening: true,
            width,
            height,
            scale,
            on_resize_request,
            redraw: Cell::new(true),
            started: Instant::now(),
            generation: 1,
            cpu: None,
            shown_cpu: false,
            accessibility: super::a11y::Bridge::new(f64::from(scale), waker),
            services: super::services::Services::default(),
            ime,
            dnd,
            cursors: super::cursor::Cursors::default(),
        })
    }
    pub(super) fn resized(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.shared.update(width, height, self.scale);
        self.redraw.set(true);
    }
    pub(super) fn resize(&self, width: u32, height: u32) -> Result<(), ShellError> {
        debug_assert!(width > 0 && height > 0 && width <= MAX_DIMENSION && height <= MAX_DIMENSION);
        let result = self
            .native
            .connection
            .configure_window(
                self.native.id.get(),
                &ConfigureWindowAux::new().width(width).height(height),
            )
            .map_err(super::error::platform)?
            .check();
        self.native.settle(result)?;
        self.native
            .connection
            .flush()
            .map_err(super::error::platform)
    }
}

impl ShellWindow for Window {
    fn request_size(&mut self, size: Size) -> Result<(), ShellError> {
        debug_assert!(!self.opening);
        negotiate(
            size,
            self.scale,
            MAX_DIMENSION,
            self.on_resize_request.as_deref(),
            &self.shared,
            || self.backend.wake(),
        )
    }

    fn copy_text(&mut self, text: &str) -> Result<(), ShellError> {
        self.services.copy(text)
    }
    fn read_text(&mut self) -> Result<String, ShellError> {
        self.services.read()
    }
    fn pick_file(
        &mut self,
        request: &arvane_shell::ShellFileDialog,
    ) -> Result<Option<std::path::PathBuf>, ShellError> {
        super::services::Services::pick_file(&self.native, request)
    }
    fn pick_files(
        &mut self,
        request: &arvane_shell::ShellFileDialog,
    ) -> Result<Option<Vec<std::path::PathBuf>>, ShellError> {
        super::services::Services::pick_files(&self.native, request)
    }
    fn save_file(
        &mut self,
        request: &arvane_shell::ShellFileDialog,
    ) -> Result<Option<std::path::PathBuf>, ShellError> {
        super::services::Services::save_file(&self.native, request)
    }
    fn pick_folder(
        &mut self,
        request: &arvane_shell::ShellFileDialog,
    ) -> Result<Option<std::path::PathBuf>, ShellError> {
        super::services::Services::pick_folder(&self.native, request)
    }
    fn open_path(&mut self, path: &std::path::Path) -> Result<(), ShellError> {
        crate::open::open_path(path)
    }
    fn reveal_path(&mut self, path: &std::path::Path) -> Result<(), ShellError> {
        crate::open::reveal_path(path)
    }
    fn recover_surface(&mut self) -> Result<(), ShellError> {
        self.release_cpu_surface();
        self.generation += 1;
        Ok(())
    }
    fn shows_cpu_frame(&self) -> bool {
        self.shown_cpu
    }
    fn release_cpu_surface(&mut self) {
        self.cpu = None;
        self.shown_cpu = false;
    }
    fn key(&self) -> u64 {
        u64::from(self.native.id.get())
    }
    fn replace_accessibility(
        &mut self,
        tree: &kivact_types::SemanticsTree,
    ) -> Result<(), arvane_shell::AccessibilityError> {
        self.accessibility.tree.replace(tree)
    }
    fn update_accessibility(
        &mut self,
        update: &kivact_types::SemanticsUpdate,
    ) -> Result<(), arvane_shell::AccessibilityError> {
        self.accessibility.tree.update(update).map(|_| ())
    }
    fn accessibility_active(&self) -> bool {
        self.accessibility.tree.active()
    }
    fn size(&self) -> Size {
        Size {
            width: self.width as f32 / self.scale,
            height: self.height as f32 / self.scale,
        }
    }
    fn physical_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn scale(&self) -> f32 {
        self.scale
    }
    fn now_ns(&self) -> u64 {
        self.started.elapsed().as_nanos().min(u64::MAX.into()) as u64
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            native_accessibility: true,
            ..Capabilities::default()
        }
    }
    fn surface_generation(&self) -> Option<u64> {
        Some(self.generation)
    }
    fn request_redraw(&self) {
        self.redraw.set(true);
    }
    fn set_title(&self, _: &str) {}
    fn set_min_size(&self, _: Option<Size>) {}
    fn reduced_motion(&self) -> bool {
        false
    }
    fn environment(&self) -> Environment {
        Environment {
            scale: self.scale,
            locale: sys_locale::get_locale(),
            ..Environment::default()
        }
    }
    fn set_fullscreen(&self, _: bool) -> Result<(), ShellError> {
        Err(unsupported("fullscreen"))
    }
    fn fullscreen(&self) -> bool {
        false
    }
    fn set_cursor(&self, cursor: Cursor) {
        if let Err(error) = self.cursors.set(&self.native, cursor) {
            self.shared.fail(error);
        }
    }
    fn set_ime_allowed(&self, allowed: bool) {
        if let Err(error) = self.ime.borrow_mut().allow(allowed) {
            self.shared.fail(error);
        }
    }
    fn set_soft_keyboard(&self, _: bool) {}
    fn set_ime_cursor_area(&self, area: Rect) {
        if let Err(error) = self.ime.borrow_mut().cursor(area, self.scale) {
            self.shared.fail(error);
        }
    }
    fn set_ime_text(&self, _: Option<arvane_shell::ShellImeText>) {}
    fn frame_period_ns(&self) -> u64 {
        arvane_shell::DEFAULT_FRAME_PERIOD_NS
    }
    fn present_with(
        &mut self,
        width: u32,
        height: u32,
        draw: &mut dyn FnMut(&mut [u32]) -> bool,
    ) -> Result<bool, ShellError> {
        if width == 0 || height == 0 {
            return Ok(false);
        }
        if (width, height) != (self.width, self.height) {
            return Err(super::error::platform(
                "CPU frame dimensions differ from the window",
            ));
        }
        if self.cpu.is_none() {
            let context =
                softbuffer::Context::new(self.native.clone()).map_err(super::error::platform)?;
            self.cpu = Some(
                softbuffer::Surface::new(&context, self.native.clone())
                    .map_err(super::error::platform)?,
            );
        }
        let surface = self
            .cpu
            .as_mut()
            .expect("invariant: CPU surface created above");
        surface
            .resize(
                NonZeroU32::new(width).expect("invariant: nonzero width"),
                NonZeroU32::new(height).expect("invariant: nonzero height"),
            )
            .map_err(super::error::platform)?;
        let mut buffer = surface.buffer_mut().map_err(super::error::platform)?;
        if !draw(&mut buffer) {
            return Ok(false);
        }
        buffer.present().map_err(super::error::platform)?;
        // softbuffer queues XCB image requests on this shared connection. Send
        // them now; the host event loop may otherwise wait without another request.
        self.native
            .connection
            .flush()
            .map_err(super::error::platform)?;
        self.shown_cpu = true;
        Ok(true)
    }
    fn with_gpu_target(
        &self,
        make: Box<dyn FnOnce(arvane_shell::GpuTarget) + Send>,
    ) -> Result<(), ShellError> {
        make(self.native.clone());
        Ok(())
    }
    fn wait_until(&mut self, ready: &mut dyn FnMut() -> bool, timeout: Duration) -> bool {
        crate::wait::until(ready, timeout)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        self.cursors.release(&self.native);
    }
}

fn unsupported(service: &str) -> ShellError {
    ShellError::unsupported(format!("{service} is unavailable for embedded X11 windows"))
}
