//! Role: Presenter: one window's renderers and its frame policy.

use super::exports::{ExportNotice, ExportRegistry};
use super::frame::{FramePlan, Presented};
use super::gpu::{MakeSurface, WindowGpu};
use super::health::{FrameHealth, LossAction};
use crate::gpu_driver::{Fallback, GpuDriver};
use crate::presentation::{GpuStartCause, Notes, Renderer, RendererChange};
use arvane_compose::{CpuCompositor, CpuOutput, Damage, GpuContext, GpuSurface, Presentation};
use arvane_protocol::{Composition, FrameReport, PixelRect};
use arvane_render::RenderError;
use arvane_shell::{BufferAge, ShellWindow};
use kivact_types::{Failure, FailureKind};
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// What drawing one frame did, before the frame policy.
pub(super) enum Drawn {
    /// The window showed the frame.
    Shown {
        renderer: Renderer,
        raster: Duration,
        present: Duration,
        /// The frame's exports, missed external frames, shader failures and stats.
        report: FrameReport,
    },
    /// The window had no buffer to give: the frame is drawn again later.
    NoBuffer {
        /// The frame's exports, missed external frames, shader failures and stats.
        report: FrameReport,
    },
}

impl Drawn {
    /// The frame's exports, missed external frames, shader failures and stats.
    fn report(&self) -> &FrameReport {
        match self {
            Self::Shown { report, .. } | Self::NoBuffer { report, .. } => report,
        }
    }
}

/// The renderers of one window and its frame policy.
pub(crate) struct Presenter {
    pub(super) cpu: CpuCompositor,
    /// The window's GPU context and surface on the driver's device.
    pub(super) surface: WindowGpu,
    pub(super) notes: Notes,
    /// What the window's surface did with the last frames that reached it.
    pub(super) health: FrameHealth,
    /// The notices for the engine about the window's GPU export pools.
    pub(super) exports: ExportRegistry,
    /// The shader programs whose failure was already noted.
    shader_failed: HashSet<u64>,
    /// The GPU limit was noted (once per window).
    gpu_limit_noted: bool,
    /// The display period, from the window's environment.
    frame_period_ns: u64,
}

impl Presenter {
    /// Stop workers needing the native window before that window is dropped.
    pub(crate) fn shutdown(&mut self, window: &mut dyn ShellWindow) {
        self.surface.shutdown(window);
    }

    /// Note the display period the window's environment reports now.
    pub(crate) fn set_frame_period(&mut self, period_ns: u64) {
        self.frame_period_ns = period_ns;
    }

    pub(crate) fn release_caches(&mut self) {
        self.cpu.release_caches();
        self.surface.release_caches();
    }

    /// Release the CPU frame state of an idle window; its next frame redraws
    /// everything. The GPU state is kept (its memory is not in the CPU frame state).
    pub(crate) fn forget_frame_state(&mut self) {
        self.cpu.invalidate();
    }

    /// A presenter noting its renderer changes to `notes`.
    pub(crate) fn new(notes: Notes) -> Self {
        Self {
            cpu: CpuCompositor::new(),
            surface: WindowGpu::default(),
            notes,
            health: FrameHealth::Healthy,
            exports: ExportRegistry::default(),
            shader_failed: HashSet::new(),
            gpu_limit_noted: false,
            frame_period_ns: kivact_types::defaults::FRAME_PERIOD_NS,
        }
    }

    /// The adapter of the GPU device drawing the window's frames, when it
    /// can import shared textures (see [`crate::Window::gpu_adapter`]).
    pub(crate) fn gpu_adapter(&self, gpu: &GpuDriver) -> Option<kivact_types::GpuAdapter> {
        if !self.surface.ready() {
            return None;
        }
        gpu.device()?.gpu.interop_adapter()
    }

    /// Import `textures` as the image `id` of the window (see
    /// [`crate::Window::import_external_image`]).
    pub(crate) fn import_external(
        &mut self,
        gpu: &mut GpuDriver,
        id: kivact_types::ExternalImageId,
        textures: kivact_types::SharedTextures,
    ) -> Result<(), Failure> {
        if !arvane_compose::GPU_BUILT {
            return Err(Failure::new(
                FailureKind::Unavailable,
                "built without GPU support",
            ));
        }
        self.sync_gpu(gpu);
        if self.gpu_adapter(gpu).is_none() {
            return Err(Failure::new(
                FailureKind::Unavailable,
                "the window has no GPU device that can import shared textures \
                 (it draws on the CPU, its device is not ready, or the adapter cannot)",
            ));
        }
        let context = self
            .surface
            .context_mut()
            .expect("invariant: a device with an adapter is ready");
        match context.device().import_external(id, textures) {
            Ok(()) => Ok(()),
            Err(failure) => {
                let error = kivact_types::Failure::from(failure.clone());
                self.gpu_failure(gpu, &failure);
                Err(error)
            }
        }
    }

