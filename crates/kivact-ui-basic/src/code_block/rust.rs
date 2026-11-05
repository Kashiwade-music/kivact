//! Role: A small Rust tokenizer that turns source text into highlighted code lines.
//!
//! Not a parser: it reads a token at a time from the whole source, so a
//! block comment or a string that spans lines keeps its tone on each of
//! them. Keywords are Rust's strict words; an uppercase identifier or a
//! primitive type is a type, an identifier before `(` a function, one
//! before `!` a macro.

use super::lines::{CodeLine, CodeSpan, CodeTone};

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

/// The primitive types, highlighted as type names.
const PRIMITIVES: &[&str] = &[
    "bool", "char", "f64", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str", "u8", "u16",
    "u32", "u64", "u128", "usize",
];

/// Highlight Rust `source` into one [`CodeLine`] per line for
/// [`code_block`](crate::code_block).
///
/// Keywords, string and character literals (raw and byte strings too),
/// line and block comments (nested, across lines), numbers, type names,
/// function names, macros, lifetimes and attributes each get their tone;
/// a final line break adds no empty line.
///
/// ```
/// use kivact_ui_basic::rust_source;
/// let lines = rust_source("fn main() {}");
/// assert_eq!(lines[0].text(), "fn main() {}");
/// ```
pub fn rust_source(source: &str) -> Vec<CodeLine> {
    let mut lines = vec![Vec::<CodeSpan>::new()];
    for (text, tone) in tokens(source) {
        for (index, part) in text.split('\n').enumerate() {
            if index > 0 {
                lines.push(Vec::new());
            }
            if part.is_empty() {
                continue;
            }
            if let Some(line) = lines.last_mut() {
                match line.last_mut() {
                    Some(last) if last.tone == tone => last.text.push_str(part),
                    _ => line.push(CodeSpan::new(part, tone)),
                }
            }
        }
    }
    if lines.last().is_some_and(Vec::is_empty) && lines.len() > 1 {
        lines.pop();
    }
    lines.into_iter().map(CodeLine).collect()
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

fn tokens(source: &str) -> Vec<(String, CodeTone)> {
    let chars: Vec<char> = source.chars().collect();
    let at = |i: usize| chars.get(i).copied().unwrap_or('\0');
    let mut out: Vec<(String, CodeTone)> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let (end, tone) = if c == '/' && at(i + 1) == '/' {
            let mut end = i;
            while end < chars.len() && chars[end] != '\n' {
                end += 1;
            }
            (end, CodeTone::Comment)
        } else if c == '/' && at(i + 1) == '*' {
            (block_comment_end(&chars, i), CodeTone::Comment)
        } else if c == '"' {
            (string_end(&chars, i), CodeTone::String)
        } else if let Some(end) = prefixed_string_end(&chars, i) {
            (end, CodeTone::String)
        } else if c == '\'' {
            if at(i + 1) == '\\' || at(i + 2) == '\'' {
                let mut end = i + 1;
                while end < chars.len() && chars[end] != '\'' {
                    end += if chars[end] == '\\' { 2 } else { 1 };
                }
                ((end + 1).min(chars.len()), CodeTone::String)
            } else {
                let mut end = i + 1;
                while end < chars.len() && is_ident(chars[end]) {
                    end += 1;
                }
                (end, CodeTone::Lifetime)
            }
        } else if c == '#' && (at(i + 1) == '[' || (at(i + 1) == '!' && at(i + 2) == '[')) {
            (attribute_end(&chars, i), CodeTone::Attribute)
        } else if c.is_ascii_digit() {
            let mut end = i;
            while end < chars.len()
                && (is_ident(chars[end]) || (chars[end] == '.' && at(end + 1).is_ascii_digit()))
            {
                end += 1;
            }
            (end, CodeTone::Number)
        } else if is_ident_start(c) {
            let mut end = i;
            while end < chars.len() && is_ident(chars[end]) {
                end += 1;
            }
            let word = collect(&chars, i, end);
            if at(end) == '!' && at(end + 1) != '=' {
                (end + 1, CodeTone::Macro)
            } else if KEYWORDS.contains(&word.as_str()) {
                (end, CodeTone::Keyword)
            } else if c.is_uppercase() || PRIMITIVES.contains(&word.as_str()) {
                (end, CodeTone::Type)
            } else if at(end) == '(' {
                (end, CodeTone::Function)
            } else {
                (end, CodeTone::Plain)
            }
        } else if c.is_whitespace() {
            let mut end = i;
            while end < chars.len() && chars[end].is_whitespace() {
                end += 1;
            }
            (end, CodeTone::Plain)
        } else {
            (i + 1, CodeTone::Punctuation)
        };
        out.push((collect(&chars, i, end), tone));
        i = end.max(i + 1);
    }
    out
}
