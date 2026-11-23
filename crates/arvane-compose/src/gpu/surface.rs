//! Role: Presenting to a window through a wgpu surface (a swapchain).
//!
//! A frame is drawn into its output texture (the window's size), then copied to the
//! swapchain over white, as the CPU frames are composited. Resizing makes a new swapchain
//! before the frame is shown. An outdated or lost surface is reconfigured and the frame
//! retried once; a timeout or an occluded window skips the frame ([`Presentation::Skipped`]);
//! anything else is an error for the facade.

use super::pass::{Drawn, view, with_pass};
use super::state::{Configured, GpuContext, GpuSurface, Presentation, Retain, Swapchain};
use super::targets;
use crate::{GpuReport, RenderError};
use arvane_protocol::{Composition, CpuFrame};
use std::time::{Duration, Instant};

/// The swapchain formats the executor can present to, in order of preference.
const SURFACE_FORMATS: [wgpu::TextureFormat; 2] = [
    wgpu::TextureFormat::Bgra8Unorm,
    wgpu::TextureFormat::Rgba8Unorm,
];

/// The colour a frame is shown over.
const WHITE: wgpu::Color = wgpu::Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

impl GpuSurface {
    /// A surface not configured yet.
    pub(super) fn new(surface: wgpu::Surface<'static>) -> Self {
        Self {
            surface,
            swapchain: Swapchain::Unconfigured,
            kept: Retain::Drop,
        }
    }

    /// Keep each presented frame for [`GpuContext::read_back`] (tests).
    pub fn keep_frames(&mut self) {
        if matches!(self.kept, Retain::Drop) {
            self.kept = Retain::Keep(None);
        }
    }
}

impl GpuContext {
    /// Draw `composition` at `scale` into a `physical` window and present it to `surface`.
    /// Also returns the time spent waiting for the surface's buffer and presenting, and what
    /// the frame produced besides pixels.
    pub fn present(
        &mut self,
        surface: &mut GpuSurface,
        composition: &Composition,
        scale: f64,
        physical: (u32, u32),
    ) -> Result<(Presentation, Duration, GpuReport), RenderError> {
        let (width, height) = physical;
        if width == 0 || height == 0 {
            return Ok((Presentation::Skipped, Duration::ZERO, GpuReport::default()));
        }
        self.reconfigure(surface, width, height, false)?;
        let Drawn { output, report } = self.draw_frame(composition, physical, scale)?;
        let shown = self.show(surface, &output);
        self.retain(surface, output);
        let (presentation, waited) = shown?;
        Ok((presentation, waited, report))
    }

    /// Present a frame drawn elsewhere to `surface`, a `width` × `height` physical-pixel window:
    /// `pixels` are its rows of 0x00RRGGBB (the CPU renderer's).
    pub fn present_pixels(
        &mut self,
        surface: &mut GpuSurface,
        pixels: &[u32],
        (width, height): (u32, u32),
    ) -> Result<(Presentation, Duration), RenderError> {
        if width == 0 || height == 0 {
            return Ok((Presentation::Skipped, Duration::ZERO));
        }
        if pixels.len() != width as usize * height as usize {
            return Err(RenderError::InvalidSize);
        }
        self.reconfigure(surface, width, height, false)?;
        let bytes: Vec<u8> = pixels
            .iter()
            .flat_map(|p| {
                [
                    ((p >> 16) & 0xff) as u8,
                    ((p >> 8) & 0xff) as u8,
                    (p & 0xff) as u8,
                    255,
                ]
            })
            .collect();
        let source = targets::target(&self.gpu.device, (width, height));
        self.gpu.queue.write_texture(
            source.as_image_copy(),
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            source.size(),
        );
        let shown = self.show(surface, &source);
        self.pool.give(source);
        shown
    }

    /// Read back the last frame presented to `surface` (over white), which keeps its frames
    /// ([`GpuSurface::keep_frames`]).
    pub fn read_back(&self, surface: &GpuSurface) -> Result<CpuFrame, RenderError> {
        let texture = match &surface.kept {
            Retain::Keep(Some(texture)) => texture,
            _ => return Err(RenderError::Surface("no frame was kept".into())),
        };
        let mut frame = self.read_frame(texture)?;
        for px in frame.rgba.as_chunks_mut::<4>().0 {
            let clear = 255 - u32::from(px[3]);
            for c in &mut px[..3] {
                *c = (u32::from(*c) + clear).min(255) as u8;
            }
            px[3] = 255;
        }
        Ok(frame)
    }

