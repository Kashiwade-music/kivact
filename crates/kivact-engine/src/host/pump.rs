//! Role: Run host turns, effects, layout and frame presentation.
//!
//! One host runs one client and every window's model. The model of a window
//! decides what the window's state means; this module carries the turns,
//! effects and frames out over Arvane's windows.

use super::client::{Boundary, Client};
use super::devtools::{
    DevtoolsHost, append_overlay_layer, apply_emulation, chord_matches, effective_scale,
    emulated_viewport,
};
use super::window::{
    FRAME_STATE_KEEP_NS, HostWindow, ImeAllowed, LastPresent, SoftKeyboard, Surface,
};
use crate::contract::{
    DevtoolsChange, DrawableTexturesHandle, EngineEmulation, FrameRecord, HostEvent, PickPhase,
    TurnOutput,
};
use crate::model::edit::state::ImeUpdate;
use crate::model::{AppModel, DeferredRequest};
use crate::{FrameLog, FrameTimes};
use arvane::{ExportNotice, Presented, ShellControl, ShellError, ShellEvent, Window, Windows};
use kivact_types::cell::{ReportOnce, Sent};
use kivact_types::{
    DrawableId, Key, KeyInput, KeyPhase, MouseButton, NodeId, PlatformRequest, PlatformResult,
    Point, PointerPhase, WindowId,
};
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
    Starting {
        /// The client's next turn deadline (`TurnOutput::Continue`).
        next_wake: Option<u64>,
    },
    /// Handling the shell's events.
    Running {
        /// The client's next turn deadline (`TurnOutput::Continue`).
        next_wake: Option<u64>,
    },
    /// The client finished (`TurnOutput::Exit`): no more turns; every return asks
    /// the shell to exit.
    Exiting,
    Closed,
}

impl HostPhase {
    /// The client's next turn deadline: kept only while the host runs turns.
    pub(super) fn next_wake(self) -> Option<u64> {
        match self {
            Self::Starting { next_wake } | Self::Running { next_wake } => next_wake,
            Self::Exiting | Self::Closed => None,
        }
    }

    /// Record the client's next turn deadline; nothing once it exits or closes.
    pub(super) fn set_next_wake(&mut self, at: Option<u64>) {
        if let Self::Starting { next_wake } | Self::Running { next_wake } = self {
            *next_wake = at;
        }
    }
}

pub(crate) struct Host {
    pub(super) client: Client,
    pub(super) phase: HostPhase,
    /// Shared with the client's writers and readers: borrowed for one call
    /// at a time, never across a call into the client.
    pub(super) app: Rc<RefCell<AppModel>>,
    /// Arvane's text, shared with the client's reader (`write::Reader`), which
    /// measures text for drawing: borrowed for one call at a time.
    pub(super) engine: Rc<RefCell<arvane::Engine>>,
    /// Embedded handle destruction can occur inside a platform callback while
    /// this Host is borrowed. Stop new turns before deferred native cleanup;
    /// an already-running client turn still settles under the normal rules.
    pub(super) close: crate::contract::CloseSignal,
    // xtask-allow(state-fields): option: absent means the system's reduced-motion preference applies
    pub(super) reduced_motion_override: Option<bool>,
    /// The platform suspended the application (in the background):
    /// nothing is drawn until it resumes. Independent of `phase`.
    pub(super) presence: AppPresence,
    /// Per-frame timings for benchmarks (`KIVACT_FRAME_LOG`), shared with
    /// every window's renderer observer.
    pub(super) frame_log: Rc<RefCell<FrameLog>>,
    /// The frame log noted that assistive technology reads the tree.
    pub(super) accessibility_noted: ReportOnce,
    /// The App-wide windows and their host state.
    pub(super) windows: std::collections::BTreeMap<WindowId, HostWindow>,
    // xtask-allow(state-fields): projection: the App motion last reported to the client, written only by Host::report_app_motion
    /// The App motion input last reported to the client.
    pub(super) app_motion: Sent<(bool, u64)>,
    /// The DevTools' overlays, pick, keys and emulation.
    pub(super) devtools: DevtoolsHost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AppPresence {
    Foreground,
    Suspended,
}

impl Host {
    pub(super) fn observe_close(&mut self) -> bool {
        if self.close.is_set() {
            self.phase = HostPhase::Closed;
        }
        self.phase == HostPhase::Closed
    }

