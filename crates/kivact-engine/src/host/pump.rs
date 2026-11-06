//! Role: Run host turns, effects, layout and frame presentation.

use super::client::{Boundary, Client};
use crate::contract::HostEvent;
use crate::model::edit::state::ImeUpdate;
use crate::model::{DeferredRequest, Model};
use crate::{FrameLog, FrameTimes};
use arvane::{Presented, ShellControl, ShellError, Window};
use kivact_types::cell::{ReportOnce, Sent};
use kivact_types::{PlatformRequest, PlatformResult};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

/// Turns one pump runs before it lets the window draw (each turn left new
/// events); the cut is reported (`HostEvent::TurnsCut`).
const TURN_LIMIT: u32 = 64;

/// Where the host is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HostPhase {
    /// Inside the shell's start (`Host::begin`): the client starts and the first
    /// pump's turns run. Platform effects wait: an embedding host is still inside
    /// its editor's open call and its window takes no size request or dialog then.
    /// The first entry after the start carries them out (`Host::enter`), and the
    /// host asks for that entry at once (`ShellControl::WakeAt(now)` while effects
    /// are queued).
    Starting,
    /// Handling the shell's events.
    Running,
    /// The client finished (`TurnOutput::exit`): no more turns; every return asks
    /// the shell to exit.
    Exiting,
    Closed,
}

/// What became of the last frame presented.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LastPresent {
    /// Shown, dropped or lost: nothing is retried (a change or a surface loss
    /// redraws).
    Settled,
    /// The window had no buffer (occluded, timed out): the frame is drawn again at
    /// `retry_at_ns`, or at once when the window regains the focus.
    Skipped { retry_at_ns: u64 },
}

