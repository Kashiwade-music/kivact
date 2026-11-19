//! Role: The heading recipe state, tokens, recipe table and default looks.

use crate::variants::Look;
use kivact_core::style::{ColorToken, Style, Theme, style};
pub(crate) fn heading(t: &Theme, s: &Look<HeadingState>) -> Style {
    let scale = match s.level {
        1 => 2.0,
        2 => 1.65,
        3 => 1.4,
        4 => 1.2,
        5 => 1.1,
        _ => 1.0,
    };
    style()
        .font_size(t.text().size * scale)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
}

/// Semantic heading level passed to its recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeadingState {
    /// Heading level, clamped to 1 through 6.
    pub level: u8,
}