    /// Drop the image `id` of the window (see
    /// [`crate::Window::drop_external_image`]).
    pub(crate) fn drop_external(&mut self, gpu: &GpuDriver, id: kivact_types::ExternalImageId) {
        self.sync_gpu(gpu);
        if let Some(context) = self.surface.context_mut() {
            context.device().drop_external(id);
        }
    }

    /// Make the GPU export pool of `sink` for `generation` (see
    /// [`crate::Window::export_drawable`]).
    pub(crate) fn export_drawable(
        &mut self,
        gpu: &GpuDriver,
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
        self.sync_gpu(gpu);
        let Some(context) = self.surface.context_mut() else {
            return Err(Failure::new(
                FailureKind::NotReady,
                "the window's GPU device is not ready (it is starting, or the window draws on the CPU)",
            ));
        };
        context
            .export_drawable(sink, generation, adapter, size)
            .map_err(kivact_types::Failure::from)
    }

    /// Drop the export pool of `sink` (see [`crate::Window::drop_export`]).
    pub(crate) fn drop_export(&mut self, gpu: &GpuDriver, sink: kivact_types::ids::DrawableId) {
        self.sync_gpu(gpu);
        if let Some(context) = self.surface.context_mut() {
            context.drop_export(sink);
        }
    }

    /// The export notices since the last call (see
    /// [`crate::Window::take_export_notices`]).
    pub(crate) fn take_export_notices(&mut self) -> Vec<ExportNotice> {
        self.exports.take()
    }

    /// Make the device lost, as a device failure does (the test control of
    /// `arvane::lose_gpu_device`). Nothing happens while the window is not ready.
    #[cfg(feature = "test")]
    pub(crate) fn lose_gpu_device(&mut self, gpu: &mut GpuDriver) {
        if !self.surface.ready() {
            return;
        }
        let error = RenderError::Backend("test: device lost".into());
        self.gpu_failure(gpu, &error);
    }

    /// The platform destroyed the window's surface: drop the GPU surface
    /// made for it (a new surface generation gets a new one with its first
    /// frame) and start the frame policy afresh.
    pub(crate) fn surface_destroyed(&mut self) {
        self.surface.drop_surface();
        self.health = FrameHealth::Healthy;
    }

    /// Make the window's GPU context and surface when the driver has a
    /// device, and move a configuring surface along, without drawing. Returns
    /// whether the GPU became ready to draw `window`'s frames (draw one now,
    /// so the switch happens while the window is idle rather than with the
    /// next input).
    pub(crate) fn prepare(&mut self, window: &mut dyn ShellWindow, gpu: &mut GpuDriver) -> bool {
        self.sync_gpu(gpu);
        if self.surface.ready() {
            return false;
        }
        let physical = window.physical_size();
        let keep_frames = window.surface().frame_sink().is_some();
        let make = |context: &GpuContext, target: &mut dyn ShellWindow| {
            make_surface(target, context, keep_frames)
        };
        self.advance_gpu(window, gpu, &make, physical);
        self.surface.ready()
    }

    /// Move the driver's device start along, then this window's context and
    /// surface on it (see [`WindowGpu::advance`]). A failed step is applied
    /// as a frame's failure is.
    fn advance_gpu(
        &mut self,
        window: &mut dyn ShellWindow,
        gpu: &mut GpuDriver,
        make: MakeSurface<'_>,
        physical: (u32, u32),
    ) {
        let was_ready = self.surface.ready();
        gpu.advance(&mut self.notes);
        let device = gpu.device();
        match self.surface.advance(device, window, make, physical) {
            Ok(Some(configure)) => gpu.announce(configure, &mut self.notes),
            Ok(None) => {}
            Err(error) => {
                self.gpu_failure(gpu, &error);
            }
        }
        self.sync_exports(was_ready);
    }

    /// Bring the window's GPU context onto the driver's device: a context on
    /// a device the driver no longer has (failed, restarted) is lost.
    fn sync_gpu(&mut self, gpu: &GpuDriver) {
        let device = gpu.device();
        if self
            .surface
            .stale(device.as_ref().map(|device| &device.gpu))
        {
            self.lose_device();
        }
    }

