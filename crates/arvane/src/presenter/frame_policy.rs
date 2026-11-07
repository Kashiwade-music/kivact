//! Role: Presenter: one window's renderers and its frame policy.

use super::exports::{ExportNotice, ExportRegistry};
use super::frame::{FramePlan, Presented};
use super::gpu::{Fallback, Gpu};
use super::health::{FrameHealth, LossAction};
use crate::ShellWaker;
use crate::error::{self, Failure, FailureKind};
use crate::presentation::{Notes, Renderer, RendererChange};
use arvane_compose::{CpuCompositor, CpuOutput, Damage, GpuContext, GpuSurface, Presentation};
use arvane_render::RenderError;
use arvane_shell::{BufferAge, ShellWindow};
use kivact_types::render::{Composition, FrameReport, PixelRect};
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// What drawing one frame did, before the frame policy.
pub(super) struct Drawn {
    pub(super) renderer: Renderer,
    /// `false`: the window had no buffer to give.
    pub(super) shown: bool,
    pub(super) raster: Duration,
    pub(super) present: Duration,
    /// The frame's exports, missed external frames, shader failures and stats.
    pub(super) report: FrameReport,
}

/// The renderers of one window and its frame policy.
pub(crate) struct Presenter {
    pub(super) cpu: CpuCompositor,
    pub(super) gpu: Gpu,
    pub(super) notes: Notes,
    /// What the window's surface did with the last frames that reached it.
    pub(super) health: FrameHealth,
    /// The GPU export pools of the window and the notices for the engine.
    pub(super) exports: ExportRegistry,
    /// The shader programs whose failure was already noted.
    shader_failed: HashSet<u64>,
    /// The limits already noted (once each per window).
    limits_noted: HashSet<String>,
}

impl Presenter {
    /// Stop workers needing the native window before that window is dropped.
    pub(crate) fn shutdown(&mut self, window: &mut dyn ShellWindow) {
        self.gpu.shutdown(window);
    }

    pub(crate) fn release_caches(&mut self) {
        self.cpu.release_caches();
        self.gpu.release_caches();
    }

    /// A presenter noting its renderer changes to `notes`, its GPU devices
    /// made as `env` says.
    pub(crate) fn new(notes: Notes, env: &crate::env::Env) -> Self {
        Self {
            cpu: CpuCompositor::new(),
            gpu: Gpu::new(env.gpu),
            notes,
            health: FrameHealth::Healthy,
            exports: ExportRegistry::default(),
            shader_failed: HashSet::new(),
            limits_noted: HashSet::new(),
        }
    }

    /// Note a renderer change.
    pub(crate) fn note(&mut self, change: RendererChange) {
        self.notes.note(change);
    }

    /// The adapter of the GPU device drawing the window's frames, when it
    /// can import shared textures (see [`crate::Window::gpu_adapter`]).
    pub(crate) fn gpu_adapter(&self) -> Option<kivact_types::GpuAdapter> {
        self.gpu.adapter()
    }

    /// Import `textures` as the image `id` of the window (see
    /// [`crate::Window::import_external_image`]).
    pub(crate) fn import_external(
        &mut self,
        id: kivact_types::ExternalImageId,
        textures: kivact_types::SharedTextures,
    ) -> Result<(), Failure> {
        if !arvane_compose::GPU_BUILT {
            return Err(Failure::new(
                FailureKind::Unavailable,
                "built without GPU support",
            ));
        }
        if self.gpu.adapter().is_none() {
            return Err(Failure::new(
                FailureKind::Unavailable,
                "the window has no GPU device that can import shared textures \
                 (it draws on the CPU, its device is not ready, or the adapter cannot)",
            ));
        }
        let was_ready = self.gpu.ready();
        let context = self
            .gpu
            .context()
            .expect("invariant: a device with an adapter is ready");
        let result = match context.import_external(id, textures) {
            Ok(()) => Ok(()),
            Err(failure) => {
                let error = error::render(failure.clone());
                self.gpu.failed(&failure, &mut self.notes);
                Err(error)
            }
        };
        self.sync_exports(was_ready);
        result
    }

