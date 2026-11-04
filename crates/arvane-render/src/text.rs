//! Role: How glyphs are rasterized, the same on the CPU and the GPU.
//!
//! Text is antialiased in grayscale (no subpixel ClearType: it breaks on
//! transparent layers and transforms), so two things make small text as
//! crisp and dark as the platform's:
//! - Hinting: both rasterizers hint vertically only (skrifa's light
//!   "smooth" target, the font's instructions or the autohinter) at the
//!   device font size, and put each run's baseline on a whole device
//!   pixel. Horizontal positions keep their subpixel precision, so the
//!   shaped advances and the layout are unchanged. This needs a
//!   translation and a uniform scale (`Transform::uniform_scale`, within a
//!   relative 1e-5, so a skew and the counter-skew of a child count as the
//!   translation they leave); under a rotation, skew or non-uniform scale
//!   glyphs are drawn unhinted (`draw.rs`).
//! - Stem darkening: outlines are dilated by [`darkening`] on each side,
//!   compensating the lighter look of thin grayscale antialiased stems
//!   (as FreeType's and macOS's stem darkening do), fading out as glyphs
//!   grow and their stems span several pixels.
//! - Positions: each glyph's horizontal origin is put on a quarter device
//!   pixel ([`subpixel`]). The GPU draws glyphs from an atlas that keeps
//!   four subpixel positions per glyph (glifo's cache), copying them
//!   pixel-aligned; the CPU draws the outlines at the same positions, so
//!   both produce the same coverage. A quarter pixel moves a glyph by at
//!   most an eighth of a pixel from its shaped position.
//!
//! Neither vello_cpu nor vello_hybrid offers a gamma or contrast
//! adjustment of the coverage; stem darkening stands in for it.

use vello_cpu::kurbo::Diagonal2;

/// Dilation per side, in device pixels, of glyphs drawn at a device
/// font size of `size` pixels: 0.1 px up to 24 px, fading linearly to none
/// at 48 px.
pub(crate) fn darkening(size: f64) -> Diagonal2 {
    const FULL: f64 = 24.0;
    const NONE: f64 = 48.0;
    const AMOUNT: f64 = 0.1;
    let amount = AMOUNT * ((NONE - size) / (NONE - FULL)).clamp(0.0, 1.0);
    Diagonal2::new(amount, amount)
}

/// `x` (logical pixels, at `scale` device pixels per logical pixel) on the
/// nearest quarter device pixel.
pub(crate) fn subpixel(x: f64, scale: f64) -> f64 {
    (x * scale * 4.0).round() / (scale * 4.0)
}
