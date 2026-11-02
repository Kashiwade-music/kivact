//! Role: Embedded ShellWindow services and synchronous CPU/GPU presentation.
use super::native::Native;
use super::state::{CpuFrame, Hold, Phase, ScaleSource, State};
use crate::embed::child::{Backend, Command, Shared};
use crate::embed::session::{Session, negotiate};
use arvane_shell::{AccessibilityError, ShellError, ShellFileDialog, ShellImeText, ShellWindow};
use arvane_shell::{ClickCounter, MakeHandler, Mouse, PlatformTree, ShellEvent, ShellWaker};
use kivact_types::{Cursor, Rect, Size};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use windows_sys::Win32::Graphics::Gdi::{
    DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW, GetMonitorInfoW, InvalidateRect,
    MONITOR_DEFAULTTONEAREST, MONITORINFOEXW, MonitorFromWindow,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetCapture;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, GetClientRect, GetCursorPos, GetParent, KillTimer, PostMessageW, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOZORDER, SetTimer, SetWindowPos, USER_TIMER_MINIMUM, WindowFromPoint,
};
pub(super) struct Window(pub(super) Rc<State>);

pub(super) fn environment(state: &State) -> kivact_types::protocol::Environment {
    let scale = state.shared.scale();
    let mouse = crate::native_input::state::MouseSettings::read();
    kivact_types::protocol::Environment {
        scale,
        frame_period_ns: frame_period(state.native.hwnd()),
        reduced_motion: crate::native_input::reduced_motion(),
        locale: crate::native_input::locale::system_locale(),
        mouse: kivact_types::protocol::MouseSettings {
            double_click_ns: u64::from(mouse.double_click_ms) * 1_000_000,
            double_click_distance: (
                mouse.double_click_px.0 as f32 / scale,
                mouse.double_click_px.1 as f32 / scale,
            ),
            wheel_lines: mouse.wheel_lines,
            wheel_chars: mouse.wheel_chars,
        },
        ..Default::default()
    }
}
fn frame_period(hwnd: isize) -> u64 {
    // SAFETY: querying the child parent retains no native resources.
    let parent = unsafe { GetParent(hwnd) };
    // SAFETY: nearest-monitor lookup reads the opaque parent HWND only.
    let monitor = unsafe { MonitorFromWindow(parent, MONITOR_DEFAULTTONEAREST) };
    // SAFETY: zeroed MONITORINFOEXW is valid output storage once cbSize is filled.
    let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    // SAFETY: zeroed DEVMODEW is valid query output storage once dmSize is filled.
    let mut mode: DEVMODEW = unsafe { std::mem::zeroed() };
    mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
    // SAFETY: info is size-tagged writable MONITORINFOEXW storage.
    let found = unsafe { GetMonitorInfoW(monitor, (&mut info as *mut MONITORINFOEXW).cast()) };
    if found == 0 {
        return arvane_shell::DEFAULT_FRAME_PERIOD_NS;
    }
    // SAFETY: device name comes from monitor info and mode declares its output size.
    let found =
        unsafe { EnumDisplaySettingsW(info.szDevice.as_ptr(), ENUM_CURRENT_SETTINGS, &mut mode) };
    if found != 0 && mode.dmDisplayFrequency > 1 {
        1_000_000_000 / u64::from(mode.dmDisplayFrequency)
    } else {
        arvane_shell::DEFAULT_FRAME_PERIOD_NS
    }
}
impl ShellWindow for Window {
    fn request_size(&mut self, size: Size) -> Result<(), ShellError> {
        debug_assert!(self.0.phase.get() != Phase::Opening);
        debug_assert_eq!(std::thread::current().id(), self.0.native.ui_thread);
        negotiate(
            size,
            self.scale(),
            super::native::MAX_DIMENSION,
            self.0.on_resize_request.as_deref(),
            &self.0.shared,
            || self.0.native.wake(),
        )
    }

