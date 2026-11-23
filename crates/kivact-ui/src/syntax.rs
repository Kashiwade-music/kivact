//! Role: The kinds of source tokens a code block colors, and the ranges they paint.

use crate::syntax_colors as colors;
use kivact_core::style::{Color, StyledRange, style};
use std::ops::Range;

/// What a source token is. A kit's tokenizer returns these; the theme
/// colors each kind through `kivact::ui::theme::syntax`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
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
    Macro,
    Lifetime,
}

/// A tokenizer of one source line: byte ranges of `line` and their kinds.
pub type Tokenizer = fn(&str) -> Vec<(Range<usize>, SyntaxKind)>;

impl SyntaxKind {
    /// The theme color of the kind.
    pub const fn color(self) -> Color {
        match self {
            Self::Keyword => colors::KEYWORD,
            Self::Type => colors::TYPE,
            Self::String => colors::STRING,
            Self::Comment => colors::COMMENT,
            Self::Operator => colors::OPERATOR,
            Self::Punctuation => colors::PUNCTUATION,
            Self::Number => colors::NUMBER,
            Self::Constant => colors::CONSTANT,
            Self::Function => colors::FUNCTION,
            Self::Variable => colors::VARIABLE,
            Self::Tag => colors::TAG,
            Self::Attribute => colors::ATTRIBUTE,
            Self::Property => colors::PROPERTY,
            Self::Macro => colors::MACRO,
            Self::Lifetime => colors::LIFETIME,
        }
    }
}

/// The colored ranges of `line` for `code_block(..).highlight(..)`:
/// tokens in order, each inside the line and after the previous one;
/// a token that overlaps the previous one or leaves the line is dropped.
pub fn syntax_ranges(line: &str, tokens: &[(Range<usize>, SyntaxKind)]) -> Vec<StyledRange> {
    let mut offset = 0;
    tokens
        .iter()
        .filter_map(|(range, kind)| {
            if range.start < offset || range.is_empty() || line.get(range.clone()).is_none() {
                return None;
            }
            offset = range.end;
            Some(StyledRange {
                range: range.clone(),
                style: style().color(kind.color()),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the syntax ranges.

    use super::*;

    #[test]
    fn overlapping_and_outside_tokens_are_dropped() {
        let ranges = syntax_ranges(
            "let x",
            &[
                (0..3, SyntaxKind::Keyword),
                (2..4, SyntaxKind::Variable),
                (4..9, SyntaxKind::Variable),
                (4..5, SyntaxKind::Variable),
            ],
        );
        assert_eq!(
            ranges.iter().map(|r| r.range.clone()).collect::<Vec<_>>(),
            [0..3, 4..5]
        );
    }
}