    /// Copy `source` (the window's size) to the surface's next texture over white, and present it.
    fn show(
        &mut self,
        surface: &mut GpuSurface,
        source: &wgpu::Texture,
    ) -> Result<(Presentation, Duration), RenderError> {
        let (width, height) = (source.width(), source.height());
        let waiting = Instant::now();
        let Some((texture, suboptimal)) = self.acquire(surface, width, height)? else {
            return Ok((Presentation::Skipped, waiting.elapsed()));
        };
        let mut waited = waiting.elapsed();
        let format = texture.texture.format();
        let target_view = view(&texture.texture);
        let source_view = view(source);
        let group = self
            .gpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Arvane present"),
                layout: &self.gpu.pipelines.apply_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source_view),
                }],
            });
        let pipeline = self.gpu.pipelines.present_for(&self.gpu.device, format);
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Arvane present"),
            });
        with_pass(
            &mut encoder,
            &target_view,
            wgpu::LoadOp::Clear(WHITE),
            |pass| {
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &group, &[]);
                pass.draw(0..3, 0..1);
            },
        );
        self.gpu.queue.submit([encoder.finish()]);
        let presenting = Instant::now();
        texture.present();
        waited += presenting.elapsed();
        if suboptimal {
            self.reconfigure(surface, width, height, true)?;
        }
        self.gpu.check()?;
        Ok((Presentation::Shown, waited))
    }

    /// Keep `output` for [`GpuContext::read_back`] when the surface keeps its frames, else pool it.
    fn retain(&mut self, surface: &mut GpuSurface, output: wgpu::Texture) {
        match &mut surface.kept {
            Retain::Keep(last) => {
                if let Some(old) = last.replace(output) {
                    self.pool.give(old);
                }
            }
            Retain::Drop => self.pool.give(output),
        }
    }

    /// The surface's next texture and whether it is suboptimal; `None` skips the frame.
    fn acquire(
        &self,
        surface: &mut GpuSurface,
        width: u32,
        height: u32,
    ) -> Result<Option<(wgpu::SurfaceTexture, bool)>, RenderError> {
        for attempt in 0..2 {
            match surface.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(texture) => return Ok(Some((texture, false))),
                wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                    return Ok(Some((texture, true)));
                }
                wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                    return Ok(None);
                }
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost
                    if attempt == 0 =>
                {
                    self.reconfigure(surface, width, height, true)?;
                }
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    return Err(RenderError::Surface(
                        "the window surface stayed lost after reconfiguring".into(),
                    ));
                }
                wgpu::CurrentSurfaceTexture::Validation => {
                    self.gpu.check()?;
                    return Err(RenderError::Backend(
                        "the window surface failed validation".into(),
                    ));
                }
            }
        }
        unreachable!("the second attempt returns")
    }

    fn reconfigure(
        &self,
        surface: &mut GpuSurface,
        width: u32,
        height: u32,
        again: bool,
    ) -> Result<(), RenderError> {
        configure(
            &self.gpu.device,
            &self.gpu.adapter,
            surface,
            width,
            height,
            again,
        )?;
        self.gpu.check()
    }
}

/// Configure `surface` (making its swapchain on the first call).
pub(super) fn configure(
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    surface: &mut GpuSurface,
    width: u32,
    height: u32,
    again: bool,
) -> Result<(), RenderError> {
    if let Swapchain::Configured(configured) = &mut surface.swapchain {
        if !again && (configured.config.width, configured.config.height) == (width, height) {
            return Ok(());
        }
        configured.config.width = width;
        configured.config.height = height;
        surface.surface.configure(device, &configured.config);
        return Ok(());
    }
    let config = surface_config(adapter, &surface.surface, width, height)?;
    surface.surface.configure(device, &config);
    surface.swapchain = Swapchain::Configured(Configured { config });
    Ok(())
}

/// The swapchain configuration: an 8-bit format, mailbox pacing where offered, opaque alpha.
fn surface_config(
    adapter: &wgpu::Adapter,
    surface: &wgpu::Surface<'static>,
    width: u32,
    height: u32,
) -> Result<wgpu::SurfaceConfiguration, RenderError> {
    let capabilities = surface.get_capabilities(adapter);
    let format = SURFACE_FORMATS
        .into_iter()
        .find(|f| capabilities.formats.contains(f))
        .ok_or_else(|| RenderError::Surface("no 8-bit unorm surface format".into()))?;
    let present_mode = if capabilities
        .present_modes
        .contains(&wgpu::PresentMode::Mailbox)
    {
        wgpu::PresentMode::Mailbox
    } else {
        wgpu::PresentMode::Fifo
    };
    let alpha_mode = if capabilities
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::Opaque)
    {
        wgpu::CompositeAlphaMode::Opaque
    } else {
        wgpu::CompositeAlphaMode::Auto
    };
    Ok(wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width,
        height,
        present_mode,
        desired_maximum_frame_latency: 2,
        alpha_mode,
        view_formats: vec![],
    })
}

/// A surface of `target` on `instance`, which `adapter` must be able to present to.
pub(super) fn make_surface<T>(
    instance: &wgpu::Instance,
    adapter: &wgpu::Adapter,
    target: T,
) -> Result<GpuSurface, RenderError>
where
    T: raw_window_handle::HasWindowHandle
        + raw_window_handle::HasDisplayHandle
        + Send
        + Sync
        + 'static,
{
    let surface = instance
        .create_surface(target)
        .map_err(|e| RenderError::Surface(e.to_string()))?;
    if !adapter.is_surface_supported(&surface) {
        return Err(RenderError::Surface(
            "the GPU adapter cannot present to this window".into(),
        ));
    }
    Ok(GpuSurface::new(surface))
}