    fn copy_text(&mut self, text: &str) -> Result<(), ShellError> {
        self.0.services.borrow_mut().copy_text(text)
    }
    fn read_text(&mut self) -> Result<String, ShellError> {
        self.0.services.borrow_mut().read_text()
    }
    fn pick_file(&mut self, request: &ShellFileDialog) -> Result<Option<PathBuf>, ShellError> {
        super::services::pick_file(&super::services::Parent::new(self.0.native.hwnd()), request)
    }
    fn pick_files(
        &mut self,
        request: &ShellFileDialog,
    ) -> Result<Option<Vec<PathBuf>>, ShellError> {
        super::services::pick_files(&super::services::Parent::new(self.0.native.hwnd()), request)
    }
    fn save_file(&mut self, request: &ShellFileDialog) -> Result<Option<PathBuf>, ShellError> {
        super::services::save_file(&super::services::Parent::new(self.0.native.hwnd()), request)
    }
    fn pick_folder(&mut self, request: &ShellFileDialog) -> Result<Option<PathBuf>, ShellError> {
        super::services::pick_folder(&super::services::Parent::new(self.0.native.hwnd()), request)
    }
    fn open_path(&mut self, path: &Path) -> Result<(), ShellError> {
        crate::open::open_path(path)
    }
    fn reveal_path(&mut self, path: &Path) -> Result<(), ShellError> {
        crate::open::reveal_path(path)
    }
    fn recover_surface(&mut self) -> Result<(), ShellError> {
        self.release_cpu_surface();
        Ok(())
    }
    fn shows_cpu_frame(&self) -> bool {
        self.0.frame.borrow().is_some()
    }
    fn release_cpu_surface(&mut self) {
        self.0.frame.replace(None);
        self.0.scratch.replace(Vec::new());
    }
    fn replace_accessibility(
        &mut self,
        tree: &kivact_types::protocol::SemanticsTree,
    ) -> Result<(), AccessibilityError> {
        let result = self.0.a11y.replace(tree);
        if result.is_ok() {
            super::a11y::publish(&self.0);
        }
        result
    }
    fn update_accessibility(
        &mut self,
        update: &kivact_types::protocol::SemanticsUpdate,
    ) -> Result<(), AccessibilityError> {
        self.0
            .a11y
            .update(update)
            .map(|_| super::a11y::publish(&self.0))
    }
    fn accessibility_active(&self) -> bool {
        self.0.a11y.active()
    }
    fn size(&self) -> Size {
        let applied = self.0.shared.applied();
        Size {
            width: applied.width as f32 / applied.scale,
            height: applied.height as f32 / applied.scale,
        }
    }
    fn physical_size(&self) -> (u32, u32) {
        self.0.shared.size()
    }
    fn scale(&self) -> f32 {
        self.0.shared.scale()
    }
    fn now_ns(&self) -> u64 {
        self.0.now()
    }
    fn capabilities(&self) -> kivact_types::Capabilities {
        kivact_types::Capabilities {
            native_accessibility: true,
            ime_reconversion: true,
            ..Default::default()
        }
    }
    fn surface_generation(&self) -> Option<u64> {
        Some(1)
    }
    fn request_redraw(&self) {
        if self.0.shared.alive() {
            // SAFETY: invalidation only targets our live child; callback dispatch is deferred.
            unsafe {
                InvalidateRect(self.0.native.hwnd(), std::ptr::null(), 0);
            }
        }
    }
    fn set_title(&self, _: &str) {}
    fn set_min_size(&self, _: Option<Size>) {}
    fn reduced_motion(&self) -> bool {
        self.environment().reduced_motion
    }
    fn environment(&self) -> kivact_types::protocol::Environment {
        environment(&self.0)
    }
    fn set_fullscreen(&self, _: bool) -> Result<(), ShellError> {
        Err(ShellError::unsupported(
            "fullscreen is unavailable for an embedded window",
        ))
    }
    fn fullscreen(&self) -> bool {
        false
    }
    fn set_cursor(&self, cursor: Cursor) {
        self.0.cursor.set(cursor);
        // The cursor is global: apply it now only while it is over this child or
        // captured by it; WM_SETCURSOR applies the stored one otherwise.
        let hwnd = self.0.native.hwnd();
        // SAFETY: GetCapture reads this UI thread's capture window only.
        let captured = unsafe { GetCapture() } == hwnd;
        let mut point = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        // SAFETY: GetCursorPos writes the local POINT.
        let located = unsafe { GetCursorPos(&mut point) } != 0;
        // SAFETY: WindowFromPoint only reads the window tree for a screen point.
        if captured || (located && unsafe { WindowFromPoint(point) } == hwnd) {
            super::input::cursor(cursor);
        }
    }
    fn set_ime_allowed(&self, allowed: bool) {
        super::ime::allowed(&self.0, allowed);
    }
    fn set_soft_keyboard(&self, _: bool) {}
    fn set_ime_cursor_area(&self, area: Rect) {
        super::ime::area(&self.0, area);
    }
    fn set_ime_text(&self, text: Option<ShellImeText>) {
        self.0.input.text.replace(text);
    }
    fn frame_period_ns(&self) -> u64 {
        frame_period(self.0.native.hwnd())
    }
    fn wait_until(&mut self, done: &mut dyn FnMut() -> bool, limit: Duration) -> bool {
        crate::wait::until(done, limit)
    }
    fn present_with(
        &mut self,
        width: u32,
        height: u32,
        draw: &mut dyn FnMut(&mut [u32]) -> bool,
    ) -> Result<bool, ShellError> {
        let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) else {
            return Ok(false);
        };
        if !self.0.shared.alive() {
            return Ok(false);
        }
        if (width, height) != self.physical_size() {
            return Err(ShellError::new(
                "frame dimensions differ from the child client size",
            ));
        }
        let count = (width as usize)
            .checked_mul(height as usize)
            .filter(|count| *count <= isize::MAX as usize / std::mem::size_of::<u32>())
            .ok_or_else(|| ShellError::new("embedded frame dimensions overflow"))?;
        let mut scratch = self.0.scratch.borrow_mut();
        let additional = count.saturating_sub(scratch.len());
        scratch.try_reserve(additional).map_err(|error| {
            ShellError::new(format!("embedded frame allocation failed: {error}"))
        })?;
        scratch.resize(count, 0);
        if !draw(&mut scratch) {
            return Ok(false);
        }
        let pixels = std::mem::take(&mut *scratch);
        drop(scratch);
        let old = self.0.frame.replace(Some(CpuFrame {
            pixels,
            width: w,
            height: h,
        }));
        if let Some(old) = old {
            *self.0.scratch.borrow_mut() = old.pixels;
        }
        super::cpu::present(&self.0)?;
        Ok(true)
    }
    fn with_gpu_target(
        &self,
        make: Box<dyn FnOnce(arvane_shell::GpuTarget) + Send>,
    ) -> Result<(), ShellError> {
        if !self.0.shared.alive() {
            return Err(ShellError::new("embedded window is closed"));
        }
        make(self.0.target.clone());
        Ok(())
    }
}

