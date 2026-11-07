//! Role: Presenting to a window through a wgpu surface (a swapchain), with no readback.
//!
//! The renderer draws straight into the surface's texture when it can
//! (the surface takes the renderer's format and the frame is the window's
//! size). Otherwise (another surface format, a scale reduced to the frame
//! budget, or a surface that keeps its frames for tests) it draws into a
//! texture that a render pass copies to the surface's texture
//! (`TextureBlitter`), enlarging a reduced frame. The scene is drawn over
//! white, as the CPU frames are composited.
//!
//! Resizing: the swapchain of a resized window is made again before the
//! frame is recorded, on the window's thread. On Vulkan alone it is made
//! on a scoped thread while the frame is recorded (the driver's work, ~3.5
//! ms on Windows' NVIDIA Vulkan driver): DirectX's swapchain calls may send
//! messages to the window's thread, which would deadlock with a join that
//! waits for them.
//!
//! Failure policy: an outdated or lost surface is reconfigured and the
//! frame retried once; a timeout or an occluded window skips the frame
//! ([`Presentation::Skipped`]); anything else is an error for the facade
//! (a surface error drops the surface, a device failure the device). After
//! a skipped frame (either kind of frame) the next one takes the surface's
//! buffer before recording the scene, so a window that stays occluded
//! costs no recording.
//!
//! Frame pacing: mailbox where the surface offers it (each frame replaces
//! the one waiting for the next vblank, without tearing), else FIFO
//! (vsync, at most two frames queued). The host draws at most one frame
//! per display period (its frame tick), so a mailbox swapchain shows every
//! frame; it never blocks the window's thread waiting for a vblank, and on
//! Windows' Vulkan drivers it is configured in milliseconds rather than
//! ~0.2 s and keeps ~80 MB less memory than FIFO. Without mailbox (FIFO)
//! a presentation waits for the vsync.
//!
//! Limits: resizing a window makes a new swapchain every frame (the
//! driver's work), and that is the floor. A larger, rounded swapchain
//! drawn into through a viewport of the window's size cannot be used:
//! Windows' Vulkan surfaces require `minImageExtent` = `maxImageExtent` =
//! the window's size (another size breaks the specification, and some
//! drivers stretch it), and wgpu's DirectX 12 swapchains are created with
//! `DXGI_SCALING_STRETCH`, stretched to the window; either way the content
//! would be wrong while the window is dragged.

use super::external::ExternalPass;
use super::state::{
    Configured, FORMAT, GpuContext, GpuSurface, LastConfigure, Presentation, Retain, Swapchain,
};
use super::texture::target_texture;
use crate::error::RenderError;
use kivact_types::Size;
use kivact_types::render::{CpuFrame, RenderScene};
use std::time::{Duration, Instant};

impl GpuSurface {
    pub(super) fn new(surface: wgpu::Surface<'static>) -> Self {
        Self {
            surface,
            swapchain: Swapchain::Unconfigured,
            raster: None,
            keep: Retain::Drop,
        }
    }

    /// The configured swapchain (a frame configures it before it records or
    /// acquires).
    fn configured(&self) -> &Configured {
        match &self.swapchain {
            Swapchain::Configured(configured) => configured,
            Swapchain::Unconfigured => unreachable!(
                "invariant: a frame configures the swapchain before it records or acquires"
            ),
        }
    }

    fn configured_mut(&mut self) -> &mut Configured {
        match &mut self.swapchain {
            Swapchain::Configured(configured) => configured,
            Swapchain::Unconfigured => unreachable!(
                "invariant: a frame configures the swapchain before it records or acquires"
            ),
        }
    }

    /// Keep each presented frame for [`GpuContext::read_back`] (tests): the
    /// frames are drawn into a texture and copied to the window.
    pub fn keep_frames(&mut self) {
        self.keep = Retain::Keep;
    }
}

impl GpuContext {
    /// Render `scene` and present it to `surface`, a `width` × `height`
    /// physical-pixel window. Also returns the time spent waiting for the
    /// surface's buffer and presenting.
    pub fn present(
        &mut self,
        surface: &mut GpuSurface,
        scene: &RenderScene,
        size: Size,
        scale: f64,
        physical: (u32, u32),
    ) -> Result<(Presentation, Duration), RenderError> {
        let mut external = std::mem::take(&mut self.external);
        let drawn = (|| {
            let pass = ExternalPass::begin(&mut external, scene)?;
            self.present_frame(surface, scene, size, scale, physical, pass)
        })();
        self.external = external;
        drawn
    }

