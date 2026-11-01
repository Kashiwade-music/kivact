//! Role: Fixed-resolution images drawn with the same `Draw` vocabulary as canvases.
//!
//! The path from a `Scene` through the engine's `paint::Context` to an
//! Arvane scene is the window's. The context keeps the resources, the
//! raster scale and the first failure; text is measured with a
//! `bridge::TextMeasurer` that needs no node tree. Only the CPU
//! rasterizer draws, and its premultiplied output is turned back into
//! straight alpha. The theme's tokens, family and line height come from
//! `Offscreen::theme`, as a window root's do.
//!
//! Invariants:
//! - A window skips a resource it cannot use and goes on; an `Offscreen`
//!   returns the first failure of an image or a text it actually uses as a
//!   `UiError` of kind `Asset` located at the call (a font that fails to
//!   register too; there is no error type of its own).
//! - Commands `Draw` drops (pixels whose size disagrees with their buffer,
//!   a box-only shadow value) and those the context counts (non-finite or
//!   negative coordinates, sizes and radii, non-finite arcs, rejected
//!   gradient stops or geometry) are `Unsupported`. They are dropped the
//!   way a canvas drops them, invalid geometry requests no resource, and
//!   nothing is dropped silently (a canvas gradient by coordinates that is
//!   rejected paints nothing and is not counted). A window sends what it
//!   dropped once per layer drawing (`HostEvent::DrawDropped { node,
//!   count }`), which the
//!   client reports to the default sink as `Unsupported`; a layer that
//!   drops again after a clean drawing is reported again.
//! - The one source of the size limits is Arvane's
//!   `validate::dimensions`/`Rasterizer::check_size`. A size of 0 or over
//!   the limit is not shrunk: it is a `Message` error (the kind of an
//!   invalid `window_size`) and the drawing closure is not called.
//! - Assets load from an inline store and complete at once. Codecs, the
//!   artwork's time and cancelling between frames are the caller's.
use super::{Draw, Size};
use crate::{Result, UiError};
use kivact_reactive::ErrorKind;
use kivact_style::Theme;
use kivact_style::asset::Asset;
use kivact_style::draw::Recording;
use std::panic::Location;

/// A tightly packed image, top row first, in sRGB with straight
/// (unassociated) alpha: exactly `width * height * 4` bytes in RGBA order,
/// without row padding. It owns its allocation, independent of the
/// [`Offscreen`] that drew it, and can move to another thread.
/// Produced by the framework; read its fields.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Draws into fixed-resolution RGBA images without an `App` or a window
/// (for exporting artwork or a frame sequence). The `paint` closure of
/// [`Offscreen::render`] is the function a `canvas` takes, so one drawing
/// function serves both.
///
/// It paints with the CPU renderer that also draws a window on the CPU and
/// the same scene lowering as a canvas; a GPU window can differ slightly in
/// anti-aliasing. Coordinates are output pixels at scale 1. The image
/// starts transparent: paint a background to make it opaque. Regions are
/// ignored, and geometry a canvas repairs or drops is repaired or dropped
/// the same way. Commands `Draw` rejects, a resource the drawing needs and
/// cannot get, and rasterization failures are errors (a [`UiError`]
/// located at the call; see [`Offscreen::render`]). A failure leaves the
/// `Offscreen` usable for the next call.
///
/// Images, SVGs and fonts load inside the call. Relative paths resolve
/// against the current directory. Keep one `Asset` and one `Offscreen`
/// across the frames of a sequence to reuse the bounded asset and shaping
/// caches. Fonts default to the system's with its fallback; register the
/// files the preview uses ([`Offscreen::font`]) and give the same
/// [`Theme`] ([`Offscreen::theme`]) for the same text.
///
/// It stays on the thread that made it (`!Send`, `!Sync`); to export in the
/// background, build it inside the worker (for example `kivact::task::blocking`)
/// and check its `CancelToken` between frames. One frame is synchronous and
/// cannot be interrupted. Dropping it releases its caches; images already
/// returned stay valid.
///
/// ```no_run
/// # use kivact_core as kivact;
/// use kivact::draw::{Offscreen, Rect};
/// use kivact::style::rgb;
///
/// let mut renderer = Offscreen::new();
/// let image = renderer.render(640, 360, |draw, _size| {
///     draw.fill_rect(Rect::new(10.0, 10.0, 80.0, 40.0), rgb(0xff3300));
/// })?;
/// assert_eq!(image.rgba.len(), 640 * 360 * 4);
/// # Ok::<(), kivact::UiError>(())
/// ```
///
/// The renderer cannot move to another thread:
///
/// ```compile_fail
/// # use kivact_core as kivact;
/// fn requires_send<T: Send>() {}
/// requires_send::<kivact::draw::Offscreen>();
/// ```
pub struct Offscreen {
    engine: kivact_engine::bridge::Offscreen,
    theme: Theme,
}

