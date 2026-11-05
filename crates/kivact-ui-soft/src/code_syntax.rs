//! Role: Tokenize source languages for canonical code-block line styling.
use crate::SyntaxColors;
use kivact::style::StyledRange;
use std::ops::Range;
/// A syntax coloring category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Keyword,
    Type,
    String,
    Comment,
    Operator,
    Punctuation,
    Number,
    Constant,
    Function,
    Variable,
    Tag,
    Attribute,
    Property,
}

type Tokenizer = fn(&str) -> Vec<(Range<usize>, TokenKind)>;

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
    Named(String),
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
    pub fn label(&self) -> &str {
        match self {
            Self::Named(name) if name == "plaintext" => "",
            Self::Named(name) => name,
            Self::Custom(language) => &language.name,
        }
    }
    fn tokenize(&self, line: &str) -> Vec<(Range<usize>, TokenKind)> {
        match self {
            Self::Named(name) => scan(line, name),
            Self::Custom(language) => (language.tokenizer)(line),
        }
    }
}

/// Convert tokenizer byte ranges into standard editor line styles.
pub fn code_highlight(
    language: &CodeLanguageInput,
    line: &str,
    palette: &SyntaxColors,
) -> Vec<StyledRange> {
    language
        .tokenize(line)
        .into_iter()
        .map(|(range, kind)| {
            let color = match kind {
                TokenKind::Keyword => palette.keyword,
                TokenKind::Type => palette.r#type,
                TokenKind::String => palette.string,
                TokenKind::Comment => palette.comment,
                TokenKind::Operator => palette.operator,
                TokenKind::Punctuation => palette.punctuation,
                TokenKind::Number => palette.number,
                TokenKind::Constant => palette.constant,
                TokenKind::Function => palette.function,
                TokenKind::Variable => palette.variable,
                TokenKind::Tag => palette.tag,
                TokenKind::Attribute => palette.attribute,
                TokenKind::Property => palette.property,
            };

            StyledRange {
                range,
                color: Some(color),
                background: None,
                decoration: None,
            }
        })
        .collect()
}
fn scan(line: &str, language: &str) -> Vec<(Range<usize>, TokenKind)> {
    const LANGUAGES: &str = "typescript javascript tsx jsx ts js json html xml svg css scss less python py bash sh zsh shell php hack yaml yml markdown md c h cpp c++ cc cxx hpp hh sql rust rs kotlin kt kts swift go golang";
    const KEYWORDS: &str = "as async await break case catch class const continue def default delete do else enum export extends fn for from func function if impl import in interface let match mod move mut new package pass private pub public raise ref return self static struct super switch throw trait try type use var void where while with yield SELECT FROM WHERE INSERT INTO UPDATE DELETE JOIN GROUP BY ORDER LIMIT CREATE TABLE";
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
        let (length, kind) = if tail.starts_with("//")
            || (hash_comments && first == '#')
            || (language == "sql" && tail.starts_with("--"))
        {
            (tail.len(), Some(TokenKind::Comment))
        } else if let Some(comment) = tail.strip_prefix("/*") {
            (
                comment.find("*/").map_or(tail.len(), |end| end + 4),
                Some(TokenKind::Comment),
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
                TokenKind::Property
            } else {
                TokenKind::String
            };
            (end, Some(kind))
        } else if first.is_ascii_digit() {
            (
                tail.bytes()
                    .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.'))
                    .count(),
                Some(TokenKind::Number),
            )
        } else if first.is_alphabetic() || first == '_' {
            let end = tail
                .char_indices()
                .find(|(_, ch)| !ch.is_alphanumeric() && *ch != '_')
                .map_or(tail.len(), |(end, _)| end);
            let word = &tail[..end];
            let kind = if matches!(word, "true" | "false" | "null" | "None" | "nil") {
                Some(TokenKind::Constant)
            } else if KEYWORDS.split_whitespace().any(|keyword| keyword == word) {
                Some(TokenKind::Keyword)
            } else if tail[end..].trim_start().starts_with('(') {
                Some(TokenKind::Function)
            } else if first.is_uppercase() {
                Some(TokenKind::Type)
            } else {
                None
            };
            (end, kind)
        } else {
            let kind = if "+-*/%=!<>&|^~?:".contains(first) {
                Some(TokenKind::Operator)
            } else if "{}()[];,.@".contains(first) {
                Some(TokenKind::Punctuation)
            } else {
                None
            };
            (first.len_utf8(), kind)
        };
        if let Some(kind) = kind {
            tokens.push((offset..offset + length, kind));
        }
        offset += length;
    }
    tokens
}
