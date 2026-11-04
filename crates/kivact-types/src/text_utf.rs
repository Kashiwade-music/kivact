//! Role: Convert clamped UTF-8 byte positions and platform UTF-16 code-unit offsets.

/// Largest UTF-8 character boundary no greater than byte, clamped to the text.
pub fn floor_char_boundary(text: &str, byte: usize) -> usize {
    let mut byte = byte.min(text.len());
    while !text.is_char_boundary(byte) {
        byte -= 1;
    }
    byte
}

/// UTF-16 code units before a byte position, flooring partial UTF-8 characters.
///
/// Byte positions beyond the text are clamped to its end. This counts UTF-16
/// code units, not Unicode scalar values or grapheme clusters.
pub fn utf16_offset(text: &str, byte: usize) -> usize {
    text[..floor_char_boundary(text, byte)]
        .encode_utf16()
        .count()
}

/// UTF-8 byte position at a UTF-16 offset, clamped to the text's end.
///
/// An offset inside a surrogate pair rounds forward to the end of that Unicode
/// scalar value, preserving the editing and engine IME conversion contract.
pub fn byte_offset_from_utf16(text: &str, units: usize) -> usize {
    let mut seen = 0;
    for (byte, character) in text.char_indices() {
        if seen >= units {
            return byte;
        }
        seen += character.len_utf16();
    }
    text.len()
}
