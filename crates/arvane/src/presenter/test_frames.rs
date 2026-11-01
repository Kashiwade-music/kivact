//! Role: Presents test-window frames and keeps them as pixels (feature `testing`).

use super::frame::Presented;
#[cfg(feature = "gpu")]
use super::frame_policy::make_surface;
use super::frame_policy::{Drawn, Presenter};
use crate::error::{self, Error, ErrorKind};
use crate::presentation::Renderer;
use arvane_shell::ShellWindow;
use kivact_types::CpuFrame;
use kivact_types::{Scene, Size};
use std::time::{Duration, Instant};

impl Presenter {
    /// Create the GPU renderer now, on this thread (test windows). Fails
    /// with [`ErrorKind::Unavailable`] when no hardware adapter exists or
    /// GPU support is not built in.
    pub(crate) fn enable_gpu(&mut self) -> Result<(), Error> {
        #[cfg(feature = "gpu")]
        return self.gpu.enable(&mut self.notes);
        #[cfg(not(feature = "gpu"))]
        Err(Error::with(
            ErrorKind::Unavailable,
            "Arvane was built without GPU support (feature `gpu`)",
        ))
    }

    /// The rasterizer the next frame uses (after a pending GPU start
    /// finished).
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

    /// Rasterize `scene` as `plan` says and keep it as a frame (test
    /// windows): through the GPU surface of the real test window `gpu`
    /// (read back), else offscreen, on the GPU (read back) when it is
    /// enabled or on the CPU; then apply the frame policy (`now_ns`: the
    /// window's clock; `period_ns`: its display's refresh period). Also
    /// returns the frame and its rasterizer, when it was drawn.
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
}

/// The 0x00RRGGBB pixels of a `width` × `height` window showing `frame`,
/// enlarging a frame drawn at a reduced scale by nearest neighbor.
pub(crate) fn surface_pixels(frame: &kivact_types::CpuFrame, width: u32, height: u32) -> Vec<u32> {
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
