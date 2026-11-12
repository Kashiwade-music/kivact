//! Role: Fixed-resolution images and SVG documents drawn with `Draw`.
//!
//! The path from a `DrawList` through the engine's `paint::Context` to an
//! Arvane scene is the window's. The context keeps the resources, the
//! raster scale and the first failure; text is measured with a
//! `bridge::TextMeasurer` that needs no node tree. Only the CPU
//! rasterizer draws, and its premultiplied output is turned back into
//! straight alpha. The theme's tokens, family and line height come from
//! `Offscreen::theme`, as a window root's do. `render_svg` writes the same
//! recording as an SVG document: `kivact-svg` writes it and asks this module
//! for the text baselines and the drawn images.
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
//! - The SVG document of `render_svg` records the drawing once, with the
//!   same theme, fonts and text measurer as `render`. What SVG 1.1 cannot
//!   express is an `Unsupported` error located at the call, never a silent
//!   omission.
//! - The one source of the size limits is Arvane's
//!   `validate::dimensions`/`Rasterizer::check_size`. A size of 0 or over
//!   the limit is not shrunk: it is a `Message` error (the kind of an
//!   invalid `window_size`) and the drawing closure is not called.
//! - Assets load from an inline store and complete at once. Codecs, the
//!   artwork's time and cancelling between frames are the caller's.

use crate::draw::Draw;
use crate::geometry::Size;
use crate::{Result, UiError};
use kivact_engine::api::{OffscreenError, OffscreenRenderer, TextMeasurer};
use kivact_runtime::ErrorKind;
use kivact_style::Theme;
use kivact_style::asset::Asset;
use kivact_style::draw::Recording;
use kivact_svg::{Resources, SvgError};
use kivact_types::{ComputedTextStyle, DrawCmd, DrawList, Pixels, TextRun};
use std::panic::Location;
use std::sync::Arc;

/// Draws into fixed-resolution RGBA images without an `App` or a window
/// (for exporting artwork or a frame sequence). The `paint` closure of
/// [`Offscreen::render`] is the function a `canvas` takes, so one drawing
/// function serves both; [`Offscreen::render_svg`] writes the same drawing
/// as an SVG document.
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
/// use kivact::draw::{Offscreen};
/// use kivact::geometry::{Rect};
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
    engine: OffscreenRenderer,
    theme: Theme,
}

/// A drawing recorded once, with the text configuration that measured it.
/// `render` and `render_svg` both write from one of these.
struct Recorded {
    scene: DrawList,
    inherited: ComputedTextStyle,
    measurer: TextMeasurer,
}

/// What the SVG writer asks of one `Offscreen`: the text measurer of the
/// recording and the renderer that draws an image for the document.
struct SvgHost<'a> {
    engine: &'a mut OffscreenRenderer,
    measurer: &'a TextMeasurer,
    inherited: &'a ComputedTextStyle,
}

impl Resources for SvgHost<'_> {
    fn fallback_family(&self) -> Option<&str> {
        self.inherited.family.as_deref()
    }

    fn baseline(&self, run: &TextRun) -> f64 {
        self.measurer.measure(run).baseline
    }

    fn rasterize(
        &mut self,
        cmd: DrawCmd,
        width: u32,
        height: u32,
    ) -> std::result::Result<Pixels, SvgError> {
        if !OffscreenRenderer::accepts_size(width, height) {
            return Err(SvgError::Malformed(format!(
                "an image of {width}x{height} pixels is too large to embed"
            )));
        }
        let scene = Arc::new(DrawList {
            cmds: vec![cmd],
            regions: Vec::new(),
        });
        let image = self
            .engine
            .render(&scene, (width, height), self.inherited)
            .map_err(|error| match error {
                OffscreenError::Dropped(_) => SvgError::Unsupported(
                    "SVG cannot express an image the renderer drops".to_owned(),
                ),
                OffscreenError::Failed(message) => SvgError::Resource(message.to_string()),
            })?;
        Ok(Pixels {
            width: image.width,
            height: image.height,
            rgba: image.rgba.into(),
        })
    }
}

