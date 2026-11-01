//! Role: Scene presentation and frame policy.
//!
//! Presenting scenes to a window: the renderers, their life cycle and the
//! frame policy behind [`crate::Window::present`]. Each window has one
//! presenter; the host only hands it a scene and learns what happened
//! ([`Presented`]).
//!
//! Frames go through the window's GPU surface when the GPU renderer is
//! ready (feature `gpu`), else they are rasterized on the CPU straight into
//! the window's CPU surface. A native window's presenter starts the GPU on
//! a worker thread with the window (the first frames are drawn on the CPU)
//! and moves the start along on the shell's wakes (`gpu.rs`).
//!
//! Invariants:
//! - A frame the GPU cannot draw because its device failed, or because the
//!   window's surface keeps failing, is drawn on the CPU in the same call,
//!   so neither loses a frame or ends the application.
//! - A frame the window has no buffer for (timed out, occluded) is not
//!   shown: it is to be drawn again a display period later, the wait
//!   doubling while it stays so, up to 16 periods ([`Presented::Skipped`]).
//! - A scene vello_hybrid cannot draw though the device is fine (an atlas
//!   or intermediate texture it cannot allocate, a missing texture
//!   binding: `RenderError::GpuLimit`) is drawn by vello_cpu for that
//!   frame alone and uploaded through the window's GPU surface (a window
//!   with a swapchain does not show what is presented through GDI).
//! - A failed presentation recreates the window's surface
//!   ([`Presented::SurfaceLost`]); the third in a row is an error (a shown
//!   frame counts again from zero), as is a surface that cannot be made
//!   again.
//! - A scene that cannot be drawn is [`Presented::Dropped`]; so is, on
//!   every renderer and before anything is drawn, a frame of a window of
//!   no pixels (minimized, sized 0: `Content`) or without a surface
//!   (destroyed by the platform: `Unavailable`). The resize that gives the
//!   window pixels back asks for a frame, so a dropped one is not retried.
//! - [`Presented::Shown`] means the window accepted the frame, not that it
//!   was scanned out.
//! - Renderer changes are noted to the window's observer as they happen
//!   ([`RendererChange`]: one line starting with `renderer=`), and so is an
//!   external frame the GPU did not draw (not ready within 100 ms, or
//!   already given back to its engine), once per frame value.
//! - Images an external render engine shares (feature `gpu-interop`) are
//!   imported into the window's own device and go with it: a device
//!   failure, a new device or the presenter's drop releases every frame
//!   of them (`arvane-render`'s `gpu/external.rs`).

#[cfg(feature = "gpu")]
mod gpu;

use crate::error::{self, Error, ErrorKind};
use crate::presentation::{Notes, Renderer, RendererChange};
use crate::{ShellError, ShellWaker};
use arvane_shell::ShellWindow;
use std::time::{Duration, Instant};
#[cfg(feature = "testing")]
use ui_types::CpuFrame;
use ui_types::{Scene, Size};

/// Surface failures in a row after which presenting is an error.
const LOSSES: u8 = 3;
/// A skipped frame is drawn again after at most 2^4 = 16 display periods.
const MAX_BACKOFF: u32 = 4;

/// A frame's logical extent and the physical scale it is rasterized at:
/// the window's display scale, reduced for a window beyond the renderers'
/// frame budget (each side 16,384 px, 64 MiB), whose frame is then
/// enlarged to the window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FramePlan {
    /// Logical extent; it never rasterizes beyond the window.
    pub size: Size,
    /// Physical pixels per logical pixel the frame is drawn at.
    pub scale: f32,
}

impl FramePlan {
    /// The plan for a window of `physical` pixels at display `scale`.
    pub fn new((width, height): (u32, u32), scale: f32) -> Self {
        let (size, scale) = arvane_render::raster_plan(width, height, scale);
        Self { size, scale }
    }
}

