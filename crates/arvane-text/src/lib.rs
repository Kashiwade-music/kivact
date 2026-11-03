//! Role: Arvane text: shaping, line breaking and cursor geometry over parley.
//!
//! Responsibility: the only crate that names parley. Shaped text leaves this
//! crate as `kivact-types` glyph runs (font bytes, glyph ids, positions), so
//! the render crate never needs parley.
//!
//! Parley does the system font fallback, shaping, bidi and line breaking
//! (dictionary breaking of Japanese and other scripts without spaces
//! included). A paragraph tries its requested family, or each family of a
//! CSS family list in order (a list or a quoted name is read by parley's
//! CSS parser; a list it cannot read is one family name; a name of a list
//! that is not installed is skipped), glyph by glyph, then the platform
//! UI font (`system-ui`, then `sans-serif`), then the platform's symbol
//! fonts, and before the symbol fonts the color emoji font for a text that
//! may hold emoji (`build.rs`); a span with a family falls back from it
//! the same way, glyph by glyph. A variable font's weight (`wght`) reaches
//! both renderers as the run's normalized coordinates, which they apply to
//! the outlines. A paragraph is shaped once per text, spans and shaping
//! style (`layouts.rs`): another width only breaks its lines again, and what is
//! not shaping (colors, decorations, highlight bands, alignment, overflow,
//! line clamp) is applied to the shaped layout afterwards (`output.rs`).
//! The "…" of an ellipsis is shaped once per paragraph style and kept
//! (at most `ELLIPSES`, then all are forgotten). A text of white space
//! alone is broken only at its line breaks, so its spaces hang past any
//! width (`layout_at`). [`TextEngine::has_family`] answers whether a
//! family is installed; [`TextEngine::isolated`] never sees system fonts
//! (deterministic tests).
//!
//! On Linux, system fonts are found through fontconfig, loaded at run time
//! (fontique's `fontconfig-dlopen`): a build needs no fontconfig
//! development package or pkg-config, a run needs `libfontconfig.so.1`.
//!
//! Invariant: parley types never appear in the public API.

#![forbid(unsafe_code)]

mod build;
mod cache;
mod error;
mod inline;
mod layouts;
mod metrics;
mod output;
mod text_engine;
mod transform;

pub use error::TextError;
pub use text_engine::TextEngine;
