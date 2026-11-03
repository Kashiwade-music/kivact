//! Role: Read the Soft tokens of the active theme.
use crate::Tokens;
use kivact::style::current_theme;
use std::sync::Arc;

/// The current theme's Neutral tokens. Read within a style closure to
/// follow theme changes.
pub fn tokens() -> Arc<Tokens> {
    current_theme().get()
}
