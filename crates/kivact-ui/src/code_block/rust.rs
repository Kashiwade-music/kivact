//! Role: The Rust tokens of a code block's source, read as one text.
//!
//! Not a parser: it reads a token at a time from the whole source, so a
//! block comment or a string that spans lines keeps its kind on each of
//! them. Keywords are Rust's strict words; an uppercase identifier or a
//! primitive type is a type, an identifier before `(` a function, one
//! before `!` a macro.

use crate::syntax::SyntaxKind;
use std::ops::Range;

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

/// The primitive types, highlighted as type names.
const PRIMITIVES: &[&str] = &[
    "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str", "u8", "u16",
    "u32", "u64", "u128", "usize",
];

/// The Rust tokens of each line of `source`, read as one text: a block
/// comment or a string that spans lines keeps its kind on every line.
///
/// Entry `n` holds the tokens of the `n`th line, the lines being
/// `source.split('\n')`; a token's range is inside that line. Text that
/// has no color (identifiers, spaces and the like) has no entry. Give a
/// line's tokens to [`crate::syntax_ranges`]. Lines are split like `str::split('\n')`, so a trailing newline leaves an empty last line; `Plain` text is not returned.
pub fn rust_tokens(source: &str) -> Vec<Vec<(Range<usize>, SyntaxKind)>> {
    let mut starts = vec![0];
    starts.extend(source.match_indices('\n').map(|(at, _)| at + 1));
    let mut lines = vec![Vec::new(); starts.len()];
    for (range, kind) in tokens(source) {
        let mut at = range.start;
        while at < range.end {
            let line = starts.partition_point(|&start| start <= at) - 1;
            let line_end = starts.get(line + 1).map_or(source.len(), |next| next - 1);
            let end = range.end.min(line_end);
            if end > at {
                lines[line].push((at - starts[line]..end - starts[line], kind));
            }
            at = end + 1;
        }
    }
    lines
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn collect(chars: &[char], from: usize, to: usize) -> String {
    chars[from..to.min(chars.len())].iter().collect()
}

/// The colored tokens of `source`, as byte ranges of it.
fn tokens(source: &str) -> Vec<(Range<usize>, SyntaxKind)> {
    let chars: Vec<char> = source.chars().collect();
    let bytes: Vec<usize> = source
        .char_indices()
        .map(|(at, _)| at)
        .chain(std::iter::once(source.len()))
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let (end, kind) = token_at(&chars, i);
        if let Some(kind) = kind {
            out.push((bytes[i]..bytes[end], kind));
        }
        i = end.max(i + 1);
    }
    out
}

/// The end (exclusive, in chars) of the token at `i`, and its kind if it
/// is colored.
fn token_at(chars: &[char], i: usize) -> (usize, Option<SyntaxKind>) {
    let at = |k: usize| chars.get(k).copied().unwrap_or('\0');
    let c = chars[i];
    if c == '/' && at(i + 1) == '/' {
        let mut end = i;
        while end < chars.len() && chars[end] != '\n' {
            end += 1;
        }
        (end, Some(SyntaxKind::Comment))
    } else if c == '/' && at(i + 1) == '*' {
        (block_comment_end(chars, i), Some(SyntaxKind::Comment))
    } else if c == '"' {
        (string_end(chars, i), Some(SyntaxKind::String))
    } else if let Some(end) = prefixed_string_end(chars, i) {
        (end, Some(SyntaxKind::String))
    } else if c == '\'' {
        quote_token(chars, i)
    } else if c == '#' && (at(i + 1) == '[' || (at(i + 1) == '!' && at(i + 2) == '[')) {
        (attribute_end(chars, i), Some(SyntaxKind::Attribute))
    } else if c.is_ascii_digit() {
        (number_end(chars, i), Some(SyntaxKind::Number))
    } else if is_ident_start(c) {
        word_token(chars, i)
    } else if c.is_whitespace() {
        (whitespace_end(chars, i), None)
    } else {
        (i + 1, Some(SyntaxKind::Punctuation))
    }
}

/// The end and kind of a token that starts with `'`: a character literal,
/// or a lifetime or loop label.
fn quote_token(chars: &[char], i: usize) -> (usize, Option<SyntaxKind>) {
    let at = |k: usize| chars.get(k).copied().unwrap_or('\0');
    if at(i + 1) == '\\' || at(i + 2) == '\'' {
        let mut end = i + 1;
        while end < chars.len() && chars[end] != '\'' {
            end += if chars[end] == '\\' { 2 } else { 1 };
        }
        ((end + 1).min(chars.len()), Some(SyntaxKind::String))
    } else {
        let mut end = i + 1;
        while end < chars.len() && is_ident(chars[end]) {
            end += 1;
        }
        (end, Some(SyntaxKind::Lifetime))
    }
}

