//! Role: Tokenize source languages for canonical code-block line styling.
use kivact::style::StyledRange;
use kivact::ui::{SyntaxKind, Tokenizer, syntax_ranges};
use std::ops::Range;
/// A caller-supplied code language.
#[derive(Clone)]
pub struct CodeLanguage {
    pub(crate) name: String,
    pub(crate) identifiers: Vec<String>,
    tokenizer: Tokenizer,
}

impl CodeLanguage {
    /// Define a language with a tokenizer returning ordered UTF-8 byte ranges.
    pub fn custom(
        name: impl Into<String>,
        identifiers: impl IntoIterator<Item = impl Into<String>>,
        tokenizer: Tokenizer,
    ) -> Self {
        Self {
            name: name.into(),
            identifiers: identifiers.into_iter().map(Into::into).collect(),
            tokenizer,
        }
    }
}

/// The built-in identifier or custom language used for highlighting.
#[derive(Clone)]
pub enum CodeLanguageInput {
    /// Built-in language identifier.
    Named(String),
    /// Caller-supplied tokenizer and language identifiers.
    Custom(CodeLanguage),
}

impl From<&str> for CodeLanguageInput {
    fn from(value: &str) -> Self {
        Self::Named(value.to_ascii_lowercase())
    }
}
impl From<String> for CodeLanguageInput {
    fn from(value: String) -> Self {
        Self::Named(value.to_ascii_lowercase())
    }
}
impl From<CodeLanguage> for CodeLanguageInput {
    fn from(value: CodeLanguage) -> Self {
        Self::Custom(value)
    }
}

impl CodeLanguageInput {
    /// The visible language name; plaintext has no header label.
    pub fn label(&self) -> &str {
        match self {
            Self::Named(name) if name == "plaintext" => "",
            Self::Named(name) => name,
            Self::Custom(language) => &language.name,
        }
    }
    fn tokenize(&self, line: &str) -> Vec<(Range<usize>, SyntaxKind)> {
        match self {
            Self::Named(name) => scan(line, name),
            Self::Custom(language) => (language.tokenizer)(line),
        }
    }
}

/// Convert one line's tokenizer ranges into standard editor line styles.
pub fn code_highlight(language: &CodeLanguageInput, line: &str) -> Vec<StyledRange> {
    syntax_ranges(line, &language.tokenize(line))
}
fn scan(line: &str, language: &str) -> Vec<(Range<usize>, SyntaxKind)> {
    const LANGUAGES: &str = "typescript javascript tsx jsx ts js json html xml svg css scss less python py bash sh zsh shell php hack yaml yml markdown md c h cpp c++ cc cxx hpp hh sql rust rs kotlin kt kts swift go golang";
    if !LANGUAGES.split_whitespace().any(|name| name == language) {
        return Vec::new();
    }
    let hash_comments = matches!(
        language,
        "python" | "py" | "bash" | "sh" | "zsh" | "shell" | "php" | "yaml" | "yml"
    );
    let mut tokens = Vec::new();
    let mut offset = 0;
    while offset < line.len() {
        let tail = &line[offset..];
        let Some(first) = tail.chars().next() else {
            break;
        };
        let (length, kind) = token(tail, first, language, hash_comments);
        if let Some(kind) = kind {
            tokens.push((offset..offset + length, kind));
        }
        offset += length;
    }
    tokens
}

fn token(
    tail: &str,
    first: char,
    language: &str,
    hash_comments: bool,
) -> (usize, Option<SyntaxKind>) {
    if tail.starts_with("//")
        || (hash_comments && first == '#')
        || (language == "sql" && tail.starts_with("--"))
    {
        (tail.len(), Some(SyntaxKind::Comment))
    } else if let Some(comment) = tail.strip_prefix("/*") {
        (
            comment.find("*/").map_or(tail.len(), |end| end + 4),
            Some(SyntaxKind::Comment),
        )
    } else if matches!(first, '"' | '\'' | '`') {
        let mut escaped = false;
        let mut end = tail.len();
        for (index, character) in tail.char_indices().skip(1) {
            if character == first && !escaped {
                end = index + character.len_utf8();
                break;
            }
            escaped = character == '\\' && !escaped;
        }
        let kind = if language == "json" && tail[end..].trim_start().starts_with(':') {
            SyntaxKind::Property
        } else {
            SyntaxKind::String
        };
        (end, Some(kind))
    } else if first.is_ascii_digit() {
        (
            tail.bytes()
                .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.'))
                .count(),
            Some(SyntaxKind::Number),
        )
    } else if first.is_alphabetic() || first == '_' {
        word_token(tail, first)
    } else {
        let kind = if "+-*/%=!<>&|^~?:".contains(first) {
            Some(SyntaxKind::Operator)
        } else if "{}()[];,.@".contains(first) {
            Some(SyntaxKind::Punctuation)
        } else {
            None
        };
        (first.len_utf8(), kind)
    }
}

fn word_token(tail: &str, first: char) -> (usize, Option<SyntaxKind>) {
    const KEYWORDS: &str = "as async await break case catch class const continue def default delete do else enum export extends fn for from func function if impl import in interface let match mod move mut new package pass private pub public raise ref return self static struct super switch throw trait try type use var void where while with yield SELECT FROM WHERE INSERT INTO UPDATE DELETE JOIN GROUP BY ORDER LIMIT CREATE TABLE";
    let end = tail
        .char_indices()
        .find(|(_, ch)| !ch.is_alphanumeric() && *ch != '_')
        .map_or(tail.len(), |(end, _)| end);
    let word = &tail[..end];
    let kind = if matches!(word, "true" | "false" | "null" | "None" | "nil") {
        Some(SyntaxKind::Constant)
    } else if KEYWORDS.split_whitespace().any(|keyword| keyword == word) {
        Some(SyntaxKind::Keyword)
    } else if tail[end..].trim_start().starts_with('(') {
        Some(SyntaxKind::Function)
    } else if first.is_uppercase() {
        Some(SyntaxKind::Type)
    } else {
        None
    };
    (end, kind)
}
