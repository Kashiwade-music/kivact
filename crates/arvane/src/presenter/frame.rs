//! Role: A frame's raster plan, and what presenting it did.

use crate::error::{Failure, FailureKind};
use crate::presentation::Renderer;
use kivact_types::Size;
use kivact_types::ids::DrawableId;
use kivact_types::render::{ExportedFrame, FrameStats};
use std::time::Duration;

/// A frame's logical extent and the physical scale it is rasterized at:
/// the window's display scale, reduced for a window beyond the renderers'
/// frame budget (each side 16,384 px, 64 MiB), whose frame is then
/// enlarged to the window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FramePlan {
    /// Logical extent; it never rasterizes beyond the window.
    pub size: Size,
    /// Physical pixels per logical pixel the frame is drawn at.
    pub scale: f64,
}

impl FramePlan {
    /// The plan for a window of `physical` pixels at display `scale`.
    pub fn new((width, height): (u32, u32), scale: f64) -> Self {
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
        /// The drawable frames this presentation exported (§10.6).
        exports: Vec<ExportedFrame>,
        /// The drawables whose export waits for a free slot; their frames
        /// come on a later frame (§8.12).
        pending_exports: Vec<DrawableId>,
        /// The work counts of the frame (§8.15).
        stats: FrameStats,
    },
    /// Presentation is temporarily unavailable (no buffer, occlusion, or
    /// GPU surface configuration): nothing new was shown. Draw again at
    /// `retry_at_ns` in [`crate::Window::now_ns`]'s clock, or as soon as the
    /// window is focused again. Consecutive skips back off from one display
    /// period to sixteen; a shown frame or a lost surface resets that backoff.
    Skipped {
        /// When to draw the frame again.
        retry_at_ns: u64,
    },
    /// The scene or its size cannot be drawn: nothing was shown and the
    /// window keeps its picture until the next change. So it is, on every
    /// renderer, for a window of no pixels (minimized): its resize asks
    /// for the next frame.
    Dropped(Failure),
    /// The window's surface failed and was made again: nothing new was
    /// shown. Request another frame immediately; Arvane has already
    /// performed the recovery. A shown or skipped frame resets the failure count.
    SurfaceLost(Failure),
}

/// What presenting to a window without a surface does: nothing is drawn
/// (the host draws again when the window has a surface again).
pub(crate) fn no_surface() -> Presented {
    Presented::Dropped(Failure::new(
        FailureKind::Unavailable,
        "the window has no surface (the platform destroyed it)",
    ))
}

/// What presenting to a window of no pixels does (minimized, or sized to
/// nothing; `physical` is its size): nothing is drawn, on any renderer,
/// and the resize that gives it pixels again asks for a frame. `None` for
/// a window with pixels.
pub(crate) fn no_pixels((width, height): (u32, u32)) -> Option<Presented> {
    (width == 0 || height == 0).then(|| {
        Presented::Dropped(Failure::new(
            FailureKind::Content,
            format!("the window has no pixels to draw ({width} x {height})"),
        ))
    })
}
