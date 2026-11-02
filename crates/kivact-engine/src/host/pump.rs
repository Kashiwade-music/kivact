//! Role: The `Host` and its pump: shell events, turns, layout, effects and presenting.

use super::client::{Boundary, Client};
use crate::model::Model;
use crate::{FrameLog, FrameTimes};
use arvane::{Presented, ShellControl, ShellError, Window};
use kivact_wire::{FileDialogRequest, HostEvent, PlatformRequest, PlatformResult};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

/// Turns one pump runs before it lets the window draw (each turn left new
/// events); the cut is reported (`HostEvent::TurnsCut`).
const TURN_LIMIT: u32 = 64;

pub(crate) struct Host {
    pub(super) client: Client,
    pub(super) starting: bool,
    /// Shared with the client's reader (`write::Reader`): borrowed for one
    /// call at a time, never across a call into the client.
    pub(super) model: Rc<RefCell<Model>>,
    /// Arvane's text, shared with the client's reader (`write::Reader`), which
    /// measures text for drawing: borrowed for one call at a time.
    pub(super) engine: Rc<RefCell<arvane::Engine>>,
    /// When to draw a frame the window skipped again
    /// (`Presented::Skipped`).
    pub(super) repaint_at: Option<u64>,
    /// The window skipped the last frame drawn (no buffer: occluded, timed
    /// out); a focused window draws again at once.
    pub(super) skipping: bool,
    pub(super) next_wake: Option<u64>,
    pub(super) exit: bool,
    /// Embedded handle destruction can occur inside a platform callback while
    /// this Host is borrowed. Stop new turns before deferred native cleanup;
    /// an already-running client turn still settles under the normal rules.
    pub(super) embedded_closing: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    /// The window's environment as last read: a change of the system's
    /// theme is found by comparing with it (the theme reported at start is
    /// the host's own, not the snapshot's).
    pub(super) environment: kivact_types::protocol::Environment,
    /// The platform suspended the application (in the background):
    /// nothing is drawn until it resumes.
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
    pub(super) adapter: Option<kivact_wire::GpuAdapter>,
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
    pub(super) fn effects<W: Window>(&mut self, window: &mut W) {
        if self.starting || self.closing() {
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
                            .update_env(|env| env.fullscreen = fullscreen);
                        PlatformResult::Done
                    })
                }
                PlatformRequest::CopyText(text) => {
                    window.copy_text(&text).map(|()| PlatformResult::Done)
                }
                PlatformRequest::ReadText => window.read_text().map(PlatformResult::Text),
                PlatformRequest::PickFile(p) => {
                    window.pick_file(&dialog(p)).map(PlatformResult::Path)
                }
                PlatformRequest::PickFiles(p) => {
                    window.pick_files(&dialog(p)).map(PlatformResult::Paths)
                }
                PlatformRequest::SaveFile(p) => {
                    window.save_file(&dialog(p)).map(PlatformResult::Path)
                }
                PlatformRequest::PickFolder(p) => {
                    window.pick_folder(&dialog(p)).map(PlatformResult::Path)
                }
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
                if e.is_unsupported() {
                    PlatformResult::Unsupported
                } else {
                    PlatformResult::Failed(e.to_string())
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
    pub(super) fn externals<W: Window>(&mut self, window: &mut W) {
        use crate::model::ExternalWrite;
        use kivact_wire::ExternalImport;
        let writes = std::mem::take(&mut self.model.borrow_mut().external_writes);
        for write in writes {
            match write {
                ExternalWrite::Import(image, textures) => {
                    let result = match window.import_external_image(image, textures) {
                        Ok(()) => ExternalImport::Ready {
                            generation: window.gpu_adapter().map_or(0, |a| a.generation),
                        },
                        Err(e) if e.kind() == arvane::ErrorKind::Unavailable => {
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
    fn adapter_changed<W: Window>(&mut self, window: &W) {
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
    pub(super) fn present<W: Window>(
        &mut self,
        window: &mut W,
        started: Instant,
    ) -> Result<Presented, arvane::Error> {
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
    pub(super) fn turn<W: Window>(
        &mut self,
        events: &[HostEvent],
        boundaries: &[Boundary],
        window: &mut W,
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
        self.exit = output.exit;
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
    pub(super) fn pump<W: Window>(&mut self, window: &mut W) -> Result<ShellControl, ShellError> {
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
                if self.exit || !model.has_events() {
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
        let text = self
            .model
            .borrow_mut()
            .ime_changed(&mut self.engine.borrow_mut());
        if let Some(text) = text {
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
        let after_frame = self.paint_due() && window.defers_accessibility();
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
    pub(super) fn defer<W: Window>(&mut self, window: &mut W) -> ShellControl {
        if self.paint_due() {
            window.request_redraw();
        }
        // The IME may ask about the text right after the input that
        // edited it (a reconversion after a selection).
        let text = self
            .model
            .borrow_mut()
            .ime_changed(&mut self.engine.borrow_mut());
        if let Some(text) = text {
            window.set_ime_text(text);
        }
        self.show_cursor(window);
        self.control(window)
    }
    /// Show the cursor the hover chain asks for (the model's rule),
    /// when it changed.
    fn show_cursor<W: Window>(&mut self, window: &W) {
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
    fn control<W: Window>(&self, window: &W) -> ShellControl {
        let model = self.model.borrow();
        let frame_deadline = if model.frame.surface_active() {
            model.frame_deadline()
        } else {
            None
        };
        let next = if model.has_events() || !model.effects.is_empty() {
            Some(window.now_ns())
        } else {
            [self.next_wake, frame_deadline, self.repaint_at]
                .into_iter()
                .flatten()
                .min()
        };
        if self.exit {
            ShellControl::Exit
        } else {
            next.map_or(ShellControl::Wait, ShellControl::WakeAt)
        }
    }
}

fn dialog(p: FileDialogRequest) -> arvane::ShellFileDialog {
    arvane::ShellFileDialog {
        title: p.title,
        directory: p.directory,
        file_name: p.file_name,
        filters: p.filters,
    }
}
