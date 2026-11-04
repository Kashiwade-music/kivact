//! Role: Presenter: one window's renderers and the policy applied to each frame.

use super::frame::Presented;
#[cfg(feature = "gpu")]
use super::gpu::Fallback;
use super::health::FrameHealth;
use crate::error::{self, Failure, FailureKind};
use crate::presentation::{Notes, Renderer, RendererChange};
use crate::{ShellError, ShellWaker};
#[cfg(feature = "gpu")]
use arvane_render::RenderError;
use arvane_shell::ShellWindow;
use kivact_types::Size;
use kivact_types::render::RenderScene;
use std::time::{Duration, Instant};

/// What drawing one frame did, before the frame policy.
pub(super) struct Drawn {
    pub(super) renderer: Renderer,
    /// `false`: the window had no buffer to give.
    pub(super) shown: bool,
    pub(super) raster: Duration,
    pub(super) present: Duration,
}

/// The renderers of one window and its frame policy.
pub(crate) struct Presenter {
    pub(super) cpu: arvane_render::CpuRenderer,
    #[cfg(feature = "gpu")]
    pub(super) gpu: super::gpu::Gpu,
    pub(super) notes: Notes,
    /// What the window's surface did with the last frames that reached it.
    pub(super) health: FrameHealth,
}

impl Presenter {
    /// Stop workers needing the native window before that window is dropped.
    pub(crate) fn shutdown(&mut self, window: &mut dyn ShellWindow) {
        #[cfg(feature = "gpu")]
        self.gpu.shutdown(window);
        #[cfg(not(feature = "gpu"))]
        let _ = window;
    }

    pub(crate) fn release_caches(&mut self) {
        self.cpu.release_caches();
        #[cfg(feature = "gpu")]
        self.gpu.release_caches();
    }

    /// A presenter noting its renderer changes to `notes`, its GPU devices
    /// made as `env` says.
    pub(crate) fn new(notes: Notes, env: &crate::env::Env) -> Self {
        #[cfg(not(feature = "gpu"))]
        let _ = env;
        Self {
            cpu: arvane_render::CpuRenderer::new(),
            #[cfg(feature = "gpu")]
            gpu: super::gpu::Gpu::new(env.gpu),
            notes,
            health: FrameHealth::Healthy,
        }
    }

    /// Note a renderer change.
    pub(crate) fn note(&mut self, change: RendererChange) {
        self.notes.note(change);
    }

    /// Note the external frames the GPU did not draw (each once).
    pub(super) fn note_missed(&mut self) {
        #[cfg(feature = "gpu")]
        for frame in self.gpu.take_missed() {
            self.notes
                .note(RendererChange::ExternalFrameMissed { frame });
        }
    }

    /// The adapter of the GPU device drawing the window's frames, when it
    /// can import shared textures (see [`crate::Window::gpu_adapter`]).
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn gpu_adapter(&self) -> Option<kivact_types::GpuAdapter> {
        self.gpu.adapter()
    }

    /// Import `textures` as the image `id` of the window (see
    /// [`crate::Window::import_external_image`]).
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn import_external(
        &mut self,
        id: kivact_types::ExternalImageId,
        textures: kivact_types::SharedTextures,
    ) -> Result<(), Failure> {
        if self.gpu.adapter().is_none() {
            return Err(Failure::new(
                FailureKind::Unavailable,
                "the window has no GPU device that can import shared textures \
                 (it draws on the CPU, its device is not ready, or the adapter cannot)",
            ));
        }
        let context = self
            .gpu
            .context()
            .expect("invariant: a device with an adapter is ready");
        match context.import_external(id, textures) {
            Ok(()) => Ok(()),
            Err(failure) => {
                let error = error::render(failure.clone());
                self.gpu.failed(&failure, &mut self.notes);
                Err(error)
            }
        }
    }

    /// Drop the image `id` of the window (see
    /// [`crate::Window::drop_external_image`]).
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn drop_external(&mut self, id: kivact_types::ExternalImageId) {
        if let Some(context) = self.gpu.context() {
            context.drop_external(id);
        }
    }