    /// [`GpuContext::present`] between the external frames' hand-offs.
    fn present_frame(
        &mut self,
        surface: &mut GpuSurface,
        scene: &RenderScene,
        size: Size,
        scale: f64,
        (width, height): (u32, u32),
        pass: ExternalPass<'_>,
    ) -> Result<(Presentation, Duration), RenderError> {
        let (raster_size, info) = self.dimensions(scene, size, scale)?;
        if width == 0 || height == 0 {
            return Ok((Presentation::Skipped, Duration::ZERO));
        }
        // On Vulkan, a new swapchain for a resized window is made while
        // the frame is recorded (below). Elsewhere it is made here first:
        // DirectX's swapchain calls may send messages to the window's
        // thread, which would wait for a join that waits for them.
        let alongside = self.adapter.get_info().backend == wgpu::Backend::Vulkan
            && matches!(
                &surface.swapchain,
                Swapchain::Configured(c) if (c.config.width, c.config.height) != (width, height)
            );
        if !alongside {
            self.reconfigure(surface, width, height, false)?;
        }
        let format = surface.configured().config.format;
        let direct =
            surface.keep == Retain::Drop && format == FORMAT && raster_size == (width, height);
        prepare_raster(&self.device, surface, direct, raster_size);
        // After a skipped frame, take the buffer first: no recording while
        // the window stays occluded.
        let early = if !alongside && surface.configured().last == LastConfigure::Skipped {
            let waiting = Instant::now();
            let acquired = self.acquire(surface, width, height)?;
            let waited = waiting.elapsed();
            match acquired {
                Some(acquired) => Some((acquired, waited)),
                None => return Ok((Presentation::Skipped, waited)),
            }
        } else {
            None
        };
        // White under the scene, as the CPU frames are composited.
        if alongside {
            // Making the new swapchain is the driver's work (~3.5 ms on
            // Windows' NVIDIA Vulkan driver): a scoped thread does it while
            // this one records the frame. Vulkan's swapchain calls do not
            // message the window's thread.
            self.encode_alongside(
                scene,
                scale,
                raster_size,
                &info,
                pass.frame(),
                |device, adapter| configure(device, adapter, surface, width, height, false),
            )?;
            self.check()?;
        } else {
            self.encode(scene, scale, raster_size, &info, true, pass.frame());
        }
        let ((texture, suboptimal), mut waited) = match early {
            Some(early) => early,
            None => {
                let waiting = Instant::now();
                let acquired = self.acquire(surface, width, height)?;
                let waited = waiting.elapsed();
                let Some(acquired) = acquired else {
                    return Ok((Presentation::Skipped, waited));
                };
                (acquired, waited)
            }
        };
        let encoder = self.draw_presentation(surface, &texture.texture, raster_size, direct)?;
        self.queue.submit([encoder.finish()]);
        let submitted = pass.submit();
        let presenting = Instant::now();
        texture.present();
        waited += presenting.elapsed();
        if suboptimal {
            self.reconfigure(surface, width, height, true)?;
        }
        self.check()?;
        submitted.finish()?;
        Ok((Presentation::Shown, waited))
    }

