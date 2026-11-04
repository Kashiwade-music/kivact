//! Role: The bundled Tuffy test font, drawn the same on every machine.

use std::sync::Arc;

/// The bundled test font (Tuffy, public domain; its license is
/// `assets/fonts/Tuffy-LICENSE.txt`). It has no CJK glyphs.
static TEST_FONT: &[u8] = include_bytes!("../assets/fonts/Tuffy.ttf");

/// The bundled test font (Tuffy, one family, no CJK glyphs): an
/// [`crate::Engine::with_fonts_only`] given it shapes, lays out and draws
/// text the same on every machine.
pub fn test_font() -> Arc<[u8]> {
    Arc::from(TEST_FONT)
}
