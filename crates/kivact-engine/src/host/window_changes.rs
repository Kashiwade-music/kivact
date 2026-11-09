//! Role: Apply window changes and finish one pump's per-window work.
//!
//! The client writes its window changes into the shared `AppModel` during a
//! turn; the host performs them after it (Arvane's windows exist only here),
//! starts each new window's model from the window it opened, and finishes
//! every window after the turns.

use super::devtools::{apply_emulation, effective_scale, emulated_viewport};
use super::pump::Host;
use super::window::{HostWindow, surface_of};
use crate::contract::HostEvent;
use crate::model::{AppClock, WindowChange};
use arvane::Windows;
use kivact_types::{Environment, WindowId};

impl Host {
    /// Perform the window changes of the turn, in order.
    pub(super) fn apply_window_changes(&mut self, windows: &mut Windows<'_>) {
        let changes = self.app.borrow_mut().take_window_changes();
        for change in changes {
            match change {
                WindowChange::Open(request) => {
                    let id = request.id;
                    match windows.open(request.clone()) {
                        Ok(()) => {
                            self.windows
                                .insert(id, HostWindow::new(request.mode, request.parent));
                            let (environment, surface_active) = self.start_window(id, windows);
                            self.app.borrow().push_event(HostEvent::WindowOpened {
                                window: id,
                                environment,
                                surface_active,
                            });
                        }
                        Err(error) => {
                            self.app.borrow_mut().remove_window(id);
                            self.windows.remove(&id);
                            self.app.borrow().push_event(HostEvent::WindowFailed {
                                window: id,
                                reason: error.to_string(),
                            });
                        }
                    }
                }
                WindowChange::Close(id) => {
                    if self.windows.remove(&id).is_some() {
                        windows.close(id);
                        self.app.borrow_mut().remove_window(id);
                    }
                    self.devtools.overlays.remove(&id);
                    if self.devtools.pick == Some(id) {
                        self.devtools.pick = None;
                    }
                }
            }
        }
    }

    /// Start window `id`'s model from the window: its environment, surface,
    /// viewport and frame clock, and the main window's theme. Returns the
    /// environment and whether the window can draw.
    pub(super) fn start_window(
        &mut self,
        id: WindowId,
        windows: &mut Windows<'_>,
    ) -> (Environment, bool) {
        let window = windows
            .get(id)
            .expect("invariant: a started window is open");
        let native = window.environment();
        let size = window.size();
        let scale = window.scale();
        let fullscreen = window.fullscreen();
        let now = window.now_ns();
        let emulation = self.devtools.emulation;
        let viewport = emulated_viewport(size, scale, emulation.scale);
        let paint_scale = effective_scale(scale, emulation.scale);
        let surface = surface_of(&window);
        self.windows
            .get_mut(&id)
            .expect("invariant: a started window has host state")
            .environment
            .update(native.clone());
        self.windows
            .get_mut(&id)
            .expect("invariant: a started window has host state")
            .surface = surface;
        let drawable = self.drawable(id);
        let theme = self
            .app
            .borrow()
            .model(WindowId::MAIN)
            .map(|model| model.frame.theme_hint())
            .unwrap_or_default();
        let env = {
            let mut app = self.app.borrow_mut();
            let model = app
                .model_mut(id)
                .expect("invariant: a started window has a model");
            model.set_viewport(viewport);
            model.frame.set_now(now);
            let mut env = native;
            env.scale_factor = paint_scale;
            env.window_size = size;
            env.fullscreen = fullscreen;
            env.theme_hint = theme;
            env.reduced_motion = self.reduced_motion_override.unwrap_or(env.reduced_motion);
            apply_emulation(emulation, &mut env);
            model.start_env(env.clone(), drawable);
            env
        };
        if let Some(model) = self.app.borrow_mut().model_mut(id) {
            model.paint.set_scale(paint_scale);
        }
        (env, drawable)
    }

