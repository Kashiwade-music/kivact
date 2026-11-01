//! Role: Shell event conversion and the client's start; callbacks stay on the UI thread.
//!
//! Input goes to the model (`model::pointer::shell`, `model::keys`,
//! `model::edit`), which owns what it means; the clipboard work a key
//! leaves (copy, paste) is done here, with the window's clipboard.
//!
//! Invariants:
//! - What the client sees of the window's environment is one `HostEnv`,
//!   kept by the model's frame group. At start the window's environment
//!   (`Window::environment`: scale, refresh period, reduced motion, theme,
//!   safe area, mouse settings, locale), its size and fullscreen state and
//!   its capabilities (`Window::capabilities`) are reported in `HostInfo`.
//!   The theme is reported light at start and follows the system's from
//!   its first change.
//! - Every change goes through `Model::update_env`, which sends one
//!   `EnvChanged` (the whole value) only when the value differs from the
//!   last one sent. `ShellEvent::EnvironmentChanged` reads the environment
//!   again (theme, reduced motion, refresh period, safe area, locale); the
//!   mouse settings are Arvane's (it counts clicks and scales wheels). A
//!   new scale, size or fullscreen state arrives as `Resized`.
//! - The host's locale (`Environment::locale`, a BCP 47 tag or `None`) is
//!   `HostEnv::locale`; when it is read again is Arvane's.
//! - Frames are drawn only while the application is resumed
//!   (`Suspended`/`Resumed`), the window has a surface
//!   (`Window::surface_generation`, `SurfaceDestroyed`/`SurfaceCreated`)
//!   and is not empty (`HostEnv::surface_active`). In between no redraw is
//!   asked for and no frame tick is sent (the client's frame clock stops);
//!   a resume and a new surface draw again.
//! - A press and a release carry the click count Arvane counted
//!   (`PointerInput::clicks`; a release has its press's), and a
//!   wheel the amount Arvane made logical pixels; the model is given them
//!   as they are. A wheel's native scroll runs the layout pass at once, so
//!   `ScrollChanged` is reported without waiting for the next frame.
//! - A touch or pen contact (`ShellEvent::Pointer`) is given to the model
//!   as its pointer's primary-button press, move and release
//!   (`Model::contact`), with Arvane's tap count; what a contact does is
//!   the model's. The window losing the focus cancels every pointer.
//! - `MemoryWarning` releases unused asset and text engine caches
//!   (`Engine::release_caches`: registered fonts and owned runs already
//!   returned are kept) and is reported to the client.
use super::Host;
use crate::KeyResult;
use crate::host::HostWaker;
use arvane::Window;
use arvane::{NativeWindow, Presented, ShellControl, ShellError, ShellEvent, ShellWaker};
use kivact_lower::conversion as c;
use kivact_wire::*;
impl arvane::Handler for Host {
    fn start(
        &mut self,
        window: &mut NativeWindow<'_>,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        let store_waker = waker.clone();
        let store = arvane::asset::AssetStore::new(move || store_waker.wake());
        self.begin(window, HostWaker::new(move || waker.wake()), store)
    }
    fn event(
        &mut self,
        event: ShellEvent,
        window: &mut NativeWindow<'_>,
    ) -> Result<ShellControl, ShellError> {
        self.handle(event, window)
    }
}
impl Host {
    /// Start the client in `window`, loading images with `store`: the
    /// shell's start, and the test driver's.
    pub(super) fn begin<W: Window>(
        &mut self,
        window: &mut W,
        waker: HostWaker,
        store: arvane::asset::AssetStore,
    ) -> Result<ShellControl, ShellError> {
        self.environment = window.environment();
        let env = {
            let mut model = self.model.borrow_mut();
            model.set_viewport(window.size());
            // The theme is the one the model starts with (light); the
            // rest is the window's.
            let env = model.frame.start_env();
            env.scale = window.scale();
            env.reduced_motion = self.environment.reduced_motion;
            env.frame_period_ns = self.environment.frame_period_ns;
            env.safe_area = c::insets(self.environment.safe_area);
            env.window = window.size();
            env.locale = self.environment.locale.clone();
            env.fullscreen = window.fullscreen();
            env.clone()
        };
        self.frame_log
            .borrow_mut()
            .note(&format!("frame_period_ns={}", env.frame_period_ns));
        let info = HostInfo {
            capabilities: window.capabilities(),
            now_ns: window.now_ns(),
            env,
        };
        {
            let mut model = self.model.borrow_mut();
            model.paint.set_scale(window.scale());
            model.set_store(store);
        }
        let tree = super::write::Writer::new(&self.model, &self.engine);
        if let Some(diagnostic) = self.client.start(&info, waker, tree) {
            return Err(ShellError::new(diagnostic));
        }
        self.effects(window);
        if self.closing() {
            return Ok(ShellControl::default());
        }
        #[cfg(feature = "gpu-interop")]
        self.externals(window);
        self.layout();
        self.pump(window)
    }
    /// Handle one shell event in `window`: the shell's events, and the
    /// ones the test driver injects.
    pub(super) fn handle<W: Window>(
        &mut self,
        event: ShellEvent,
        window: &mut W,
    ) -> Result<ShellControl, ShellError> {
        if self.closing() {
            return Ok(ShellControl::default());
        }
        self.starting = false;
        self.effects(window);
        if self.closing() {
            return Ok(ShellControl::default());
        }
        self.model.borrow_mut().frame.set_now(window.now_ns());
        if stops_smooth_scrolls(&event) {
            self.model.borrow_mut().frame.stop_smooth_scrolls();
        }
        if matches!(event, ShellEvent::File { .. }) {
            self.layout();
        }
        let input = matches!(
            event,
            ShellEvent::Pointer(_)
                | ShellEvent::Wheel { .. }
                | ShellEvent::Modifiers(_)
                | ShellEvent::Key { .. }
                | ShellEvent::Preedit { .. }
                | ShellEvent::Commit(_)
                | ShellEvent::ImeSelection { .. }
                | ShellEvent::ImeDisabled
        );
        if hit_tests(&event) {
            // Deferred turns leave edits laid out lazily; hit test current geometry.
            self.layout();
        }
        match event {
            ShellEvent::Redraw => {
                let surface_active = self.model.borrow().frame.surface_active();
                if surface_active {
                    let started = std::time::Instant::now();
                    self.layout();
                    match self.present(window, started) {
                        Ok(Presented::Shown { .. }) => {
                            self.model.borrow_mut().paint.drawn();
                            self.skipping = false;
                        }
                        // The window had no buffer (timed out, occluded):
                        // the picture is drawn again when Arvane says.
                        Ok(Presented::Skipped { retry_at_ns }) => {
                            self.model.borrow_mut().paint.drawn();
                            self.skipping = true;
                            self.repaint_at = Some(retry_at_ns);
                        }
                        // Content cannot render; the next change redraws.
                        Ok(Presented::Dropped(_)) => self.model.borrow_mut().paint.drawn(),
                        // The window made its surface again: report the
                        // loss and draw again.
                        Ok(Presented::SurfaceLost(error)) => {
                            let mut model = self.model.borrow_mut();
                            model.set_surface(false);
                            let revision = model.applied();
                            model.push(HostEvent::HostFailure {
                                revision,
                                reason: error.to_string(),
                            });
                            model.set_surface(true);
                            model.invalidate_paint();
                        }
                        Err(error) => return Err(ShellError::new(error.to_string())),
                    }
                }
            }
            ShellEvent::Wake => {
                if self.repaint_at.is_some_and(|t| t <= window.now_ns()) {
                    self.repaint_at = None;
                    self.model.borrow_mut().invalidate_paint();
                }
                let events = {
                    let mut model = self.model.borrow_mut();
                    model.tick(window.now_ns());
                    // The asset store's worker wakes the shell too; its
                    // reports are events, delivered by the pump below.
                    model.poll_assets();
                    model.has_events()
                };
                // A due wake without events still gives the client its turn.
                if !events && !self.exit {
                    self.turn(&[], &[], window)?;
                }
            }
            ShellEvent::Resized => {
                let size = window.size();
                let drawable = self.drawable(window);
                let mut model = self.model.borrow_mut();
                model.set_viewport(size);
                model.invalidate_paint();
                model.update_env(|env| {
                    env.scale = window.scale();
                    env.window = size;
                    env.fullscreen = window.fullscreen();
                    env.surface_active = drawable;
                });
                drop(model);
                self.layout();
            }
            ShellEvent::CloseRequested => self.model.borrow_mut().push(HostEvent::CloseRequested),
            ShellEvent::Focused(false) => self
                .model
                .borrow_mut()
                .cancel_pointer(&mut self.engine.borrow_mut()),
            // A window coming back (after skipped frames) draws at once.
            ShellEvent::Focused(true) if self.skipping => {
                self.repaint_at = None;
                self.model.borrow_mut().invalidate_paint();
            }
            ShellEvent::Focused(true) => {}
            ShellEvent::Suspended => {
                if !self.suspended {
                    self.suspended = true;
                    self.model
                        .borrow_mut()
                        .cancel_pointer(&mut self.engine.borrow_mut());
                    self.report_drawable(window);
                    self.model
                        .borrow_mut()
                        .push(HostEvent::Lifecycle(LifecycleEvent::Suspended));
                }
            }
            ShellEvent::Resumed => {
                if self.suspended {
                    self.suspended = false;
                    self.report_drawable(window);
                    self.model
                        .borrow_mut()
                        .push(HostEvent::Lifecycle(LifecycleEvent::Resumed));
                    self.model.borrow_mut().invalidate_paint();
                }
            }
            ShellEvent::SurfaceDestroyed => self.report_drawable(window),
            ShellEvent::SurfaceCreated => {
                self.report_drawable(window);
                self.model.borrow_mut().invalidate_paint();
            }
            // Rebuildable host caches are released before application callbacks.
            ShellEvent::MemoryWarning => {
                window.release_caches();
                self.engine.borrow_mut().release_caches();
                self.model.borrow().paint.scene_cache.borrow_mut().clear();
                self.model.borrow().assets.borrow_mut().release_caches();
                self.frame_log.borrow_mut().note("memory_warning");
                self.model
                    .borrow_mut()
                    .push(HostEvent::Lifecycle(LifecycleEvent::MemoryWarning));
            }
            ShellEvent::Pointer(input) => self
                .model
                .borrow_mut()
                .pointer_input(input, &mut self.engine.borrow_mut()),
            ShellEvent::Wheel { x, y } => {
                self.model
                    .borrow_mut()
                    .wheel(Point { x, y }, &mut self.engine.borrow_mut());
                // A native scroll is reported (`ScrollChanged`) by the
                // layout pass: run it now, so the client's turn is due at
                // once instead of waiting for the next paint.
                self.layout();
            }
            ShellEvent::Modifiers(m) => self.model.borrow_mut().modifiers = m,
            ShellEvent::File {
                phase,
                files,
                position,
            } => self.model.borrow_mut().file(phase, files, position),
            ShellEvent::KeyUp { key, code } => self.model.borrow_mut().key_up(key, code),
            ShellEvent::Key {
                key,
                physical,
                code,
                repeat,
                text,
            } => {
                let result = self.model.borrow_mut().key(
                    key,
                    physical,
                    code,
                    repeat,
                    text,
                    &mut self.engine.borrow_mut(),
                );
                match result {
                    // A cut already removed the text.
                    KeyResult::Copy(text) => {
                        let _ = window.copy_text(&text);
                    }
                    KeyResult::Paste(id) => {
                        if let Ok(text) = window.read_text() {
                            self.model
                                .borrow_mut()
                                .paste(id, &text, &mut self.engine.borrow_mut());
                        }
                    }
                    KeyResult::Reported | KeyResult::Consumed | KeyResult::Text(_) => {}
                }
            }
            ShellEvent::Preedit { text, selection } => {
                self.model.borrow_mut().preedit(&text, selection)
            }
            ShellEvent::Commit(text) => self.model.borrow_mut().commit(&text),
            ShellEvent::ImeSelection { start, end } => {
                self.model.borrow_mut().ime_select(start, end)
            }
            ShellEvent::ImeDisabled => self.model.borrow_mut().ime_disabled(),
            ShellEvent::EnvironmentChanged => self.environment_changed(window),
            ShellEvent::Accessibility {
                node,
                action,
                value,
            } => self
                .model
                .borrow_mut()
                .accessibility_action(node, action, value),
        }
        if self.closing() {
            return Ok(ShellControl::default());
        }
        if input {
            return Ok(self.defer(window));
        }
        self.pump(window)
    }
    /// Whether frames can be drawn (`HostEnv::surface_active`; no frame
    /// ticks otherwise): the application is in the foreground, the window
    /// has a surface and is not empty (minimized).
    fn drawable<W: Window>(&self, window: &W) -> bool {
        let size = window.size();
        !self.suspended
            && window.surface_generation().is_some()
            && size.width > 0.0
            && size.height > 0.0
    }
    /// Report whether frames can be drawn, when that changed.
    fn report_drawable<W: Window>(&mut self, window: &W) {
        let drawable = self.drawable(window);
        self.model.borrow_mut().set_surface(drawable);
    }
    /// The window's environment may have changed: what the client sees of
    /// it (theme, reduced motion, the refresh period frames are paced at,
    /// the safe area, the locale) is reported in one `EnvChanged` when any
    /// of it did. The mouse settings are Arvane's own (it counts clicks and
    /// scales wheels).
    fn environment_changed<W: Window>(&mut self, window: &W) {
        let now = window.environment();
        let before = std::mem::replace(&mut self.environment, now.clone());
        let mut model = self.model.borrow_mut();
        let period_before = model.frame.period_ns();
        model.update_env(|env| {
            if now.dark != before.dark {
                env.theme_hint = if now.dark {
                    ThemeHint::Dark
                } else {
                    ThemeHint::Light
                };
            }
            env.reduced_motion = now.reduced_motion;
            // Another monitor: the frame period follows its refresh rate.
            env.frame_period_ns = now.frame_period_ns;
            env.safe_area = c::insets(now.safe_area);
            env.locale = now.locale.clone();
        });
        let period = model.frame.period_ns();
        drop(model);
        if period != period_before {
            self.frame_log
                .borrow_mut()
                .note(&format!("frame_period_ns={period}"));
        }
    }
}

/// Whether `event` stops the smooth scrolls in progress: a wheel, a mouse
/// press, or any event of a touch or pen contact.
fn stops_smooth_scrolls(event: &ShellEvent) -> bool {
    match event {
        ShellEvent::Wheel { .. } => true,
        ShellEvent::Pointer(input) => {
            input.kind != PointerKind::Mouse || input.phase == PointerPhase::Down
        }
        _ => false,
    }
}

/// Whether `event` is hit tested: a wheel, the mouse's moves and buttons
/// (not its leave or cancel), or any event of a contact.
fn hit_tests(event: &ShellEvent) -> bool {
    match event {
        ShellEvent::Wheel { .. } => true,
        ShellEvent::Pointer(input) => {
            input.kind != PointerKind::Mouse
                || !matches!(input.phase, PointerPhase::Leave | PointerPhase::Cancel)
        }
        _ => false,
    }
}