impl Default for Offscreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Offscreen {
    pub fn new() -> Self {
        Self {
            engine: OffscreenRenderer::new(),
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
            .map_err(|message| failed(ErrorKind::Asset, message.to_string(), at))
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
    ) -> Result<Pixels> {
        let at = Location::caller();
        if !OffscreenRenderer::accepts_size(width, height) {
            return Err(failed(
                ErrorKind::Message,
                format!("invalid or excessive image size {width}x{height}"),
                at,
            ));
        }
        let size = Size::new(width as f64, height as f64);
        let Recorded {
            scene, inherited, ..
        } = self.record(size, paint, at)?;
        let image = self
            .engine
            .render(&Arc::new(scene), (width, height), &inherited)
            .map_err(|error| match error {
                OffscreenError::Dropped(_) => rejected(at),
                OffscreenError::Failed(message) => {
                    failed(ErrorKind::Asset, message.to_string(), at)
                }
            })?;
        Ok(Pixels {
            width: image.width,
            height: image.height,
            rgba: image.rgba.into(),
        })
    }

    /// Write `paint` as an SVG 1.1 document of `width` × `height` logical
    /// pixels. The document's `width`, `height` and `viewBox` are these
    /// values, and it has no background: paint one to make it opaque.
    /// `paint` runs once with the same `Draw` vocabulary, theme and fonts as
    /// [`Offscreen::render`], so one `Offscreen` serves the PNG and the SVG
    /// export. `paint` does not run when the size is invalid.
    ///
    /// Images are rasterized by the same CPU renderer at the size they are
    /// drawn and embedded as PNG data URIs. Fonts are not embedded: the
    /// document names the families, which the viewer must have installed.
    ///
    /// Not expressible in SVG 1.1, and therefore `ErrorKind::Unsupported`:
    /// paragraphs the host shaped, external GPU frames, drawables' subtrees,
    /// border images, image fills and shader fills, background layers sized
    /// or placed by hand, more than one background layer, repeating and
    /// conic gradients, radial gradients sized by a keyword, OpenType font
    /// features and text decorations, blend modes, masks, filters and 3D
    /// transforms on layers.
    ///
    /// The errors, located at the call:
    ///
    /// - `ErrorKind::Message`: the size is not a positive finite pair, or
    ///   the drawing does not nest (a pop without its push, or a push left
    ///   open).
    /// - `ErrorKind::Unsupported`: the drawing holds a command, value or
    ///   property listed above, or a value that is not finite.
    /// - `ErrorKind::Asset`: an image or text the drawing uses failed to load
    ///   or to be measured.
    ///
    /// ```
    /// # use kivact_core as kivact;
    /// use kivact::draw::Offscreen;
    /// use kivact::geometry::Rect;
    /// use kivact::style::rgb;
    ///
    /// let document = Offscreen::new().render_svg(64.0, 32.0, |draw, _size| {
    ///     draw.fill_rect(Rect::new(0.0, 0.0, 64.0, 32.0), rgb(0xff3300));
    /// })?;
    /// assert!(document.starts_with("<?xml"));
    /// # Ok::<(), kivact::UiError>(())
    /// ```
    #[track_caller]
    pub fn render_svg(
        &mut self,
        width: f64,
        height: f64,
        paint: impl FnOnce(&mut Draw, Size),
    ) -> Result<String> {
        let at = Location::caller();
        if !(width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0) {
            return Err(failed(
                ErrorKind::Message,
                format!("invalid SVG size {width}x{height}"),
                at,
            ));
        }
        let size = Size::new(width, height);
        let Recorded {
            scene,
            inherited,
            measurer,
        } = self.record(size, paint, at)?;
        let mut host = SvgHost {
            engine: &mut self.engine,
            measurer: &measurer,
            inherited: &inherited,
        };
        kivact_svg::write(&scene, width, height, &mut host).map_err(|error| svg_error(error, at))
    }

    /// The recording prelude of `render` and `render_svg`: records `paint`
    /// at `size` with the theme's inherited text style and a measurer of it.
    /// A command `Draw` rejects is an `Unsupported` error located at `at`.
    fn record(
        &mut self,
        size: Size,
        paint: impl FnOnce(&mut Draw, Size),
        at: &'static Location<'static>,
    ) -> Result<Recorded> {
        let inherited = ComputedTextStyle {
            family: self.theme.text().family.clone(),
            line_height: self.theme.text().line_height,
            ..Default::default()
        };
        let measurer = self.engine.text_measurer(inherited.clone());
        let Recording {
            scene, unsupported, ..
        } = kivact_style::internal::record(size, &self.theme, |draw| {
            draw.set_text_measurer(measurer.clone());
            paint(draw, size);
        });
        if unsupported != 0 {
            return Err(rejected(at));
        }
        Ok(Recorded {
            scene,
            inherited,
            measurer,
        })
    }
}

/// The error of a drawing that holds a command `Draw` rejects.
fn rejected(at: &'static Location<'static>) -> UiError {
    failed(
        ErrorKind::Unsupported,
        "the drawing holds a command Draw rejects",
        at,
    )
}

/// The error of an SVG export that the writer refused, located at the call.
fn svg_error(error: SvgError, at: &'static Location<'static>) -> UiError {
    match error {
        SvgError::Unsupported(message) => failed(ErrorKind::Unsupported, message, at),
        SvgError::Malformed(message) => failed(ErrorKind::Message, message, at),
        SvgError::Resource(message) => failed(ErrorKind::Asset, message, at),
    }
}

fn failed(kind: ErrorKind, message: impl Into<String>, at: &'static Location<'static>) -> UiError {
    UiError::new(kind, message, Some(at))
}