    /// The models' windows, ascending by id.
    pub(super) fn window_ids(&self) -> Vec<WindowId> {
        self.app.borrow().windows()
    }

    /// Whether window `id` can draw now.
    pub(super) fn drawable(&self, id: WindowId) -> bool {
        self.phase != HostPhase::Closed
            && self.presence == AppPresence::Foreground
            && self
                .windows
                .get(&id)
                .is_some_and(|window| matches!(window.surface, Surface::Drawable { .. }))
    }

    /// The first step of every entry after the shell's start (`handle`, the test
    /// driver's `inject`): the start is over, and the platform effects queued
    /// meanwhile are carried out.
    pub(super) fn enter(&mut self, windows: &mut Windows<'_>) {
        if let HostPhase::Starting { next_wake } = self.phase {
            self.phase = HostPhase::Running { next_wake };
        }
        self.effects(windows);
    }

    /// Queue `event` and pump (the test driver's injection).
    #[cfg(feature = "testing")]
    pub(super) fn inject(
        &mut self,
        event: HostEvent,
        windows: &mut Windows<'_>,
    ) -> Result<ShellControl, ShellError> {
        if self.observe_close() {
            return Ok(ShellControl::Wait);
        }
        self.enter(windows);
        if self.phase == HostPhase::Closed {
            return Ok(ShellControl::Wait);
        }
        self.app.borrow().push_event(event);
        self.pump(windows)
    }

    /// Carry out every window's deferred effects.
    pub(super) fn effects(&mut self, windows: &mut Windows<'_>) {
        if matches!(self.phase, HostPhase::Starting { .. } | HostPhase::Closed) {
            return;
        }
        for id in self.window_ids() {
            let effects = {
                let mut app = self.app.borrow_mut();
                app.model_mut(id)
                    .map(|model| std::mem::take(&mut model.effects))
                    .unwrap_or_default()
            };
            for (request, effect) in effects {
                let result = match effect {
                    DeferredRequest::Platform(request) => {
                        let fullscreen_request = matches!(request, PlatformRequest::Fullscreen(_));
                        let Some(mut window) = windows.get(id) else {
                            continue;
                        };
                        let result = window.perform(request);
                        if fullscreen_request {
                            let fullscreen = window.fullscreen();
                            if let Some(model) = self.app.borrow_mut().model_mut(id) {
                                model.update_env(|env, _| env.fullscreen = fullscreen);
                            }
                        }
                        result
                    }
                    DeferredRequest::ReducedMotion(override_value) => {
                        self.reduced_motion_override = override_value;
                        self.apply_reduced_motion();
                        PlatformResult::Done
                    }
                    DeferredRequest::Inject(inputs) => {
                        if windows.get(id).is_some() {
                            windows.inject(id, inputs, request);
                        }
                        continue;
                    }
                    DeferredRequest::Emulate(emulation) => {
                        self.apply_emulation(emulation, windows);
                        PlatformResult::Done
                    }
                    DeferredRequest::TimeScale(scale) => {
                        if scale.is_finite() {
                            let now = windows.now_ns();
                            for id in self.window_ids() {
                                if let Some(model) = self.app.borrow_mut().model_mut(id) {
                                    model.set_time_scale(now, scale);
                                }
                            }
                        }
                        PlatformResult::Done
                    }
                    DeferredRequest::DemandGpu => {
                        windows.demand_gpu(id);
                        continue;
                    }
                };
                if self.observe_close() {
                    return;
                }
                self.app
                    .borrow()
                    .push_event(HostEvent::EffectResult { request, result });
            }
        }
    }