    /// The window's environment may have changed: what the client sees of
    /// it (theme, reduced motion, the refresh period frames are paced at,
    /// the safe area, the locale) is reported in one `EnvChanged` when any
    /// of it did. The mouse settings are Arvane's own (it counts clicks and
    /// scales wheels).
    pub(super) fn environment_changed(&mut self, id: WindowId, window: &arvane::Window<'_>) {
        let mut now = window.environment();
        let before = match self.windows.get_mut(&id) {
            Some(host_window) => host_window
                .environment
                .get()
                .expect("invariant: environment is set at start")
                .clone(),
            None => return,
        };
        now.fonts = self.app.borrow().assets.borrow().fonts();
        if let Some(host_window) = self.windows.get_mut(&id) {
            host_window.environment.update(now.clone());
        }
        let period_before = {
            let app = self.app.borrow();
            app.model(id).map_or(0, |model| model.frame.period_ns())
        };
        if let Some(model) = self.app.borrow_mut().model_mut(id) {
            let reduced_motion_override = self.reduced_motion_override;
            let emulation = self.devtools.emulation;
            model.update_env(|env, _| {
                if now.theme_hint != before.theme_hint {
                    env.theme_hint = now.theme_hint;
                }
                env.reduced_motion = reduced_motion_override.unwrap_or(now.reduced_motion);
                // Another monitor: the frame period follows its refresh rate.
                env.frame_period_ns = now.frame_period_ns;
                env.safe_area = now.safe_area.sanitize();
                env.locale = now.locale.clone();
                env.pointer = now.pointer;
                env.keyboard_inset = now.keyboard_inset;
                apply_emulation(emulation, env);
            });
        }
        let period = {
            let app = self.app.borrow();
            app.model(id).map_or(0, |model| model.frame.period_ns())
        };
        if period != period_before {
            self.frame_log
                .borrow_mut()
                .note(&format!("frame_period_ns={period}"));
        }
    }

    /// Finish window `id` after the turns: its title and minimum size, IME,
    /// cursor, redraw request and accessibility publication.
    pub(super) fn finish_window(&mut self, id: WindowId, windows: &mut Windows<'_>) {
        let title = {
            let mut app = self.app.borrow_mut();
            app.model_mut(id)
                .and_then(|model| model.window_request.take())
        };
        if let Some((title, min_size)) = title
            && let Some(window) = windows.get(id)
        {
            window.set_title(&title);
            window.set_min_size(min_size);
        }
        if let Some(mut window) = windows.get(id) {
            self.synchronize_ime(id, &mut window);
        }
        self.show_cursor(id, windows);
        let now = windows.now_ns();
        let retry = {
            let app = self.app.borrow();
            app.model(id)
                .is_some_and(|model| model.drawables.retry_due(now))
        };
        if retry && let Some(model) = self.app.borrow_mut().model_mut(id) {
            model.paint.request_redraw();
        }
        // Without a surface (minimized) a redraw draws nothing and leaves
        // the paint dirty, so asking for one would spin; the restore asks.
        let paint_due = self.paint_due(id);
        if paint_due && let Some(window) = windows.get(id) {
            window.request_redraw();
        }
        // With a frame to draw, the native window updates assistive
        // technology after it (the redraw pumps again): after a layout pass
        // the update is O(nodes) on both sides of AccessKit, and the frame
        // need not wait for it.
        if !paint_due
            && let Some(mut window) = windows.get(id)
            && let Some(model) = self.app.borrow_mut().model_mut(id)
        {
            model.accessibility(&mut window, &mut self.engine.borrow_mut());
        }
    }

    /// Stop or resume the App's motion clock and report its input when it
    /// changed.
    pub(super) fn finish_motion(&mut self, now: u64) {
        let drawable = self.windows.keys().any(|id| self.drawable(*id));
        let mut app = self.app.borrow_mut();
        match app.clock {
            AppClock::Running if !drawable => {
                app.clock = AppClock::Stopped { since_ns: now };
            }
            AppClock::Stopped { since_ns } if drawable => {
                app.motion.offset_ns = app
                    .motion
                    .offset_ns
                    .saturating_add(now.saturating_sub(since_ns));
                app.motion.resumed_ns = now;
                app.clock = AppClock::Running;
            }
            _ => {}
        }
        let period = if drawable {
            let id = self
                .windows
                .keys()
                .copied()
                .find(|id| self.drawable(*id))
                .expect("invariant: a drawable window exists");
            app.model(id).map_or(0, |model| model.frame.period_ns())
        } else {
            app.model(WindowId::MAIN)
                .map_or(0, |model| model.frame.period_ns())
        };
        if self.app_motion.update((drawable, period)).is_some() {
            app.events.push(HostEvent::AppMotion {
                drawable,
                frame_period_ns: period,
            });
        }
    }
}