    /// The platform destroyed the window's surface: drop the GPU surface
    /// made for it (a new surface generation gets a new one with its first
    /// frame) and start the frame policy afresh.
    pub(crate) fn surface_destroyed(&mut self) {
        #[cfg(feature = "gpu")]
        self.gpu.drop_surface();
        self.health = FrameHealth::Healthy;
    }

    /// Start the GPU renderer on a worker thread; frames are drawn on the
    /// CPU until it is ready, and when no hardware adapter is available,
    /// or GPU support is not built in, for the whole session. `waker`
    /// wakes the window's thread when a start step finishes
    /// ([`Presenter::prepare`] then).
    pub(crate) fn start_gpu(&mut self, waker: ShellWaker) {
        #[cfg(feature = "gpu")]
        self.gpu
            .start(std::sync::Arc::new(move || waker.wake()), &mut self.notes);
        #[cfg(not(feature = "gpu"))]
        {
            let _ = waker;
            self.notes.note(RendererChange::Cpu {
                reason: "built without GPU support".into(),
            });
        }
    }

    /// Move a GPU start along for `window` without drawing: once the
    /// device is ready, make the window's surface and configure it on a
    /// worker. Returns whether the GPU became ready to draw `window`'s
    /// frames (draw one now, so the switch happens while the window is
    /// idle rather than with the next input).
    pub(crate) fn prepare(&mut self, window: &dyn ShellWindow) -> bool {
        #[cfg(feature = "gpu")]
        {
            if self.gpu.ready() {
                return false;
            }
            let make = |context: &arvane_render::GpuContext| make_surface(window, context, false);
            self.gpu
                .advance(&mut self.notes, &make, window.physical_size());
            self.gpu.ready()
        }
        #[cfg(not(feature = "gpu"))]
        {
            let _ = window;
            false
        }
    }

