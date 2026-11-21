//! Role: One window's part of the GPU renderer: its context and swapchain on the driver's device.
//!
//! A window whose driver has a ready device makes its context and its
//! surface (on the window's thread), and a worker configures the swapchain
//! ([`WindowGpu::advance`]); frames go to the GPU once it is configured. A
//! surface the platform destroys is dropped with it and made again by the
//! window's next frame. A device the window no longer has (failed, or
//! restarted) is dropped with its context, and the window's export pools
//! are reported lost ([`WindowGpu::lose`]). Failures are classified by the
//! driver ([`crate::gpu_driver`]).

use crate::gpu_driver::{Polled, ReadyDevice, Wake, Worker};
use arvane_compose::{GpuContext, GpuDevice, GpuSurface, Presentation};
use arvane_protocol::{Composition, CpuFrame, ExportLoss, FrameReport};
use arvane_render::RenderError;
use arvane_shell::ShellWindow;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Makes the window's surface on the window's thread.
pub(crate) type MakeSurface<'a> =
    &'a dyn Fn(&GpuContext, &mut dyn ShellWindow) -> Result<GpuSurface, RenderError>;

/// The window's configured surface, and how long configuring took.
type Configured = Result<(GpuSurface, Duration), RenderError>;

pub(crate) enum DeviceSurface {
    Pending,
    Live(Box<GpuSurface>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigureTarget {
    Current,
    Destroyed,
}

/// One window's GPU renderer on the driver's device.
#[derive(Default)]
pub(crate) enum WindowGpu {
    /// No device: the window draws on the CPU.
    #[default]
    Off,
    /// A worker configures the window's swapchain; the context waits here.
    Configuring {
        /// Boxed: a context is large.
        context: Box<GpuContext>,
        worker: Worker<Configured>,
        surface: ConfigureTarget,
    },
    /// The window draws on the device; `surface` is made by the next frame
    /// when `Pending`.
    Ready {
        /// Boxed: a context is large.
        context: Box<GpuContext>,
        surface: DeviceSurface,
    },
}

impl WindowGpu {
    /// Whether frames go to the GPU now.
    pub(crate) fn ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    /// Whether a worker is configuring the window's swapchain.
    pub(crate) fn configuring(&self) -> bool {
        matches!(self, Self::Configuring { .. })
    }

    /// The context of a ready window.
    pub(crate) fn context_mut(&mut self) -> Option<&mut GpuContext> {
        match self {
            Self::Ready { context, .. } => Some(&mut **context),
            Self::Off | Self::Configuring { .. } => None,
        }
    }

    /// Whether the window's context is not on `device` (none, or another
    /// one): the window must lose it.
    pub(crate) fn stale(&self, device: Option<&Rc<GpuDevice>>) -> bool {
        match self {
            Self::Off => false,
            Self::Configuring { context, .. } | Self::Ready { context, .. } => {
                !device.is_some_and(|device| Rc::ptr_eq(device, context.device()))
            }
        }
    }

    /// Drop the window's context and surface; returns the losses of its
    /// export pools.
    pub(crate) fn lose(&mut self) -> Vec<ExportLoss> {
        match std::mem::take(self) {
            Self::Off => Vec::new(),
            Self::Configuring { context, .. } | Self::Ready { context, .. } => {
                context.export_losses()
            }
        }
    }

    /// The window's surface was destroyed by the platform: it is made again
    /// by the next frame (a swapchain being configured is dropped when its
    /// worker hands it back).
    pub(crate) fn drop_surface(&mut self) {
        match self {
            Self::Ready { surface, .. } => *surface = DeviceSurface::Pending,
            Self::Configuring { surface, .. } => *surface = ConfigureTarget::Destroyed,
            Self::Off => {}
        }
    }

    /// Release rebuildable validation metadata under memory pressure.
    pub(crate) fn release_caches(&mut self) {
        match self {
            Self::Ready { context, .. } | Self::Configuring { context, .. } => {
                context.release_caches();
            }
            Self::Off => {}
        }
    }

    /// Stop the configuring worker that needs the native window, before the
    /// window is dropped.
    pub(crate) fn shutdown(&mut self, window: &mut dyn ShellWindow) {
        if let Self::Configuring { worker, .. } = std::mem::take(self) {
            worker.shutdown(window);
        }
    }

    /// Move the window's GPU start along on `device` (none: nothing starts):
    /// once the driver has a device the window has no context for, make
    /// its context and surface and hand the surface to a worker to
    /// configure. Returns how long configuring took when the window became
    /// ready now. A failed surface or configuring is returned as the error,
    /// after the window dropped what failed.
    pub(crate) fn advance(
        &mut self,
        device: Option<ReadyDevice>,
        window: &mut dyn ShellWindow,
        make: MakeSurface<'_>,
        physical: (u32, u32),
    ) -> Result<Option<Duration>, RenderError> {
        match std::mem::take(self) {
            Self::Off => {
                let Some(ReadyDevice { gpu, wake }) = device else {
                    return Ok(None);
                };
                let context = Box::new(GpuContext::new(gpu));
                let surface = match make(&context, window) {
                    Ok(surface) => surface,
                    Err(error) => {
                        *self = Self::Ready {
                            context,
                            surface: DeviceSurface::Pending,
                        };
                        return Err(error);
                    }
                };
                self.configure(wake, context, surface, physical)?;
                Ok(None)
            }
            Self::Configuring {
                context,
                worker,
                surface,
            } => match worker.poll() {
                Polled::Working(worker) => {
                    *self = Self::Configuring {
                        context,
                        worker,
                        surface,
                    };
                    Ok(None)
                }
                // The platform destroyed the surface meanwhile: the next frame makes a new one.
                Polled::Sent(_) if surface == ConfigureTarget::Destroyed => {
                    *self = Self::Ready {
                        context,
                        surface: DeviceSurface::Pending,
                    };
                    Ok(None)
                }
                Polled::Sent(Ok((surface, configure))) => {
                    *self = Self::Ready {
                        context,
                        surface: DeviceSurface::Live(Box::new(surface)),
                    };
                    Ok(Some(configure))
                }
                Polled::Sent(Err(error)) => {
                    *self = Self::Ready {
                        context,
                        surface: DeviceSurface::Pending,
                    };
                    Err(error)
                }
                Polled::Stopped => Err(RenderError::Backend(
                    "GPU surface configuration stopped".into(),
                )),
            },
            ready @ Self::Ready { .. } => {
                *self = ready;
                Ok(None)
            }
        }
    }

    /// Hand `surface` to a worker that configures it for `physical`; the
    /// window is configuring until the worker hands it back.
    fn configure(
        &mut self,
        wake: Wake,
        context: Box<GpuContext>,
        mut surface: GpuSurface,
        (width, height): (u32, u32),
    ) -> Result<(), RenderError> {
        let configurer = context.device().configurer();
        let worker = Worker::spawn("arvane-gpu-configure", wake, move || {
            let since = Instant::now();
            let configured = configurer.configure(&mut surface, width, height);
            configured.map(|()| (surface, since.elapsed()))
        })
        .ok_or_else(|| RenderError::Backend("the GPU worker thread cannot start".into()))?;
        *self = Self::Configuring {
            context,
            worker,
            surface: ConfigureTarget::Current,
        };
        Ok(())
    }

    /// Present to the window on the GPU, its surface made by `make` when it
    /// has none; `None` when the window is not ready (draw on the CPU).
    pub(crate) fn present(
        &mut self,
        window: &mut dyn ShellWindow,
        make: MakeSurface<'_>,
        composition: &Composition,
        scale: f64,
        physical: (u32, u32),
    ) -> Option<Result<(Presentation, Duration, FrameReport), RenderError>> {
        let Self::Ready { context, surface } = self else {
            return None;
        };
        let surface = match window_surface(context, surface, window, make) {
            Ok(surface) => surface,
            Err(error) => return Some(Err(error)),
        };
        Some(context.present(surface, composition, scale, physical))
    }

    /// Present `pixels` (0x00RRGGBB rows of the window's size, drawn on the
    /// CPU) through the window's GPU surface; `None` when the window is not
    /// ready.
    pub(crate) fn present_pixels(
        &mut self,
        window: &mut dyn ShellWindow,
        make: MakeSurface<'_>,
        pixels: &[u32],
        physical: (u32, u32),
    ) -> Option<Result<(Presentation, Duration), RenderError>> {
        let Self::Ready { context, surface } = self else {
            return None;
        };
        let surface = match window_surface(context, surface, window, make) {
            Ok(surface) => surface,
            Err(error) => return Some(Err(error)),
        };
        Some(context.present_pixels(surface, pixels, physical))
    }

    /// Read back the last frame presented to the window.
    pub(crate) fn read_back(&self) -> Option<Result<CpuFrame, RenderError>> {
        let Self::Ready { context, surface } = self else {
            return None;
        };
        match surface {
            DeviceSurface::Pending => None,
            DeviceSurface::Live(surface) => Some(context.read_back(surface)),
        }
    }
}

/// The window's surface, made by `make` when there is none.
fn window_surface<'a>(
    context: &GpuContext,
    surface: &'a mut DeviceSurface,
    window: &mut dyn ShellWindow,
    make: MakeSurface<'_>,
) -> Result<&'a mut GpuSurface, RenderError> {
    if matches!(surface, DeviceSurface::Pending) {
        *surface = DeviceSurface::Live(Box::new(make(context, window)?));
    }
    match surface {
        DeviceSurface::Live(surface) => Ok(surface),
        DeviceSurface::Pending => unreachable!("invariant: the surface was just made"),
    }
}
