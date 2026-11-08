//! Role: Defines renderer choices and presentation change notifications.

use kivact_types::WindowId;
use kivact_types::{ExternalFrame, GpuAdapter};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::time::Duration;

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
    /// A user shader's pipeline failed to build; its fallback is drawn from
    /// now on. Noted once per program (`renderer=gpu shader_failed ...`).
    ShaderFailed {
        /// The driver's message.
        message: String,
    },
    /// A frame went beyond the GPU renderer's limits or the compositor's
    /// fixed bounds (`renderer=gpu limit_exceeded what=...`). Noted once per
    /// kind of limit and window.
    LimitExceeded {
        /// Which limit was exceeded.
        what: String,
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
            Self::ShaderFailed { message } => write!(f, "renderer=gpu shader_failed {message}"),
            Self::LimitExceeded { what } => write!(f, "renderer=gpu limit_exceeded what={what}"),
        }
    }
}

/// Observes the renderer changes of a window.
pub type Observer = Rc<RefCell<dyn FnMut(WindowId, &RendererChange)>>;

/// Where a presenter notes its renderer changes.
pub(crate) struct Notes {
    observer: Option<Observer>,
    window: WindowId,
}

impl Notes {
    pub(crate) fn new(observer: Option<Observer>, window: WindowId) -> Self {
        Self { observer, window }
    }

    pub(crate) fn note(&mut self, change: RendererChange) {
        if let Some(observer) = &self.observer {
            observer.borrow_mut()(self.window, &change);
        }
    }
}
