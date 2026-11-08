//! Role: Convert shell events and start client callbacks on the UI thread.
//!
//! Input goes to the target window's model (`model::pointer::shell`,
//! `model::keys`, `model::edit`), which owns what it means; the clipboard
//! work a key leaves (copy, paste) is done here, with the window's
//! clipboard.
//!
//! Invariants:
//! - What the client sees of one window's environment is one `Environment`,
//!   kept by that window's model. At start each window's environment
//!   (`Window::environment`: scale, refresh period, reduced motion, theme,
//!   safe area, mouse settings, locale), its size and fullscreen state, the
//!   loop's capabilities (`Windows::capabilities`) and whether frames can be
//!   drawn (`Environment::surface_active`) are reported in `HostInfo`.
//!   The theme is reported light at start and follows the system's from
//!   its first change.
//! - Every change goes through `Model::update_env`, which sends one
//!   `EnvChanged` (the whole value) only when the value differs from the
//!   last one sent.
//! - The host's locale (`Environment::locale`, a BCP 47 tag or `None`) is
//!   `Environment::locale`; when it is read again is Arvane's.
//! - Frames are drawn only while the application is resumed
//!   (`Suspended`/`Resumed`), the window has a surface
//!   (`Window::surface_generation`, `SurfaceDestroyed`/`SurfaceCreated`)
//!   and is not empty (`Environment::surface_active`). In between no redraw is
//!   asked for and no frame tick is sent (the client's frame clock stops);
//!   a resume and a new surface draw again.
//! - A window with an open modal child drops Pointer, Wheel, Key, Ime, File
//!   and Accessibility input before it reaches the parent's model.
//! - The window losing the focus cancels every pointer of that window.
use super::pump::{AppPresence, Host, HostPhase};
use super::window::LastPresent;
use crate::KeyResult;
use crate::contract::{HostEvent, HostInfo};
use crate::host::HostWaker;
use arvane::{EventTarget, ShellControl, ShellError, ShellEvent, Window, Windows};
use kivact_types::{
    KeyInput, KeyPhase, LifecycleEvent, PlatformRequest, PlatformResult, PointerKind, PointerPhase,
    WindowId, WindowMode,
};
impl Host {
    /// Start the client in the main window, loading images with the App's
    /// store: the shell's start, and the test driver's.
    pub(super) fn begin(
        &mut self,
        windows: &mut Windows<'_>,
        waker: HostWaker,
    ) -> Result<ShellControl, ShellError> {
        if self.observe_close() {
            return Ok(ShellControl::Wait);
        }
        let (env, drawable) = self.start_window(WindowId::MAIN, windows);
        self.frame_log
            .borrow_mut()
            .note(&format!("frame_period_ns={}", env.frame_period_ns));
        let info = HostInfo {
            capabilities: windows.capabilities(),
            now_ns: windows.now_ns(),
            env,
            surface_active: drawable,
        };
        let tree = super::write::Writer::new(&self.app, &self.engine);
        self.client.start(&info, waker, tree);
        if self.observe_close() {
            return Ok(ShellControl::Wait);
        }
        self.apply_window_changes(windows);
        if self.observe_close() {
            return Ok(ShellControl::Wait);
        }
        self.externals(windows);
        self.settle();
        self.pump(windows)
    }

    /// Handle one shell event: a loop event once, or one window's event in
    /// that window.
    pub(super) fn handle(
        &mut self,
        target: EventTarget,
        event: ShellEvent,
        windows: &mut Windows<'_>,
    ) -> Result<ShellControl, ShellError> {
        if self.observe_close() {
            return Ok(ShellControl::Wait);
        }
        match target {
            EventTarget::Loop => self.handle_loop(event, windows),
            EventTarget::Window(id) => self.handle_window(id, event, windows),
        }
    }

