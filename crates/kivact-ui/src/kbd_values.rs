//! Role: Supply passive key-cap glyphs and spoken labels to theme recipes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KbdKeyState {
    pub glyph: String,
    pub label: String,
}
