//! Role: The `Host` and its pump: shell events, turns, layout, effects and presenting.

use super::client::{Boundary, Client};
use crate::contract::HostEvent;
use crate::model::Model;
use crate::model::edit::state::ImeUpdate;
use crate::{FrameLog, FrameTimes};
use arvane::{Presented, ShellControl, ShellError, Window};
use kivact_types::{PlatformRequest, PlatformResult};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

/// Turns one pump runs before it lets the window draw (each turn left new
/// events); the cut is reported (`HostEvent::TurnsCut`).
const TURN_LIMIT: u32 = 64;

/// Where the host is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
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
}

/// What became of the last frame presented.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LastFrame {
    /// Shown, dropped or lost: nothing is retried (a change or a surface loss
    /// redraws).
    Settled,
    /// The window had no buffer (occluded, timed out): the frame is drawn again at
    /// `retry_at_ns`, or at once when the window regains the focus.
    Skipped { retry_at_ns: u64 },
}

pub(crate) struct Host {
    pub(super) client: Client,
    pub(super) phase: Phase,
    /// Shared with the client's reader (`write::Reader`): borrowed for one
    /// call at a time, never across a call into the client.
    pub(super) model: Rc<RefCell<Model>>,
    /// Arvane's text, shared with the client's reader (`write::Reader`), which
    /// measures text for drawing: borrowed for one call at a time.
    pub(super) engine: Rc<RefCell<arvane::Engine>>,
    /// What became of the last frame presented (`Presented`).
    pub(super) last_frame: LastFrame,
    pub(super) next_wake: Option<u64>,
    /// Embedded handle destruction can occur inside a platform callback while
    /// this Host is borrowed. Stop new turns before deferred native cleanup;
    /// an already-running client turn still settles under the normal rules.
    pub(super) embedded_closing: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    /// The window's environment as last read: a change of the system's
    /// theme is found by comparing with it (the theme reported at start is
    /// the host's own, not the snapshot's).
    pub(super) environment: kivact_types::Environment,
    pub(super) reduced_motion_override: Option<bool>,
    /// The platform suspended the application (in the background):
    /// nothing is drawn until it resumes. Independent of `phase`.
    pub(super) suspended: bool,
    /// The software keyboard was asked for (an editable text input has
    /// the focus).
    pub(super) soft_keyboard: bool,
    /// Per-frame timings for benchmarks (`KIVACT_FRAME_LOG`), shared with
    /// the window's renderer observer.
    pub(super) frame_log: Rc<RefCell<FrameLog>>,
    /// The frame log noted that assistive technology reads the tree.
    pub(super) accessibility_noted: bool,
    /// The window's adapter the client was told of last
    /// (`HostEvent::GpuAdapterChanged`).
    #[cfg(feature = "gpu-interop")]
    pub(super) adapter: Option<kivact_types::GpuAdapter>,
}