impl Default for Offscreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Offscreen {
    pub fn new() -> Self {
        Self {
            engine: kivact_engine::bridge::Offscreen::new(),
            theme: Theme::default(),
        }
    }

    /// Resolve color tokens with `theme`, which also supplies the text
    /// family and line height that text without its own takes (what a
    /// window's root carries): pass the preview's theme.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Load and register a TTF, OTF or TTC file now, returning the family
    /// names it adds. A new font clears the shaping caches; images already
    /// returned are unaffected. A file that is not a usable font is an
    /// `ErrorKind::Asset` error.
    #[track_caller]
    pub fn font(&mut self, font: impl Into<Asset>) -> Result<Vec<String>> {
        let at = Location::caller();
        self.engine
            .font(&font.into())
            .map_err(|message| failed(ErrorKind::Asset, message, at))
    }

    /// Draw `paint` into an image of exactly `width` × `height` pixels;
    /// never shrunk to fit a budget. `paint` runs once, with the `Draw` and
    /// its [`Size`], and not at all when the size is invalid.
    ///
    /// The errors, located at the call:
    ///
    /// - `ErrorKind::Message`: the size is zero or beyond what windows draw
    ///   (each side and the area are limited by the renderer).
    /// - `ErrorKind::Unsupported`: the drawing holds a command that cannot
    ///   be drawn: one `Draw` rejects (raw `Pixels` whose length does not
    ///   match their size, a canvas shadow with spread or inset) or one
    ///   whose values are invalid (a non-finite or negative coordinate,
    ///   size, radius or arc, a gradient stop that is not finite, in `0..=1`
    ///   and in order; a rejected canvas gradient by coordinates paints
    ///   nothing and is not an error). A canvas reports the same commands
    ///   as an `Unsupported` error and draws the rest.
    /// - `ErrorKind::Asset`: a resource used by the drawing (an image, an
    ///   SVG), the shaping of its text or the rasterization failed.
    #[track_caller]
    pub fn render(
        &mut self,
        width: u32,
        height: u32,
        paint: impl FnOnce(&mut Draw, Size),
    ) -> Result<RgbaImage> {
        let at = Location::caller();
        if !kivact_engine::bridge::Offscreen::accepts_size(width, height) {
            return Err(failed(
                ErrorKind::Message,
                format!("invalid or excessive image size {width}x{height}"),
                at,
            ));
        }
        let size = Size::new(width as f32, height as f32);
        let inherited = kivact_wire::ComputedText {
            family: self.theme.text().family.clone(),
            line_height: Some(self.theme.text().line_height),
            ..Default::default()
        };
        let measurer = self.engine.text_measurer(inherited.clone());
        let Recording {
            scene,
            unsupported: invalid,
            ..
        } = Draw::record(size, &self.theme, |draw| {
            draw.set_text_measurer(measurer);
            paint(draw, size);
        });
        let rejected = || {
            failed(
                ErrorKind::Unsupported,
                "the drawing holds a command Draw rejects",
                at,
            )
        };
        if invalid != 0 {
            return Err(rejected());
        }
        let image = self
            .engine
            .render(&std::sync::Arc::new(scene), (width, height), &inherited)
            .map_err(|error| match error {
                kivact_engine::bridge::OffscreenError::Dropped(_) => rejected(),
                kivact_engine::bridge::OffscreenError::Failed(message) => {
                    failed(ErrorKind::Asset, message, at)
                }
            })?;
        Ok(RgbaImage {
            width: image.width,
            height: image.height,
            rgba: image.rgba,
        })
    }
}

fn failed(kind: ErrorKind, message: impl Into<String>, at: &'static Location<'static>) -> UiError {
    UiError::new(kind, message, Some(at))
}