    /// Drop the image `id` of the window (see
    /// [`crate::Window::drop_external_image`]).
    pub(crate) fn drop_external(&mut self, id: kivact_types::ExternalImageId) {
        if let Some(context) = self.gpu.context() {
            context.drop_external(id);
        }
    }

    /// Make the GPU export pool of `sink` for `generation` (see
    /// [`crate::Window::export_drawable`]).
    pub(crate) fn export_drawable(
        &mut self,
        sink: kivact_types::ids::DrawableId,
        generation: u64,
        adapter: &kivact_types::GpuAdapter,
        size: (u32, u32),
    ) -> Result<kivact_types::SharedTextures, Failure> {
        if !arvane_compose::GPU_INTEROP_BUILT {
            return Err(Failure::new(
                FailureKind::Unavailable,
                "built without GPU interop support (feature gpu-interop)",
            ));
        }
        let Some(context) = self.gpu.context() else {
            return Err(Failure::new(
                FailureKind::NotReady,
                "the window's GPU device is not ready (it is starting, or the window draws on the CPU)",
            ));
        };
        let textures = context
            .export_drawable(sink, generation, adapter, size)
            .map_err(error::render)?;
        self.exports.made(sink, generation);
        Ok(textures)
    }

    /// Drop the export pool of `sink` (see [`crate::Window::drop_export`]).
    pub(crate) fn drop_export(&mut self, sink: kivact_types::ids::DrawableId) {
        self.exports.dropped(sink);
        if let Some(context) = self.gpu.context() {
            context.drop_export(sink);
        }
    }

    /// The export notices since the last call (see
    /// [`crate::Window::take_export_notices`]).
    pub(crate) fn take_export_notices(&mut self) -> Vec<ExportNotice> {
        self.exports.take()
    }

    /// Make the device lost, as a device failure does (the test control of
    /// `arvane::spi::lose_gpu_device`). Nothing happens without a device.
    pub(crate) fn lose_gpu_device(&mut self) {
        if !self.gpu.ready() {
            return;
        }
        let error = RenderError::Backend("test: device lost".into());
        self.gpu.failed(&error, &mut self.notes);
        self.sync_exports(true);
    }

    /// The platform destroyed the window's surface: drop the GPU surface
    /// made for it (a new surface generation gets a new one with its first
    /// frame) and start the frame policy afresh.
    pub(crate) fn surface_destroyed(&mut self) {
        self.gpu.drop_surface();
        self.health = FrameHealth::Healthy;
    }

    /// Start the GPU renderer on a worker thread; frames are drawn on the
    /// CPU until it is ready, and when no hardware adapter is available,
    /// or GPU support is not built in, for the whole session. `waker`
    /// wakes the window's thread when a start step finishes
    /// ([`Presenter::prepare`] then).
    pub(crate) fn start_gpu(&mut self, waker: ShellWaker) {
        self.gpu.start(
            std::sync::Arc::new(move || waker.wake(crate::WakeReasons::GPU_STEP)),
            &mut self.notes,
        );
    }

    /// Move a GPU start along for `window` without drawing: once the
    /// device is ready, make the window's surface and configure it on a
    /// worker. Returns whether the GPU became ready to draw `window`'s
    /// frames (draw one now, so the switch happens while the window is
    /// idle rather than with the next input).
    pub(crate) fn prepare(&mut self, window: &dyn ShellWindow) -> bool {
        if self.gpu.ready() {
            return false;
        }
        let make = |context: &GpuContext| make_surface(window, context, window.keeps_frames());
        self.gpu
            .advance(&mut self.notes, &make, window.physical_size());
        self.sync_exports(false);
        self.gpu.ready()
    }

