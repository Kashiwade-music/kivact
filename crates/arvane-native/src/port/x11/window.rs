//! Role: Implements embedded X11 window services and synchronous presentation.

use super::conn::Native;
use super::notifier::{MAX_DIMENSION, Notifier};
use crate::port::embed::child::{ChildPort as _, Shared};
use crate::port::embed::session::{Session, negotiate};
use crate::present::CpuSurface;
use arvane_shell::{ShellError, ShellWindow};
use kivact_types::Environment;
use kivact_types::{Capabilities, Cursor, Rect, Size};
use kivact_types::{PlatformRequest, PlatformResult};
use std::cell::{Cell, RefCell};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use x11rb::connection::Connection;
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ChildRedraw {
    Clean,
    Needed,
}

pub(super) struct Window {
    pub(super) shared: Arc<Shared>,
    pub(super) backend: Arc<Notifier>,
    pub(super) session: Rc<Session>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) scale: f64,
    /// The frame period (ns) of the display the window's center is on.
    frame_period_ns: u64,
    on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    pub(super) redraw: Cell<ChildRedraw>,
    started: Instant,
    cpu: CpuSurface<Arc<Native>, Arc<Native>>,
    pub(super) accessibility: super::a11y::Bridge,
    services: super::services::Services,
    pub(super) ime: RefCell<super::ime::Ime>,
    pub(super) ime_driver: RefCell<arvane_shell::pipeline::ImeDriver>,
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
        scale: f64,
        on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
        waker: arvane_shell::ShellWaker,
    ) -> Result<Self, ShellError> {
        let (width, height) = size;
        shared.update(width, height, scale);
        let ime = super::ime::Ime::new(&native);
        let dnd = super::dnd::Dnd::new(&native)?;
        let frame_period_ns = randr_frame_period(&native, (width, height));
        Ok(Self {
            native,
            session: Rc::new(Session::new(shared.clone())),
            shared,
            backend,
            width,
            height,
            scale,
            frame_period_ns,
            on_resize_request,
            redraw: Cell::new(ChildRedraw::Needed),
            started: Instant::now(),
            cpu: CpuSurface::Released,
            accessibility: super::a11y::Bridge::new(scale, waker),
            services: super::services::Services::default(),
            ime,
            ime_driver: RefCell::default(),
            dnd,
            cursors: super::cursor::Cursors::default(),
        })
    }
    pub(super) fn resized(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.shared.update(width, height, self.scale);
        self.redraw.set(ChildRedraw::Needed);
    }
    /// Read the frame period again after a configure; `true` when it changed.
    pub(super) fn refresh_frame_period(&mut self) -> bool {
        let period = randr_frame_period(&self.native, (self.width, self.height));
        let changed = period != self.frame_period_ns;
        self.frame_period_ns = period;
        changed
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
    fn perform(&mut self, request: PlatformRequest) -> PlatformResult {
        match request {
            PlatformRequest::SaveState { .. } => return PlatformResult::Done,
            PlatformRequest::CopyText(text) => {
                self.services.copy(&text).map(|()| PlatformResult::Done)
            }
            PlatformRequest::ReadText => self.services.read().map(PlatformResult::Text),
            PlatformRequest::PickFile(request) => {
                super::services::Services::pick_file(&self.native, &request)
                    .map(PlatformResult::Path)
            }
            PlatformRequest::PickFiles(request) => {
                super::services::Services::pick_files(&self.native, &request)
                    .map(PlatformResult::Paths)
            }
            PlatformRequest::SaveFile(request) => {
                super::services::Services::save_file(&self.native, &request)
                    .map(PlatformResult::Path)
            }
            PlatformRequest::PickFolder(request) => {
                super::services::Services::pick_folder(&self.native, &request)
                    .map(PlatformResult::Path)
            }
            PlatformRequest::OpenPath(path) => {
                crate::open::open_path(&path).map(|()| PlatformResult::Done)
            }
            PlatformRequest::RevealPath(path) => {
                crate::open::reveal_path(&path).map(|()| PlatformResult::Done)
            }
            PlatformRequest::RequestWindowSize(size) => {
                debug_assert!(!self.session.opening());
                negotiate(
                    size,
                    self.scale,
                    MAX_DIMENSION,
                    self.on_resize_request.as_deref(),
                    &self.shared,
                    || self.backend.wake(),
                )
                .map(|()| PlatformResult::Done)
            }
            PlatformRequest::Fullscreen(_) => return PlatformResult::Unsupported,
        }
        .unwrap_or_else(|error| match error {
            ShellError::Unsupported(_) => PlatformResult::Unsupported,
            ShellError::Failed(failure) => PlatformResult::Failed(failure),
        })
    }
    fn cpu_format(&self) -> arvane_shell::CpuFormat {
        arvane_shell::CpuFormat::Xrgb8888
    }

    fn keeps_frames(&self) -> bool {
        false
    }
    fn keep_frame(&mut self, _frame: arvane_shell::CpuFrame) {
        unreachable!("invariant: native windows do not retain presented frames");
    }

    fn recover_surface(&mut self) -> Result<(), ShellError> {
        self.release_cpu_surface();
        Ok(())
    }
    fn shows_cpu_frame(&self) -> bool {
        self.cpu.shows_frame()
    }
    fn release_cpu_surface(&mut self) {
        self.cpu.release();
    }
    fn replace_accessibility(
        &mut self,
        tree: &kivact_types::render::SemanticsTree,
    ) -> Result<(), arvane_shell::AccessibilityError> {
        self.accessibility.tree.replace(tree)
    }
    fn update_accessibility(
        &mut self,
        update: &kivact_types::render::SemanticsUpdate,
    ) -> Result<(), arvane_shell::AccessibilityError> {
        self.accessibility.tree.update(update).map(|_| ())
    }
    fn size(&self) -> Size {
        Size {
            width: self.width as f64 / self.scale,
            height: self.height as f64 / self.scale,
        }
    }
    fn physical_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn scale(&self) -> f64 {
        self.scale
    }
    fn now_ns(&self) -> u64 {
        self.started.elapsed().as_nanos().min(u64::MAX.into()) as u64
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            state_restoration: false,
            clipboard_text: true,
            file_dialogs: true,
            open_path: true,
            reveal_path: true,
            fullscreen: false,
            min_size: false,
            command_key: kivact_types::CommandKey::Ctrl,
            context_menu_on: kivact_types::ContextMenuTrigger::Release,
            native_accessibility: true,
            ..Capabilities::default()
        }
    }
    fn surface_generation(&self) -> Option<u64> {
        // The X window is never re-created, so its surface keeps the first generation.
        Some(1)
    }
    fn request_redraw(&self) {
        self.redraw.set(ChildRedraw::Needed);
    }
    fn set_title(&self, _: &str) {}
    fn set_min_size(&self, _: Option<Size>) {}
    fn reduced_motion(&self) -> bool {
        crate::native_input::reduced_motion()
    }
    fn environment(&self) -> Environment {
        Environment {
            scale_factor: self.scale,
            window_size: self.size(),
            locale: sys_locale::get_locale(),
            frame_period_ns: self.frame_period_ns,
            reduced_motion: self.reduced_motion(),
            ..Environment::default()
        }
    }

    fn fullscreen(&self) -> bool {
        false
    }
    fn set_cursor(&self, cursor: Cursor) {
        if let Err(error) = self.cursors.set(&self.native, cursor) {
            self.shared.fail(error);
        }
    }
    fn set_ime_purpose(&self, _purpose: kivact_types::InputPurpose) {}
    fn set_ime_allowed(&self, allowed: bool) {
        self.ime_driver.borrow_mut().set_allowed(
            &mut super::ime::Port {
                ime: &mut self.ime.borrow_mut(),
                scale: self.scale,
            },
            allowed,
        );
    }
    fn set_soft_keyboard(&self, _: bool) {}
    fn set_ime_cursor_area(&self, area: Rect) {
        // Native pixels keep a scale change visible to the shared output cache.
        let scale = self.scale;
        let area = Rect {
            x: area.x * scale,
            y: area.y * scale,
            width: area.width * scale,
            height: area.height * scale,
        };
        self.ime_driver.borrow_mut().set_cursor_area(
            &mut super::ime::Port {
                ime: &mut self.ime.borrow_mut(),
                scale: 1.0,
            },
            area,
        );
    }
    fn set_ime_text(&self, text: Option<arvane_shell::ShellImeText>) {
        self.ime_driver.borrow_mut().set_text(
            &mut super::ime::Port {
                ime: &mut self.ime.borrow_mut(),
                scale: self.scale,
            },
            text,
        );
    }
    fn frame_period_ns(&self) -> u64 {
        self.frame_period_ns
    }
    fn present_with(
        &mut self,
        width: u32,
        height: u32,
        damage: Option<&[kivact_types::render::PixelRect]>,
        draw: &mut dyn FnMut(&mut [u32], arvane_shell::BufferAge) -> bool,
    ) -> Result<bool, ShellError> {
        let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) else {
            return Ok(false);
        };
        if (width, height) != (self.width, self.height) {
            return Err(super::error::platform(
                "CPU frame dimensions differ from the window",
            ));
        }
        let native = self.native.clone();
        let presented =
            self.cpu
                .present(w, h, || crate::present::surface(&native), damage, draw)?;
        if presented {
            // softbuffer queues XCB image requests on this shared connection. Send
            // them now; the host event loop may otherwise wait without another request.
            self.native
                .connection
                .flush()
                .map_err(super::error::platform)?;
        }
        Ok(presented)
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

