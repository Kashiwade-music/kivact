//! Role: Read the Soft tokens of the active theme.
use crate::Tokens;
use kivact::style::{ThemeRef, use_tokens};

/// The current theme's Neutral tokens. Read within a style closure to
/// follow theme changes.
pub fn tokens() -> ThemeRef<Tokens> {
    use_tokens::<Tokens>()
}