    /// Apply the frame policy to what drawing a frame did (`now_ns`: the
    /// window's clock after it, `period_ns`: its display's refresh period,
    /// `recover` makes the window's surface again).
    pub(super) fn settle(
        &mut self,
        drawn: Result<Drawn, Failure>,
        (now_ns, period_ns): (u64, u64),
        renderer: Renderer,
        window: &mut dyn ShellWindow,
        composition: &Composition,
        plan: FramePlan,
    ) -> Result<Presented, Failure> {
        // A frame that was not shown leaves the compositor's records ahead
        // of what the window holds: the next frame redraws everything.
        if !matches!(&drawn, Ok(drawn) if drawn.shown) {
            self.cpu.invalidate();
        }
        match drawn {
            Ok(drawn) if drawn.shown => {
                self.health.shown();
                Ok(Presented::Shown {
                    renderer: drawn.renderer,
                    raster: drawn.raster,
                    present: drawn.present,
                    exports: drawn.report.exports,
                    pending_exports: drawn.report.pending_exports,
                    stats: drawn.report.stats,
                })
            }
            // The surface had no buffer (timed out, occluded): the picture
            // is drawn again a display period later, backing off to 16
            // periods while it stays so.
            Ok(_) => {
                let wait = self.health.skipped(period_ns);
                Ok(Presented::Skipped {
                    retry_at_ns: now_ns.saturating_add(wait),
                })
            }
            Err(error) if error.kind == FailureKind::Surface => {
                match self.health.lost(renderer) {
                    LossAction::Fail => return Err(error),
                    LossAction::FallbackToCpu => {
                        self.gpu.fallback_to_cpu(&mut self.notes, &error);
                        let drawn = self.draw_native(window, composition, plan);
                        return self.settle(
                            drawn,
                            (now_ns, period_ns),
                            Renderer::Cpu,
                            window,
                            composition,
                            plan,
                        );
                    }
                    LossAction::Recover => {}
                }
                window
                    .recover_surface()
                    .map_err(|e| Failure::new(FailureKind::Surface, e.to_string()))?;
                Ok(Presented::SurfaceLost(error))
            }
            Err(error) => Ok(Presented::Dropped(error)),
        }
    }

    /// Render `composition` as `plan` says and show it in the native `window`
    /// (see the module invariants).
    pub(crate) fn present_native(
        &mut self,
        window: &mut dyn ShellWindow,
        composition: &Composition,
        plan: FramePlan,
    ) -> Result<Presented, Failure> {
        let was_ready = self.gpu.ready();
        let drawn = self.draw_native(window, composition, plan);
        if let Ok(drawn) = &drawn {
            self.note_report(&drawn.report);
        }
        let (now, period) = (window.now_ns(), window.frame_period_ns());
        let renderer = if self.gpu.ready() {
            Renderer::Gpu
        } else {
            Renderer::Cpu
        };
        let presented = self.settle(drawn, (now, period), renderer, window, composition, plan);
        self.sync_exports(was_ready);
        presented
    }

    /// Note what a frame's report says that the window must report: the
    /// external frames it did not draw, and each shader program that failed
    /// (once per program).
    fn note_report(&mut self, report: &FrameReport) {
        for frame in &report.missed {
            self.notes
                .note(RendererChange::ExternalFrameMissed { frame: *frame });
        }
        for (hash, message) in &report.shader_failures {
            if self.shader_failed.insert(*hash) {
                self.notes.note(RendererChange::ShaderFailed {
                    message: message.clone(),
                });
            }
        }
    }

    /// Note that a frame went beyond the limit `what` (once per kind and window).
    fn note_limit(&mut self, what: &str) {
        if self.limits_noted.insert(what.to_owned()) {
            self.notes.note(RendererChange::LimitExceeded {
                what: what.to_owned(),
            });
        }
    }

    /// Report the engine's export notices when the GPU's readiness changed:
    /// the pools go when the device goes, and `Available` comes when a device
    /// becomes ready.
    fn sync_exports(&mut self, was_ready: bool) {
        match (was_ready, self.gpu.ready()) {
            (true, false) => self.exports.lost_all(),
            (false, true) => self.exports.available(),
            _ => {}
        }
    }

