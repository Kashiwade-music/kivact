//! Role: The highlighted lines a code block shows: tones, spans and lines.

/// The syntax category of a [`CodeSpan`]; each tone but `Plain` and
/// `Punctuation` has its own color ([`CodeColors`](crate::CodeColors)).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CodeTone {
    /// Identifiers and anything else.
    #[default]
    Plain,
    /// Language keywords.
    Keyword,
    /// String and character literals.
    String,
    /// Numeric literals.
    Number,
    /// Comments.
    Comment,
    /// Called or defined functions.
    Function,
    /// Type names.
    Type,
    /// Macro invocations.
    Macro,
    /// Lifetimes and loop labels such as `'a`.
    Lifetime,
    /// Operators and delimiters.
    Punctuation,
    /// Attributes such as `#[derive(..)]`.
    Attribute,
}

/// A run of code text in one tone.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CodeSpan {
    /// The text, without a line break.
    pub text: String,
    /// How the text is highlighted.
    pub tone: CodeTone,
}

impl CodeSpan {
    /// A span of `text` in `tone`.
    pub fn new(text: impl Into<String>, tone: CodeTone) -> Self {
        Self {
            text: text.into(),
            tone,
        }
    }
}

/// One line of a code block: its spans in order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CodeLine(pub Vec<CodeSpan>);

impl CodeLine {
    /// A line of unhighlighted `text`.
    pub fn plain(text: impl Into<String>) -> Self {
        Self(vec![CodeSpan::new(text, CodeTone::Plain)])
    }

    /// The line's text: its spans' texts in order.
    pub fn text(&self) -> String {
        self.0.iter().map(|span| span.text.as_str()).collect()
    }
}
