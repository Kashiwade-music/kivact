//! Role: Embedded ShellWindow services and synchronous CPU/GPU presentation.
use super::native::Native;
use super::state::State;
use crate::embed::child::{Backend, Command, Shared};
use crate::embed::session::{Session, contain, negotiate};
use arvane_shell::{AccessibilityError, ShellError, ShellFileDialog, ShellImeText, ShellWindow};
use arvane_shell::{ClickCounter, MakeHandler, Mouse, ShellEvent, ShellWaker};
use kivact_types::{Cursor, Rect, Size};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetCapture;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
pub(super) struct Window(pub(super) Rc<State>);
pub(super) fn environment(state: &State) -> kivact_types::Environment {
    let scale = state.shared.scale();
    let mouse = crate::native_input::state::MouseSettings::read();
    kivact_types::Environment {
        scale,
        frame_period_ns: frame_period(state.native.hwnd()),
        reduced_motion: crate::native_input::reduced_motion(),
        locale: crate::native_input::locale::system_locale(),
        mouse: kivact_types::MouseSettings {
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
        debug_assert!(!self.0.opening.get());
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
        self.0.cpu_shown.get()
    }
    fn release_cpu_surface(&mut self) {
        *self.0.pixels.borrow_mut() = Vec::new();
        *self.0.scratch.borrow_mut() = Vec::new();
        self.0.pixel_size.set((0, 0));
        self.0.cpu_shown.set(false);
    }
    fn key(&self) -> u64 {
        self.0.native.hwnd() as u64
    }
    fn replace_accessibility(
        &mut self,
        tree: &kivact_types::SemanticsTree,
    ) -> Result<(), AccessibilityError> {
        let result = self
            .0
            .a11y
            .borrow()
            .as_ref()
            .expect("live child has UIA adapter")
            .tree
            .replace(tree);
        if result.is_ok() {
            super::a11y::publish(&self.0);
        }
        result
    }
    fn update_accessibility(
        &mut self,
        update: &kivact_types::SemanticsUpdate,
    ) -> Result<(), AccessibilityError> {
        let result = self
            .0
            .a11y
            .borrow()
            .as_ref()
            .expect("live child has UIA adapter")
            .tree
            .update(update);
        match result {
            Ok(_) => {
                super::a11y::publish(&self.0);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
    fn accessibility_active(&self) -> bool {
        self.0
            .a11y
            .borrow()
            .as_ref()
            .is_some_and(|bridge| bridge.tree.active())
    }
    fn size(&self) -> Size {
        let (w, h) = self.physical_size();
        Size {
            width: w as f32 / self.scale(),
            height: h as f32 / self.scale(),
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
        if self.0.shared.alive.load(Ordering::Acquire) {
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
    fn environment(&self) -> kivact_types::Environment {
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
        if width == 0 || height == 0 || !self.0.shared.alive.load(Ordering::Acquire) {
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
        std::mem::swap(&mut *self.0.pixels.borrow_mut(), &mut *scratch);
        self.0.pixel_size.set((width, height));
        drop(scratch);
        self.0.cpu_shown.set(true);
        super::cpu::present(&self.0)?;
        Ok(true)
    }
    fn with_gpu_target(
        &self,
        make: Box<dyn FnOnce(arvane_shell::GpuTarget) + Send>,
    ) -> Result<(), ShellError> {
        if !self.0.shared.alive.load(Ordering::Acquire) {
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
        explicit_scale: bool,
        on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    ) -> Self {
        Self {
            a11y: RefCell::new(None),
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
            busy: Cell::new(false),
            commands_active: Cell::new(false),
            closing: Cell::new(false),
            resources_leaked: Cell::new(false),
            opening: Cell::new(true),
            original_proc: Cell::new(None),
            cursor: Cell::new(kivact_types::Cursor::Auto),
            pending_surrogate: Cell::new(None),
            mouse: Cell::new(Mouse::default()),
            clicks: RefCell::new(ClickCounter::default()),
            environment: RefCell::new(kivact_types::Environment::default()),
            explicit_scale: Cell::new(explicit_scale),
            pixels: RefCell::new(Vec::new()),
            scratch: RefCell::new(Vec::new()),
            pixel_size: Cell::new((0, 0)),
            cpu_shown: Cell::new(false),
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
            if shared.alive.load(Ordering::Acquire)
                && !native.wake_pending.swap(true, Ordering::AcqRel)
            {
                // SAFETY: posting uses only the opaque HWND and never accesses UI state.
                if unsafe { PostMessageW(native.hwnd(), super::native::WAKE, 0, 0) } == 0 {
                    native.wake_pending.store(false, Ordering::Release);
                }
            }
        })
    }
    pub(super) fn start(self: &Rc<Self>, make: MakeHandler) {
        self.busy.set(true);
        self.session
            .borrow_mut()
            .start(make, &mut Window(self.clone()), self.waker());
        self.busy.set(false);
        if self.closing.get() {
            self.dispose();
        }
        if self.shared.stopped() {
            self.stop();
        } else {
            self.control();
        }
    }
    pub(super) fn event(self: &Rc<Self>, event: ShellEvent) {
        if self.closing.get() || self.shared.stopped() {
            return;
        }
        self.events.borrow_mut().push_back(event);
        if !self.opening.get() {
            self.drain();
        }
    }
    pub(super) fn drain(self: &Rc<Self>) {
        if self.busy.get() || self.opening.get() {
            return;
        }
        loop {
            if self.closing.get() || self.shared.stopped() {
                break;
            }
            let event = self.events.borrow_mut().pop_front();
            let Some(event) = event else {
                break;
            };
            self.invoke(event);
        }
        let actions = self
            .a11y
            .borrow()
            .as_ref()
            .map(|bridge| bridge.tree.drain_actions())
            .unwrap_or_default();
        for event in actions {
            self.event(event);
        }
        self.commands();
    }
    fn invoke(self: &Rc<Self>, event: ShellEvent) {
        if self.busy.replace(true) {
            return;
        }
        self.session
            .borrow_mut()
            .deliver(event, &mut Window(self.clone()));
        self.busy.set(false);
        if self.closing.get() {
            self.dispose();
        }
        if self.shared.stopped() {
            self.stop();
        } else {
            self.control();
        }
    }
    fn control(self: &Rc<Self>) {
        if self.shared.stopped() || self.closing.get() {
            return;
        }
        let next_wake_ns = self.session.borrow().control.next_wake_ns;
        let hwnd = self.native.hwnd();
        // SAFETY: cancel only the HWND-scoped timer owned by this child.
        unsafe {
            KillTimer(hwnd, super::native::TIMER);
        }
        if let Some(deadline) = next_wake_ns {
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
        if self.busy.get() || self.opening.get() || self.commands_active.replace(true) {
            return;
        }
        loop {
            if self.shared.stopped() || self.closing.get() {
                break;
            }
            let command = self
                .shared
                .commands
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .pop_front();
            match command {
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
                    if !self.shared.alive.load(Ordering::Acquire) {
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
                    self.explicit_scale.set(true);
                    self.shared.set_scale_value(scale);
                    self.event(ShellEvent::Resized);
                }
                None => break,
            }
        }
        self.commands_active.set(false);
        if self.shared.stopped() {
            self.stop();
        }
    }
    pub(super) fn destroy(self: &Rc<Self>) {
        if self.shared.alive.load(Ordering::Acquire) {
            // SAFETY: all callers run on the creating UI thread; wndproc marks dead.
            unsafe {
                DestroyWindow(self.native.hwnd());
            }
        }
    }
    pub(super) fn dispose(self: &Rc<Self>) {
        if self.busy.get() || self.resources_leaked.get() {
            return;
        }
        self.busy.set(true);
        if !self.session.borrow_mut().close(&mut Window(self.clone())) {
            self.leak_resources();
            return;
        }
        self.cpu_shown.set(false);
        self.pixel_size.set((0, 0));
        self.pixels.borrow_mut().clear();
        self.scratch.borrow_mut().clear();
        // Explicit close disposes before DestroyWindow; parent destruction enters
        // through WM_DESTROY. Both paths disconnect before adapter destruction.
        super::a11y::disconnect(self);
        let a11y = self.a11y.borrow_mut().take();
        if let Err(error) = contain(|| drop(a11y), "embedded accessibility destruction panicked") {
            self.shared.fail(error);
            tracing::warn!(target: "kivact", "embedded accessibility destruction panicked; leaking remaining native resources");
            self.leak_resources();
            return;
        }
        // SAFETY: this child owns the HWND-scoped timer; cleanup runs on its UI thread.
        unsafe {
            KillTimer(self.native.hwnd(), super::native::TIMER);
        }
        self.events.borrow_mut().clear();
        self.busy.set(false);
    }
    fn leak_resources(self: &Rc<Self>) {
        // Native resources and their callbacks may outlive all owner guards.
        crate::module::retain_module();
        self.resources_leaked.set(true);
        self.busy.set(false);
        // A later WM_DESTROY or the final handle drop must not retry teardown.
        std::mem::forget(self.clone());
    }
    pub(super) fn stop(self: &Rc<Self>) {
        self.shared.request_close();
        if self.busy.get() {
            return;
        }
        self.closing.set(true);
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
            let previous = self.shared.size();
            self.shared.update(
                (rect.right - rect.left).max(0) as u32,
                (rect.bottom - rect.top).max(0) as u32,
                self.shared.scale(),
            );
            return previous != self.shared.size();
        }
        false
    }
}