/// What [`crate::Window::present`] did, independent of the chosen renderer.
/// Arvane owns rasterization and surface recovery; the host owns scheduling
/// the next call. A skipped or lost frame must be retried with the host's
/// current scene and frame plan, since input, size or scale may have changed.
/// `Err` instead of this result means recovery failed or the surface failed
/// three times in a row; that window cannot continue presenting.
#[derive(Clone, Debug, PartialEq)]
pub enum Presented {
    /// The window accepted the frame. Native windows hand it to their
    /// presentation surface; test windows retain its pixels and mirror
    /// them when a real window is enabled. This does not acknowledge
    /// display scanout.
    Shown {
        /// The rasterizer that drew the frame.
        renderer: Renderer,
        /// Rasterizing the scene.
        raster: Duration,
        /// Waiting for the window's buffer and handing the frame over.
        present: Duration,
    },
    /// Presentation is temporarily unavailable (no buffer, occlusion, or
    /// GPU surface configuration): nothing new was shown. Draw again at
    /// `retry_at_ns` in [`crate::Window::now_ns`]'s clock, or as soon as the
    /// window is focused again. Consecutive skips back off from one display
    /// period to sixteen; a shown frame resets that backoff.
    Skipped {
        /// When to draw the frame again.
        retry_at_ns: u64,
    },
    /// The scene or its size cannot be drawn: nothing was shown and the
    /// window keeps its picture until the next change. So it is, on every
    /// renderer, for a window of no pixels (minimized): its resize asks
    /// for the next frame.
    Dropped(Error),
    /// The window's surface failed and was made again: nothing new was
    /// shown. Request another frame immediately; Arvane has already
    /// performed the recovery. A shown frame resets the failure count.
    SurfaceLost(Error),
}

/// What presenting to a window without a surface does: nothing is drawn
/// (the host draws again when the window has a surface again).
pub(crate) fn no_surface() -> Presented {
    Presented::Dropped(Error::with(
        ErrorKind::Unavailable,
        "the window has no surface (the platform destroyed it)",
    ))
}

/// What presenting to a window of no pixels does (minimized, or sized to
/// nothing; `physical` is its size): nothing is drawn, on any renderer,
/// and the resize that gives it pixels again asks for a frame. `None` for
/// a window with pixels.
pub(crate) fn no_pixels((width, height): (u32, u32)) -> Option<Presented> {
    (width == 0 || height == 0).then(|| {
        Presented::Dropped(Error::with(
            ErrorKind::Content,
            format!("the window has no pixels to draw ({width} x {height})"),
        ))
    })
}

/// What drawing one frame did, before the frame policy.
struct Drawn {
    renderer: Renderer,
    /// `false`: the window had no buffer to give.
    shown: bool,
    raster: Duration,
    present: Duration,
}