    /// Apply the frame policy to what drawing a frame did (`now_ns`: the
    /// window's clock after it, `period_ns`: its display's refresh period,
    /// `recover` makes the window's surface again).
    pub(super) fn settle(
        &mut self,
        drawn: Result<Drawn, Failure>,
        now_ns: u64,
        period_ns: u64,
        recover: impl FnOnce() -> Result<(), ShellError>,
    ) -> Result<Presented, Failure> {
        match drawn {
            Ok(drawn) if drawn.shown => {
                self.health.shown();
                Ok(Presented::Shown {
                    renderer: drawn.renderer,
                    raster: drawn.raster,
                    present: drawn.present,
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
                if self.health.lost() {
                    return Err(error);
                }
                recover().map_err(|e| Failure::new(FailureKind::Surface, e.to_string()))?;
                Ok(Presented::SurfaceLost(error))
            }
            Err(error) => Ok(Presented::Dropped(error)),
        }
    }

    /// Render `scene` as `plan` says and show it in the native `window`
    /// (see the module invariants).
    pub(crate) fn present_native(
        &mut self,
        window: &mut dyn ShellWindow,
        scene: &RenderScene,
        plan: crate::FramePlan,
    ) -> Result<Presented, Failure> {
        let drawn = self.draw_native(window, scene, plan.size, plan.scale);
        self.note_missed();
        let (now, period) = (window.now_ns(), window.frame_period_ns());
        self.settle(drawn, now, period, || window.recover_surface())
    }

    /// Render `scene` (logical `size`, drawn at `scale`) and show it in
    /// `window`. A scene rasterized smaller than the window (a scale
    /// reduced to the frame budget) is enlarged to it.
    ///
    /// Errors: [`FailureKind::Content`] for a scene or size that cannot be
    /// drawn (nothing is shown), [`FailureKind::Surface`] when the window's
    /// surface fails (the next frame makes a new GPU surface, and a window
    /// whose GPU surface keeps failing is drawn on the CPU).
    fn draw_native(
        &mut self,
        window: &mut dyn ShellWindow,
        scene: &RenderScene,
        size: Size,
        scale: f64,
    ) -> Result<Drawn, Failure> {
        let physical = window.physical_size();
        #[cfg(feature = "gpu")]
        {
            let make = |context: &arvane_render::GpuContext| make_surface(window, context, false);
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
                });
            }
            let presented = self.gpu.present(&make, scene, size, scale, physical);
            match presented {
                None => {}
                Some(Ok((presentation, present))) => {
                    window.release_cpu_surface();
                    return Ok(Drawn {
                        renderer: Renderer::Gpu,
                        shown: presentation == arvane_render::Presentation::Shown,
                        raster: started.elapsed().saturating_sub(present),
                        present,
                    });
                }
                // vello_cpu draws what vello_hybrid cannot (the device is
                // fine); the frame goes through the window's GPU surface.
                Some(Err(arvane_render::RenderError::GpuLimit(_))) => {
                    let started = Instant::now();
                    let mut pixels = vec![0; physical.0 as usize * physical.1 as usize];
                    self.cpu
                        .render_0rgb(scene, size, scale, physical, &mut pixels)
                        .map_err(error::render)?;
                    let raster = started.elapsed();
                    match self.gpu.present_pixels(&make, &pixels, physical) {
                        Some(Ok((presentation, present))) => {
                            return Ok(Drawn {
                                renderer: Renderer::Cpu,
                                shown: presentation == arvane_render::Presentation::Shown,
                                raster,
                                present,
                            });
                        }
                        Some(Err(e)) => self.gpu_failed(e)?,
                        None => {}
                    }
                }
                Some(Err(e)) => self.gpu_failed(e)?,
            }
        }
        let started = Instant::now();
        let mut raster = Duration::ZERO;
        let renderer = &mut self.cpu;
        let mut render_error = None;
        let backend = window.present_with(physical.0, physical.1, &mut |buffer| {
            let drawn = renderer.render_0rgb(scene, size, scale, physical, buffer);
            raster = started.elapsed();
            match drawn {
                Ok(()) => true,
                Err(error) => {
                    render_error = Some(error);
                    false
                }
            }
        });
        if let Some(error) = render_error {
            return Err(error::render(error));
        }
        let shown = backend.map_err(|e| Failure::new(FailureKind::Surface, e.to_string()))?;
        Ok(Drawn {
            renderer: Renderer::Cpu,
            shown,
            raster,
            present: started.elapsed().saturating_sub(raster),
        })
    }

    /// Handle a failed GPU frame: a surface error that keeps the window on
    /// the GPU is returned (`FailureKind::Surface`: the frame policy makes the
    /// surface again); after a device failure, a surface that kept failing
    /// or a scene beyond the GPU renderer's limits the frame is drawn on
    /// the CPU (`Ok`); any other error is returned.
    #[cfg(feature = "gpu")]
    pub(super) fn gpu_failed(&mut self, failure: RenderError) -> Result<(), Failure> {
        match self.gpu.failed(&failure, &mut self.notes) {
            Fallback::Cpu => Ok(()),
            Fallback::Surface => Err(Failure::new(FailureKind::Surface, failure.to_string())),
            // The CPU renderer draws what vello_hybrid cannot.
            Fallback::Other if matches!(failure, RenderError::GpuLimit(_)) => Ok(()),
            Fallback::Other => Err(error::render(failure)),
        }
    }
}

/// Create the surface inside the backend's owner-thread callback, never by
/// retrieving a native handle from a GPU worker. Readback remains render policy.
#[cfg(feature = "gpu")]
pub(super) fn make_surface(
    window: &dyn ShellWindow,
    context: &arvane_render::GpuContext,
    keep_frames: bool,
) -> Result<arvane_render::GpuSurface, arvane_render::RenderError> {
    let make = context.surface_maker();
    let (sender, made) = std::sync::mpsc::sync_channel(1);
    window
        .with_gpu_target(Box::new(move |target: arvane_shell::GpuTarget| {
            let _ = sender.send(make(target));
        }))
        .map_err(|e| arvane_render::RenderError::Surface(e.to_string()))?;
    let mut surface = made
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| arvane_render::RenderError::Surface("the window is gone".into()))??;
    if keep_frames {
        surface.keep_frames();
    }
    Ok(surface)
}
