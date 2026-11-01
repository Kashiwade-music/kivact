//! Role: Arvane: the public facade of the Arvane layer.
//!
//! Responsibility: the only crate outside Arvane may depend on (via
//! `kivact-engine`). It wires the adapter crates together; the
//! adapters share `kivact-types` values and portable `arvane-shell` contracts. Text is the [`Engine`]'s; windows ([`Window`]: the native
//! [`NativeWindow`] run by [`Shell`], and the test windows) present scenes
//! through their own presenters, which own the renderers, the GPU life
//! cycle and the frame policy (`presenter.rs`).
//!
//! Invariants:
//! - Public signatures use Arvane contracts, neutral `kivact-types` values and
//!   standard library types.
//!   No type from winit, taffy, parley, vello_cpu, vello_hybrid, wgpu,
//!   softbuffer, accesskit, image, resvg or skrifa is reachable from it.
//! - Arvane depends on no Kivact type (the wire included); a host lowers its
//!   own values to Arvane's. OS differences end in the windows (`Window`,
//!   `arvane-native`'s shell), GPU and CPU differences in the presenter and
//!   `arvane-render`.
//! - Textures an application's render engine shares with a window
//!   (feature `gpu-interop`) are imported into that window's own GPU device
//!   ([`Window::import_external_image`]): an image belongs to one window,
//!   and another window's scene that names it draws nothing. The `unsafe`
//!   this needs is in `kivact-gpu-share`, below `arvane-render`.
//! - What a value means where it crosses (logical coordinates,
//!   straight-alpha input pixels, premultiplied frames, a text's scalar and
//!   glyph positions and its edit byte offsets, `Arc` identity) is written
//!   on its type.
//!
//! The adapters are `arvane-text` (shaping), `arvane-render` (drawing),
//! `arvane-asset` ([`asset`]: images, SVG, fonts) and `arvane-native`
//! (windows, input, accessibility). Portable shell vocabulary and input
//! normalization and validated accessibility trees live in `arvane-shell`;
//! native decoding stays in the backend. Inside each, the values shared by
//! several modules (this crate's renderer values in `presentation.rs`,
//! `kivact-types`' image data, text's errors and metrics, assets' limits and
//! image state, render's errors, gradients, GPU state and texture making,
//! the shell's values, native input state and the shared accessibility
//! tree) sit in modules of their own, which the modules doing the work
//! refer to.

#![forbid(unsafe_code)]

#[cfg(feature = "embed")]
pub mod embed;

mod env;
mod error;
mod presentation;
mod presenter;
mod rasterizer;
mod shutdown;
/// Windows for tests (feature `testing`).
#[cfg(feature = "testing")]
pub mod test_display;
mod window;

use kivact_types::{RichText, ShapedText, Size, TextMetrics, TextStyle};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// Images, SVG and font files: loading, decoding, the image cache and its
/// worker, fit geometry and font family resolution.
pub mod asset {
    //! Role: Explicit image and font services exposed by the Arvane facade.
    pub use arvane_asset::{
        Asset, AssetErrorKind, AssetStore, FamilyChoice, FamilyResolution, ImageState, PixelSize,
        Pixels, Region, fit_image, load_font,
    };
}
pub use arvane_render::{MAX_SCENE_DEPTH, glyph_ink_bounds};
pub use arvane_shell::{
    Accessibility, AccessibilityError, DEFAULT_FRAME_PERIOD_NS, ShellControl, ShellError,
    ShellEvent, ShellFileDialog, ShellImeText, ShellOptions, ShellWaker,
};
pub use error::{Error, ErrorKind};
pub use presentation::{Renderer, RendererChange};
pub use presenter::{FramePlan, Presented};
pub use rasterizer::Rasterizer;

pub use window::{Handler, NativeWindow, Shell, Window};

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
    text: RefCell<arvane_text::TextEngine>,
    thread: PhantomData<Rc<()>>,
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

pub use arvane_native::retain_module;
pub use shutdown::detached_threads;
