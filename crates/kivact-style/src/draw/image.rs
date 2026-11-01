//! Role: Image values for the painter: pixel buffers and drawing options.
use crate::geometry::Rect;
use std::sync::Arc;

/// Straight-alpha RGBA8 pixels. Keep the same Arc for unchanged pixels;
/// a new frame uses a new Arc. Renderer conversions expire after unused frames.
///
/// Two `Pixels` are equal when their sizes match and they share the `Arc`
/// (the contents are never compared), so a new `Arc` every frame is cheap.
/// They bypass decoding and the asset store, and Arvane's conversion and GPU
/// upload caches drop them after a frame that does not use them. A buffer
/// whose length disagrees with its size draws nothing and is reported as
/// `Unsupported`.
#[derive(Clone, Debug)]
pub struct Pixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}
impl PartialEq for Pixels {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && Arc::ptr_eq(&self.rgba, &other.rgba)
    }
}
impl Pixels {
    pub fn new(width: u32, height: u32, rgba: Arc<[u8]>) -> Self {
        Self {
            width,
            height,
            rgba,
        }
    }
}
/// Image sampling and opacity, applied to the whole drawn image
/// (`Draw::image_with`, `Draw::pixels`).
///
/// A crop does not copy pixels: it is drawn with a clip and a paint
/// transform. An SVG is rasterized at the scale of the `Draw`'s transforms
/// times the ratio of the source rectangle to the destination (within the
/// asset cache's limits).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageOptions {
    /// The part of the image to draw, in the image's own pixels (`None`:
    /// the whole image).
    pub source: Option<Rect>,
    /// Opacity of the whole image, 0 to 1.
    pub opacity: f32,
    /// Nearest-neighbour sampling, on the CPU and the GPU alike.
    pub pixelated: bool,
}
impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            source: None,
            opacity: 1.0,
            pixelated: false,
        }
    }
}

kivact_reactive::constant_value!(Pixels);

/// One frame of an image an external render engine shares with the window
/// (feature `gpu-interop`), made by `kivact::gpu_interop::ExternalImage::frame`
/// and drawn with `Draw::gpu_frame`. A plain value: keep it in a signal
/// the canvas reads, and a new frame redraws the canvas.
///
/// It knows the size of the textures it was imported with
/// ([`GpuFrame::width`], [`GpuFrame::height`]): the pixels of the frame,
/// which `ImageOptions::source` of `Draw::gpu_frame` counts in, as
/// `Pixels::width` and `Pixels::height` are for `Draw::pixels`.
///
/// Its RGBA8 values are composited as sRGB-encoded values (premultiplied
/// alpha), as `Pixels` and CSS colors are: an engine that computes in
/// linear light encodes to sRGB before it hands the frame over.
#[cfg(feature = "gpu-interop")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpuFrame {
    pub(crate) frame: kivact_wire::ExternalFrame,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

#[cfg(feature = "gpu-interop")]
impl GpuFrame {
    /// The frame `frame` of the shared textures of `width` by `height`
    /// pixels (what `ExternalImage::frame` and its publisher make).
    pub fn new(frame: kivact_wire::ExternalFrame, width: u32, height: u32) -> Self {
        Self {
            frame,
            width,
            height,
        }
    }

    /// The width in pixels of the textures this frame belongs to (the
    /// `width` of the `SharedTextures` that were imported).
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height in pixels of the textures this frame belongs to (the
    /// `height` of the `SharedTextures` that were imported).
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[cfg(feature = "gpu-interop")]
kivact_reactive::constant_value!(GpuFrame);