    /// Handle an event about the loop as a whole once.
    fn handle_loop(
        &mut self,
        event: ShellEvent,
        windows: &mut Windows<'_>,
    ) -> Result<ShellControl, ShellError> {
        self.enter(windows);
        if self.phase == HostPhase::Closed {
            return Ok(ShellControl::Wait);
        }
        match event {
            ShellEvent::SaveState => self
                .app
                .borrow()
                .push_event(HostEvent::Lifecycle(LifecycleEvent::SaveState)),
            ShellEvent::Wake(_) => self.wake(windows)?,
            ShellEvent::Suspended => {
                if self.presence == AppPresence::Foreground {
                    self.presence = AppPresence::Suspended;
                    self.cancel_pointers();
                    self.report_drawable_all(windows);
                    self.app
                        .borrow()
                        .push_event(HostEvent::Lifecycle(LifecycleEvent::Suspended));
                }
            }
            ShellEvent::Resumed => {
                if self.presence == AppPresence::Suspended {
                    self.presence = AppPresence::Foreground;
                    self.report_drawable_all(windows);
                    self.app
                        .borrow()
                        .push_event(HostEvent::Lifecycle(LifecycleEvent::Resumed));
                    self.invalidate_paint_all();
                }
            }
            // Rebuildable host caches are released before application callbacks.
            ShellEvent::MemoryWarning => {
                for id in self.window_ids() {
                    if let Some(mut window) = windows.get(id) {
                        window.release_caches();
                    }
                }
                self.engine.borrow_mut().release_caches();
                for id in self.window_ids() {
                    if let Some(model) = self.app.borrow_mut().model_mut(id) {
                        model.paint.scene_cache.borrow_mut().clear();
                    }
                }
                self.app.borrow().assets.borrow_mut().release_caches();
                self.frame_log.borrow_mut().note("memory_warning");
                self.app
                    .borrow()
                    .push_event(HostEvent::Lifecycle(LifecycleEvent::MemoryWarning));
            }
            _ => unreachable!("invariant: matching loop event family"),
        }
        self.pump(windows)
    }

    /// Handle one window's event in that window.
    fn handle_window(
        &mut self,
        id: WindowId,
        event: ShellEvent,
        windows: &mut Windows<'_>,
    ) -> Result<ShellControl, ShellError> {
        if !self.windows.contains_key(&id) {
            return Ok(ShellControl::Wait);
        }
        self.enter(windows);
        if self.phase == HostPhase::Closed {
            return Ok(ShellControl::Wait);
        }
        let now = windows.now_ns();
        if let Some(model) = self.app.borrow_mut().model_mut(id) {
            model.frame.set_now(now);
            if stops_smooth_scrolls(&event) {
                model.frame.stop_smooth_scrolls();
            }
        }
        if matches!(event, ShellEvent::File(_)) {
            self.settle_window(id);
        }
        // Key releases remain turn boundaries so pending edits reach the client.
        let input = matches!(
            event,
            ShellEvent::Pointer(_)
                | ShellEvent::Wheel(_)
                | ShellEvent::Modifiers(_)
                | ShellEvent::Key(KeyInput {
                    phase: KeyPhase::Down { .. },
                    ..
                })
                | ShellEvent::Ime(_)
        );
        if hit_tests(&event) {
            // Deferred turns leave edits laid out lazily; hit test current geometry.
            self.settle_window(id);
        }
        // A modal child owns its parent's input while it is open.
        if self.discards_for_modal_parent(id, &event) {
            return Ok(self.defer(id, windows));
        }
        match event {
            ShellEvent::Redraw => self.redraw(id, windows)?,
            ShellEvent::Resized => self.resized(id, windows),
            event @ (ShellEvent::Pointer(_)
            | ShellEvent::Wheel(_)
            | ShellEvent::Modifiers(_)
            | ShellEvent::File(_)
            | ShellEvent::Key(_)
            | ShellEvent::Ime(_)) => self.input_event(event, id, windows),
            event @ (ShellEvent::EnvironmentChanged
            | ShellEvent::AccessibilityActivated
            | ShellEvent::AccessibilityDeactivated
            | ShellEvent::Accessibility { .. }) => self.accessibility_event(event, id, windows),
            event => self.lifecycle_event(event, id, windows),
        }
        if self.phase == HostPhase::Closed {
            return Ok(ShellControl::Wait);
        }
        if input {
            return Ok(self.defer(id, windows));
        }
        self.pump(windows)
    }

    /// Whether `id` has an open modal child that owns the input in `event`.
    fn discards_for_modal_parent(&self, id: WindowId, event: &ShellEvent) -> bool {
        let modal_child = self
            .windows
            .values()
            .any(|window| window.mode == WindowMode::Modal && window.parent == Some(id));
        modal_child
            && matches!(
                event,
                ShellEvent::Pointer(_)
                    | ShellEvent::Wheel(_)
                    | ShellEvent::Key(_)
                    | ShellEvent::Ime(_)
                    | ShellEvent::File(_)
                    | ShellEvent::Accessibility { .. }
            )
    }

