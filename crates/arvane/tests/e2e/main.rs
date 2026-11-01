//! Role: Arvane contract test suite.
//!
//! Arvane's contract tests: what a host sees through the `arvane` facade
//! alone (no Kivact crate), on real test windows
//! (`arvane::test_display`): one UI thread per process, windows reused from
//! its pool, frames drawn on the CPU (bit for bit the same everywhere)
//! unless a test asks for the GPU, text shaped with the bundled test font
//! only.
//!
//! One test binary (`required-features = ["testing"]`; `cargo test -p
//! arvane --all-features`), its modules split by the part of the upper
//! face they hold:
//! - `present`: scenes to frame pixels (rects, rounded corners, clips,
//!   translucency, straight-alpha images, glyph coverage and baseline, a
//!   shadow's reach, the reduced scale of a window beyond the frame
//!   budget, an invalid scene dropped as `Content`, GPU against CPU, the
//!   window-less `Rasterizer`).
//! - `presenter`: `Shown` and its renderer, an occluded window's `Skipped`
//!   and its doubling wait, `Dropped` without a surface or pixels,
//!   `SurfaceLost` and the third loss's error, the surface made again, the
//!   forced renderer.
//! - `text`: measuring as shaping, wrapping at another width, families,
//!   registering a font drops the caches, fallback, error kinds, line
//!   heights, lines and their ranges, glyph ink.
//! - `text_style`: letter and word spacing, kerning, justification, `tnum`
//!   and `ss01`, a variable font's weight, decoration positions, ellipsis
//!   and line clamp, case conversion moving the spans, span colors, bands
//!   and decorations, per-glyph fallback inside a span, invalid spans, the
//!   first baseline.
//! - `input`: click and tap counts, contacts' ids and kinds, a lost
//!   contact canceled, keys and the IME in order.
//! - `accessibility`: replacement and updates, an invalid tree or update
//!   keeping the accepted tree, assistive technology's requests checked,
//!   relations, list and tree item properties and roles on the platform
//!   node, active descendants, text runs.
//! - `environment`: the environment snapshot, capabilities, the software
//!   keyboard and the IME requests, the clipboard, dialogs, open and
//!   reveal.
//! - `assets`: the store's worker and waker, PNG, EXIF-rotated JPEG and
//!   SVG by content, a failure reported once, identity.
//! - `transform`: rotation and skew, nesting, clips, images and paints
//!   under transforms, a flattening transform, rotated and scaled text.
//! - `layers`: group opacity, polygon and elliptical clips, the 17 blend
//!   modes against the W3C formulas, blur and drop shadow, the mask and
//!   filter order, group nesting checks.
//! - `paints`: radial (circle, ellipse, two circles) and conic gradients,
//!   repeat, reflect and hard stops, image patterns (both axes and one),
//!   gradient-dot tiles, a pattern's color matrix, degenerate gradients.
//! - `shapes`: elliptical corners and polygons, arcs drawn where
//!   `Shape::contains` hits, fill rules, caps, joins, dashes and dots.
//! - `effects`: inset and outer elliptical shadows, knockout, offset and
//!   spread, a backdrop's blur leaving the outside alone, its color
//!   matrix and groups.
//! - `images`: images through a color matrix, pixels that change every
//!   frame (CPU and GPU), the whole vocabulary on the GPU against the CPU
//!   (transformed scenes included, three frames).
//! - `external` (feature `gpu-interop`, Windows and Linux): textures an
//!   external render engine shares (`gpu-interop-producer`), on Direct3D
//!   12 and Vulkan: frames exact in four places, an engine drawing ahead,
//!   a frame never ready given up after 100 ms, a dropped image, refused
//!   mismatches, two windows' own images, a CPU window drawing none.
//!
//! Determinism: frames are drawn on the CPU (bit for bit the same), text
//! with the bundled test font only (Tuffy,
//! `arvane::test_display::test_font`), the clock is virtual. Image
//! fixtures are in `crates/arvane/tests/fixtures/`. The tests of OpenType
//! features and variable weights use Inter
//! (`crates/arvane/tests/fonts/InterVariable.ttf`, SIL OFL, with a `wght`
//! axis, `tnum`, `ss01` and `cv11`) through `Engine::with_fonts_only`. A
//! GPU frame matches the CPU frame when the mean channel difference is at
//! most 1, at most 0.5% of the pixels differ by more than 64 and at least
//! 99% by 8 or less (`support.rs`). A GPU test is skipped, saying why,
//! only when there is no hardware adapter or no feature `gpu`
//! (`arvane::test_display::gpu_unavailable`).

mod accessibility;
mod assets;
mod effects;
mod environment;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
mod external;
mod images;
mod input;
mod layers;
mod paints;
mod present;
mod presenter;
mod shapes;
mod support;
mod text;
mod text_style;
mod transform;

#[cfg(feature = "embed")]
mod embed;
