//! Role: Defines renderer choices and presentation change notifications.

use std::{fmt, time::Duration};
use ui_types::{ExternalFrame, GpuAdapter};

/// The rasterizer behind a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Renderer {
    /// vello_cpu, into the window's CPU surface.
    Cpu,
    /// vello_hybrid on the GPU, into the window's GPU surface.
    Gpu,
}

/// A change of the renderer that draws a window's frames, or a note of
/// what it could not draw.
///
/// Its text (`Display`) is one line starting with `renderer=`.
#[derive(Clone, Debug, PartialEq)]
pub enum RendererChange {
    /// The renderer a native window starts with: the CPU, and whether the
    /// GPU is starting (`renderer=cpu gpu=starting`; else `renderer=cpu`).
    Start {
        /// The GPU is starting.
        gpu: bool,
    },
    /// Frames are drawn on the GPU from now on (`renderer=gpu adapter=…
    /// init_ms=… configure_ms=…`).
    Gpu {
        /// The adapter and backend ("NVIDIA ... (Vulkan)").
        adapter: String,
        /// Creating the device.
        init: Duration,
        /// Configuring the window's first swapchain, when it was configured
        /// in the background.
        configure: Option<Duration>,
        /// The adapter, when the device can import textures an external
        /// render engine shares (feature `gpu-interop`): a new
        /// [`GpuAdapter::generation`] means the images imported before are
        /// gone. The renderer changing to the CPU means none can be.
        interop: Option<GpuAdapter>,
    },
    /// Frames are drawn on the CPU from now on, or until the GPU restarts
    /// (`renderer=cpu <reason>`).
    Cpu {
        /// Why.
        reason: String,
    },
    /// An external frame a scene named was not drawn: it was not ready
    /// within the frame's wait (100 ms), or the window had already given
    /// it back to its engine, or the pool has no such texture
    /// (`renderer=gpu external_frame_missed image=… slot=… value=…`).
    /// Noted once per frame value.
    ExternalFrameMissed {
        /// The frame.
        frame: ExternalFrame,
    },
}

impl fmt::Display for RendererChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ms = |d: &Duration| d.as_secs_f64() * 1000.0;
        match self {
            Self::Start { gpu: true } => f.write_str("renderer=cpu gpu=starting"),
            Self::Start { gpu: false } => f.write_str("renderer=cpu"),
            Self::Gpu {
                adapter,
                init,
                configure,
                ..
            } => {
                write!(f, "renderer=gpu adapter={adapter} init_ms={:.1}", ms(init))?;
                if let Some(configure) = configure {
                    write!(f, " configure_ms={:.1}", ms(configure))?;
                }
                Ok(())
            }
            Self::Cpu { reason } => write!(f, "renderer=cpu {reason}"),
            Self::ExternalFrameMissed { frame } => write!(
                f,
                "renderer=gpu external_frame_missed image={} slot={} value={}",
                frame.image.0, frame.slot, frame.value
            ),
        }
    }
}

/// Observes a window's renderer changes.
pub(crate) type Observer = Box<dyn FnMut(&RendererChange)>;

/// Where a presenter notes its renderer changes.
#[derive(Default)]
pub(crate) struct Notes(Option<Observer>);

impl Notes {
    pub(crate) fn new(observer: Option<Observer>) -> Self {
        Self(observer)
    }

    pub(crate) fn note(&mut self, change: RendererChange) {
        if let Some(observer) = &mut self.0 {
            observer(&change);
        }
    }
}