/// The frame period of the RandR CRTC that holds the center of the window, or the default period
/// where RandR cannot tell (no RandR, no CRTC, mode or rate).
fn randr_frame_period(native: &Native, size: (u32, u32)) -> u64 {
    read_randr_period(native, size).unwrap_or(kivact_types::defaults::FRAME_PERIOD_NS)
}

fn read_randr_period(native: &Native, (width, height): (u32, u32)) -> Option<u64> {
    let connection = &native.connection;
    let root = connection.setup().roots.get(native.screen as usize)?.root;
    let origin = connection
        .translate_coordinates(native.id.get(), root, 0, 0)
        .ok()?
        .reply()
        .ok()?;
    let center_x = i32::from(origin.dst_x) + (width / 2) as i32;
    let center_y = i32::from(origin.dst_y) + (height / 2) as i32;
    let resources = connection
        .randr_get_screen_resources_current(root)
        .ok()?
        .reply()
        .ok()?;
    for crtc in &resources.crtcs {
        let info = connection
            .randr_get_crtc_info(*crtc, resources.config_timestamp)
            .ok()?
            .reply()
            .ok()?;
        let (left, top) = (i32::from(info.x), i32::from(info.y));
        let inside = info.mode != 0
            && (left..left + i32::from(info.width)).contains(&center_x)
            && (top..top + i32::from(info.height)).contains(&center_y);
        if !inside {
            continue;
        }
        let mode = resources.modes.iter().find(|mode| mode.id == info.mode)?;
        // A frame lasts htotal * vtotal dot clocks: the period is that over the dot clock.
        let dots = u64::from(mode.htotal) * u64::from(mode.vtotal);
        let dot_clock = u64::from(mode.dot_clock);
        if dots == 0 || dot_clock == 0 {
            return None;
        }
        return Some((dots * 1_000_000_000 + dot_clock / 2) / dot_clock);
    }
    None
}