    fn redraw(&mut self, id: WindowId, windows: &mut Windows<'_>) -> Result<(), ShellError> {
        if self.drawable(id) {
            let started = std::time::Instant::now();
            self.settle_window(id);
            match self.present(id, windows, started) {
                Ok(arvane::Presented::Shown { .. }) => {
                    if let Some(model) = self.app.borrow_mut().model_mut(id) {
                        model.paint.drawn();
                    }
                    self.windows
                        .get_mut(&id)
                        .expect("invariant: a drawn window has host state")
                        .last_present = LastPresent::Settled;
                }
                // The window had no buffer (timed out, occluded):
                // the picture is drawn again when Arvane says.
                Ok(arvane::Presented::Skipped { retry_at_ns }) => {
                    if let Some(model) = self.app.borrow_mut().model_mut(id) {
                        model.paint.drawn();
                        model.drawables.skipped(retry_at_ns);
                    }
                    self.windows
                        .get_mut(&id)
                        .expect("invariant: a drawn window has host state")
                        .last_present = LastPresent::Skipped { retry_at_ns };
                }
                // Content cannot render; the next change redraws.
                Ok(arvane::Presented::Dropped(_)) => {
                    if let Some(model) = self.app.borrow_mut().model_mut(id) {
                        model.paint.drawn();
                        model.drawables.dropped();
                    }
                    self.windows
                        .get_mut(&id)
                        .expect("invariant: a drawn window has host state")
                        .last_present = LastPresent::Settled;
                }
                // The window made its surface again: report the
                // loss and draw again.
                Ok(arvane::Presented::SurfaceLost(error)) => {
                    let revision = self.app.borrow().events().applied();
                    self.app.borrow().push_event(HostEvent::HostFailure {
                        window: id,
                        revision,
                        reason: error.to_string(),
                    });
                    if let Some(model) = self.app.borrow_mut().model_mut(id) {
                        model.invalidate_paint();
                    }
                    self.windows
                        .get_mut(&id)
                        .expect("invariant: a drawn window has host state")
                        .last_present = LastPresent::Settled;
                }
                Err(error) => return Err(ShellError::Failed(error)),
            }
        }
        Ok(())
    }