    /// Apply the reduced-motion override to every window's environment.
    fn apply_reduced_motion(&mut self) {
        let override_value = self.reduced_motion_override;
        let systems: Vec<(WindowId, bool)> = self
            .windows
            .iter()
            .map(|(id, window)| {
                let system = window
                    .environment
                    .get()
                    .is_some_and(|environment| environment.reduced_motion);
                (*id, system)
            })
            .collect();
        for (id, system) in systems {
            if let Some(model) = self.app.borrow_mut().model_mut(id) {
                model.update_env(|env, _| {
                    env.reduced_motion = override_value.unwrap_or(system);
                });
            }
        }
    }

    /// Hand every window the imports and drops of shared textures the turn
    /// wrote, in order; answer each import.
    pub(super) fn externals(&mut self, windows: &mut Windows<'_>) {
        use crate::model::ExternalWrite;
        use kivact_types::ExternalImport;
        for id in self.window_ids() {
            let writes = {
                let mut app = self.app.borrow_mut();
                app.model_mut(id)
                    .map(|model| std::mem::take(&mut model.external_writes))
                    .unwrap_or_default()
            };
            for write in writes {
                let Some(mut window) = windows.get(id) else {
                    continue;
                };
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
                        self.app
                            .borrow()
                            .push_event(HostEvent::ExternalImported { image, result });
                    }
                    ExternalWrite::Drop(image) => window.drop_external_image(image),
                }
            }
        }
    }

    /// Tell the client when window `id`'s adapter changed.
    fn adapter_changed(&mut self, id: WindowId, window: &Window<'_>) {
        let adapter = window.gpu_adapter();
        let host_window = self
            .windows
            .get_mut(&id)
            .expect("invariant: an adapter is read for an open window");
        let changed = match &adapter {
            Some(value) => host_window.reported_adapter.update(value.clone()).is_some(),
            None if host_window.reported_adapter.get().is_some() => {
                host_window.reported_adapter = Sent::new();
                true
            }
            None => false,
        };
        if changed {
            self.app.borrow().push_event(HostEvent::GpuAdapterChanged {
                window: id,
                adapter,
            });
        }
    }

    /// Draw and show one frame of window `id`. `started` is when the redraw
    /// began (its layout included); the frame log gets the scene, raster and
    /// present spans of a frame shown.
    pub(super) fn present(
        &mut self,
        id: WindowId,
        windows: &mut Windows<'_>,
        started: Instant,
    ) -> Result<Presented, kivact_types::Failure> {
        let mut window = windows
            .get(id)
            .expect("invariant: a presented window is open");
        let plan = window.frame_plan();
        let scale = effective_scale(plan.scale, self.devtools.emulation.scale);
        self.export_pools(id, &mut window);
        {
            let mut app = self.app.borrow_mut();
            let model = app
                .model_mut(id)
                .expect("invariant: a presented window has a model");
            model.scroll_inputs(&mut self.engine.borrow_mut());
            // Images are rasterized for the scale the frame is drawn at.
            model.paint.set_scale(scale);
        }
        let t_scene = Instant::now();
        let overlay = self.devtools.overlays.get(&id).cloned();
        let composition = {
            let mut app = self.app.borrow_mut();
            let model = app
                .model_mut(id)
                .expect("invariant: a presented window has a model");
            let size = model.window();
            let mut composition = model.composition(&mut self.engine.borrow_mut());
            if let Some(overlay) = &overlay {
                append_overlay_layer(
                    overlay,
                    &mut composition,
                    &mut self.engine.borrow_mut(),
                    size,
                );
            }
            composition
        };
        let t_raster = Instant::now();
        let presented = window.present(&composition, plan)?;
        let end_ns = window.now_ns();
        self.export_presented(id, &presented, end_ns);
        if let Presented::Shown {
            renderer,
            raster,
            present,
            stats,
            ..
        } = &presented
        {
            self.windows
                .get_mut(&id)
                .expect("invariant: a presented window has host state")
                .last_stats = Some(*stats);
            let logging =
                self.frame_log.borrow().enabled() || self.app.borrow().log.frames_recording();
            if logging {
                let times = FrameTimes {
                    work: Instant::now() - started,
                    scene: t_raster - t_scene,
                    raster: *raster,
                    present: *present,
                };
                self.record_frame(id, end_ns, times, stats, *renderer);
            }
        }
        Ok(presented)
    }

    /// Record one shown frame in the frame log and the DevTools' ring.
    fn record_frame(
        &self,
        id: WindowId,
        end_ns: u64,
        times: FrameTimes,
        stats: &kivact_types::render::FrameStats,
        renderer: arvane::Renderer,
    ) {
        let log = self.app.borrow().log.clone();
        let mut frame_log = self.frame_log.borrow_mut();
        if frame_log.enabled() {
            frame_log.frame(times, stats);
        }
        drop(frame_log);
        if !log.frames_recording() {
            return;
        }
        let (revision, frame_period_ns) = {
            let app = self.app.borrow();
            let period = app.model(id).map_or(0, |model| model.frame.period_ns());
            (app.events().applied(), period)
        };
        log.frame(|| FrameRecord {
            window: id,
            revision,
            end_ns,
            work_ns: times.work.as_nanos() as u64,
            scene_ns: times.scene.as_nanos() as u64,
            raster_ns: times.raster.as_nanos() as u64,
            present_ns: times.present.as_nanos() as u64,
            stats: *stats,
            gpu: matches!(renderer, arvane::Renderer::Gpu),
            frame_period_ns,
        });
    }

    /// One client turn: it writes the tree (each write applied at once),
    /// then its window changes and platform effects are answered and the tree
    /// laid out before anything else runs, so the next turn sees the effect
    /// results and the new geometry.
    pub(super) fn turn(
        &mut self,
        events: &[HostEvent],
        boundaries: &[Boundary],
        windows: &mut Windows<'_>,
    ) -> Result<(), ShellError> {
        if self.phase == HostPhase::Closed {
            return Ok(());
        }
        let now = windows.now_ns();
        for id in self.window_ids() {
            let drawable = self.drawable(id);
            if let Some(model) = self.app.borrow_mut().model_mut(id) {
                model.set_surface(drawable);
                model.frame.set_now(now);
            }
        }
        let output = self.client.turn(events, boundaries, now);
        if self.observe_close() {
            return Ok(());
        }
        match output {
            TurnOutput::Continue { next_wake_ns } => self.phase.set_next_wake(next_wake_ns),
            TurnOutput::Exit => self.phase = HostPhase::Exiting,
        }
        self.apply_window_changes(windows);
        self.apply_devtools_changes();
        self.effects(windows);
        if self.phase == HostPhase::Closed {
            return Ok(());
        }
        self.externals(windows);
        self.settle();
        Ok(())
    }

    /// Settle every window's model.
    pub(super) fn settle(&mut self) {
        for id in self.window_ids() {
            self.settle_window(id);
        }
    }

    /// Settle one window's model.
    pub(super) fn settle_window(&mut self, id: WindowId) {
        if let Some(model) = self.app.borrow_mut().model_mut(id) {
            model.settle(&mut self.engine.borrow_mut());
        }
    }

    pub(super) fn pump(&mut self, windows: &mut Windows<'_>) -> Result<ShellControl, ShellError> {
        if self.phase == HostPhase::Closed {
            return Ok(ShellControl::Wait);
        }
        let now = windows.now_ns();
        for id in self.windows.keys().copied().collect::<Vec<_>>() {
            if let Some(mut window) = windows.get(id) {
                self.adapter_changed(id, &window);
                self.export_notices(&mut window);
            }
        }
        self.tick_windows(now);
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
                for id in self.window_ids() {
                    if let Some(model) = self.app.borrow_mut().model_mut(id) {
                        model.poll_assets();
                        model.settle(&mut self.engine.borrow_mut());
                    }
                }
                if !self.app.borrow().events().has_events() {
                    for id in self.window_ids() {
                        if let Some(model) = self.app.borrow_mut().model_mut(id) {
                            model.snapshot_requests(&mut self.engine.borrow_mut());
                        }
                    }
                }
                let has_events = self.app.borrow().events().has_events();
                if self.phase == HostPhase::Exiting || !has_events {
                    None
                } else if turns == TURN_LIMIT {
                    // Every turn left more events: the client keeps answering
                    // its own reports. Say so, and let the window draw.
                    self.app.borrow().push_event(HostEvent::TurnsCut { turns });
                    None
                } else {
                    // Every queued event, with the commit boundaries among
                    // them: all of their revisions are laid out now.
                    let (events, boundaries) = self.app.borrow().events().drain();
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
            self.turn(&events, &boundaries, windows)?;
        }
        for id in self.windows.keys().copied().collect::<Vec<_>>() {
            self.finish_window(id, windows);
        }
        self.finish_motion(now);
        Ok(self.control(now))
    }

    /// Tick every window's frame clock; only the tick source reports a
    /// `FrameTick` and sees the core's requests.
    pub(super) fn tick_windows(&mut self, now: u64) {
        let source = self.windows.keys().copied().find(|id| self.drawable(*id));
        let mut app = self.app.borrow_mut();
        let clock = app.motion();
        let AppModel {
            windows,
            client_frames,
            events,
            ..
        } = &mut *app;
        for id in windows.keys().copied().collect::<Vec<_>>() {
            let client = (Some(id) == source).then_some(&mut *client_frames);
            if let Some(model) = windows.get_mut(&id)
                && let Some(stamp) = model.tick(now, client)
            {
                events.push(HostEvent::FrameTick {
                    window: id,
                    now_ns: stamp.now_ns,
                    estimated_present_ns: stamp.estimated_present_ns,
                    clock,
                });
            }
        }
    }

    /// The window's export notices (GPU available or lost) can come from
    /// a wake that queues no event: they are events here, so the turn that
    /// answers them runs.
    fn export_notices(&mut self, window: &mut Window<'_>) {
        for notice in window.take_export_notices() {
            let event = match notice {
                ExportNotice::Lost(loss) => HostEvent::DrawableExportLost(loss),
                ExportNotice::Available => HostEvent::GpuExportAvailable,
            };
            self.app.borrow().push_event(event);
        }
    }

    /// Show the cursor the hover chain of window `id` asks for (the
    /// model's rule), when it changed.
    pub(super) fn show_cursor(&mut self, id: WindowId, windows: &mut Windows<'_>) {
        let cursor = match self.app.borrow_mut().model_mut(id) {
            Some(model) => model.cursor(),
            None => return,
        };
        let show = match self.app.borrow_mut().model_mut(id) {
            Some(model) => model.pointer.show(cursor),
            None => return,
        };
        if show && let Some(window) = windows.get(id) {
            window.set_cursor(cursor);
        }
    }

    /// Make the GPU export pools the drawables of window `id` ask for: a
    /// pool is made for a new generation, dropped when no longer wanted, and
    /// asked anew from the drawable when its size changed. The window's
    /// answer goes to the client as `DrawableTextures`.
    fn export_pools(&mut self, id: WindowId, window: &mut Window<'_>) {
        let (wanted, unwanted) = {
            let mut app = self.app.borrow_mut();
            let Some(model) = app.model_mut(id) else {
                return;
            };
            let wanted = model.wanted_pools();
            let ids: Vec<DrawableId> = wanted.iter().map(|w| w.id).collect();
            let unwanted = model.drawables.unwanted_pools(&ids);
            (wanted, unwanted)
        };
        for drawable in unwanted {
            window.drop_export(drawable);
            if let Some(model) = self.app.borrow_mut().model_mut(id) {
                model.drawables.set_pool(drawable, None);
            }
        }
        for want in wanted {
            let current = {
                let mut app = self.app.borrow_mut();
                app.model_mut(id)
                    .and_then(|model| model.drawables.pool_of(want.id))
            };
            match current {
                Some((generation, size)) if generation == want.generation && size == want.size => {
                    continue;
                }
                Some((generation, _)) if generation == want.generation => {
                    window.drop_export(want.id);
                    if let Some(model) = self.app.borrow_mut().model_mut(id) {
                        model.drawables.set_pool(want.id, None);
                        model.push(HostEvent::DrawableResized { id: want.id });
                    }
                    continue;
                }
                _ => {}
            }
            let result = window.export_drawable(want.id, want.generation, &want.adapter, want.size);
            if let Some(model) = self.app.borrow_mut().model_mut(id) {
                let result = match result {
                    Ok(textures) => {
                        model
                            .drawables
                            .set_pool(want.id, Some((want.generation, want.size)));
                        Ok(DrawableTexturesHandle::new(textures))
                    }
                    Err(failure) => Err(failure),
                };
                model.push(HostEvent::DrawableTextures {
                    id: want.id,
                    generation: want.generation,
                    result,
                });
            }
        }
    }

    /// Hand over the frames of window `id`'s shown presentation and decide
    /// its drawables' export retries from the pending exports.
    fn export_presented(&mut self, id: WindowId, presented: &Presented, now_ns: u64) {
        let Presented::Shown {
            exports,
            pending_exports,
            ..
        } = presented
        else {
            return;
        };
        let mut app = self.app.borrow_mut();
        let Some(model) = app.model_mut(id) else {
            return;
        };
        for frame in exports {
            model.push(HostEvent::DrawableFrame(frame.clone()));
        }
        let period = model.frame.period_ns();
        model.drawables.presented(pending_exports, now_ns, period);
    }

    /// A frame of window `id` is due: the picture changed and it has a
    /// surface.
    pub(super) fn paint_due(&self, id: WindowId) -> bool {
        let app = self.app.borrow();
        app.model(id)
            .is_some_and(|model| model.paint.needs_redraw())
            && self.drawable(id)
    }

    /// When window `id`'s skipped frame is drawn again.
    fn retry_at(&self, id: WindowId) -> Option<u64> {
        match self.windows.get(&id)?.last_present {
            LastPresent::Shown { .. } | LastPresent::Settled => None,
            LastPresent::Skipped { retry_at_ns } => Some(retry_at_ns),
        }
    }

    /// When window `id` releases the CPU frame state of its last shown
    /// frame ([`FRAME_STATE_KEEP_NS`] after it).
    fn release_at(&self, id: WindowId) -> Option<u64> {
        match self.windows.get(&id)?.last_present {
            LastPresent::Shown { at_ns } => Some(at_ns.saturating_add(FRAME_STATE_KEEP_NS)),
            LastPresent::Settled | LastPresent::Skipped { .. } => None,
        }
    }

    /// Release the CPU frame state of every window whose last shown frame
    /// is [`FRAME_STATE_KEEP_NS`] old at `now`, unless it has a frame due
    /// (that frame shows again). The window keeps showing its picture;
    /// its next frame redraws everything.
    pub(super) fn release_idle_frames(&mut self, now: u64, windows: &mut Windows<'_>) {
        for id in self.windows.keys().copied().collect::<Vec<_>>() {
            if self.release_at(id).is_none_or(|at| at > now) || self.paint_due(id) {
                continue;
            }
            if let Some(mut window) = windows.get(id) {
                window.forget_frame_state();
            }
            self.windows
                .get_mut(&id)
                .expect("invariant: the released window has host state")
                .last_present = LastPresent::Settled;
        }
    }

    /// IME state and purpose, the software keyboard and the cursor of window
    /// `id`'s settled model.
    pub(super) fn synchronize_ime(&mut self, id: WindowId, window: &mut Window<'_>) {
        let caret = {
            let mut app = self.app.borrow_mut();
            let Some(model) = app.model_mut(id) else {
                return;
            };
            model
                .caret(&mut self.engine.borrow_mut())
                .or_else(|| model.session_area())
        };
        let allowed = if caret.is_some() {
            ImeAllowed::Yes
        } else {
            ImeAllowed::No
        };
        let changed = self
            .windows
            .get_mut(&id)
            .and_then(|host_window| host_window.ime_allowed.update(allowed));
        if let Some(allowed) = changed {
            window.set_ime_allowed(*allowed == ImeAllowed::Yes);
        }
        // The software keyboard follows the focus of an editable text
        // input (a no-op where the platform has none).
        let keyboard = if caret.is_some() {
            SoftKeyboard::Shown
        } else {
            SoftKeyboard::Hidden
        };
        let changed = self
            .windows
            .get_mut(&id)
            .and_then(|host_window| host_window.soft_keyboard.update(keyboard));
        if let Some(keyboard) = changed {
            window.set_soft_keyboard(*keyboard == SoftKeyboard::Shown);
        }
        if let Some(caret) = caret {
            window.set_ime_cursor_area(caret);
        }
        let purpose = self.app.borrow().model(id).map(|model| model.ime_purpose());
        if let Some(purpose) = purpose {
            window.set_ime_purpose(purpose);
        }
        let update = match self.app.borrow_mut().model_mut(id) {
            Some(model) => model.ime_changed(&mut self.engine.borrow_mut()),
            None => ImeUpdate::Unchanged,
        };
        if let ImeUpdate::Set(text) = update {
            window.set_ime_text(text);
        }
    }

    /// Input is applied to the host model at once, but the client turn waits
    /// until the native queue is drained: a due wake runs it for every input
    /// that arrived meanwhile, so bursts (typing, drags) cost one turn,
    /// layout and frame instead of one each.
    pub(super) fn defer(&mut self, id: WindowId, windows: &mut Windows<'_>) -> ShellControl {
        if self.paint_due(id)
            && let Some(window) = windows.get(id)
        {
            window.request_redraw();
        }
        // The IME may ask about the text right after the input that
        // edited it (a reconversion after a selection).
        let purpose = self.app.borrow().model(id).map(|model| model.ime_purpose());
        if let Some(purpose) = purpose
            && let Some(window) = windows.get(id)
        {
            window.set_ime_purpose(purpose);
        }
        let update = match self.app.borrow_mut().model_mut(id) {
            Some(model) => model.ime_changed(&mut self.engine.borrow_mut()),
            None => ImeUpdate::Unchanged,
        };
        if let ImeUpdate::Set(text) = update
            && let Some(window) = windows.get(id)
        {
            window.set_ime_text(text);
        }
        self.show_cursor(id, windows);
        let now = windows.now_ns();
        self.control(now)
    }

    fn control(&self, now_ns: u64) -> ShellControl {
        if self.phase == HostPhase::Closed {
            return ShellControl::Wait;
        }
        let app = self.app.borrow();
        let source = self.windows.keys().copied().find(|id| self.drawable(*id));
        let frame_deadline = self
            .windows
            .keys()
            .copied()
            .filter(|id| self.drawable(*id))
            .filter_map(|id| {
                let model = app.model(id)?;
                let client = (Some(id) == source).then_some(&app.client_frames);
                model.frame_deadline(client)
            })
            .min();
        let effects_pending = app.windows.values().any(|model| !model.effects.is_empty());
        let next = if app.events().has_events() || effects_pending {
            Some(now_ns)
        } else {
            [
                self.phase.next_wake(),
                frame_deadline,
                self.windows
                    .keys()
                    .filter_map(|id| self.retry_at(*id))
                    .min(),
                // A window with a frame due keeps its state for that frame:
                // its release is not a wake.
                self.windows
                    .keys()
                    .filter(|id| !self.paint_due(**id))
                    .filter_map(|id| self.release_at(*id))
                    .min(),
                // A retry that is due was asked for by the redraw of `pump`,
                // which runs the export; only a later retry is a wake.
                app.windows
                    .values()
                    .filter_map(|model| model.drawables.retry_deadline())
                    .filter(|at| *at > now_ns)
                    .min(),
            ]
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

impl Host {
    /// Drop `event` for the DevTools when it is a start key or while `id`
    /// is picking; the DevTools event is queued instead.
    pub(super) fn intercept_devtools(&mut self, id: WindowId, event: &ShellEvent) -> bool {
        if let ShellEvent::Key(input) = event
            && matches!(input.phase, KeyPhase::Down { .. })
            && let Some(pick) = self.start_key(input)
        {
            self.app
                .borrow()
                .push_event(HostEvent::DevtoolsKey { window: id, pick });
            return true;
        }
        if self.devtools.pick != Some(id) {
            return false;
        }
        match event {
            ShellEvent::Pointer(input) => {
                // Every other pointer event is swallowed while picking, and emits nothing.
                let phase = match input.phase {
                    PointerPhase::Move => PickPhase::Hover,
                    PointerPhase::Down(press) if press.button == MouseButton::Primary => {
                        PickPhase::Select
                    }
                    _ => return true,
                };
                let node = self.pick_node(id, input.position);
                self.app.borrow().push_event(HostEvent::DevtoolsPick {
                    window: id,
                    node,
                    phase,
                });
                true
            }
            ShellEvent::Wheel(_) => true,
            ShellEvent::Key(input)
                if input.key == Key::Escape && matches!(input.phase, KeyPhase::Down { .. }) =>
            {
                self.app.borrow().push_event(HostEvent::DevtoolsPick {
                    window: id,
                    node: None,
                    phase: PickPhase::Cancel,
                });
                true
            }
            _ => false,
        }
    }

    /// Whether the press `input` is a start key; `Some(true)` for a pick key.
    fn start_key(&self, input: &KeyInput) -> Option<bool> {
        let pick = self
            .devtools
            .keys
            .pick
            .iter()
            .any(|chord| chord_matches(chord, input));
        if pick {
            return Some(true);
        }
        self.devtools
            .keys
            .open
            .iter()
            .any(|chord| chord_matches(chord, input))
            .then_some(false)
    }

    /// The node under `point` for pick mode, ignoring modals.
    fn pick_node(&self, id: WindowId, point: Point) -> Option<NodeId> {
        self.app
            .borrow()
            .model(id)
            .and_then(|model| model.hit_any(point))
    }

    /// Apply the DevTools changes of the turn to `DevtoolsHost`.
    pub(super) fn apply_devtools_changes(&mut self) {
        let changes = self.app.borrow_mut().take_devtools_changes();
        for change in changes {
            match change {
                DevtoolsChange::Overlay(window, overlay) => {
                    self.devtools.overlays.insert(window, overlay);
                    if let Some(model) = self.app.borrow_mut().model_mut(window) {
                        model.paint.request_redraw();
                    }
                }
                DevtoolsChange::Pick(pick) => self.devtools.pick = pick,
                DevtoolsChange::Keys(keys) => self.devtools.keys = keys,
            }
        }
    }

    /// Apply the emulation to every window after `DeferredRequest::Emulate`.
    pub(super) fn apply_emulation(
        &mut self,
        emulation: EngineEmulation,
        windows: &mut Windows<'_>,
    ) {
        self.devtools.emulation = emulation;
        let ids = self.window_ids();
        let mut sizes = Vec::new();
        for id in ids {
            if let Some(window) = windows.get(id) {
                sizes.push((id, window.size(), window.scale()));
            }
        }
        for (id, size, native_scale) in sizes {
            let viewport = emulated_viewport(size, native_scale, emulation.scale);
            let scale = effective_scale(native_scale, emulation.scale);
            if let Some(model) = self.app.borrow_mut().model_mut(id) {
                model.set_viewport(viewport);
                model.paint.set_scale(scale);
                model.update_env(|env, _| apply_emulation(emulation, env));
            }
        }
    }
}