/// The renderers of one window and its frame policy.
pub(crate) struct Presenter {
    cpu: arvane_render::CpuRenderer,
    #[cfg(feature = "gpu")]
    gpu: gpu::Gpu,
    notes: Notes,
    /// Surface failures in a row.
    losses: u8,
    /// Frames skipped in a row (no buffer).
    skipped: u32,
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
            gpu: gpu::Gpu::new(env.gpu),
            notes,
            losses: 0,
            skipped: 0,
        }
    }

    /// Note a renderer change.
    pub(crate) fn note(&mut self, change: RendererChange) {
        self.notes.note(change);
    }

    /// Note the external frames the GPU did not draw (each once).
    fn note_missed(&mut self) {
        #[cfg(feature = "gpu")]
        for frame in self.gpu.take_missed() {
            self.notes
                .note(RendererChange::ExternalFrameMissed { frame });
        }
    }

    /// The adapter of the GPU device drawing the window `key`'s frames,
    /// when it can import shared textures (see [`crate::Window::gpu_adapter`]).
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn gpu_adapter(&self, key: u64) -> Option<ui_types::GpuAdapter> {
        self.gpu.adapter(key)
    }

    /// Import `textures` as the image `id` of the window `key` (see
    /// [`crate::Window::import_external_image`]).
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn import_external(
        &mut self,
        key: u64,
        id: ui_types::ExternalImageId,
        textures: ui_types::SharedTextures,
    ) -> Result<(), Error> {
        if self.gpu.adapter(key).is_none() {
            return Err(Error::with(
                ErrorKind::Unavailable,
                "the window has no GPU device that can import shared textures \
                 (it draws on the CPU, its device is not ready, or the adapter cannot)",
            ));
        }
        let context = self
            .gpu
            .context(key)
            .expect("invariant: a window with an adapter has a context");
        match context.import_external(id, textures) {
            Ok(()) => Ok(()),
            Err(failure @ arvane_render::RenderError::Backend(_)) => {
                let error = error::render(failure.clone());
                self.gpu.device_failed(&failure, &mut self.notes);
                Err(error)
            }
            Err(failure) => Err(error::render(failure)),
        }
    }

    /// Drop the image `id` of the window (see
    /// [`crate::Window::drop_external_image`]).
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn drop_external(&mut self, id: ui_types::ExternalImageId) {
        if let Some(context) = self.gpu.ready_context() {
            context.drop_external(id);
        }
    }

    /// The platform destroyed the surface of the window `key`: drop the
    /// GPU surface made for it (a new surface generation gets a new one
    /// with its first frame) and start the frame policy afresh.
    pub(crate) fn surface_destroyed(&mut self, key: u64) {
        #[cfg(feature = "gpu")]
        self.gpu.drop_surface(key);
        #[cfg(not(feature = "gpu"))]
        let _ = key;
        self.losses = 0;
        self.skipped = 0;
    }

    /// Start the GPU renderer on a worker thread; frames are drawn on the
    /// CPU until it is ready, and when no hardware adapter is available,
    /// or GPU support is not built in, for the whole session. `waker`
    /// wakes the window's thread when a start step finishes
    /// ([`Presenter::prepare`] then).
    pub(crate) fn start_gpu(&mut self, waker: ShellWaker) {
        #[cfg(feature = "gpu")]
        self.gpu.start(std::sync::Arc::new(move || waker.wake()));
        #[cfg(not(feature = "gpu"))]
        {
            let _ = waker;
            self.notes.note(RendererChange::Cpu {
                reason: "built without GPU support".into(),
            });
        }
    }

    /// Create the GPU renderer now, on this thread (test windows). Fails
    /// with [`ErrorKind::Unavailable`] when no hardware adapter exists or
    /// GPU support is not built in.
    #[cfg(feature = "testing")]
    pub(crate) fn enable_gpu(&mut self) -> Result<(), Error> {
        #[cfg(feature = "gpu")]
        return self.gpu.enable(&mut self.notes);
        #[cfg(not(feature = "gpu"))]
        Err(Error::with(
            ErrorKind::Unavailable,
            "Arvane was built without GPU support (feature `gpu`)",
        ))
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
            let key = window.key();
            let make = |context: &arvane_render::GpuContext| make_surface(window, context, false);
            self.gpu
                .advance(&mut self.notes, Some((key, &make, window.physical_size())));
            self.gpu.ready()
        }
        #[cfg(not(feature = "gpu"))]
        {
            let _ = window;
            false
        }
    }

    /// The rasterizer the next frame uses (after a pending GPU start
    /// finished).
    #[cfg(feature = "testing")]
    pub(crate) fn renderer(&mut self) -> Renderer {
        #[cfg(feature = "gpu")]
        {
            self.gpu.advance(&mut self.notes, None);
            if self.gpu.ready() {
                return Renderer::Gpu;
            }
        }
        Renderer::Cpu
    }

    /// Apply the frame policy to a frame the platform refused before it
    /// was drawn (test windows): it had no buffer to give (`failure`
    /// `None`: the window is hidden), or its surface failed.
    #[cfg(feature = "testing")]
    pub(crate) fn refused(
        &mut self,
        failure: Option<Error>,
        now_ns: u64,
        period_ns: u64,
    ) -> Result<Presented, Error> {
        let drawn = match failure {
            None => Ok(Drawn {
                renderer: Renderer::Cpu,
                shown: false,
                raster: Duration::ZERO,
                present: Duration::ZERO,
            }),
            Some(error) => Err(error),
        };
        self.settle(drawn, now_ns, period_ns, || Ok(()))
    }

    /// Apply the frame policy to what drawing a frame did (`now_ns`: the
    /// window's clock after it, `period_ns`: its display's refresh period,
    /// `recover` makes the window's surface again).
    fn settle(
        &mut self,
        drawn: Result<Drawn, Error>,
        now_ns: u64,
        period_ns: u64,
        recover: impl FnOnce() -> Result<(), ShellError>,
    ) -> Result<Presented, Error> {
        match drawn {
            Ok(drawn) if drawn.shown => {
                self.losses = 0;
                self.skipped = 0;
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
                let wait = period_ns << self.skipped.min(MAX_BACKOFF);
                self.skipped = self.skipped.saturating_add(1);
                Ok(Presented::Skipped {
                    retry_at_ns: now_ns.saturating_add(wait),
                })
            }
            Err(error) if error.kind() == ErrorKind::Surface => {
                self.losses += 1;
                if self.losses >= LOSSES {
                    return Err(error);
                }
                recover().map_err(|e| Error::with(ErrorKind::Surface, e.to_string()))?;
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
        scene: &Scene,
        plan: crate::FramePlan,
    ) -> Result<Presented, Error> {
        let drawn = self.draw_native(window, scene, plan.size, plan.scale);
        self.note_missed();
        let (now, period) = (window.now_ns(), window.frame_period_ns());
        self.settle(drawn, now, period, || window.recover_surface())
    }

    /// Render `scene` (logical `size`, drawn at `scale`) and show it in
    /// `window`. A scene rasterized smaller than the window (a scale
    /// reduced to the frame budget) is enlarged to it.
    ///
    /// Errors: [`ErrorKind::Content`] for a scene or size that cannot be
    /// drawn (nothing is shown), [`ErrorKind::Surface`] when the window's
    /// surface fails (the next frame makes a new GPU surface, and a window
    /// whose GPU surface keeps failing is drawn on the CPU).
    fn draw_native(
        &mut self,
        window: &mut dyn ShellWindow,
        scene: &Scene,
        size: Size,
        scale: f32,
    ) -> Result<Drawn, Error> {
        let physical = window.physical_size();
        #[cfg(feature = "gpu")]
        {
            let key = window.key();
            let make = |context: &arvane_render::GpuContext| make_surface(window, context, false);
            self.gpu
                .advance(&mut self.notes, Some((key, &make, physical)));
            let started = Instant::now();
            // While a worker makes the window's swapchain, a frame presented
            // through GDI may not show (a flip-model swapchain replaces what
            // the window shows): the window keeps its last CPU frame and
            // the frame is drawn again later (the first GPU frame follows
            // the configuration at once).
            if self.gpu.configuring(key) && window.shows_cpu_frame() {
                return Ok(Drawn {
                    renderer: Renderer::Cpu,
                    shown: false,
                    raster: Duration::ZERO,
                    present: Duration::ZERO,
                });
            }
            let presented = self.gpu.present(key, &make, scene, size, scale, physical);
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
                    match self.gpu.present_pixels(key, &make, &pixels, physical) {
                        Some(Ok((presentation, present))) => {
                            return Ok(Drawn {
                                renderer: Renderer::Cpu,
                                shown: presentation == arvane_render::Presentation::Shown,
                                raster,
                                present,
                            });
                        }
                        Some(Err(e)) => self.gpu_failed(key, e)?,
                        None => {}
                    }
                }
                Some(Err(e)) => self.gpu_failed(key, e)?,
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
        let shown = backend.map_err(|e| Error::with(ErrorKind::Surface, e.to_string()))?;
        Ok(Drawn {
            renderer: Renderer::Cpu,
            shown,
            raster,
            present: started.elapsed().saturating_sub(raster),
        })
    }

    /// Rasterize `scene` as `plan` says and keep it as a frame (test
    /// windows): through the GPU surface of the real test window `gpu`
    /// (read back), else offscreen, on the GPU (read back) when it is
    /// enabled or on the CPU; then apply the frame policy (`now_ns`: the
    /// window's clock; `period_ns`: its display's refresh period). Also
    /// returns the frame and its rasterizer, when it was drawn.
    #[cfg(feature = "testing")]
    pub(crate) fn present_test(
        &mut self,
        window: &mut dyn ShellWindow,
        native_gpu_surface: bool,
        scene: &Scene,
        plan: crate::FramePlan,
        physical: (u32, u32),
        (now_ns, period_ns): (u64, u64),
    ) -> Result<(Presented, Option<(CpuFrame, Renderer)>), Error> {
        let drawn = match native_gpu_surface {
            true => self.present_test_window(window, scene, plan.size, plan.scale, physical),
            false => {
                let started = Instant::now();
                match self.render(scene, plan.size, plan.scale) {
                    Ok(frame) => {
                        let pixels = surface_pixels(&frame, physical.0, physical.1);
                        let shown = window
                            .present_with(physical.0, physical.1, &mut |buffer| {
                                buffer.copy_from_slice(&pixels);
                                true
                            })
                            .map_err(|e| Error::with(ErrorKind::Surface, e.to_string()));
                        shown.map(|shown| {
                            let drawn = Drawn {
                                renderer: self.renderer(),
                                shown,
                                raster: started.elapsed(),
                                present: Duration::ZERO,
                            };
                            (drawn, frame)
                        })
                    }
                    Err(error) => Err(error),
                }
            }
        };
        self.note_missed();
        let (drawn, frame) = match drawn {
            Ok((drawn, frame)) => {
                let renderer = drawn.renderer;
                (Ok(drawn), Some((frame, renderer)))
            }
            Err(error) => (Err(error), None),
        };
        let presented = self.settle(drawn, now_ns, period_ns, || Ok(()))?;
        Ok((presented, frame))
    }

    /// Rasterize a display list at the requested physical scale: on the
    /// GPU (read back) once enabled, else on the CPU. A GPU device failure
    /// draws the frame on the CPU.
    #[cfg(feature = "testing")]
    fn render(&mut self, scene: &Scene, size: Size, scale: f32) -> Result<CpuFrame, Error> {
        #[cfg(feature = "gpu")]
        if let Some(rendered) = self.gpu.render(scene, size, scale) {
            match rendered {
                Ok(frame) => return Ok(frame),
                // Offscreen frames have no surface (key 0 names none).
                Err(e) => self.gpu_failed(0, e)?,
            }
        }
        self.cpu.render(scene, size, scale).map_err(error::render)
    }

    /// Present `scene` to the test window `window` through a GPU surface
    /// (the test display stops drawing the frames handed to that window)
    /// and read the frame back. Needs [`Presenter::enable_gpu`]; fails
    /// with [`ErrorKind::Unavailable`] without it.
    #[cfg(feature = "testing")]
    fn present_test_window(
        &mut self,
        window: &mut dyn ShellWindow,
        scene: &Scene,
        size: Size,
        scale: f32,
        physical: (u32, u32),
    ) -> Result<(Drawn, CpuFrame), Error> {
        #[cfg(feature = "gpu")]
        if self.gpu.ready() {
            let key = window.key();
            window.release_cpu_surface();
            let started = Instant::now();
            let make = |context: &arvane_render::GpuContext| make_surface(window, context, true);
            let presented = self.gpu.present(key, &make, scene, size, scale, physical);
            let (presentation, present) = match presented {
                Some(Ok(presented)) => presented,
                Some(Err(e)) => {
                    self.gpu_failed(key, e)?;
                    return Err(Error::with(ErrorKind::Backend, "the GPU frame failed"));
                }
                None => return Err(Error::with(ErrorKind::Backend, "the GPU device failed")),
            };
            let frame = self
                .gpu
                .read_back(key)
                .ok_or_else(|| Error::with(ErrorKind::Backend, "the GPU device failed"))?
                .map_err(error::render)?;
            let drawn = Drawn {
                renderer: Renderer::Gpu,
                shown: presentation == arvane_render::Presentation::Shown,
                raster: started.elapsed().saturating_sub(present),
                present,
            };
            return Ok((drawn, frame));
        }
        let _ = (window, scene, size, scale, physical);
        Err(Error::with(
            ErrorKind::Unavailable,
            "the GPU renderer is not enabled",
        ))
    }

    /// Handle a failed GPU frame: content and surface errors are returned;
    /// after a device failure, a scene beyond the GPU renderer's limits, or
    /// a surface failure that moves the window to the CPU, the frame is
    /// drawn on the CPU (`Ok`).
    #[cfg(feature = "gpu")]
    fn gpu_failed(&mut self, key: u64, failure: arvane_render::RenderError) -> Result<(), Error> {
        use arvane_render::RenderError;
        match failure {
            // A window whose surface keeps failing is drawn on the CPU.
            RenderError::Surface(_) if self.gpu.surface_failed(key, &failure, &mut self.notes) => {
                Ok(())
            }
            RenderError::Surface(_) => Err(Error::with(ErrorKind::Surface, failure.to_string())),
            // The CPU renderer draws what vello_hybrid cannot.
            RenderError::GpuLimit(_) => Ok(()),
            RenderError::Backend(_) | RenderError::NoAdapter(_) => {
                self.gpu.device_failed(&failure, &mut self.notes);
                Ok(())
            }
            _ => Err(error::render(failure)),
        }
    }
}

/// Create the surface inside the backend's owner-thread callback, never by
/// retrieving a native handle from a GPU worker. Readback remains render policy.
#[cfg(feature = "gpu")]
fn make_surface(
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

/// The 0x00RRGGBB pixels of a `width` × `height` window showing `frame`,
/// enlarging a frame drawn at a reduced scale by nearest neighbor.
#[cfg(feature = "testing")]
pub(crate) fn surface_pixels(frame: &ui_types::CpuFrame, width: u32, height: u32) -> Vec<u32> {
    let pixels = frame.to_softbuffer();
    if (frame.width, frame.height) == (width, height) || frame.width == 0 || frame.height == 0 {
        return pixels;
    }
    let source = |target: u32, extent: u32, source: u32| {
        ((u64::from(target) * u64::from(source) / u64::from(extent)) as usize)
            .min(source as usize - 1)
    };
    let mut out = Vec::with_capacity(width as usize * height as usize);
    for y in 0..height {
        let row = source(y, height, frame.height) * frame.width as usize;
        out.extend((0..width).map(|x| pixels[row + source(x, width, frame.width)]));
    }
    out
}
