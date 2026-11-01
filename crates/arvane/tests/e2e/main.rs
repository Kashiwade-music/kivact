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
//! face they hold: `present` and `presenter` (scenes to frame pixels and
//! the frame policy), `text` and `text_style`, `input`, `accessibility`,
//! `environment`, `assets`, `transform`, `layers`, `paints`, `shapes`,
//! `effects`, `images`, `external` (feature `gpu-interop`, Windows and
//! Linux: textures an external render engine shares) and `embed` (feature
//! `embed`).
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