    /// The window's context is gone: its export pools are reported lost.
    fn lose_device(&mut self) {
        let losses = self.surface.lose();
        self.exports.lost(losses);
    }

    /// Apply a GPU failure of this window (see the driver's failure policy):
    /// the window's surface is dropped, or its context is lost with the device.
    fn gpu_failure(&mut self, gpu: &mut GpuDriver, failure: &RenderError) -> Fallback {
        let fallback = gpu.failed(failure, &mut self.notes);
        match fallback {
            Fallback::Surface => self.surface.drop_surface(),
            Fallback::Cpu => self.lose_device(),
            Fallback::Other => {}
        }
        fallback
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
        gpu: &mut GpuDriver,
        (composition, plan): (&Composition, FramePlan),
    ) -> Result<Presented, Failure> {
        // Every frame that was drawn notes its report, including one drawn
        // after a surface loss on the CPU.
        if let Ok(drawn) = &drawn {
            self.note_report(drawn.report());
        }
        // A frame that was not shown leaves the compositor's records ahead
        // of what the window holds: the next frame redraws everything.
        if !matches!(&drawn, Ok(Drawn::Shown { .. })) {
            self.cpu.invalidate();
        }
        match drawn {
            Ok(Drawn::Shown {
                renderer,
                raster,
                present,
                report,
            }) => {
                self.health.shown();
                Ok(Presented::Shown {
                    renderer,
                    raster,
                    present,
                    exports: report.exports,
                    pending_exports: report.pending_exports,
                    stats: report.stats,
                })
            }
            // The surface had no buffer (timed out, occluded): the picture
            // is drawn again a display period later, backing off to 16
            // periods while it stays so.
            Ok(Drawn::NoBuffer { .. }) => {
                let wait = self.health.skipped(period_ns);
                Ok(Presented::Skipped {
                    retry_at_ns: now_ns.saturating_add(wait),
                })
            }
            Err(error) if error.kind == FailureKind::Surface => {
                match self.health.lost(renderer) {
                    LossAction::Fail => return Err(error),
                    LossAction::FallbackToCpu => {
                        gpu.fallback_to_cpu(&mut self.notes, &error);
                        self.lose_device();
                        let drawn = self.draw_native(window, gpu, composition, plan);
                        return self.settle(
                            drawn,
                            (now_ns, period_ns),
                            Renderer::Cpu,
                            window,
                            gpu,
                            (composition, plan),
                        );
                    }
                    LossAction::Recover => {}
                }
                window
                    .surface()
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
        gpu: &mut GpuDriver,
        composition: &Composition,
        plan: FramePlan,
    ) -> Result<Presented, Failure> {
        self.sync_gpu(gpu);
        let was_ready = self.surface.ready();
        let drawn = self.draw_native(window, gpu, composition, plan);
        let (now, period) = (window.now_ns(), self.frame_period_ns);
        if gpu.waits()
            && let Ok(drawn) = &drawn
        {
            if drawn.report().shader_fallback {
                gpu.demand(GpuStartCause::Shader, &mut self.notes);
            } else if let Drawn::Shown {
                renderer: Renderer::Cpu,
                raster,
                present,
                ..
            } = drawn
            {
                gpu.note_cpu_frame(
                    *raster + *present,
                    Duration::from_nanos(period),
                    &mut self.notes,
                );
            }
        }
        let renderer = if self.surface.ready() {
            Renderer::Gpu
        } else {
            Renderer::Cpu
        };
        let presented = self.settle(
            drawn,
            (now, period),
            renderer,
            window,
            gpu,
            (composition, plan),
        );
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

    fn note_gpu_limit(&mut self) {
        if !self.gpu_limit_noted {
            self.gpu_limit_noted = true;
            self.notes.note(RendererChange::GpuLimit);
        }
    }

    /// Report the engine's `Available` notice when the window became ready
    /// for exports now (its pools are reported lost with the context, see
    /// [`Self::lose_device`]).
    fn sync_exports(&mut self, was_ready: bool) {
        if !was_ready && self.surface.ready() {
            self.exports.available();
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
        gpu: &mut GpuDriver,
        composition: &Composition,
        plan: FramePlan,
    ) -> Result<Drawn, Failure> {
        let physical = window.physical_size();
        let keep_frames = window.surface().frame_sink().is_some();
        let make = |context: &GpuContext, target: &mut dyn ShellWindow| {
            make_surface(target, context, keep_frames)
        };
        self.advance_gpu(window, gpu, &make, physical);
        let started = Instant::now();
        // While a worker makes the window's swapchain, a frame presented
        // through GDI may not show (a flip-model swapchain replaces what
        // the window shows): the window keeps its last CPU frame and
        // the frame is drawn again later (the first GPU frame follows
        // the configuration at once).
        if self.surface.configuring() && window.surface().shows_cpu_frame() {
            return Ok(Drawn::NoBuffer {
                report: FrameReport::default(),
            });
        }
        match self
            .surface
            .present(window, &make, composition, plan.scale, physical)
        {
            None => {}
            Some(Ok((presentation, present, report))) => {
                window.surface().release_cpu_surface();
                if presentation == Presentation::Shown {
                    self.keep_shown_frame(window, gpu);
                    return Ok(Drawn::Shown {
                        renderer: Renderer::Gpu,
                        raster: started.elapsed().saturating_sub(present),
                        present,
                        report,
                    });
                }
                return Ok(Drawn::NoBuffer { report });
            }
            // vello_cpu draws what vello_hybrid cannot (the device is
            // fine); the frame goes through the window's GPU surface.
            Some(Err(RenderError::GpuLimit(_))) => {
                self.note_gpu_limit();
                let started = Instant::now();
                let (pixels, report) = self.cpu_pixels(composition, physical, plan.scale)?;
                let raster = started.elapsed();
                match self
                    .surface
                    .present_pixels(window, &make, &pixels, physical)
                {
                    Some(Ok((presentation, present))) => {
                        // The window's CPU surface holds an older frame; the next CPU frame
                        // must not be damage-merged onto it.
                        window.surface().release_cpu_surface();
                        if presentation == Presentation::Shown {
                            self.keep_shown_frame(window, gpu);
                            return Ok(Drawn::Shown {
                                renderer: Renderer::Cpu,
                                raster,
                                present,
                                report,
                            });
                        }
                        return Ok(Drawn::NoBuffer { report });
                    }
                    Some(Err(e)) => self.gpu_failed(gpu, e)?,
                    None => {}
                }
            }
            Some(Err(e)) => self.gpu_failed(gpu, e)?,
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
            .map_err(kivact_types::Failure::from)?;
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
            .map_err(kivact_types::Failure::from)?;
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
        let backend =
            window
                .surface()
                .present_with(physical.0, physical.1, rects, &mut |buffer, age| {
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
        let present = started.elapsed().saturating_sub(raster);
        if shown {
            Ok(Drawn::Shown {
                renderer: Renderer::Cpu,
                raster,
                present,
                report,
            })
        } else {
            Ok(Drawn::NoBuffer { report })
        }
    }

    /// Keep a copy of the GPU frame just shown when `window` keeps frames. A failed read-back
    /// leaves the frame shown: the failure only moves the renderer along for the next frame.
    fn keep_shown_frame(&mut self, window: &mut dyn ShellWindow, gpu: &mut GpuDriver) {
        if window.surface().frame_sink().is_none() {
            return;
        }
        match self.surface.read_back() {
            Some(Ok(frame)) => {
                if let Some(sink) = window.surface().frame_sink() {
                    sink.keep(frame);
                }
            }
            Some(Err(e)) => {
                if let Err(failure) = self.gpu_failed(gpu, e) {
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
    pub(super) fn gpu_failed(
        &mut self,
        gpu: &mut GpuDriver,
        failure: RenderError,
    ) -> Result<(), Failure> {
        let was_ready = self.surface.ready();
        let fallback = self.gpu_failure(gpu, &failure);
        self.sync_exports(was_ready);
        match fallback {
            Fallback::Cpu => Ok(()),
            Fallback::Surface => Err(Failure::new(FailureKind::Surface, failure.to_string())),
            // The CPU renderer draws what vello_hybrid cannot.
            Fallback::Other if matches!(failure, RenderError::GpuLimit(_)) => Ok(()),
            Fallback::Other => Err(kivact_types::Failure::from(failure)),
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
    window: &mut dyn ShellWindow,
    context: &GpuContext,
    keep_frames: bool,
) -> Result<GpuSurface, RenderError> {
    let make = context.device().surface_maker();
    let (sender, made) = std::sync::mpsc::sync_channel(1);
    window
        .surface()
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
