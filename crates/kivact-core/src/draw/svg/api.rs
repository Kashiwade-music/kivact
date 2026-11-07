//! Role: The public `Svg` exporter: records a drawing and writes it as one SVG 1.1 document.
//!
//! The drawing is recorded exactly as `Offscreen` records it, so one paint
//! closure serves both. The text baseline is measured with the same text
//! engine that the recording uses, so a text lands where a window draws it.
//! Fonts that the document names are not embedded: the viewer needs them
//! installed under the same family names.

use super::emit::Emitter;
use crate::geometry::Size;
use crate::{Result, UiError};
use kivact_runtime::ErrorKind;
use kivact_style::Theme;
use kivact_style::asset::Asset;
use kivact_style::draw::{Draw, Recording};
use kivact_types::ComputedTextStyle;
use std::panic::Location;

/// Writes drawings as SVG 1.1 documents without an `App` or a window.
///
/// `render` takes the same paint closure as [`Offscreen::render`]
/// (`crate::draw::Offscreen`), with logical pixels as the unit. The output
/// has no background: paint one to make it opaque. Anything the SVG format
/// cannot express is an error, never a silent omission.
///
/// Images are rasterized by the same CPU renderer as `Offscreen` at the size
/// they are drawn, and embedded as PNG. Paragraphs, external frames, font
/// features, text decorations, blend modes, masks, filters, radial gradients
/// with keyword sizes and conic gradients are not supported.
///
/// It stays on the thread that made it (`!Send`, `!Sync`).
pub struct Svg {
    engine: kivact_engine::api::OffscreenRenderer,
    theme: Theme,
}

impl Default for Svg {
    fn default() -> Self {
        Self::new()
    }
}

impl Svg {
    /// A writer with the default theme and no registered fonts.
    pub fn new() -> Self {
        Self {
            engine: kivact_engine::api::OffscreenRenderer::new(),
            theme: Theme::default(),
        }
    }

    /// Resolve color tokens with `theme`, which also supplies the text family
    /// and line height that text without its own takes.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Register a TTF, OTF or TTC file, returning the family names it adds.
    /// The text of the drawing is measured with the registered fonts, so
    /// register the fonts the document names. A file that is not a usable
    /// font is an `ErrorKind::Asset` error.
    #[track_caller]
    pub fn font(&mut self, font: impl Into<Asset>) -> Result<Vec<String>> {
        let at = Location::caller();
        self.engine
            .font(&font.into())
            .map_err(|message| UiError::new(ErrorKind::Asset, message.to_string(), Some(at)))
    }

    /// Write `paint` as an SVG document of `width` × `height` logical pixels
    /// (`width`, `height` and the `viewBox` are the same values). `paint` runs
    /// once, and not at all when the size is not a positive finite pair.
    ///
    /// The errors, located at the call:
    ///
    /// - `ErrorKind::Message`: the size is not a positive finite pair, or the
    ///   drawing does not nest (a pop without its push).
    /// - `ErrorKind::Unsupported`: the drawing holds a command, value or
    ///   property SVG 1.1 cannot express (see the type's documentation).
    /// - `ErrorKind::Asset`: an image or text the drawing uses failed to load
    ///   or to be measured.
    #[track_caller]
    pub fn render(
        &mut self,
        width: f64,
        height: f64,
        paint: impl FnOnce(&mut Draw, Size),
    ) -> Result<String> {
        let at = Location::caller();
        if !(width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0) {
            return Err(UiError::new(
                ErrorKind::Message,
                format!("invalid SVG size {width}x{height}"),
                Some(at),
            ));
        }
        let size = Size::new(width, height);
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
            return Err(UiError::new(
                ErrorKind::Unsupported,
                "the drawing holds a command Draw rejects",
                Some(at),
            ));
        }
        let mut emitter = Emitter::new(&mut self.engine, &measurer, &inherited, at);
        for cmd in &scene.cmds {
            emitter.command(cmd)?;
        }
        emitter.finish(width, height)
    }
}