impl State {
    pub(super) fn new(
        shared: Arc<Shared>,
        native: Arc<Native>,
        instance: isize,
        scale_source: ScaleSource,
        on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    ) -> Self {
        Self {
            a11y: PlatformTree::new(f64::from(shared.scale())),
            uia: RefCell::new(None),
            services: RefCell::new(crate::services::Services::default()),
            input: crate::native_input::state::NativeInput::default(),
            target: Arc::new(super::target::Target {
                instance,
                native: native.clone(),
                shared: shared.clone(),
            }),
            session: RefCell::new(Session::new(shared.clone())),
            shared,
            native,
            on_resize_request,
            events: RefCell::new(VecDeque::new()),
            phase: Cell::new(Phase::Opening),
            in_callback: Cell::new(false),
            draining: Cell::new(false),
            original_proc: Cell::new(None),
            cursor: Cell::new(kivact_types::Cursor::Auto),
            pending_surrogate: Cell::new(None),
            mouse: Cell::new(Mouse::default()),
            clicks: RefCell::new(ClickCounter::default()),
            environment: RefCell::new(kivact_types::protocol::Environment::default()),
            scale_source: Cell::new(scale_source),
            frame: RefCell::new(None),
            scratch: RefCell::new(Vec::new()),
            started: Instant::now(),
        }
    }
    pub(super) fn now(&self) -> u64 {
        self.started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
    }
    pub(super) fn waker(&self) -> ShellWaker {
        let shared = Arc::downgrade(&self.shared);
        let native = Arc::downgrade(&self.native);
        ShellWaker::new(move || {
            let (Some(shared), Some(native)) = (shared.upgrade(), native.upgrade()) else {
                return;
            };
            if shared.alive() && !native.wake_pending.swap(true, Ordering::AcqRel) {
                // SAFETY: posting uses only the opaque HWND and never accesses UI state.
                if unsafe { PostMessageW(native.hwnd(), super::native::WAKE, 0, 0) } == 0 {
                    native.wake_pending.store(false, Ordering::Release);
                }
            }
        })
    }
    pub(super) fn start(self: &Rc<Self>, make: MakeHandler) {
        {
            let _callback = Hold::enter(&self.in_callback)
                .expect("invariant: open runs no App callback before start");
            self.session
                .borrow_mut()
                .start(make, &mut Window(self.clone()), self.waker());
        }
        if self.shared.stopped() {
            self.stop();
        } else {
            self.control();
        }
    }
    pub(super) fn event(self: &Rc<Self>, event: ShellEvent) {
        if self.shared.stopped() {
            return;
        }
        self.events.borrow_mut().push_back(event);
        if self.phase.get() == Phase::Live {
            self.drain();
        }
    }
    pub(super) fn drain(self: &Rc<Self>) {
        if self.in_callback.get() || self.phase.get() != Phase::Live {
            return;
        }
        loop {
            if self.shared.stopped() {
                break;
            }
            let event = self.events.borrow_mut().pop_front();
            let Some(event) = event else {
                break;
            };
            self.invoke(event);
        }
        for event in self.a11y.drain_actions() {
            self.event(event);
        }
        self.commands();
    }
    fn invoke(self: &Rc<Self>, event: ShellEvent) {
        {
            let Some(_callback) = Hold::enter(&self.in_callback) else {
                return;
            };
            self.session
                .borrow_mut()
                .deliver(event, &mut Window(self.clone()));
        }
        if self.shared.stopped() {
            self.stop();
        } else {
            self.control();
        }
    }
    fn control(self: &Rc<Self>) {
        if self.shared.stopped() {
            return;
        }
        let deadline = self.session.borrow().deadline();
        let hwnd = self.native.hwnd();
        // SAFETY: cancel only the HWND-scoped timer owned by this child.
        unsafe {
            KillTimer(hwnd, super::native::TIMER);
        }
        if let Some(deadline) = deadline {
            let remaining = deadline.saturating_sub(self.now()) / 1_000_000;
            let ms = if remaining <= 17 {
                USER_TIMER_MINIMUM
            } else {
                remaining.saturating_sub(1).min(u64::from(u32::MAX)) as u32
            };
            // SAFETY: this timer belongs to the live child and dispatches on its UI thread.
            if unsafe { SetTimer(hwnd, super::native::TIMER, ms, None) } == 0 {
                self.shared.fail(ShellError::new("SetTimer failed"));
                self.stop();
            }
        }
    }
    pub(super) fn commands(self: &Rc<Self>) {
        if self.in_callback.get() || self.phase.get() != Phase::Live {
            return;
        }
        let Some(drain) = Hold::enter(&self.draining) else {
            return;
        };
        while !self.shared.stopped() {
            match self.shared.pop_command() {
                Some(Command::SetSize {
                    width: w,
                    height: h,
                    generation,
                }) => {
                    // A host correction made during negotiation wins even when
                    // its command was queued before this accepted request.
                    if !self.shared.current(generation) {
                        continue;
                    }
                    if !self.shared.alive() {
                        continue;
                    }
                    // SAFETY: native resizing occurs only on the HWND's UI thread.
                    if unsafe {
                        SetWindowPos(
                            self.native.hwnd(),
                            0,
                            0,
                            0,
                            w as i32,
                            h as i32,
                            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                        )
                    } == 0
                    {
                        self.shared.fail(ShellError::new("SetWindowPos failed"));
                    }
                }
                Some(Command::SetScale(scale)) => {
                    self.scale_source.set(ScaleSource::Host);
                    self.shared.set_scale(scale);
                    self.event(ShellEvent::Resized);
                }
                None => break,
            }
        }
        drop(drain);
        if self.shared.stopped() {
            self.stop();
        }
    }
    pub(super) fn destroy(self: &Rc<Self>) {
        if self.shared.alive() {
            // SAFETY: all callers run on the creating UI thread; wndproc marks dead.
            unsafe {
                DestroyWindow(self.native.hwnd());
            }
        }
    }
    /// Close the handler and release the child's resources, exactly once.
    pub(super) fn dispose(self: &Rc<Self>) {
        if !matches!(self.phase.get(), Phase::Opening | Phase::Live) {
            return;
        }
        let closed = {
            // Deferred while App code runs: the callback's caller stops the child.
            let Some(_callback) = Hold::enter(&self.in_callback) else {
                return;
            };
            self.phase.set(Phase::Disposed);
            self.session.borrow_mut().close(&mut Window(self.clone()))
        };
        if !closed {
            self.leak_resources();
            return;
        }
        self.frame.replace(None);
        self.scratch.replace(Vec::new());
        // Explicit close disposes before DestroyWindow; parent destruction enters
        // through WM_DESTROY. Both paths disconnect before adapter destruction.
        super::a11y::disconnect(self);
        drop(self.uia.borrow_mut().take());
        // SAFETY: this child owns the HWND-scoped timer; cleanup runs on its UI thread.
        unsafe {
            KillTimer(self.native.hwnd(), super::native::TIMER);
        }
        self.events.borrow_mut().clear();
    }
    fn leak_resources(self: &Rc<Self>) {
        // Native resources and their callbacks may outlive all owner guards.
        crate::module::retain_module();
        self.phase.set(Phase::Leaked);
        // A later WM_DESTROY or the final handle drop must not retry teardown.
        std::mem::forget(self.clone());
    }
    pub(super) fn stop(self: &Rc<Self>) {
        self.shared.request_close();
        if self.in_callback.get() || !matches!(self.phase.get(), Phase::Opening | Phase::Live) {
            return;
        }
        self.dispose();
        self.destroy();
    }
    pub(super) fn geometry(&self) -> bool {
        let mut rect = windows_sys::Win32::Foundation::RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        // SAFETY: GetClientRect writes the supplied RECT on the child's UI thread.
        if unsafe { GetClientRect(self.native.hwnd(), &mut rect) } != 0 {
            return self.shared.set_size(
                (rect.right - rect.left).max(0) as u32,
                (rect.bottom - rect.top).max(0) as u32,
            );
        }
        false
    }
}