/// The end (exclusive) of the number whose first digit is at `i`.
fn number_end(chars: &[char], i: usize) -> usize {
    let at = |k: usize| chars.get(k).copied().unwrap_or('\0');
    let mut end = i;
    while end < chars.len()
        && (is_ident(chars[end]) || (chars[end] == '.' && at(end + 1).is_ascii_digit()))
    {
        end += 1;
    }
    end
}

/// The end and kind of the identifier, keyword or macro name at `i`.
fn word_token(chars: &[char], i: usize) -> (usize, Option<SyntaxKind>) {
    let at = |k: usize| chars.get(k).copied().unwrap_or('\0');
    let mut end = i;
    while end < chars.len() && is_ident(chars[end]) {
        end += 1;
    }
    if at(end) == '!' && at(end + 1) != '=' {
        return (end + 1, Some(SyntaxKind::Macro));
    }
    let word = collect(chars, i, end);
    let kind = if KEYWORDS.contains(&word.as_str()) {
        Some(SyntaxKind::Keyword)
    } else if chars[i].is_uppercase() || PRIMITIVES.contains(&word.as_str()) {
        Some(SyntaxKind::Type)
    } else if at(end) == '(' {
        Some(SyntaxKind::Function)
    } else {
        None
    };
    (end, kind)
}

/// The end (exclusive) of the whitespace run at `i`.
fn whitespace_end(chars: &[char], i: usize) -> usize {
    let mut end = i;
    while end < chars.len() && chars[end].is_whitespace() {
        end += 1;
    }
    end
}

/// The end (exclusive) of a string literal whose opening quote is at `start`.
fn string_end(chars: &[char], start: usize) -> usize {
    let mut i = start + 1;
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 2,
            '"' => return i + 1,
            _ => i += 1,
        }
    }
    chars.len()
}

/// The end (exclusive) of the block comment whose `/*` is at `start`;
/// block comments nest.
fn block_comment_end(chars: &[char], start: usize) -> usize {
    let at = |i: usize| chars.get(i).copied().unwrap_or('\0');
    let (mut depth, mut i) = (0usize, start);
    while i < chars.len() {
        if chars[i] == '/' && at(i + 1) == '*' {
            depth += 1;
            i += 2;
        } else if chars[i] == '*' && at(i + 1) == '/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += 1;
        }
    }
    chars.len()
}

/// The end (exclusive) of a byte, C or raw string literal (`b"`, `c"`,
/// `r"`, `r#"`, `br#"`) whose prefix is at `start`, if one starts there.
fn prefixed_string_end(chars: &[char], start: usize) -> Option<usize> {
    let at = |i: usize| chars.get(i).copied().unwrap_or('\0');
    if start > 0 && is_ident(chars[start - 1]) {
        return None;
    }
    let mut i = start;
    if matches!(at(i), 'b' | 'c') {
        i += 1;
    }
    if at(i) != 'r' {
        return (i > start && at(i) == '"').then(|| string_end(chars, i));
    }
    i += 1;
    let hashes = chars[i.min(chars.len())..]
        .iter()
        .take_while(|&&c| c == '#')
        .count();
    i += hashes;
    if at(i) != '"' {
        return None;
    }
    i += 1;
    while i < chars.len() {
        if chars[i] == '"' && (1..=hashes).all(|k| at(i + k) == '#') {
            return Some(i + 1 + hashes);
        }
        i += 1;
    }
    Some(chars.len())
}

/// The end (exclusive) of the attribute whose `#` is at `start`.
fn attribute_end(chars: &[char], start: usize) -> usize {
    let mut depth = 0usize;
    let mut i = start;
    while i < chars.len() {
        match chars[i] {
            '[' => depth += 1,
            ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    chars.len()
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the Rust tokens of a source read as one text.

    use super::*;

    #[test]
    fn rust_tokens_keeps_a_block_comment_kind_on_each_line() {
        let lines = rust_tokens("/* one\ntwo */ fn x() {}");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], [(0..6, SyntaxKind::Comment)]);
        assert_eq!(lines[1][0], (0..6, SyntaxKind::Comment));
        assert_eq!(lines[1][1], (7..9, SyntaxKind::Keyword));
    }

    #[test]
    fn rust_tokens_keeps_a_string_kind_across_a_line_break() {
        let lines = rust_tokens("let s = \"a\nb\";");
        assert_eq!(
            lines[0],
            [
                (0..3, SyntaxKind::Keyword),
                (6..7, SyntaxKind::Punctuation),
                (8..10, SyntaxKind::String),
            ]
        );
        assert_eq!(
            lines[1],
            [(0..2, SyntaxKind::String), (2..3, SyntaxKind::Punctuation)]
        );
    }

    #[test]
    fn rust_tokens_uses_byte_offsets_for_multibyte_text() {
        let lines = rust_tokens("// é\nfn");
        assert_eq!(lines[0], [(0..5, SyntaxKind::Comment)]);
        assert_eq!(lines[1], [(0..2, SyntaxKind::Keyword)]);
    }
}
