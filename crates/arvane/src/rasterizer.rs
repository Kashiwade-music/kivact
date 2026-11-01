//! Role: CPU image output independent of windows, using the window rasterizer.
use crate::Error;
use kivact_types::Size;
use kivact_types::protocol::{CpuFrame, Scene};

/// Reusable CPU rasterizer for fixed-resolution, transparent image output.
/// Its caches belong to this value and are released when it is dropped.
#[derive(Default)]
pub struct Rasterizer {
    renderer: arvane_render::CpuRenderer,
}

impl Rasterizer {
    /// Release rebuildable validation metadata; returned images remain owned by callers.
    pub fn release_caches(&mut self) {
        self.renderer.release_caches();
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a `width` × `height` image can be rendered: the frame budget
    /// of windows (each side and the area are limited). An image beyond it
    /// is an error, never reduced. Fails with [`ErrorKind::Content`].
    ///
    /// [`ErrorKind::Content`]: crate::ErrorKind::Content
    pub fn check_size(width: u32, height: u32) -> Result<(), Error> {
        arvane_render::dimensions(size(width, height), 1.0)
            .map(|_| ())
            .map_err(crate::error::render)
    }

    /// Render `scene` into exactly `width` × `height` pixels at scale 1,
    /// without compositing over a background. Returns premultiplied sRGB
    /// RGBA8; an invalid size or scene is an error. An error leaves the
    /// rasterizer usable for the next call.
    pub fn render(&mut self, scene: &Scene, width: u32, height: u32) -> Result<CpuFrame, Error> {
        self.renderer
            .render(scene, size(width, height), 1.0)
            .map_err(crate::error::render)
    }
}

/// `u32` sides above 2^24 round when converted, which only makes them larger
/// than the budget's limit of 16,384.
fn size(width: u32, height: u32) -> Size {
    Size {
        width: width as f32,
        height: height as f32,
    }
}