    /// Draw `composition` into the native window's frame, on the GPU when it
    /// is ready and else on the CPU, at the window's physical size and
    /// `plan.scale`.
    ///
    /// Errors: [`FailureKind::Content`] for a composition or size that cannot
    /// be drawn (nothing is shown), [`FailureKind::Surface`] when the window's
    /// surface fails (the next frame makes a new GPU surface, and a window
    /// whose GPU surface keeps failing is drawn on the CPU).
    fn draw_native(
        &mut self,
        window: &mut dyn ShellWindow,
        composition: &Composition,
        plan: FramePlan,
    ) -> Result<Drawn, Failure> {
        let physical = window.physical_size();
        let make = |context: &GpuContext| make_surface(window, context, window.keeps_frames());
        self.gpu.advance(&mut self.notes, &make, physical);
        let started = Instant::now();
        // While a worker makes the window's swapchain, a frame presented
        // through GDI may not show (a flip-model swapchain replaces what
        // the window shows): the window keeps its last CPU frame and
        // the frame is drawn again later (the first GPU frame follows
        // the configuration at once).
        if self.gpu.configuring() && window.shows_cpu_frame() {
            return Ok(Drawn {
                renderer: Renderer::Cpu,
                shown: false,
                raster: Duration::ZERO,
                present: Duration::ZERO,
                report: FrameReport::default(),
            });
        }
        match self.gpu.present(&make, composition, plan.scale, physical) {
            None => {}
            Some(Ok((presentation, present, report))) => {
                window.release_cpu_surface();
                if presentation == Presentation::Shown {
                    self.keep_shown_frame(window);
                }
                return Ok(Drawn {
                    renderer: Renderer::Gpu,
                    shown: presentation == Presentation::Shown,
                    raster: started.elapsed().saturating_sub(present),
                    present,
                    report,
                });
            }
            // vello_cpu draws what vello_hybrid cannot (the device is
            // fine); the frame goes through the window's GPU surface.
            Some(Err(RenderError::GpuLimit(_))) => {
                self.note_limit("gpu_limit");
                let started = Instant::now();
                let (pixels, report) = self.cpu_pixels(composition, physical, plan.scale)?;
                let raster = started.elapsed();
                match self.gpu.present_pixels(&make, &pixels, physical) {
                    Some(Ok((presentation, present))) => {
                        if presentation == Presentation::Shown {
                            self.keep_shown_frame(window);
                        }
                        return Ok(Drawn {
                            renderer: Renderer::Cpu,
                            shown: presentation == Presentation::Shown,
                            raster,
                            present,
                            report,
                        });
                    }
                    Some(Err(e)) => self.gpu_failed(e)?,
                    None => {}
                }
            }
            Some(Err(e)) => self.gpu_failed(e)?,
        }
        self.draw_cpu_native(window, composition, plan.scale, physical)
    }

    /// Draw `composition` on the CPU at the window's `physical` size and
    /// `scale`: the window's 0x00RRGGBB pixels over white, and the frame's
    /// report.
    fn cpu_pixels(
        &mut self,
        composition: &Composition,
        physical: (u32, u32),
        scale: f64,
    ) -> Result<(Vec<u32>, FrameReport), Failure> {
        let output = self
            .cpu
            .frame(composition, physical, scale)
            .map_err(error::render)?;
        let pixels = output
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(pixel_0rgb)
            .collect();
        Ok((pixels, output.report))
    }

    /// Rasterize and present through the window's CPU buffer: only the damaged
    /// rectangles when the buffer still holds the previous frame.
    fn draw_cpu_native(
        &mut self,
        window: &mut dyn ShellWindow,
        composition: &Composition,
        scale: f64,
        physical: (u32, u32),
    ) -> Result<Drawn, Failure> {
        let started = Instant::now();
        let output = self
            .cpu
            .frame(composition, physical, scale)
            .map_err(error::render)?;
        let raster = started.elapsed();
        let CpuOutput {
            rgba,
            width,
            damage,
            report,
            ..
        } = output;
        let rects = match &damage {
            Damage::Full => None,
            Damage::Rects(rects) => Some(rects.as_slice()),
        };
        let backend = window.present_with(physical.0, physical.1, rects, &mut |buffer, age| {
            match rects {
                Some(rects) if age == BufferAge::Previous => {
                    for rect in rects {
                        write_rect(buffer, rgba, width, *rect);
                    }
                }
                _ => {
                    for (px, src) in buffer.iter_mut().zip(rgba.as_chunks::<4>().0) {
                        *px = pixel_0rgb(src);
                    }
                }
            }
            true
        });
        let shown = backend.map_err(|e| Failure::new(FailureKind::Surface, e.to_string()))?;
        Ok(Drawn {
            renderer: Renderer::Cpu,
            shown,
            raster,
            present: started.elapsed().saturating_sub(raster),
            report,
        })
    }