    /// Draw directly or copy the retained raster into the acquired texture.
    fn draw_presentation(
        &mut self,
        surface: &GpuSurface,
        texture: &wgpu::Texture,
        raster_size: (u32, u32),
        direct: bool,
    ) -> Result<wgpu::CommandEncoder, RenderError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Arvane present"),
            });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        if direct {
            self.draw(&mut encoder, raster_size, &view)?;
        } else {
            let raster = surface
                .raster
                .as_ref()
                .expect("invariant: the raster texture was just made")
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.draw(&mut encoder, raster_size, &raster)?;
            surface
                .configured()
                .blitter
                .copy(&self.device, &mut encoder, &raster, &view);
        }
        Ok(encoder)
    }

    /// Present a frame drawn elsewhere to `surface`, a `width` × `height`
    /// physical-pixel window: `pixels` are its rows of 0x00RRGGBB (the CPU
    /// renderer's), uploaded and copied to the surface's texture (a window
    /// with a swapchain shows nothing presented any other way). Also
    /// returns the time spent waiting for the surface's buffer and
    /// presenting.
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
        if surface
            .raster
            .as_ref()
            .is_none_or(|t| (t.width(), t.height()) != (width, height))
        {
            surface.raster = Some(target_texture(&self.device, width, height));
        }
        let raster = surface
            .raster
            .as_ref()
            .expect("invariant: the raster texture was just made");
        // 0x00RRGGBB in little-endian bytes is B, G, R, 0: the renderer's
        // BGRA format once the alpha is opaque.
        let bytes: Vec<u8> = pixels
            .iter()
            .flat_map(|p| (p | 0xff00_0000).to_le_bytes())
            .collect();
        self.queue.write_texture(
            raster.as_image_copy(),
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            raster.size(),
        );
        let waiting = Instant::now();
        let acquired = self.acquire(surface, width, height)?;
        let mut waited = waiting.elapsed();
        let Some((texture, suboptimal)) = acquired else {
            return Ok((Presentation::Skipped, waited));
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Arvane present pixels"),
            });
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let source = surface
            .raster
            .as_ref()
            .expect("invariant: the raster texture was just made")
            .create_view(&wgpu::TextureViewDescriptor::default());
        surface
            .configured()
            .blitter
            .copy(&self.device, &mut encoder, &source, &view);
        self.queue.submit([encoder.finish()]);
        let presenting = Instant::now();
        texture.present();
        waited += presenting.elapsed();
        if suboptimal {
            self.reconfigure(surface, width, height, true)?;
        }
        self.check()?;
        Ok((Presentation::Shown, waited))
    }

    /// Read back the last frame presented to `surface` (composited over
    /// white), which keeps its frames ([`GpuSurface::keep_frames`]).
    pub fn read_back(&self, surface: &GpuSurface) -> Result<CpuFrame, RenderError> {
        let texture = surface
            .raster
            .as_ref()
            .filter(|_| surface.keep == Retain::Keep)
            .ok_or_else(|| RenderError::Surface("no frame was kept".into()))?;
        self.read_texture(texture)
    }

    /// The surface's next texture and whether it is suboptimal; `None`
    /// skips the frame. Notes whether a buffer came ([`Configured::skipped`]).
    fn acquire(
        &self,
        surface: &mut GpuSurface,
        width: u32,
        height: u32,
    ) -> Result<Option<(wgpu::SurfaceTexture, bool)>, RenderError> {
        let acquired = self.next_texture(surface, width, height)?;
        surface.configured_mut().last = if acquired.is_none() {
            LastConfigure::Skipped
        } else {
            LastConfigure::Applied
        };
        Ok(acquired)
    }

    /// The surface's next texture and whether it is suboptimal; `None`
    /// skips the frame. An outdated or lost surface is reconfigured once.
    fn next_texture(
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
                    self.check()?;
                    return Err(RenderError::Backend(
                        "the window surface failed validation".into(),
                    ));
                }
            }
        }
        unreachable!("the second attempt returns")
    }

    /// Configure `surface`'s swapchain for a `width` × `height` window.
    /// Any thread may do it; the first configuration of a window can take
    /// a few hundred milliseconds on some drivers.
    pub fn configure(
        &self,
        surface: &mut GpuSurface,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        self.reconfigure(surface, width.max(1), height.max(1), true)
    }

    /// Configure the swapchain for `width` × `height` when it is not, or
    /// `again`.
    fn reconfigure(
        &self,
        surface: &mut GpuSurface,
        width: u32,
        height: u32,
        again: bool,
    ) -> Result<(), RenderError> {
        configure(&self.device, &self.adapter, surface, width, height, again)?;
        self.check()
    }
}

/// Configure `surface`'s swapchain for `width` × `height` when it is not,
/// or `again`.
fn configure(
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
    surface.swapchain = Swapchain::Configured(Configured {
        blitter: wgpu::util::TextureBlitter::new(device, config.format),
        config,
        last: LastConfigure::Applied,
    });
    Ok(())
}

fn surface_config(
    adapter: &wgpu::Adapter,
    surface: &wgpu::Surface<'static>,
    width: u32,
    height: u32,
) -> Result<wgpu::SurfaceConfiguration, RenderError> {
    let capabilities = surface.get_capabilities(adapter);
    // The renderer's format, else another non-sRGB 8-bit format (the
    // renderer's values are already encoded, the copy must not encode
    // them again).
    let format = [FORMAT, wgpu::TextureFormat::Rgba8Unorm]
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

impl GpuContext {
    /// A surface presenting to the window `target` (any window handle that
    /// keeps its window alive on any thread) with this context's device.
    /// Call it on the thread that runs the window's event loop (windowing
    /// systems hand out window handles there only), or through
    /// [`GpuContext::surface_maker`].
    pub fn surface<T>(&self, target: T) -> Result<GpuSurface, RenderError>
    where
        T: raw_window_handle::HasWindowHandle
            + raw_window_handle::HasDisplayHandle
            + Send
            + Sync
            + 'static,
    {
        make_surface(&self.instance, &self.adapter, target)
    }

    /// [`GpuContext::surface`] for another thread: a window's own thread
    /// makes the surface and hands it back.
    pub fn surface_maker<T>(
        &self,
    ) -> impl FnOnce(T) -> Result<GpuSurface, RenderError> + Send + 'static + use<T>
    where
        T: raw_window_handle::HasWindowHandle
            + raw_window_handle::HasDisplayHandle
            + Send
            + Sync
            + 'static,
    {
        let (instance, adapter) = (self.instance.clone(), self.adapter.clone());
        move |target| make_surface(&instance, &adapter, target)
    }
}

fn make_surface<T>(
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

/// Keep an intermediate raster only when direct swapchain rendering is unavailable.
fn prepare_raster(
    device: &wgpu::Device,
    surface: &mut GpuSurface,
    direct: bool,
    raster_size: (u32, u32),
) {
    if direct {
        // Up to 64 MiB that frames drawn straight into the swapchain
        // do not need; made again when one is drawn for a copy.
        surface.raster = None;
    } else if surface
        .raster
        .as_ref()
        .is_none_or(|t| (t.width(), t.height()) != raster_size)
    {
        surface.raster = Some(target_texture(device, raster_size.0, raster_size.1));
    }
}
