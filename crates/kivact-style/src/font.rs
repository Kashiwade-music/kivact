//! Role: The core font variables of a theme: sizes, weights, line height and family.

use kivact_types::{Length, LineHeight, ThemeVar};

use crate::space::length_vars;
use crate::theme::FontFamily;

length_vars! {
    /// The font size of text that sets none, in logical px.
    SIZE_VAR / SIZE = "font.size", 14.0;
    /// A small font size.
    SIZE_SMALL_VAR / SIZE_SMALL = "font.size_small", 12.0;
    /// A large font size.
    SIZE_LARGE_VAR / SIZE_LARGE = "font.size_large", 18.0;
}

/// The weight of body text.
pub static WEIGHT_VAR: ThemeVar<u16> = ThemeVar::new("font.weight", 400);
/// The weight of strong text (a heading, a selected tab).
pub static WEIGHT_STRONG_VAR: ThemeVar<u16> = ThemeVar::new("font.weight_strong", 600);
/// The line height of text that sets none (the root's text style
/// carries it and every text below inherits it).
pub static LINE_HEIGHT_VAR: ThemeVar<LineHeight> =
    ThemeVar::new("font.line_height", LineHeight::Scale(1.35));
/// The family list of text that sets none.
pub static FAMILY_VAR: ThemeVar<FontFamily> = ThemeVar::new("font.family", FontFamily::SYSTEM);