pub(crate) struct Host {
    pub(super) client: Client,
    pub(super) phase: HostPhase,
    /// Shared with the client's reader (`write::Reader`): borrowed for one
    /// call at a time, never across a call into the client.
    pub(super) model: Rc<RefCell<Model>>,
    /// Arvane's text, shared with the client's reader (`write::Reader`), which
    /// measures text for drawing: borrowed for one call at a time.
    pub(super) engine: Rc<RefCell<arvane::Engine>>,
    /// What became of the last frame presented (`Presented`).
    pub(super) last_present: LastPresent,
    // xtask-allow(state-fields): independent client deadline, valid in every host phase.
    pub(super) next_wake: Option<u64>,
    /// Embedded handle destruction can occur inside a platform callback while
    /// this Host is borrowed. Stop new turns before deferred native cleanup;
    /// an already-running client turn still settles under the normal rules.
    pub(super) close: crate::contract::CloseSignal,
    /// The window's environment as last read: a change of the system's
    /// theme is found by comparing with it (the theme reported at start is
    /// the host's own, not the snapshot's).
    pub(super) environment: Sent<kivact_types::Environment>,
    // xtask-allow(state-fields): optional user setting, independent of host lifecycle.
    pub(super) reduced_motion_override: Option<bool>,
    /// The platform suspended the application (in the background):
    /// nothing is drawn until it resumes. Independent of `phase`.
    pub(super) presence: AppPresence,
    pub(super) surface: Surface,
    /// The software keyboard was asked for (an editable text input has
    /// the focus).
    pub(super) soft_keyboard: kivact_types::cell::Sent<SoftKeyboard>,
    pub(super) ime_allowed: kivact_types::cell::Sent<ImeAllowed>,
    /// Per-frame timings for benchmarks (`KIVACT_FRAME_LOG`), shared with
    /// the window's renderer observer.
    pub(super) frame_log: Rc<RefCell<FrameLog>>,
    /// The frame log noted that assistive technology reads the tree.
    pub(super) accessibility_noted: ReportOnce,
    /// The window's adapter the client was told of last
    /// (`HostEvent::GpuAdapterChanged`).
    pub(super) adapter: Sent<kivact_types::GpuAdapter>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AppPresence {
    Foreground,
    Suspended,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Surface {
    Drawable { generation: u64 },
    Unavailable(Unavailable),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Unavailable {
    NoSurface,
    ZeroSize,
}

impl Host {
    pub(super) fn observe_close(&mut self) -> bool {
        if self.close.is_set() {
            self.phase = HostPhase::Closed;
        }
        self.phase == HostPhase::Closed
    }
    pub(super) fn drawable(&self) -> bool {
        self.phase != HostPhase::Closed
            && self.presence == AppPresence::Foreground
            && matches!(self.surface, Surface::Drawable { .. })
    }
    /// The first step of every entry after the shell's start (`handle`, the test
    /// driver's `inject`): the start is over, and the platform effects queued
    /// meanwhile are carried out.
    pub(super) fn enter(&mut self, window: &mut Window<'_>) {
        if self.phase == HostPhase::Starting {
            self.phase = HostPhase::Running;
        }
        self.effects(window);
    }
    /// Queue `event` and pump (the test driver's injection).
    #[cfg(feature = "testing")]
    pub(super) fn inject(
        &mut self,
        event: HostEvent,
        window: &mut Window<'_>,
    ) -> Result<ShellControl, ShellError> {
        if self.observe_close() {
            return Ok(ShellControl::Wait);
        }
        self.enter(window);
        if self.phase == HostPhase::Closed {
            return Ok(ShellControl::Wait);
        }
        self.model.borrow_mut().push(event);
        self.pump(window)
    }
    pub(super) fn effects(&mut self, window: &mut Window<'_>) {
        if self.phase == HostPhase::Starting || self.phase == HostPhase::Closed {
            return;
        }
        let effects = std::mem::take(&mut self.model.borrow_mut().effects);
        for (request, effect) in effects {
            let result = match effect {
                DeferredRequest::Platform(request) => {
                    let fullscreen_request = matches!(request, PlatformRequest::Fullscreen(_));
                    let result = window.perform(request);
                    if fullscreen_request {
                        let fullscreen = window.fullscreen();
                        self.model
                            .borrow_mut()
                            .update_env(|env, _| env.fullscreen = fullscreen);
                    }
                    result
                }
                DeferredRequest::ReducedMotion(override_value) => {
                    self.reduced_motion_override = override_value;
                    self.model.borrow_mut().update_env(|env, _| {
                        env.reduced_motion = override_value.unwrap_or(
                            self.environment
                                .get()
                                .expect("invariant: environment is set at start")
                                .reduced_motion,
                        );
                    });
                    PlatformResult::Done
                }
            };
            if self.observe_close() {
                return;
            }
            self.model
                .borrow_mut()
                .push(HostEvent::EffectResult { request, result });
        }
    }
    /// Hand the window the imports and drops of shared textures the turn
    /// wrote, in order; answer each import.
    pub(super) fn externals(&mut self, window: &mut Window<'_>) {
        use crate::model::ExternalWrite;
        use kivact_types::ExternalImport;
        let writes = std::mem::take(&mut self.model.borrow_mut().external_writes);
        for write in writes {
            match write {
                ExternalWrite::Import(image, textures) => {
                    let result = match window.import_external_image(image, textures) {
                        Ok(()) => ExternalImport::Ready {
                            generation: window.gpu_adapter().map_or(0, |a| a.generation),
                        },
                        Err(e) if e.kind == kivact_types::FailureKind::Unavailable => {
                            ExternalImport::Unavailable(e.to_string())
                        }
                        Err(e) => ExternalImport::Failed(e.to_string()),
                    };
                    self.model
                        .borrow_mut()
                        .push(HostEvent::ExternalImported { image, result });
                }
                ExternalWrite::Drop(image) => window.drop_external_image(image),
            }
        }
    }
    /// Tell the client when the window's adapter changed.
    fn adapter_changed(&mut self, window: &Window<'_>) {
        let adapter = window.gpu_adapter();
        let changed = match &adapter {
            Some(value) => self.adapter.update(value.clone()).is_some(),
            None if self.adapter.get().is_some() => {
                self.adapter = Sent::new();
                true
            }
            None => false,
        };
        if changed {
            self.model
                .borrow_mut()
                .push(HostEvent::GpuAdapterChanged(adapter));
        }
    }
    /// Draw and show one frame. `started` is when the redraw began (its
    /// layout included); the frame log gets the scene, raster and present
    /// spans of a frame shown.
    pub(super) fn present(
        &mut self,
        window: &mut Window<'_>,
        started: Instant,
    ) -> Result<Presented, kivact_types::Failure> {
        let mut model = self.model.borrow_mut();
        model.scroll_inputs(&mut self.engine.borrow_mut());
        let plan = window.frame_plan();
        // Images are rasterized for the scale the frame is drawn at.
        model.paint.set_scale(plan.scale);
        let t_scene = Instant::now();
        let scene = model.scene(&mut self.engine.borrow_mut());
        drop(model);
        let t_raster = Instant::now();
        let presented = window.present(&scene, plan)?;
        if let Presented::Shown {
            raster, present, ..
        } = presented
        {
            let mut frame_log = self.frame_log.borrow_mut();
            if frame_log.enabled() {
                let end = Instant::now();
                frame_log.frame(FrameTimes {
                    work: end - started,
                    scene: t_raster - t_scene,
                    raster,
                    present,
                });
            }
        }
        Ok(presented)
    }
    /// One client turn: it writes the tree (each write applied at once),
    /// then its platform effects are answered and the tree laid out before
    /// anything else runs, so the next turn sees the effect results and the
    /// new geometry.
    pub(super) fn turn(
        &mut self,
        events: &[HostEvent],
        boundaries: &[Boundary],
        window: &mut Window<'_>,
    ) -> Result<(), ShellError> {
        if self.phase == HostPhase::Closed {
            return Ok(());
        }
        let drawable = self.drawable();
        self.model.borrow_mut().set_surface(drawable);
        self.model.borrow_mut().frame.set_now(window.now_ns());
        let output = self.client.turn(events, boundaries, window.now_ns());
        if self.observe_close() {
            return Ok(());
        }
        self.next_wake = output.next_wake_ns;
        if output.exit {
            self.phase = HostPhase::Exiting;
        }
        self.effects(window);
        if self.phase == HostPhase::Closed {
            return Ok(());
        }
        self.externals(window);
        self.settle();
        Ok(())
    }
    pub(super) fn settle(&mut self) {
        self.model
            .borrow_mut()
            .settle(&mut self.engine.borrow_mut());
    }
    pub(super) fn pump(&mut self, window: &mut Window<'_>) -> Result<ShellControl, ShellError> {
        if self.phase == HostPhase::Closed {
            return Ok(ShellControl::Wait);
        }
        self.adapter_changed(window);
        self.model.borrow_mut().tick(window.now_ns());
        let mut turns = 0;
        loop {
            if self.phase == HostPhase::Closed {
                // Native handle close owns shutdown; exit=true here could race
                // that request and be diagnosed as an App-requested failure.
                return Ok(ShellControl::Wait);
            }
            // The store's finished work (natural sizes, rasters, failures)
            // lands before each turn, so the turn sees its reports; every
            // turn starts on a laid-out tree.
            let next = {
                let mut model = self.model.borrow_mut();
                model.poll_assets();
                model.settle(&mut self.engine.borrow_mut());
                if !model.has_events() {
                    model.snapshot_requests(&mut self.engine.borrow_mut());
                }
                if self.phase == HostPhase::Exiting || !model.has_events() {
                    None
                } else if turns == TURN_LIMIT {
                    // Every turn left more events: the client keeps answering
                    // its own reports. Say so, and let the window draw.
                    model.push(HostEvent::TurnsCut { turns });
                    None
                } else {
                    // Every queued event, with the commit boundaries among
                    // them: all of their revisions are laid out now.
                    let (events, boundaries) = model.events.drain();
                    let boundaries = boundaries
                        .into_iter()
                        .map(|(events, revision)| Boundary { events, revision })
                        .collect::<Vec<_>>();
                    Some((events, boundaries))
                }
            };
            // The callback may read or write the model: no borrow is held.
            let Some((events, boundaries)) = next else {
                break;
            };
            turns += 1;
            self.turn(&events, &boundaries, window)?;
        }
        let title = self.model.borrow_mut().window_request.take();
        if let Some((title, min_size)) = title {
            window.set_title(&title);
            window.set_min_size(min_size);
        }
        self.synchronize_ime(window);
        self.show_cursor(window);
        // Without a surface (minimized) a redraw draws nothing and leaves
        // the paint dirty, so asking for one would spin; the restore asks.
        if self.paint_due() {
            window.request_redraw();
        }
        // With a frame to draw, the native window updates assistive
        // technology after it (the redraw pumps again): after a layout pass
        // the update is O(nodes) on both sides of AccessKit, and the frame
        // need not wait for it.
        let after_frame = self.paint_due();
        if !after_frame {
            self.model
                .borrow_mut()
                .accessibility(window, &mut self.engine.borrow_mut());
        }
        Ok(self.control(window))
    }
    fn synchronize_ime(&mut self, window: &mut Window<'_>) {
        let caret = {
            let mut model = self.model.borrow_mut();
            // A native editor's caret first; else the caret of a node with an
            // IME session.
            model
                .caret(&mut self.engine.borrow_mut())
                .or_else(|| model.session_area())
        };
        let allowed = if caret.is_some() {
            ImeAllowed::Yes
        } else {
            ImeAllowed::No
        };
        if let Some(allowed) = self.ime_allowed.update(allowed) {
            window.set_ime_allowed(*allowed == ImeAllowed::Yes);
        }
        // The software keyboard follows the focus of an editable text
        // input (a no-op where the platform has none).
        let keyboard = if caret.is_some() {
            SoftKeyboard::Shown
        } else {
            SoftKeyboard::Hidden
        };
        if let Some(keyboard) = self.soft_keyboard.update(keyboard) {
            window.set_soft_keyboard(*keyboard == SoftKeyboard::Shown);
        }
        if let Some(caret) = caret {
            window.set_ime_cursor_area(caret);
        }
        window.set_ime_purpose(self.model.borrow().ime_purpose());
        let update = self
            .model
            .borrow_mut()
            .ime_changed(&mut self.engine.borrow_mut());
        if let ImeUpdate::Set(text) = update {
            window.set_ime_text(text);
        }
    }
    /// Input is applied to the host model at once, but the client turn waits
    /// until the native queue is drained: a due wake runs it for every input
    /// that arrived meanwhile, so bursts (typing, drags) cost one turn,
    /// layout and frame instead of one each.
    pub(super) fn defer(&mut self, window: &mut Window<'_>) -> ShellControl {
        if self.paint_due() {
            window.request_redraw();
        }
        // The IME may ask about the text right after the input that
        // edited it (a reconversion after a selection).
        window.set_ime_purpose(self.model.borrow().ime_purpose());
        let update = self
            .model
            .borrow_mut()
            .ime_changed(&mut self.engine.borrow_mut());
        if let ImeUpdate::Set(text) = update {
            window.set_ime_text(text);
        }
        self.show_cursor(window);
        self.control(window)
    }
    /// Show the cursor the hover chain asks for (the model's rule),
    /// when it changed.
    fn show_cursor(&mut self, window: &Window<'_>) {
        let mut model = self.model.borrow_mut();
        let cursor = model.cursor();
        if model.pointer.show(cursor) {
            window.set_cursor(cursor);
        }
    }
    /// A frame is due: the picture changed and there is a surface.
    fn paint_due(&self) -> bool {
        let model = self.model.borrow();
        model.paint.needs_redraw() && self.drawable()
    }
    /// When the skipped frame is drawn again.
    fn retry_at(&self) -> Option<u64> {
        match self.last_present {
            LastPresent::Settled => None,
            LastPresent::Skipped { retry_at_ns } => Some(retry_at_ns),
        }
    }
    fn control(&self, window: &Window<'_>) -> ShellControl {
        if self.phase == HostPhase::Closed {
            return ShellControl::Wait;
        }
        let model = self.model.borrow();
        let frame_deadline = if self.drawable() {
            model.frame_deadline()
        } else {
            None
        };
        let next = if model.has_events() || !model.effects.is_empty() {
            Some(window.now_ns())
        } else {
            [self.next_wake, frame_deadline, self.retry_at()]
                .into_iter()
                .flatten()
                .min()
        };
        if self.phase == HostPhase::Exiting {
            ShellControl::Exit
        } else {
            next.map_or(ShellControl::Wait, ShellControl::WakeAt)
        }
    }
}

#[derive(PartialEq)]
pub(super) enum SoftKeyboard {
    Hidden,
    Shown,
}
#[derive(PartialEq)]
pub(super) enum ImeAllowed {
    No,
    Yes,
}