impl Host {
    #[cfg(feature = "embed")]
    pub(crate) fn with_embedded_closing(
        mut self,
        closing: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        self.embedded_closing = Some(closing);
        self
    }
    pub(super) fn closing(&self) -> bool {
        self.embedded_closing
            .as_ref()
            .is_some_and(|closing| closing.load(std::sync::atomic::Ordering::Acquire))
    }
    /// The first step of every entry after the shell's start (`handle`, the test
    /// driver's `inject`): the start is over, and the platform effects queued
    /// meanwhile are carried out.
    pub(super) fn enter(&mut self, window: &mut Window<'_>) {
        if self.phase == Phase::Starting {
            self.phase = Phase::Running;
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
        if self.closing() {
            return Ok(ShellControl::Wait);
        }
        self.enter(window);
        if self.closing() {
            return Ok(ShellControl::Wait);
        }
        self.model.borrow_mut().push(event);
        self.pump(window)
    }
    pub(super) fn effects(&mut self, window: &mut Window<'_>) {
        if self.phase == Phase::Starting || self.closing() {
            return;
        }
        let effects = std::mem::take(&mut self.model.borrow_mut().effects);
        for (request, effect) in effects {
            if self.closing() {
                return;
            }
            let result = match effect {
                PlatformRequest::RequestWindowSize(size) => {
                    window.request_size(size).map(|_| PlatformResult::Done)
                }
                PlatformRequest::Fullscreen(fullscreen) => {
                    window.set_fullscreen(fullscreen).map(|()| {
                        let fullscreen = window.fullscreen();
                        self.model
                            .borrow_mut()
                            .update_env(|env, _| env.fullscreen = fullscreen);
                        PlatformResult::Done
                    })
                }
                PlatformRequest::ReducedMotion(override_value) => {
                    self.reduced_motion_override = override_value;
                    self.model.borrow_mut().update_env(|env, _| {
                        env.reduced_motion =
                            override_value.unwrap_or(self.environment.reduced_motion);
                    });
                    Ok(PlatformResult::Done)
                }
                PlatformRequest::CopyText(text) => {
                    window.copy_text(&text).map(|()| PlatformResult::Done)
                }
                PlatformRequest::ReadText => window.read_text().map(PlatformResult::Text),
                PlatformRequest::PickFile(p) => window.pick_file(&p).map(PlatformResult::Path),
                PlatformRequest::PickFiles(p) => window.pick_files(&p).map(PlatformResult::Paths),
                PlatformRequest::SaveFile(p) => window.save_file(&p).map(PlatformResult::Path),
                PlatformRequest::PickFolder(p) => window.pick_folder(&p).map(PlatformResult::Path),
                PlatformRequest::OpenPath(path) => {
                    window.open_path(&path).map(|()| PlatformResult::Done)
                }
                PlatformRequest::RevealPath(path) => {
                    window.reveal_path(&path).map(|()| PlatformResult::Done)
                }
                PlatformRequest::RegisterFont(_) => {
                    unreachable!("font registrations are answered by the write")
                }
            }
            .unwrap_or_else(|e| {
                // A service the platform does not have is `Unsupported`
                // (`ErrorKind::Unsupported` in the App), not a failure.
                match e {
                    arvane::ShellError::Unsupported(_) => PlatformResult::Unsupported,
                    arvane::ShellError::Failed(failure) => PlatformResult::Failed(failure),
                }
            });
            if self.closing() {
                return;
            }
            self.model
                .borrow_mut()
                .push(HostEvent::EffectResult { request, result });
        }
    }
    /// Hand the window the imports and drops of shared textures the turn
    /// wrote, in order; answer each import.
    #[cfg(feature = "gpu-interop")]
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
    #[cfg(feature = "gpu-interop")]
    fn adapter_changed(&mut self, window: &Window<'_>) {
        let adapter = window.gpu_adapter();
        if adapter != self.adapter {
            self.adapter = adapter.clone();
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
        if self.closing() {
            return Ok(());
        }
        self.model.borrow_mut().frame.set_now(window.now_ns());
        let output = self.client.turn(events, boundaries, window.now_ns());
        if self.closing() {
            return Ok(());
        }
        self.next_wake = output.next_wake_ns;
        if output.exit {
            self.phase = Phase::Exiting;
        }
        self.effects(window);
        if self.closing() {
            return Ok(());
        }
        #[cfg(feature = "gpu-interop")]
        self.externals(window);
        self.layout();
        Ok(())
    }
    pub(super) fn layout(&mut self) {
        self.model
            .borrow_mut()
            .relayout(&mut self.engine.borrow_mut());
    }
    pub(super) fn pump(&mut self, window: &mut Window<'_>) -> Result<ShellControl, ShellError> {
        if self.closing() {
            return Ok(ShellControl::Wait);
        }
        #[cfg(feature = "gpu-interop")]
        self.adapter_changed(window);
        self.model.borrow_mut().tick(window.now_ns());
        let mut turns = 0;
        loop {
            if self.closing() {
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
                model.relayout(&mut self.engine.borrow_mut());
                if self.phase == Phase::Exiting || !model.has_events() {
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
        let active = window.accessibility_active();
        if active && !self.accessibility_noted {
            // Assistive technology (a UI Automation client) asked for the
            // tree: from now on it is kept up to date; benchmarks need to
            // know.
            self.accessibility_noted = true;
            self.frame_log.borrow_mut().note("accessibility=on");
        }
        let caret = {
            let mut model = self.model.borrow_mut();
            // A native editor's caret first; else the caret of a node with an
            // IME session.
            model
                .caret(&mut self.engine.borrow_mut())
                .or_else(|| model.session_area())
        };
        window.set_ime_allowed(caret.is_some());
        // The software keyboard follows the focus of an editable text
        // input (a no-op where the platform has none).
        if caret.is_some() != self.soft_keyboard {
            self.soft_keyboard = caret.is_some();
            window.set_soft_keyboard(self.soft_keyboard);
        }
        if let Some(caret) = caret {
            window.set_ime_cursor_area(caret);
        }
        let update = self
            .model
            .borrow_mut()
            .ime_changed(&mut self.engine.borrow_mut());
        if let ImeUpdate::Set(text) = update {
            window.set_ime_text(text);
        }
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
                .accessibility(window, active, &mut self.engine.borrow_mut());
        }
        Ok(self.control(window))
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
        model.paint.needs_redraw() && model.frame.surface_active()
    }
    /// When the skipped frame is drawn again.
    fn retry_at(&self) -> Option<u64> {
        match self.last_frame {
            LastFrame::Settled => None,
            LastFrame::Skipped { retry_at_ns } => Some(retry_at_ns),
        }
    }
    fn control(&self, window: &Window<'_>) -> ShellControl {
        let model = self.model.borrow();
        let frame_deadline = if model.frame.surface_active() {
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
        if self.phase == Phase::Exiting {
            ShellControl::Exit
        } else {
            next.map_or(ShellControl::Wait, ShellControl::WakeAt)
        }
    }
}
