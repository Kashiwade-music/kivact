//! Role: The heading recipe state, tokens, recipe table and default looks.

use crate::variants::Look;
use kivact_core::style::{Style, Theme, style};
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
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR) * scale)
        .weight(t.get(&kivact_core::style::font::WEIGHT_STRONG_VAR))
        .color(kivact_core::style::color::TEXT)
}

/// Semantic heading level passed to its recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeadingState {
    /// Heading level, clamped to 1 through 6.
    pub level: u8,
}