    fn wake(&mut self, windows: &mut Windows<'_>) -> Result<(), ShellError> {
        let now = windows.now_ns();
        for id in self.windows.keys().copied().collect::<Vec<_>>() {
            let retry_due = self.windows.get(&id).is_some_and(|window| {
                matches!(
                    window.last_present,
                    LastPresent::Skipped { retry_at_ns } if retry_at_ns <= now
                )
            });
            if retry_due {
                self.windows
                    .get_mut(&id)
                    .expect("invariant: the retried window has host state")
                    .last_present = LastPresent::Settled;
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.invalidate_paint();
                }
            }
        }
        self.tick_windows(now);
        let mut events = false;
        for id in self.window_ids() {
            if let Some(model) = self.app.borrow_mut().model_mut(id) {
                // The asset store's worker wakes the shell too; its
                // reports are events, delivered by the pump below.
                model.poll_assets();
                model.settle(&mut self.engine.borrow_mut());
            }
            events |= self.app.borrow().events().has_events();
        }
        // A due wake without events still gives the client its turn.
        if !events && self.phase != HostPhase::Exiting {
            self.turn(&[], &[], windows)?;
        }
        Ok(())
    }

    fn resized(&mut self, id: WindowId, windows: &mut Windows<'_>) {
        let Some(window) = windows.get(id) else {
            return;
        };
        self.refresh_surface(id, &window);
        let drawable = self.drawable(id);
        let size = window.size();
        let scale = window.scale();
        let fullscreen = window.fullscreen();
        if let Some(model) = self.app.borrow_mut().model_mut(id) {
            model.set_viewport(size);
            model.invalidate_paint();
            model.update_env(|env, surface_active| {
                env.scale_factor = scale;
                env.window_size = size;
                env.fullscreen = fullscreen;
                *surface_active = drawable;
            });
        }
        self.settle_window(id);
    }

    fn lifecycle_event(&mut self, event: ShellEvent, id: WindowId, windows: &mut Windows<'_>) {
        match event {
            ShellEvent::KeyboardInset(insets) => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.update_env(|env, _| env.keyboard_inset = insets.sanitize());
                }
            }
            ShellEvent::CloseRequested => self
                .app
                .borrow()
                .push_event(HostEvent::CloseRequested { window: id }),
            ShellEvent::Focused(false) => {
                let mut app = self.app.borrow_mut();
                if app.focused_window() == Some(id) {
                    app.focused_window = None;
                }
                if let Some(model) = app.model_mut(id) {
                    model.key_down_targets.0.clear();
                    model.cancel_pointer(&mut self.engine.borrow_mut());
                }
            }
            ShellEvent::Focused(true) => {
                self.app.borrow_mut().focused_window = Some(id);
                // A window coming back (after skipped frames) draws at once.
                let skipped = self.windows.get(&id).is_some_and(|window| {
                    matches!(window.last_present, LastPresent::Skipped { .. })
                });
                if skipped {
                    self.windows
                        .get_mut(&id)
                        .expect("invariant: a focused window has host state")
                        .last_present = LastPresent::Settled;
                    if let Some(model) = self.app.borrow_mut().model_mut(id) {
                        model.invalidate_paint();
                    }
                }
            }
            ShellEvent::SurfaceDestroyed => self.report_drawable(id, windows),
            ShellEvent::SurfaceCreated => {
                self.report_drawable(id, windows);
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.invalidate_paint();
                }
            }
            ShellEvent::Injected { request } => {
                self.app.borrow().push_event(HostEvent::InjectDelivered {
                    window: id,
                    request,
                })
            }
            _ => unreachable!("invariant: matching shell event family"),
        }
    }

    fn input_event(&mut self, event: ShellEvent, id: WindowId, windows: &mut Windows<'_>) {
        match event {
            ShellEvent::Pointer(input) => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.pointer_input(input, &mut self.engine.borrow_mut());
                }
            }
            ShellEvent::Wheel(input) => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.wheel(input, &mut self.engine.borrow_mut());
                }
                // A native scroll is reported (`ScrollChanged`) by the
                // layout pass: run it now, so the client's turn is due at
                // once instead of waiting for the next paint.
                self.settle_window(id);
            }
            ShellEvent::Modifiers(m) => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.report_modifiers(m);
                }
            }
            ShellEvent::File(input) => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.file(input.phase, input.files, input.position);
                }
            }
            ShellEvent::Key(input) => self.key_event(input, id, windows),
            ShellEvent::Ime(input) => self.ime_event(input, id),
            _ => unreachable!("invariant: matching shell event family"),
        }
    }

    fn key_event(
        &mut self,
        input: kivact_types::KeyInput,
        id: WindowId,
        windows: &mut Windows<'_>,
    ) {
        let result = match self.app.borrow_mut().model_mut(id) {
            Some(model) => {
                let physical = physical_letter(input.code);
                model.key(input, physical, &mut self.engine.borrow_mut())
            }
            None => return,
        };
        match result {
            // A cut already removed the text.
            KeyResult::Copy(text) => {
                if let Some(mut window) = windows.get(id) {
                    let _ = window.perform(PlatformRequest::CopyText(text));
                }
            }
            KeyResult::Paste(node) => {
                if let Some(mut window) = windows.get(id)
                    && let PlatformResult::Text(text) = window.perform(PlatformRequest::ReadText)
                    && let Some(model) = self.app.borrow_mut().model_mut(id)
                {
                    model.paste(node, &text, &mut self.engine.borrow_mut());
                }
            }
            KeyResult::Reported | KeyResult::Consumed | KeyResult::Text(_) => {}
        }
    }

    fn ime_event(&mut self, input: kivact_types::ImeInput, id: WindowId) {
        match input {
            kivact_types::ImeInput::Preedit { text, selection } => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.preedit(&text, selection.map(|r| (r.start, r.end)));
                }
            }
            kivact_types::ImeInput::Commit(text) => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.commit(&text);
                }
            }
            kivact_types::ImeInput::Selection(range) => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.ime_select(range.start, range.end);
                }
            }
            kivact_types::ImeInput::End => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.ime_disabled();
                }
            }
        }
    }

    fn accessibility_event(&mut self, event: ShellEvent, id: WindowId, windows: &mut Windows<'_>) {
        match event {
            ShellEvent::EnvironmentChanged => {
                if let Some(window) = windows.get(id) {
                    self.environment_changed(id, &window);
                }
            }
            ShellEvent::AccessibilityActivated => {
                if let Some(model) = self.app.borrow_mut().model_mut(id)
                    && model.a11y.publish == crate::a11y::state::A11yPublish::Inactive
                {
                    model.a11y.publish = crate::a11y::state::A11yPublish::AwaitingFirstTree;
                    model.a11y.invalidate();
                    if self.accessibility_noted.note() {
                        self.frame_log.borrow_mut().note("accessibility=on");
                    }
                }
            }
            ShellEvent::AccessibilityDeactivated => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.a11y.publish = crate::a11y::state::A11yPublish::Inactive;
                }
            }
            ShellEvent::Accessibility { node, request } => {
                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                    model.accessibility_action(node, request);
                }
            }
            _ => unreachable!("invariant: matching shell event family"),
        }
    }

    /// Refresh window `id`'s surface state from the window.
    fn refresh_surface(&mut self, id: WindowId, window: &Window<'_>) {
        let surface = super::window::surface_of(window);
        if let Some(host_window) = self.windows.get_mut(&id) {
            host_window.surface = surface;
        }
    }

    /// Report whether window `id` can draw, when that changed.
    fn report_drawable(&mut self, id: WindowId, windows: &mut Windows<'_>) {
        if let Some(window) = windows.get(id) {
            self.refresh_surface(id, &window);
        }
        let drawable = self.drawable(id);
        if let Some(model) = self.app.borrow_mut().model_mut(id) {
            model.set_surface(drawable);
        }
    }

    fn report_drawable_all(&mut self, windows: &mut Windows<'_>) {
        for id in self.windows.keys().copied().collect::<Vec<_>>() {
            self.report_drawable(id, windows);
        }
    }

    fn cancel_pointers(&mut self) {
        for id in self.window_ids() {
            if let Some(model) = self.app.borrow_mut().model_mut(id) {
                model.cancel_pointer(&mut self.engine.borrow_mut());
            }
        }
    }

    fn invalidate_paint_all(&mut self) {
        for id in self.window_ids() {
            if let Some(model) = self.app.borrow_mut().model_mut(id) {
                model.invalidate_paint();
            }
        }
    }
}

