//! Role: Engine: text shaping, measuring and fonts for its creating UI thread.

use crate::asset;
use crate::error::{self, Error};
use kivact_types::{RichText, ShapedText, Size, TextMetrics, TextStyle};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// Arvane's text (shaping, measuring, fonts) for use on the creating UI
/// thread (`!Send`, `!Sync`).
///
/// ```compile_fail
/// fn requires_send<T: Send>() {}
/// requires_send::<arvane::Engine>();
/// ```
pub struct Engine {
    /// In a cell only so [`Engine::font_families`] can fill its snapshot
    /// through `&self`; every other use goes through `get_mut`.
    pub(super) text: RefCell<arvane_text::TextEngine>,
    pub(super) thread: PhantomData<Rc<()>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Create isolated adapter state without opening a native window.
    pub fn new() -> Self {
        Self::with_text(arvane_text::TextEngine::new())
    }

    /// Create adapter state whose text sees only `fonts`, never the
    /// system's: they become every generic family and the fallback of every
    /// script, so shaping, layout and frames are the same on every machine
    /// (deterministic tests). Text none of them covers draws a missing
    /// glyph. Fails when a font holds no usable face.
    pub fn with_fonts_only(fonts: &[Arc<[u8]>]) -> Result<Self, Error> {
        Ok(Self::with_text(
            arvane_text::TextEngine::isolated(fonts).map_err(error::text)?,
        ))
    }

    fn with_text(text: arvane_text::TextEngine) -> Self {
        Self {
            text: RefCell::new(text),
            thread: PhantomData,
        }
    }

    /// Register application-owned font bytes and return the available family names.
    ///
    /// New fonts can change fallback, so cached shaping and measurement are
    /// discarded, and so are the answers of [`Engine::has_font_family`].
    pub fn register_font(&mut self, bytes: Arc<[u8]>) -> Result<Vec<String>, Error> {
        self.text
            .get_mut()
            .register_font(bytes)
            .map_err(error::text)
    }

    /// System and registered font names, sorted and unique, preserving case.
    /// The names are enumerated on the first call and kept until the next
    /// successful application font registration; [`Engine::release_caches`]
    /// keeps them.
    pub fn font_families(&self) -> Vec<String> {
        self.text.borrow_mut().family_names()
    }

    /// Release the text adapter's caches
    /// when the platform is short of memory: paragraph layouts, measured
    /// and shaped snapshots, ellipses, font copies and shaping scratch.
    ///
    /// The font database and registered fonts remain. Snapshots already
    /// returned to a caller keep their shared font bytes and remain valid;
    /// later measurements and shapes rebuild the released work. Window
    /// renderers and the asset store own their caches separately.
    pub fn release_caches(&mut self) {
        self.text.get_mut().release_caches();
    }

    /// Whether a registered or system font has the family `family`, ignoring
    /// case.
    ///
    /// Answers are cached until [`Engine::register_font`] or
    /// [`Engine::release_caches`].
    pub fn has_font_family(&mut self, family: &str) -> bool {
        self.text.get_mut().has_family(family)
    }

    /// Resolve a requested font family against this engine's system and
    /// registered fonts, using Arvane's table for the current platform.
    ///
    /// Named families are trimmed and compared without case; a missing
    /// name selects the system UI font and sets `missing`. Empty names
    /// and CSS generic families never set it. The returned choice can be
    /// used as [`TextStyle::family`] (`SystemUi` means `None`); the host
    /// decides whether and where to report a missing name.
    ///
    /// Comma-separated CSS lists (and quoted names) are retained for
    /// per-glyph selection by the text engine, before its system fallback.
    /// Unavailable names in a list are skipped, without a missing-family
    /// diagnostic; malformed lists fall back to the system fonts.
    pub fn resolve_font_family(&mut self, name: &str) -> asset::FamilyResolution {
        let source = name.trim();
        if source.contains(',') || source.starts_with(['\'', '"']) {
            return asset::FamilyResolution {
                choice: asset::FamilyChoice::Family(source.to_owned()),
                missing: false,
            };
        }
        arvane_asset::resolve_family(name, arvane_asset::Os::current(), |family| {
            self.has_font_family(family)
        })
    }

    /// Shape a paragraph (a `&str`, or a [`RichText`] with spans),
    /// applying fallback fonts and optional line wrapping.
    ///
    /// Results and failures are cached by text, spans, style and width
    /// across calls. A paragraph is shaped once per text and shaping style
    /// (colors, decorations, highlights, alignment, overflow and line clamp
    /// aside); another width only breaks its lines again. Input text and
    /// spans are borrowed for this call; the returned snapshot owns its
    /// runs and font bytes independently of this engine and its caches.
    pub fn shape<'t>(
        &mut self,
        text: impl Into<RichText<'t>>,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> Result<Arc<ShapedText>, Error> {
        self.text
            .get_mut()
            .shape(text.into(), style, max_width)
            .map_err(error::text)
    }

    /// [`Engine::shape`] for drawing only: the same size and glyph runs,
    /// without carets (`carets` is empty), which cost about as much as
    /// breaking the lines. Cached likewise.
    pub fn shape_glyphs<'t>(
        &mut self,
        text: impl Into<RichText<'t>>,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> Result<Arc<ShapedText>, Error> {
        self.text
            .get_mut()
            .shape_glyphs(text.into(), style, max_width)
            .map_err(error::text)
    }

    /// The extent [`Engine::shape`] would give a paragraph, without glyph
    /// runs or carets (for layout). Cached by text, spans, style (except
    /// what only drawing reads: colors, decorations, highlights) and
    /// width; it shares the shaped paragraph with `shape`.
    pub fn measure_text<'t>(
        &mut self,
        text: impl Into<RichText<'t>>,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> Result<Size, Error> {
        self.text_metrics(text, style, max_width)
            .map(|metrics| metrics.size)
    }

    /// [`Engine::measure_text`] with the first line's baseline. Cached
    /// likewise (one cache).
    pub fn text_metrics<'t>(
        &mut self,
        text: impl Into<RichText<'t>>,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> Result<TextMetrics, Error> {
        self.text
            .get_mut()
            .measure(text.into(), style, max_width)
            .map_err(error::text)
    }
}
