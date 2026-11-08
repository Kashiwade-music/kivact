//! Role: Lexical checks of a user's shader source, and the offset map back to it.
//!
//! The map takes offsets of the assembled module back to the user's source.

use crate::error::ShaderError;

/// Where the user's source starts in the assembled module.
pub(crate) struct Placement<'a> {
    source: &'a str,
    start: usize,
}

impl<'a> Placement<'a> {
    /// Places `source` at byte `start` of the assembled text.
    pub(crate) fn new(source: &'a str, start: usize) -> Self {
        Self { source, start }
    }

    /// The error for a problem at `offset` of the assembled text, or with no
    /// place in the module when `offset` is `None`. A place outside the user's
    /// source is reported as an internal error.
    pub(crate) fn error(&self, offset: Option<usize>, message: impl Into<String>) -> ShaderError {
        let message = message.into();
        let Some(at) = offset else {
            return ShaderError::whole(message);
        };
        match at.checked_sub(self.start) {
            Some(inside) if inside <= self.source.len() => {
                let (line, column) = position(self.source, inside);
                ShaderError::new(message, line, column)
            }
            _ => ShaderError::whole(format!("internal: {message}")),
        }
    }

    /// An error in the generated code, which a correct user source never causes.
    pub(crate) fn internal(&self, message: impl Into<String>) -> ShaderError {
        ShaderError::whole(format!("internal: {}", message.into()))
    }
}

/// The 1-based line and byte column of `offset` in `source`.
pub(crate) fn position(source: &str, offset: usize) -> (u32, u32) {
    let before = source.get(..offset).unwrap_or(source);
    let line_start = before.rfind('\n').map_or(0, |at| at + 1);
    let line = before.matches('\n').count() + 1;
    (line as u32, (before.len() - line_start + 1) as u32)
}

/// One identifier or attribute name in the user's code, with its byte offset.
struct Word<'a> {
    text: &'a str,
    start: usize,
    /// Whether the word follows an `@`, so it names an attribute.
    attribute: bool,
}

/// Rejects the parts of WGSL that a user source may not use, with a position.
pub(crate) fn check_source(source: &str) -> Result<(), ShaderError> {
    for word in words(source)? {
        let Some(message) = forbidden(&word) else {
            continue;
        };
        let (line, column) = position(source, word.start);
        return Err(ShaderError::new(message, line, column));
    }
    Ok(())
}

/// The reason a word may not appear in a user source, if there is one.
fn forbidden(word: &Word<'_>) -> Option<String> {
    if word.attribute {
        return match word.text {
            "group" | "binding" | "vertex" | "fragment" | "compute" => Some(format!(
                "`@{}` is not allowed in a shader source; Kivact generates the bindings and entry points",
                word.text
            )),
            _ => None,
        };
    }
    match word.text {
        "enable" => Some("`enable` directives are not allowed in a shader source".to_owned()),
        "override" => Some("`override` declarations are not allowed in a shader source".to_owned()),
        name if name.starts_with("kivact_") => Some(format!(
            "`{name}` is reserved: names starting with `kivact_` belong to Kivact"
        )),
        _ => None,
    }
}

/// Lists the identifiers of `source`, skipping comments.
fn words(source: &str) -> Result<Vec<Word<'_>>, ShaderError> {
    let bytes = source.as_bytes();
    let mut words = Vec::new();
    let mut attribute = false;
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if bytes[i..].starts_with(b"//") {
            i = bytes[i..]
                .iter()
                .position(|&b| b == b'\n')
                .map_or(bytes.len(), |end| i + end);
        } else if bytes[i..].starts_with(b"/*") {
            i = skip_block_comment(source, i)?;
        } else if byte.is_ascii_whitespace() {
            i += 1;
        } else if byte == b'@' {
            attribute = true;
            i += 1;
        } else if byte.is_ascii_alphanumeric() || byte == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            // A number token such as `1e5` is not a word; only names are checked.
            if !byte.is_ascii_digit() {
                words.push(Word {
                    text: &source[start..i],
                    start,
                    attribute,
                });
            }
            attribute = false;
        } else {
            attribute = false;
            i += 1;
        }
    }
    Ok(words)
}

/// Returns the offset just past the block comment that starts at `start`.
/// WGSL block comments nest.
fn skip_block_comment(source: &str, start: usize) -> Result<usize, ShaderError> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut i = start;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"/*") {
            depth += 1;
            i += 2;
        } else if bytes[i..].starts_with(b"*/") {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return Ok(i);
            }
        } else {
            i += 1;
        }
    }
    let (line, column) = position(source, start);
    Err(ShaderError::new("unterminated block comment", line, column))
}