    /// Keep a copy of the GPU frame just shown when `window` keeps frames. A failed read-back
    /// leaves the frame shown: the failure only moves the renderer along for the next frame.
    fn keep_shown_frame(&mut self, window: &mut dyn ShellWindow) {
        if !window.keeps_frames() {
            return;
        }
        match self.gpu.read_back() {
            Some(Ok(frame)) => window.keep_frame(frame),
            Some(Err(e)) => {
                if let Err(failure) = self.gpu_failed(e) {
                    tracing::warn!(target: "kivact", %failure, "could not read back a shown GPU frame");
                }
            }
            None => {}
        }
    }

    /// Handle a failed GPU frame: a surface error that keeps the window on
    /// the GPU is returned (`FailureKind::Surface`: the frame policy makes the
    /// surface again); after a device failure, a surface that kept failing
    /// or a scene beyond the GPU renderer's limits the frame is drawn on
    /// the CPU (`Ok`); any other error is returned.
    pub(super) fn gpu_failed(&mut self, failure: RenderError) -> Result<(), Failure> {
        let was_ready = self.gpu.ready();
        let fallback = self.gpu.failed(&failure, &mut self.notes);
        self.sync_exports(was_ready);
        match fallback {
            Fallback::Cpu => Ok(()),
            Fallback::Surface => Err(Failure::new(FailureKind::Surface, failure.to_string())),
            // The CPU renderer draws what vello_hybrid cannot.
            Fallback::Other if matches!(failure, RenderError::GpuLimit(_)) => Ok(()),
            Fallback::Other => Err(error::render(failure)),
        }
    }
}

/// The 0x00RRGGBB pixel of premultiplied RGBA8 `p`, composited over white
/// (the same formula as `CpuFrame::to_softbuffer`).
fn pixel_0rgb(p: &[u8; 4]) -> u32 {
    let white = 255 - u32::from(p[3]);
    ((u32::from(p[0]) + white).min(255) << 16)
        | ((u32::from(p[1]) + white).min(255) << 8)
        | (u32::from(p[2]) + white).min(255)
}

/// Write the pixels of `rect` from the frame `rgba` (`width` px wide) into
/// the window's `buffer` (the same width), clipped to both.
fn write_rect(buffer: &mut [u32], rgba: &[u8], width: u32, rect: PixelRect) {
    let row_pixels = width as usize;
    let rows = rgba.len() / 4 / row_pixels.max(1);
    let x_end = rect.x.saturating_add(rect.width).min(width);
    let y_end = rect.y.saturating_add(rect.height).min(rows as u32);
    for y in rect.y..y_end {
        for x in rect.x..x_end {
            let index = y as usize * row_pixels + x as usize;
            let (Some(px), Some(src)) = (buffer.get_mut(index), rgba.get(index * 4..index * 4 + 4))
            else {
                continue;
            };
            *px = pixel_0rgb(src.try_into().expect("four bytes"));
        }
    }
}

/// Create the surface inside the backend's owner-thread callback, never by
/// retrieving a native handle from a GPU worker. Readback remains render policy.
pub(super) fn make_surface(
    window: &dyn ShellWindow,
    context: &GpuContext,
    keep_frames: bool,
) -> Result<GpuSurface, RenderError> {
    let make = context.surface_maker();
    let (sender, made) = std::sync::mpsc::sync_channel(1);
    window
        .with_gpu_target(Box::new(move |target: arvane_shell::GpuTarget| {
            let _ = sender.send(make(target));
        }))
        .map_err(|e| RenderError::Surface(e.to_string()))?;
    let mut surface = made
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| RenderError::Surface("the window is gone".into()))??;
    if keep_frames {
        surface.keep_frames();
    }
    Ok(surface)
}