/// Whether `event` stops the smooth scrolls in progress: a wheel, a mouse
/// press, or any event of a touch or pen contact.
fn stops_smooth_scrolls(event: &ShellEvent) -> bool {
    match event {
        ShellEvent::Wheel(_) => true,
        ShellEvent::Pointer(input) => {
            input.kind != PointerKind::Mouse || matches!(input.phase, PointerPhase::Down(_))
        }
        _ => false,
    }
}

/// Whether `event` is hit tested: a wheel, the mouse's moves and buttons
/// (not its leave or cancel), or any event of a contact.
fn hit_tests(event: &ShellEvent) -> bool {
    match event {
        ShellEvent::Wheel(_) => true,
        ShellEvent::Pointer(input) => {
            input.kind != PointerKind::Mouse
                || !matches!(input.phase, PointerPhase::Leave | PointerPhase::Cancel)
        }
        _ => false,
    }
}

fn physical_letter(code: kivact_types::KeyCode) -> Option<char> {
    match code {
        kivact_types::KeyCode::KeyA => Some('a'),
        kivact_types::KeyCode::KeyB => Some('b'),
        kivact_types::KeyCode::KeyC => Some('c'),
        kivact_types::KeyCode::KeyD => Some('d'),
        kivact_types::KeyCode::KeyE => Some('e'),
        kivact_types::KeyCode::KeyF => Some('f'),
        kivact_types::KeyCode::KeyG => Some('g'),
        kivact_types::KeyCode::KeyH => Some('h'),
        kivact_types::KeyCode::KeyI => Some('i'),
        kivact_types::KeyCode::KeyJ => Some('j'),
        kivact_types::KeyCode::KeyK => Some('k'),
        kivact_types::KeyCode::KeyL => Some('l'),
        kivact_types::KeyCode::KeyM => Some('m'),
        kivact_types::KeyCode::KeyN => Some('n'),
        kivact_types::KeyCode::KeyO => Some('o'),
        kivact_types::KeyCode::KeyP => Some('p'),
        kivact_types::KeyCode::KeyQ => Some('q'),
        kivact_types::KeyCode::KeyR => Some('r'),
        kivact_types::KeyCode::KeyS => Some('s'),
        kivact_types::KeyCode::KeyT => Some('t'),
        kivact_types::KeyCode::KeyU => Some('u'),
        kivact_types::KeyCode::KeyV => Some('v'),
        kivact_types::KeyCode::KeyW => Some('w'),
        kivact_types::KeyCode::KeyX => Some('x'),
        kivact_types::KeyCode::KeyY => Some('y'),
        kivact_types::KeyCode::KeyZ => Some('z'),
        _ => None,
    }
}
